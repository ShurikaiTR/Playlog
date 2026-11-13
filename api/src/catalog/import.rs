//! Kataloğa oyun ekleme ve güncelleme. Veri dış kaynaklardan (Wikidata, Steam...) derlenip
//! bu biçimde gönderilir; aynı oyun dış kimliğinden tanınır ve ikinci kez eklenmez.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::PgConnection;
use uuid::Uuid;

use super::slug::{self, slugify};

const KINDS: &[&str] = &[
    "game",
    "dlc",
    "expansion",
    "bundle",
    "remake",
    "remaster",
    "port",
];
const STATUSES: &[&str] = &["draft", "published", "hidden"];
const PRECISIONS: &[&str] = &["day", "month", "quarter", "year"];
const MAX_URL_LEN: usize = 2000;
/// İlişki başına üst sınır; hatalı bir kaynak binlerce satır yazamasın.
const MAX_ITEMS: usize = 200;

#[derive(Deserialize)]
pub struct GameInput {
    /// Verilmezse başlıktan üretilir. Var olan oyunun slug'ı içe aktarmayla değişmez.
    pub slug: Option<String>,
    pub title: String,
    pub kind: Option<String>,
    /// Ek paket ya da yeniden yapımın ana oyunu; önceden eklenmiş olmalı.
    pub parent_slug: Option<String>,
    /// Serinin adı; yoksa oluşturulur.
    pub series: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    /// Verilmezse çıkışların en erkeni kullanılır.
    pub release_date: Option<NaiveDate>,
    pub release_precision: Option<String>,
    pub cover_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub logo_url: Option<String>,
    pub time_to_beat_minutes: Option<i32>,
    pub pc_requirements: Option<serde_json::Value>,
    /// Yeni oyunda varsayılan `draft`; var olan oyunda verilmezse durum korunur.
    pub status: Option<String>,
    #[serde(default)]
    pub releases: Vec<ReleaseInput>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub themes: Vec<String>,
    #[serde(default)]
    pub modes: Vec<String>,
    #[serde(default)]
    pub perspectives: Vec<String>,
    #[serde(default)]
    pub developers: Vec<String>,
    #[serde(default)]
    pub publishers: Vec<String>,
    /// Kaynak -> o kaynaktaki kimlik, örn. `{"steam": "292030", "wikidata": "Q11163283"}`.
    #[serde(default)]
    pub external_ids: BTreeMap<String, String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    #[serde(default)]
    pub artworks: Vec<String>,
    #[serde(default)]
    pub trailers: Vec<TrailerInput>,
    #[serde(default)]
    pub languages: Vec<LanguageInput>,
    /// Kurul -> derece, örn. `{"pegi": "18", "esrb": "M"}`.
    #[serde(default)]
    pub age_ratings: BTreeMap<String, String>,
    #[serde(default)]
    pub alt_titles: Vec<String>,
}

#[derive(Deserialize)]
pub struct ReleaseInput {
    /// Platformun slug'ı (`pc`, `ps5`, `switch`...).
    pub platform: String,
    pub region: Option<String>,
    pub date: Option<NaiveDate>,
    pub precision: Option<String>,
}

#[derive(Deserialize)]
pub struct TrailerInput {
    pub provider: String,
    pub video_id: String,
}

#[derive(Deserialize)]
pub struct LanguageInput {
    pub language: String,
    #[serde(default)]
    pub interface: bool,
    #[serde(default)]
    pub audio: bool,
    #[serde(default)]
    pub subtitles: bool,
}

pub enum ImportError {
    /// Girdi hatalı; kod istemciye döner.
    Invalid(&'static str),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for ImportError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err)
    }
}

pub struct Imported {
    pub id: Uuid,
    pub slug: String,
    pub created: bool,
}

/// Tek bir oyunu ekler ya da günceller. Çağıran bunu bir transaction içinde çalıştırır;
/// hata dönerse hiçbir şey yazılmamış olur.
///
/// Güncellemede oyunun alanları ve tüm ilişkileri (çıkışlar, etiketler, şirketler, medya, diller)
/// gönderilenle değiştirilir: içe aktarma kaynağı o oyunun tam halini gönderir.
pub async fn import_game(tx: &mut PgConnection, input: GameInput) -> Result<Imported, ImportError> {
    let game = Validated::new(input)?;

    let existing = find_existing(tx, &game).await?;
    let (id, slug, created) = match existing {
        Some((id, slug)) => {
            update_game(tx, id, &game).await?;
            (id, slug, false)
        }
        None => {
            let slug = free_slug(tx, &game).await?;
            let id = insert_game(tx, &slug, &game).await?;
            (id, slug, true)
        }
    };

    replace_relations(tx, id, &game).await?;
    Ok(Imported { id, slug, created })
}

