//! Kataloğa oyun ekleme ve güncelleme. Veri dış kaynaklardan (Wikidata, Steam...) derlenip
//! bu biçimde gönderilir; aynı oyun dış kimliğinden tanınır ve ikinci kez eklenmez.

use std::collections::{BTreeMap, BTreeSet};

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
/// Uzunluk üst sınırı (karakter); veritabanındaki `games_summary_len` ile aynı.
const MAX_OVERVIEW: usize = 4000;
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
    /// İçe aktarmanın bu kaydı elle bakılacak diye işaretlediği neden; panelden kaydedilince temizlenir.
    pub review_reason: Option<String>,
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
    let mut game = Validated::new(input)?;

    let existing = find_existing(tx, &game).await?;
    let (id, slug, created) = match existing {
        Some((id, slug)) => {
            // Yeniden içe aktarma yönetici düzenlemelerinin üzerine yazmaz.
            let snapshot = Snapshot::load(tx, id).await?;
            game.keep_manual_edits(&snapshot);
            update_game(tx, id, &game).await?;
            replace_relations(tx, id, &game, &snapshot.locked).await?;
            return Ok(Imported {
                id,
                slug,
                created: false,
            });
        }
        None => {
            let slug = free_slug(tx, &game).await?;
            let id = insert_game(tx, &slug, &game).await?;
            (id, slug, true)
        }
    };

    replace_relations(tx, id, &game, &BTreeSet::new()).await?;
    Ok(Imported { id, slug, created })
}

/// Yönetim panelinden yeni oyun: içe aktarmanın aksine var olan bir oyunu asla güncellemez.
/// Slug verilmişse boş olmalıdır (eski adresler de dolu sayılır); verilmezse başlıktan üretilir.
pub async fn create_game(tx: &mut PgConnection, input: GameInput) -> Result<Imported, ImportError> {
    let game = Validated::new(input)?;
    let explicit = game
        .input
        .slug
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty());
    let slug = if explicit {
        if slug_taken(tx, &game.slug, None).await? {
            return Err(ImportError::Invalid("slug_taken"));
        }
        game.slug.clone()
    } else {
        free_slug(tx, &game).await?
    };
    let id = insert_game(tx, &slug, &game).await?;
    replace_relations(tx, id, &game, &BTreeSet::new()).await?;
    Ok(Imported {
        id,
        slug,
        created: true,
    })
}

/// Yönetim panelinden düzenleme: oyun kimliğiyle bulunur ve alanlarıyla ilişkileri gönderilenle
/// değiştirilir. Slug verilip değişmişse eskisi yönlendirme için geçmişe yazılır.
pub async fn update_game_by_id(
    tx: &mut PgConnection,
    id: Uuid,
    input: GameInput,
) -> Result<Option<Imported>, ImportError> {
    let game = Validated::new(input)?;
    let current: Option<String> =
        sqlx::query_scalar("SELECT slug::text FROM games WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some(current) = current else {
        return Ok(None);
    };

    let explicit = game
        .input
        .slug
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty());
    let slug = if explicit {
        game.slug.clone()
    } else {
        current.clone()
    };
    if slug != current {
        if slug_taken(tx, &slug, Some(id)).await? {
            return Err(ImportError::Invalid("slug_taken"));
        }
        sqlx::query("DELETE FROM game_slug_history WHERE slug = $1 AND game_id = $2")
            .bind(&slug)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE games SET slug = $2 WHERE id = $1")
            .bind(id)
            .bind(&slug)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO game_slug_history (slug, game_id) VALUES ($1, $2)
             ON CONFLICT (slug) DO NOTHING",
        )
        .bind(&current)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    // Elle değiştirilen alanlar kilitlenir: sonraki içe aktarmalar onlara dokunmaz.
    let snapshot = Snapshot::load(tx, id).await?;
    let mut locked = snapshot.locked.clone();
    locked.extend(snapshot.changed_by(&game).into_iter().map(str::to_owned));
    update_game(tx, id, &game).await?;
    sqlx::query("UPDATE games SET locked_fields = $2 WHERE id = $1")
        .bind(id)
        .bind(locked.iter().cloned().collect::<Vec<_>>())
        .execute(&mut *tx)
        .await?;
    replace_relations(tx, id, &game, &BTreeSet::new()).await?;
    Ok(Some(Imported {
        id,
        slug,
        created: false,
    }))
}

/// Slug başka bir oyunda (ya da başka bir oyunun eski adresinde) kullanılıyor mu?
async fn slug_taken(
    tx: &mut PgConnection,
    slug: &str,
    except: Option<Uuid>,
) -> Result<bool, ImportError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM games WHERE slug = $1 AND id IS DISTINCT FROM $2)
             OR EXISTS (SELECT 1 FROM game_slug_history WHERE slug = $1 AND game_id IS DISTINCT FROM $2)",
    )
    .bind(slug)
    .bind(except)
    .fetch_one(&mut *tx)
    .await?)
}

/// Biçimi denetlenmiş ve normalleştirilmiş girdi.
struct Validated {
    input: GameInput,
    slug: String,
    kind: &'static str,
    status: Option<&'static str>,
    release: (Option<NaiveDate>, &'static str),
    releases: Vec<(String, String, Option<NaiveDate>, &'static str)>,
    review: Option<String>,
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
            .is_some_and(|s| s.chars().count() > MAX_OVERVIEW)
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

        let review = input
            .review_reason
            .as_deref()
            .map(str::trim)
            .filter(|reason| !reason.is_empty())
            .map(|reason| reason.chars().take(200).collect::<String>());
        Ok(Self {
            input,
            slug,
            kind,
            status,
            release,
            releases,
            review,
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
             (slug, title, kind, parent_id, series_id, summary, release_date,
              release_precision, cover_url, backdrop_url, logo_url, time_to_beat_minutes,
              pc_requirements, status, review_reason)
         VALUES ($1, $2, $3::game_kind, $4, $5, $6, $7, $8::date_precision, $9, $10, $11,
                 $12, $13, $14::publish_status, $15)
         RETURNING id",
    )
    .bind(slug)
    .bind(&input.title)
    .bind(game.kind)
    .bind(parent)
    .bind(series)
    .bind(&input.summary)
    .bind(game.release.0)
    .bind(game.release.1)
    .bind(&input.cover_url)
    .bind(&input.backdrop_url)
    .bind(&input.logo_url)
    .bind(input.time_to_beat_minutes)
    .bind(&input.pc_requirements)
    .bind(game.status.unwrap_or("draft"))
    .bind(&game.review)
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
                -- Seri değişince eski serideki sıra ve not geçersiz olur.
                series_position = CASE WHEN series_id IS NOT DISTINCT FROM $5 THEN series_position END,
                series_note = CASE WHEN series_id IS NOT DISTINCT FROM $5 THEN series_note END,
                release_date = $7, release_precision = $8::date_precision,
                cover_url = $9, backdrop_url = $10, logo_url = $11, time_to_beat_minutes = $12,
                pc_requirements = $13,
                status = COALESCE($14::publish_status, status),
                review_reason = $15,
                updated_at = now()
          WHERE id = $1",
    )
    .bind(id)
    .bind(&input.title)
    .bind(game.kind)
    .bind(parent)
    .bind(series)
    .bind(&input.summary)
    .bind(game.release.0)
    .bind(game.release.1)
    .bind(&input.cover_url)
    .bind(&input.backdrop_url)
    .bind(&input.logo_url)
    .bind(input.time_to_beat_minutes)
    .bind(&input.pc_requirements)
    .bind(game.status)
    .bind(&game.review)
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
    locked: &BTreeSet<String>,
) -> Result<(), ImportError> {
    let input = &game.input;
    let open = |group: &str| !locked.contains(group);

    for (group, delete) in [
        ("platforms", "DELETE FROM game_releases WHERE game_id = $1"),
        ("tags", "DELETE FROM game_tags WHERE game_id = $1"),
        ("companies", "DELETE FROM game_companies WHERE game_id = $1"),
        ("media", "DELETE FROM game_media WHERE game_id = $1"),
        ("languages", "DELETE FROM game_languages WHERE game_id = $1"),
        (
            "age_ratings",
            "DELETE FROM game_age_ratings WHERE game_id = $1",
        ),
        (
            "alt_titles",
            "DELETE FROM game_alt_titles WHERE game_id = $1",
        ),
    ] {
        if open(group) {
            sqlx::query(delete).bind(id).execute(&mut *tx).await?;
        }
    }

    for (platform, region, date, precision) in game.releases.iter().filter(|_| open("platforms")) {
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
        for name in names.iter().filter(|_| open("tags")) {
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
        for name in names.iter().filter(|_| open("companies")) {
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
        for (position, url) in urls.iter().enumerate().filter(|_| open("media")) {
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
    for (position, trailer) in input.trailers.iter().enumerate().filter(|_| open("media")) {
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

    for language in input.languages.iter().filter(|_| open("languages")) {
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

    for (board, rating) in input.age_ratings.iter().filter(|_| open("age_ratings")) {
        sqlx::query("INSERT INTO game_age_ratings (game_id, board, rating) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(board)
            .bind(rating)
            .execute(&mut *tx)
            .await?;
    }

    for title in input.alt_titles.iter().filter(|_| open("alt_titles")) {
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

/// Oyunun veritabanındaki hali; yönetici düzenlemelerini kilitlemek ve korumak için.
struct Snapshot {
    title: String,
    kind: String,
    parent_slug: Option<String>,
    summary: Option<String>,
    release: (Option<NaiveDate>, String),
    cover_url: Option<String>,
    backdrop_url: Option<String>,
    logo_url: Option<String>,
    series: Option<String>,
    genres: Vec<String>,
    themes: Vec<String>,
    modes: Vec<String>,
    perspectives: Vec<String>,
    developers: Vec<String>,
    publishers: Vec<String>,
    platforms: BTreeSet<String>,
    media: BTreeSet<String>,
    languages: BTreeSet<String>,
    alt_titles: BTreeSet<String>,
    locked: BTreeSet<String>,
}

#[derive(sqlx::FromRow)]
struct GameRow {
    title: String,
    kind: String,
    parent_slug: Option<String>,
    summary: Option<String>,
    release_date: Option<NaiveDate>,
    release_precision: String,
    cover_url: Option<String>,
    backdrop_url: Option<String>,
    logo_url: Option<String>,
    series: Option<String>,
    locked_fields: Vec<String>,
}

fn blank_to_none(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

/// Ad listesini kıyaslanabilir bir kümeye çevirir (slug'a göre, tekrarsız).
fn slugs(prefix: &str, names: &[String]) -> BTreeSet<String> {
    names
        .iter()
        .map(|name| format!("{prefix}:{}", slugify(name.trim())))
        .collect()
}

impl Snapshot {
    async fn load(tx: &mut PgConnection, id: Uuid) -> Result<Self, ImportError> {
        let row: GameRow = sqlx::query_as(
            "SELECT g.title, g.kind::text AS kind, p.slug::text AS parent_slug, g.summary,
                    g.release_date, g.release_precision::text AS release_precision,
                    g.cover_url, g.backdrop_url, g.logo_url, s.name AS series, g.locked_fields
               FROM games g
               LEFT JOIN games p ON p.id = g.parent_id
               LEFT JOIN series s ON s.id = g.series_id
              WHERE g.id = $1",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let tags: Vec<(String, String)> = sqlx::query_as(
            "SELECT t.kind::text, t.name FROM game_tags gt JOIN tags t ON t.id = gt.tag_id
              WHERE gt.game_id = $1 ORDER BY t.name",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let of_kind = |kind: &str| {
            tags.iter()
                .filter(|(k, _)| k == kind)
                .map(|(_, name)| name.clone())
                .collect::<Vec<_>>()
        };
        let companies: Vec<(String, String)> = sqlx::query_as(
            "SELECT gc.role::text, c.name FROM game_companies gc JOIN companies c ON c.id = gc.company_id
              WHERE gc.game_id = $1 ORDER BY c.name",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let role = |role: &str| {
            companies
                .iter()
                .filter(|(r, _)| r == role)
                .map(|(_, name)| name.clone())
                .collect::<Vec<_>>()
        };
        let platforms: Vec<String> = sqlx::query_scalar(
            "SELECT p.slug::text FROM game_releases r JOIN platforms p ON p.id = r.platform_id
              WHERE r.game_id = $1",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let media: Vec<String> = sqlx::query_scalar(
            "SELECT kind::text || ':' || COALESCE(url, provider || ':' || video_id)
               FROM game_media WHERE game_id = $1",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let languages: Vec<String> =
            sqlx::query_scalar("SELECT language FROM game_languages WHERE game_id = $1")
                .bind(id)
                .fetch_all(&mut *tx)
                .await?;
        let alt_titles: Vec<String> =
            sqlx::query_scalar("SELECT title FROM game_alt_titles WHERE game_id = $1")
                .bind(id)
                .fetch_all(&mut *tx)
                .await?;

        Ok(Self {
            title: row.title,
            kind: row.kind,
            parent_slug: row.parent_slug,
            summary: row.summary,
            release: (row.release_date, row.release_precision),
            cover_url: row.cover_url,
            backdrop_url: row.backdrop_url,
            logo_url: row.logo_url,
            series: row.series,
            genres: of_kind("genre"),
            themes: of_kind("theme"),
            modes: of_kind("mode"),
            perspectives: of_kind("perspective"),
            developers: role("developer"),
            publishers: role("publisher"),
            platforms: platforms.into_iter().collect(),
            media: media.into_iter().collect(),
            languages: languages.into_iter().collect(),
            alt_titles: alt_titles.into_iter().collect(),
            locked: row.locked_fields.into_iter().collect(),
        })
    }

    /// Gönderilenle farklı olan alan grupları: yönetici bunları elle değiştirmiştir.
    fn changed_by(&self, game: &Validated) -> Vec<&'static str> {
        let input = &game.input;
        let mut changed = Vec::new();
        let mut check = |group: &'static str, differs: bool| {
            if differs {
                changed.push(group);
            }
        };
        check("title", self.title != input.title);
        check(
            "summary",
            blank_to_none(&self.summary) != blank_to_none(&input.summary),
        );
        check(
            "release",
            self.release != (game.release.0, game.release.1.to_owned()),
        );
        check(
            "cover",
            blank_to_none(&self.cover_url) != blank_to_none(&input.cover_url),
        );
        check(
            "backdrop",
            blank_to_none(&self.backdrop_url) != blank_to_none(&input.backdrop_url),
        );
        check(
            "logo",
            blank_to_none(&self.logo_url) != blank_to_none(&input.logo_url),
        );
        check(
            "series",
            blank_to_none(&self.series).map(slugify) != blank_to_none(&input.series).map(slugify),
        );
        let tags =
            |genres: &[String], themes: &[String], modes: &[String], perspectives: &[String]| {
                let mut all = slugs("genre", genres);
                all.extend(slugs("theme", themes));
                all.extend(slugs("mode", modes));
                all.extend(slugs("perspective", perspectives));
                all
            };
        check(
            "tags",
            tags(&self.genres, &self.themes, &self.modes, &self.perspectives)
                != tags(
                    &input.genres,
                    &input.themes,
                    &input.modes,
                    &input.perspectives,
                ),
        );
        let companies = |developers: &[String], publishers: &[String]| {
            let mut all = slugs("developer", developers);
            all.extend(slugs("publisher", publishers));
            all
        };
        check(
            "companies",
            companies(&self.developers, &self.publishers)
                != companies(&input.developers, &input.publishers),
        );
        let platforms: BTreeSet<String> = game.releases.iter().map(|r| r.0.clone()).collect();
        check("platforms", self.platforms != platforms);
        let mut media: BTreeSet<String> = input
            .screenshots
            .iter()
            .map(|url| format!("screenshot:{url}"))
            .collect();
        media.extend(input.artworks.iter().map(|url| format!("artwork:{url}")));
        media.extend(
            input
                .trailers
                .iter()
                .map(|t| format!("trailer:{}:{}", t.provider, t.video_id)),
        );
        check("media", self.media != media);
        let languages: BTreeSet<String> =
            input.languages.iter().map(|l| l.language.clone()).collect();
        check("languages", self.languages != languages);
        let alt_titles: BTreeSet<String> = input
            .alt_titles
            .iter()
            .map(|t| t.trim().to_owned())
            .collect();
        check("alt_titles", self.alt_titles != alt_titles);
        changed
    }
}

impl Validated {
    /// Yeniden içe aktarmada: kilitli alanlar kayıtlı değerinde kalır; tür, ana oyun ve durum
    /// içe aktarmayla hiç değişmez.
    fn keep_manual_edits(&mut self, stored: &Snapshot) {
        let locked = |group: &str| stored.locked.contains(group);
        self.kind = KINDS
            .iter()
            .find(|kind| **kind == stored.kind)
            .copied()
            .unwrap_or(self.kind);
        self.input.parent_slug = stored.parent_slug.clone();
        self.status = None;
        if locked("title") {
            self.input.title = stored.title.clone();
        }
        if locked("summary") {
            self.input.summary = stored.summary.clone();
        }
        if locked("release") {
            self.release = (
                stored.release.0,
                PRECISIONS
                    .iter()
                    .chain(&["tba"])
                    .find(|p| **p == stored.release.1)
                    .copied()
                    .unwrap_or("tba"),
            );
            self.input.release_date = stored.release.0;
            // Tarihi elle verilmiş bir oyun gözden geçirmeye gerek bırakmaz.
            self.review = None;
        }
        if locked("cover") {
            self.input.cover_url = stored.cover_url.clone();
        }
        if locked("backdrop") {
            self.input.backdrop_url = stored.backdrop_url.clone();
        }
        if locked("logo") {
            self.input.logo_url = stored.logo_url.clone();
        }
        if locked("series") {
            self.input.series = stored.series.clone();
        }
    }
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

/// Gerçek bir PostgreSQL ister: `TEST_DATABASE_URL=… cargo test -- --ignored locks`.
#[cfg(test)]
mod db_tests {
    use super::*;

    async fn pool() -> sqlx::PgPool {
        let url = std::env::var("TEST_DATABASE_URL").unwrap();
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    fn game(json: serde_json::Value) -> GameInput {
        serde_json::from_value(json).unwrap()
    }

    #[tokio::test]
    #[ignore = "veritabanı ister"]
    async fn locks_survive_reimport() {
        let pool = pool().await;
        let ids = serde_json::json!({ "steam": "424242", "wikidata": "Q424242" });
        let mut tx = pool.begin().await.unwrap();
        let first = import_game(
            &mut tx,
            game(serde_json::json!({
                "title": "Lock Test", "status": "published", "summary": "from steam",
                "release_date": "2020-01-01", "genres": ["Action"], "developers": ["Studio A"],
                "review_reason": "No release date found",
                "releases": [{ "platform": "pc" }], "external_ids": ids
            })),
        )
        .await
        .ok()
        .unwrap();
        assert!(first.created);
        tx.commit().await.unwrap();

        // Yönetici yalnızca özeti ve türleri değiştirir.
        let mut tx = pool.begin().await.unwrap();
        update_game_by_id(
            &mut tx,
            first.id,
            game(serde_json::json!({
                "title": "Lock Test", "slug": first.slug, "status": "published",
                "summary": "written by hand", "release_date": "2020-01-01",
                "genres": ["Action", "Puzzle"], "developers": ["Studio A"],
                "releases": [{ "platform": "pc" }], "external_ids": ids
            })),
        )
        .await
        .ok()
        .unwrap();
        tx.commit().await.unwrap();
        let (locked, review): (Vec<String>, Option<String>) =
            sqlx::query_as("SELECT locked_fields, review_reason FROM games WHERE id = $1")
                .bind(first.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(locked, vec!["summary".to_owned(), "tags".to_owned()]);
        assert_eq!(review, None);

        // Yeniden içe aktarma: kilitli alanlar kalır, açık olanlar güncellenir, durum değişmez.
        let mut tx = pool.begin().await.unwrap();
        let again = import_game(
            &mut tx,
            game(serde_json::json!({
                "title": "Lock Test (Remaster)", "status": "draft", "summary": "from steam again",
                "release_date": "2021-02-02", "genres": ["Shooter"], "developers": ["Studio B"],
                "releases": [{ "platform": "pc" }, { "platform": "ps5" }], "external_ids": ids
            })),
        )
        .await
        .ok()
        .unwrap();
        tx.commit().await.unwrap();
        assert!(!again.created);
        assert_eq!(again.id, first.id);
        let (title, summary, status, date): (String, Option<String>, String, Option<NaiveDate>) =
            sqlx::query_as(
                "SELECT title, summary, status::text, release_date FROM games WHERE id = $1",
            )
            .bind(first.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(summary.as_deref(), Some("written by hand"));
        assert_eq!(title, "Lock Test (Remaster)");
        assert_eq!(status, "published");
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 2, 2));
        let genres: Vec<String> = sqlx::query_scalar(
            "SELECT t.name FROM game_tags g JOIN tags t ON t.id = g.tag_id WHERE g.game_id = $1 ORDER BY 1",
        )
        .bind(first.id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(genres, vec!["Action".to_owned(), "Puzzle".to_owned()]);
        let companies: Vec<String> = sqlx::query_scalar(
            "SELECT c.name FROM game_companies g JOIN companies c ON c.id = g.company_id WHERE g.game_id = $1",
        )
        .bind(first.id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(companies, vec!["Studio B".to_owned()]);
        let platforms: i64 =
            sqlx::query_scalar("SELECT count(*) FROM game_releases WHERE game_id = $1")
                .bind(first.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(platforms, 2);

        sqlx::query("DELETE FROM games WHERE id = $1")
            .bind(first.id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