/// Biçimi denetlenmiş ve normalleştirilmiş girdi.
struct Validated {
    input: GameInput,
    slug: String,
    kind: &'static str,
    status: Option<&'static str>,
    release: (Option<NaiveDate>, &'static str),
    releases: Vec<(String, String, Option<NaiveDate>, &'static str)>,
}

impl Validated {
    fn new(mut input: GameInput) -> Result<Self, ImportError> {
        // PostgreSQL metin alanları NUL kabul etmez; kaynak veride nadiren de olsa çıkar.
        let clean = |text: &mut Option<String>| {
            if let Some(text) = text
                && text.contains('\0')
            {
                *text = text.replace('\0', "");
            }
        };
        clean(&mut input.summary);
        clean(&mut input.description);
        input.title = input.title.replace('\0', "").trim().to_owned();
        if input.title.is_empty() || input.title.chars().count() > 300 {
            return Err(ImportError::Invalid("invalid_title"));
        }
        let slug = match input.slug.as_deref().map(str::trim) {
            Some(given) if !given.is_empty() => given.to_owned(),
            _ => slugify(&input.title),
        };
        if !slug::is_valid(&slug) {
            return Err(ImportError::Invalid("invalid_slug"));
        }

        let kind = one_of(
            input.kind.as_deref().unwrap_or("game"),
            KINDS,
            "invalid_kind",
        )?;
        let status = input
            .status
            .as_deref()
            .map(|status| one_of(status, STATUSES, "invalid_status"))
            .transpose()?;

        if input
            .summary
            .as_deref()
            .is_some_and(|s| s.chars().count() > 500)
        {
            return Err(ImportError::Invalid("invalid_summary"));
        }
        if input
            .time_to_beat_minutes
            .is_some_and(|minutes| minutes <= 0)
        {
            return Err(ImportError::Invalid("invalid_time_to_beat"));
        }
        for url in [&input.cover_url, &input.backdrop_url, &input.logo_url]
            .into_iter()
            .flatten()
            .chain(&input.screenshots)
            .chain(&input.artworks)
        {
            check_url(url)?;
        }

        let lists: [&[String]; 9] = [
            &input.genres,
            &input.themes,
            &input.modes,
            &input.perspectives,
            &input.developers,
            &input.publishers,
            &input.screenshots,
            &input.artworks,
            &input.alt_titles,
        ];
        if lists.iter().any(|list| list.len() > MAX_ITEMS)
            || input.releases.len() > MAX_ITEMS
            || input.languages.len() > MAX_ITEMS
            || input.trailers.len() > MAX_ITEMS
        {
            return Err(ImportError::Invalid("too_many_items"));
        }

        let mut releases = Vec::with_capacity(input.releases.len());
        for release in &input.releases {
            let region = release.region.as_deref().unwrap_or("worldwide");
            let is_country = region.len() == 2 && region.bytes().all(|b| b.is_ascii_lowercase());
            if region != "worldwide" && !is_country {
                return Err(ImportError::Invalid("invalid_region"));
            }
            let (date, precision) = dated(release.date, release.precision.as_deref())?;
            releases.push((release.platform.clone(), region.to_owned(), date, precision));
        }

        // Oyunun tarihi verilmediyse çıkışların en erkeni.
        let release = match input.release_date {
            Some(date) => dated(Some(date), input.release_precision.as_deref())?,
            None => releases
                .iter()
                .filter_map(|(_, _, date, precision)| date.map(|date| (date, *precision)))
                .min_by_key(|(date, _)| *date)
                .map_or((None, "tba"), |(date, precision)| (Some(date), precision)),
        };

        for (source, external_id) in &input.external_ids {
            let source_ok = (2..=31).contains(&source.len())
                && source.starts_with(|c: char| c.is_ascii_lowercase())
                && source
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
            if !source_ok || external_id.is_empty() || external_id.len() > 200 {
                return Err(ImportError::Invalid("invalid_external_id"));
            }
        }
        for language in &input.languages {
            if !language_ok(&language.language) {
                return Err(ImportError::Invalid("invalid_language"));
            }
        }
        for (board, rating) in &input.age_ratings {
            let board_ok =
                (2..=10).contains(&board.len()) && board.bytes().all(|b| b.is_ascii_lowercase());
            if !board_ok || rating.is_empty() || rating.chars().count() > 20 {
                return Err(ImportError::Invalid("invalid_age_rating"));
            }
        }
        for trailer in &input.trailers {
            if trailer.provider.is_empty() || trailer.video_id.is_empty() {
                return Err(ImportError::Invalid("invalid_trailer"));
            }
        }

        Ok(Self {
            input,
            slug,
            kind,
            status,
            release,
            releases,
        })
    }
}

fn one_of(
    value: &str,
    allowed: &[&'static str],
    code: &'static str,
) -> Result<&'static str, ImportError> {
    allowed
        .iter()
        .find(|candidate| **candidate == value)
        .copied()
        .ok_or(ImportError::Invalid(code))
}

/// Tarih ve kesinliği birlikte geçerli kılar: tarih yoksa `tba`, varsa varsayılan `day`.
fn dated(
    date: Option<NaiveDate>,
    precision: Option<&str>,
) -> Result<(Option<NaiveDate>, &'static str), ImportError> {
    match date {
        None => Ok((None, "tba")),
        Some(date) => Ok((
            Some(date),
            one_of(precision.unwrap_or("day"), PRECISIONS, "invalid_precision")?,
        )),
    }
}

/// Görsel adresleri sayfalara girer: yalnızca https kabul edilir.
fn check_url(url: &str) -> Result<(), ImportError> {
    if url.starts_with("https://") && url.len() <= MAX_URL_LEN && !url.contains(char::is_whitespace)
    {
        Ok(())
    } else {
        Err(ImportError::Invalid("invalid_url"))
    }
}

/// BCP 47'nin kullandığımız alt kümesi: `en`, `tr`, `pt-BR`, `zh-Hans`.
fn language_ok(code: &str) -> bool {
    let mut parts = code.split('-');
    let primary_ok = parts.next().is_some_and(|primary| {
        (2..=3).contains(&primary.len()) && primary.bytes().all(|b| b.is_ascii_lowercase())
    });
    primary_ok
        && parts.all(|part| {
            (2..=8).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

/// Var olan oyunu bulur: önce dış kimliklerden, yoksa slug'dan.
async fn find_existing(
    tx: &mut PgConnection,
    game: &Validated,
) -> Result<Option<(Uuid, String)>, ImportError> {
    let mut found: Option<(Uuid, String)> = None;
    for (source, external_id) in &game.input.external_ids {
        let row: Option<(Uuid, String)> = sqlx::query_as(
            "SELECT g.id, g.slug::text
               FROM game_external_ids e JOIN games g ON g.id = e.game_id
              WHERE e.source = $1 AND e.external_id = $2
                FOR UPDATE OF g",
        )
        .bind(source)
        .bind(external_id)
        .fetch_optional(&mut *tx)
        .await?;
        match (&found, row) {
            // Kimlikler iki ayrı oyunu gösteriyor: kaynak veride bir karışıklık var.
            (Some((id, _)), Some((other, _))) if *id != other => {
                return Err(ImportError::Invalid("conflicting_external_ids"));
            }
            (None, Some(row)) => found = Some(row),
            _ => {}
        }
    }
    if found.is_some() {
        return Ok(found);
    }

    let by_slug: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM games WHERE slug = $1 FOR UPDATE")
            .bind(&game.slug)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(id) = by_slug else { return Ok(None) };
    // Aynı slug, ama aynı kaynakta başka bir kimlik: bu başka bir oyun (aynı adlı yeniden yapım gibi).
    if differs_by_external_id(tx, id, game).await? {
        return Ok(None);
    }
    Ok(Some((id, game.slug.clone())))
}

async fn differs_by_external_id(
    tx: &mut PgConnection,
    id: Uuid,
    game: &Validated,
) -> Result<bool, ImportError> {
    for (source, external_id) in &game.input.external_ids {
        let other: Option<String> = sqlx::query_scalar(
            "SELECT external_id FROM game_external_ids WHERE game_id = $1 AND source = $2 LIMIT 1",
        )
        .bind(id)
        .bind(source)
        .fetch_optional(&mut *tx)
        .await?;
        if other.is_some_and(|other| other != *external_id) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Yeni oyun için boş bir slug: önce kendisi, doluysa sonuna çıkış yılı eklenir.
async fn free_slug(tx: &mut PgConnection, game: &Validated) -> Result<String, ImportError> {
    let mut candidates = vec![game.slug.clone()];
    if let Some(date) = game.release.0 {
        candidates.push(format!("{}-{}", game.slug, date.format("%Y")));
    }
    for candidate in candidates {
        if !slug::is_valid(&candidate) {
            continue;
        }
        let taken: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM games WHERE slug = $1)
                 OR EXISTS (SELECT 1 FROM game_slug_history WHERE slug = $1)",
        )
        .bind(&candidate)
        .fetch_one(&mut *tx)
        .await?;
        if !taken {
            return Ok(candidate);
        }
    }
    Err(ImportError::Invalid("slug_taken"))
}

async fn parent_and_series(
    tx: &mut PgConnection,
    game: &Validated,
) -> Result<(Option<Uuid>, Option<Uuid>), ImportError> {
    let parent = match game.input.parent_slug.as_deref() {
        Some(parent_slug) => Some(
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM games WHERE slug = $1")
                .bind(parent_slug)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(ImportError::Invalid("unknown_parent"))?,
        ),
        None => None,
    };
    let series = match game.input.series.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => {
            let series_slug = slugify(name);
            if series_slug.is_empty() || name.chars().count() > 200 {
                return Err(ImportError::Invalid("invalid_series"));
            }
            // Çakışmada adı yerinde bırakan güncelleme, satırı RETURNING ile geri almak içindir.
            Some(
                sqlx::query_scalar::<_, Uuid>(
                    "INSERT INTO series (slug, name) VALUES ($1, $2)
                     ON CONFLICT (slug) DO UPDATE SET name = series.name
                     RETURNING id",
                )
                .bind(series_slug)
                .bind(name)
                .fetch_one(&mut *tx)
                .await?,
            )
        }
        _ => None,
    };
    Ok((parent, series))
}

async fn insert_game(
    tx: &mut PgConnection,
    slug: &str,
    game: &Validated,
) -> Result<Uuid, ImportError> {
    let (parent, series) = parent_and_series(tx, game).await?;
    let input = &game.input;
    Ok(sqlx::query_scalar(
        "INSERT INTO games
             (slug, title, kind, parent_id, series_id, summary, description, release_date,
              release_precision, cover_url, backdrop_url, logo_url, time_to_beat_minutes,
              pc_requirements, status)
         VALUES ($1, $2, $3::game_kind, $4, $5, $6, $7, $8, $9::date_precision, $10, $11, $12,
                 $13, $14, $15::publish_status)
         RETURNING id",
    )
    .bind(slug)
    .bind(&input.title)
    .bind(game.kind)
    .bind(parent)
    .bind(series)
    .bind(&input.summary)
    .bind(&input.description)
    .bind(game.release.0)
    .bind(game.release.1)
    .bind(&input.cover_url)
    .bind(&input.backdrop_url)
    .bind(&input.logo_url)
    .bind(input.time_to_beat_minutes)
    .bind(&input.pc_requirements)
    .bind(game.status.unwrap_or("draft"))
    .fetch_one(&mut *tx)
    .await?)
}

async fn update_game(tx: &mut PgConnection, id: Uuid, game: &Validated) -> Result<(), ImportError> {
    let (parent, series) = parent_and_series(tx, game).await?;
    if parent == Some(id) {
        return Err(ImportError::Invalid("unknown_parent"));
    }
    let input = &game.input;
    sqlx::query(
        "UPDATE games
            SET title = $2, kind = $3::game_kind, parent_id = $4, series_id = $5, summary = $6,
                description = $7, release_date = $8, release_precision = $9::date_precision,
                cover_url = $10, backdrop_url = $11, logo_url = $12, time_to_beat_minutes = $13,
                pc_requirements = $14,
                status = COALESCE($15::publish_status, status),
                updated_at = now()
          WHERE id = $1",
    )
    .bind(id)
    .bind(&input.title)
    .bind(game.kind)
    .bind(parent)
    .bind(series)
    .bind(&input.summary)
    .bind(&input.description)
    .bind(game.release.0)
    .bind(game.release.1)
    .bind(&input.cover_url)
    .bind(&input.backdrop_url)
    .bind(&input.logo_url)
    .bind(input.time_to_beat_minutes)
    .bind(&input.pc_requirements)
    .bind(game.status)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Oyunun ilişkilerini gönderilenle değiştirir. Dış kimlikler silinmez, yalnızca eklenir:
/// başka bir kaynağın daha önce yazdığı kimlik korunur.
async fn replace_relations(
    tx: &mut PgConnection,
    id: Uuid,
    game: &Validated,
) -> Result<(), ImportError> {
    let input = &game.input;

    for delete in [
        "DELETE FROM game_releases WHERE game_id = $1",
        "DELETE FROM game_tags WHERE game_id = $1",
        "DELETE FROM game_companies WHERE game_id = $1",
        "DELETE FROM game_media WHERE game_id = $1",
        "DELETE FROM game_languages WHERE game_id = $1",
        "DELETE FROM game_age_ratings WHERE game_id = $1",
        "DELETE FROM game_alt_titles WHERE game_id = $1",
    ] {
        sqlx::query(delete).bind(id).execute(&mut *tx).await?;
    }

    for (platform, region, date, precision) in &game.releases {
        let platform_id: i16 = sqlx::query_scalar("SELECT id FROM platforms WHERE slug = $1")
            .bind(platform)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(ImportError::Invalid("unknown_platform"))?;
        sqlx::query(
            "INSERT INTO game_releases (game_id, platform_id, region, release_date, release_precision)
             VALUES ($1, $2, $3, $4, $5::date_precision)
             ON CONFLICT (game_id, platform_id, region) DO NOTHING",
        )
        .bind(id)
        .bind(platform_id)
        .bind(region)
        .bind(date)
        .bind(precision)
        .execute(&mut *tx)
        .await?;
    }

    for (kind, names) in [
        ("genre", &input.genres),
        ("theme", &input.themes),
        ("mode", &input.modes),
        ("perspective", &input.perspectives),
    ] {
        for name in names {
            let name = name.trim();
            let tag_slug = slugify(name);
            if tag_slug.is_empty() || name.chars().count() > 80 {
                return Err(ImportError::Invalid("invalid_tag"));
            }
            sqlx::query(
                "WITH tag AS (
                     INSERT INTO tags (kind, slug, name) VALUES ($2::tag_kind, $3, $4)
                     ON CONFLICT (kind, slug) DO UPDATE SET name = tags.name
                     RETURNING id)
                 INSERT INTO game_tags (game_id, tag_id) SELECT $1, id FROM tag
                 ON CONFLICT DO NOTHING",
            )
            .bind(id)
            .bind(kind)
            .bind(tag_slug)
            .bind(name)
            .execute(&mut *tx)
            .await?;
        }
    }

    for (role, names) in [
        ("developer", &input.developers),
        ("publisher", &input.publishers),
    ] {
        for name in names {
            let name = name.trim();
            let company_slug = slugify(name);
            if company_slug.is_empty() || name.chars().count() > 200 {
                return Err(ImportError::Invalid("invalid_company"));
            }
            sqlx::query(
                "WITH company AS (
                     INSERT INTO companies (slug, name) VALUES ($2, $3)
                     ON CONFLICT (slug) DO UPDATE SET name = companies.name
                     RETURNING id)
                 INSERT INTO game_companies (game_id, company_id, role)
                 SELECT $1, id, $4::company_role FROM company
                 ON CONFLICT DO NOTHING",
            )
            .bind(id)
            .bind(company_slug)
            .bind(name)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }
    }

    for (source, external_id) in &input.external_ids {
        let owner: Uuid = sqlx::query_scalar(
            "INSERT INTO game_external_ids (source, external_id, game_id) VALUES ($1, $2, $3)
             ON CONFLICT (source, external_id) DO UPDATE SET fetched_at = now()
             RETURNING game_id",
        )
        .bind(source)
        .bind(external_id)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if owner != id {
            return Err(ImportError::Invalid("conflicting_external_ids"));
        }
    }

    for (kind, urls) in [
        ("screenshot", &input.screenshots),
        ("artwork", &input.artworks),
    ] {
        for (position, url) in urls.iter().enumerate() {
            sqlx::query(
                "INSERT INTO game_media (game_id, kind, url, position)
                 VALUES ($1, $2::media_kind, $3, $4)",
            )
            .bind(id)
            .bind(kind)
            .bind(url)
            .bind(position as i32)
            .execute(&mut *tx)
            .await?;
        }
    }
    for (position, trailer) in input.trailers.iter().enumerate() {
        sqlx::query(
            "INSERT INTO game_media (game_id, kind, provider, video_id, position)
             VALUES ($1, 'trailer', $2, $3, $4)",
        )
        .bind(id)
        .bind(&trailer.provider)
        .bind(&trailer.video_id)
        .bind(position as i32)
        .execute(&mut *tx)
        .await?;
    }

    for language in &input.languages {
        sqlx::query(
            "INSERT INTO game_languages (game_id, language, interface, audio, subtitles)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (game_id, language) DO NOTHING",
        )
        .bind(id)
        .bind(&language.language)
        .bind(language.interface)
        .bind(language.audio)
        .bind(language.subtitles)
        .execute(&mut *tx)
        .await?;
    }

    for (board, rating) in &input.age_ratings {
        sqlx::query("INSERT INTO game_age_ratings (game_id, board, rating) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(board)
            .bind(rating)
            .execute(&mut *tx)
            .await?;
    }

    for title in &input.alt_titles {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 300 {
            return Err(ImportError::Invalid("invalid_alt_title"));
        }
        sqlx::query(
            "INSERT INTO game_alt_titles (game_id, title) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(title)
        .execute(&mut *tx)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(json: serde_json::Value) -> GameInput {
        serde_json::from_value(json).unwrap()
    }

    fn code(result: Result<Validated, ImportError>) -> &'static str {
        match result {
            Ok(_) => "ok",
            Err(ImportError::Invalid(code)) => code,
            Err(ImportError::Db(_)) => "db",
        }
    }

    #[test]
    fn minimal_input_gets_defaults() {
        let game = Validated::new(input(serde_json::json!({ "title": " Half-Life 2 " })))
            .ok()
            .unwrap();
        assert_eq!(game.input.title, "Half-Life 2");
        assert_eq!(game.slug, "half-life-2");
        assert_eq!(game.kind, "game");
        assert_eq!(game.status, None);
        assert_eq!(game.release, (None, "tba"));
    }

    #[test]
    fn release_date_falls_back_to_the_earliest_release() {
        let game = Validated::new(input(serde_json::json!({
            "title": "Hades",
            "releases": [
                { "platform": "switch", "date": "2020-09-17" },
                { "platform": "pc", "date": "2018-12-01", "precision": "month" },
                { "platform": "ps5" }
            ]
        })))
        .ok()
        .unwrap();
        assert_eq!(
            game.release,
            (NaiveDate::from_ymd_opt(2018, 12, 1), "month")
        );
        assert_eq!(game.releases[2].3, "tba");
        assert_eq!(game.releases[0].1, "worldwide");
    }

    #[test]
    fn bad_fields_are_rejected_with_a_code() {
        let reject = |json| code(Validated::new(input(json)));
        assert_eq!(
            reject(serde_json::json!({ "title": "  " })),
            "invalid_title"
        );
        assert_eq!(reject(serde_json::json!({ "title": "™" })), "invalid_slug");
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "slug": "Not Valid" })),
            "invalid_slug"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "kind": "movie" })),
            "invalid_kind"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "status": "live" })),
            "invalid_status"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "cover_url": "http://x.test/a.jpg" })),
            "invalid_url"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "screenshots": ["javascript:alert(1)"] })),
            "invalid_url"
        );
        assert_eq!(
            reject(
                serde_json::json!({ "title": "A", "release_date": "2020-01-01", "release_precision": "tba" })
            ),
            "invalid_precision"
        );
        assert_eq!(
            reject(
                serde_json::json!({ "title": "A", "releases": [{ "platform": "pc", "region": "Europe" }] })
            ),
            "invalid_region"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "external_ids": { "Steam": "1" } })),
            "invalid_external_id"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "languages": [{ "language": "Turkish" }] })),
            "invalid_language"
        );
        assert_eq!(
            reject(serde_json::json!({ "title": "A", "time_to_beat_minutes": 0 })),
            "invalid_time_to_beat"
        );
    }

    #[test]
    fn language_codes_follow_bcp47_shape() {
        for ok in ["en", "tr", "pt-BR", "zh-Hans", "fil"] {
            assert!(language_ok(ok), "{ok}");
        }
        for bad in ["", "e", "EN", "english", "pt_BR", "pt-"] {
            assert!(!language_ok(bad), "{bad}");
        }
    }
}
