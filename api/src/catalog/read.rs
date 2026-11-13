//! Katalogdan okuma: oyun listesi ve tek oyunun tüm ilişkileriyle ayrıntısı.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub const DEFAULT_PAGE: i64 = 24;
pub const MAX_PAGE: i64 = 100;
/// Aramanın başlıktaki bir kelime grubuna en az bu kadar benzemesi gerekir (trigram,
/// 0-1). 0.5: "witcer" The Witcher'ı bulur, ilgisiz başlıklar gelmez.
const TYPO_TOLERANCE: f32 = 0.5;

/// Liste süzgeçleri; hepsi isteğe bağlıdır.
#[derive(Default)]
pub struct ListFilter {
    /// Başlıkta ve diğer adlarda aranır.
    pub query: Option<String>,
    /// Tür slug'ları; oyun hepsini taşımalı (daraltır).
    pub genres: Vec<String>,
    /// Platform slug'ları ya da aileleri (`pc`, `playstation`...); herhangi biri yeter (genişletir).
    pub platforms: Vec<String>,
    /// Oyun modu slug'ları; herhangi biri yeter.
    pub modes: Vec<String>,
    pub year_min: Option<i32>,
    pub year_max: Option<i32>,
    pub window: Option<Window>,
    pub sort: Sort,
    pub limit: i64,
    pub offset: i64,
}

/// Çıkış tarihine göre pencere.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Window {
    /// Bugün ya da daha önce çıkmış.
    Released,
    /// Tarihi bugünden sonra olanlar.
    Upcoming,
}

impl Window {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "released" => Some(Self::Released),
            "upcoming" => Some(Self::Upcoming),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Released => "released",
            Self::Upcoming => "upcoming",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sort {
    /// En yeni çıkış önce; arama varsa en yakın eşleşme önce.
    #[default]
    Newest,
    /// En yakın çıkış önce (yakında çıkacaklar için).
    Soonest,
    Title,
    /// Kataloğa en son eklenen önce.
    Added,
}

impl Sort {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "newest" => Some(Self::Newest),
            "soonest" => Some(Self::Soonest),
            "title" => Some(Self::Title),
            "added" => Some(Self::Added),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Newest => "newest",
            Self::Soonest => "soonest",
            Self::Title => "title",
            Self::Added => "added",
        }
    }
}

/// Kartlarda, vitrinde ve arama sonuçlarında gösterilen özet.
#[derive(Serialize, FromRow)]
pub struct GameCard {
    pub slug: String,
    pub title: String,
    pub kind: String,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub logo_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub release_precision: String,
    pub genres: Vec<String>,
    pub developers: Vec<String>,
    #[serde(skip)]
    total: i64,
}

#[derive(Serialize)]
pub struct GamePage {
    pub total: i64,
    pub items: Vec<GameCard>,
}

/// Boş liste "süzgeç yok" demektir; sorguya NULL olarak gider.
fn some(values: &[String]) -> Option<&[String]> {
    (!values.is_empty()).then_some(values)
}

/// Yalnızca yayındaki oyunlar. Sorgu sabittir; süzgeçler NULL ise devre dışı kalır.
pub async fn list(db: &PgPool, filter: &ListFilter) -> sqlx::Result<GamePage> {
    let pattern = filter
        .query
        .as_deref()
        .map(|query| format!("%{}%", escape_like(query)));
    let items: Vec<GameCard> = sqlx::query_as(
        "SELECT g.slug::text AS slug, g.title, g.kind::text AS kind, g.summary, g.cover_url,
                g.backdrop_url, g.logo_url, g.release_date,
                g.release_precision::text AS release_precision,
                COALESCE((SELECT array_agg(t.name ORDER BY t.name)
                            FROM game_tags gt JOIN tags t ON t.id = gt.tag_id
                           WHERE gt.game_id = g.id AND t.kind = 'genre'), '{}') AS genres,
                COALESCE((SELECT array_agg(c.name ORDER BY c.name)
                            FROM game_companies gc JOIN companies c ON c.id = gc.company_id
                           WHERE gc.game_id = g.id AND gc.role = 'developer'), '{}') AS developers,
                count(*) OVER () AS total
           FROM games g
          WHERE g.status = 'published'
            AND ($1::text IS NULL
                 OR g.title ILIKE $2
                 OR word_similarity($1, g.title) >= $3
                 OR EXISTS (SELECT 1 FROM game_alt_titles a
                             WHERE a.game_id = g.id AND a.title ILIKE $2))
            AND ($4::text[] IS NULL
                 OR (SELECT count(*) FROM game_tags gt JOIN tags t ON t.id = gt.tag_id
                      WHERE gt.game_id = g.id AND t.kind = 'genre' AND t.slug::text = ANY($4))
                    = cardinality($4))
            AND ($5::text[] IS NULL
                 OR EXISTS (SELECT 1 FROM game_releases r JOIN platforms p ON p.id = r.platform_id
                             WHERE r.game_id = g.id
                               AND (p.slug::text = ANY($5) OR p.family = ANY($5))))
            AND ($6::text[] IS NULL
                 OR EXISTS (SELECT 1 FROM game_tags gt JOIN tags t ON t.id = gt.tag_id
                             WHERE gt.game_id = g.id AND t.kind = 'mode' AND t.slug::text = ANY($6)))
            AND ($7::int IS NULL OR g.release_date >= make_date($7, 1, 1))
            AND ($8::int IS NULL OR g.release_date < make_date($8 + 1, 1, 1))
            AND ($9::text IS NULL
                 OR ($9 = 'released' AND g.release_date <= current_date)
                 OR ($9 = 'upcoming' AND g.release_date > current_date))
          ORDER BY CASE WHEN $10 = 'title' THEN lower(g.title) END,
                   CASE WHEN $10 = 'added' THEN g.created_at END DESC,
                   CASE WHEN $10 = 'soonest' THEN g.release_date END ASC NULLS LAST,
                   CASE WHEN $10 = 'newest' AND $1::text IS NOT NULL
                        THEN word_similarity($1, g.title) END DESC,
                   g.release_date DESC NULLS LAST,
                   g.id
          LIMIT $11 OFFSET $12",
    )
    .bind(&filter.query)
    .bind(&pattern)
    .bind(TYPO_TOLERANCE)
    .bind(some(&filter.genres))
    .bind(some(&filter.platforms))
    .bind(some(&filter.modes))
    .bind(filter.year_min)
    .bind(filter.year_max)
    .bind(filter.window.map(Window::as_str))
    .bind(filter.sort.as_str())
    .bind(filter.limit)
    .bind(filter.offset)
    .fetch_all(db)
    .await?;

    // Pencere sayacı her satırda aynıdır. Sayfa boşsa (son sayfanın ötesi) toplam ayrıca sayılmaz.
    let total = items.first().map_or(0, |card| card.total);
    Ok(GamePage { total, items })
}

/// Bir etiket türünün (tür, tema, mod) yayındaki oyunlarda kaç kez geçtiği; en yaygın önce.
#[derive(Serialize, FromRow)]
pub struct TagCount {
    pub slug: String,
    pub name: String,
    pub count: i64,
}

pub async fn tags(db: &PgPool, kind: &str) -> sqlx::Result<Vec<TagCount>> {
    sqlx::query_as(
        "SELECT t.slug::text AS slug, t.name, count(*) AS count
           FROM tags t
           JOIN game_tags gt ON gt.tag_id = t.id
           JOIN games g ON g.id = gt.game_id AND g.status = 'published'
          WHERE t.kind = $1::tag_kind
          GROUP BY t.id
          ORDER BY count(*) DESC, t.name",
    )
    .bind(kind)
    .fetch_all(db)
    .await
}

/// LIKE desenindeki özel karakterleri etkisizleştirir; kullanıcının yazdığı `%` düz karakterdir.
fn escape_like(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for c in raw.chars() {
        if matches!(c, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[derive(Serialize, FromRow)]
pub struct Named {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize, FromRow)]
pub struct GameRef {
    pub slug: String,
    pub title: String,
    pub kind: String,
    pub cover_url: Option<String>,
}

#[derive(Serialize)]
pub struct Release {
    pub platform: Platform,
    pub region: String,
    pub release_date: Option<NaiveDate>,
    pub release_precision: String,
}

#[derive(Serialize)]
pub struct Platform {
    pub slug: String,
    pub name: String,
    pub family: String,
}

#[derive(Serialize)]
pub struct Trailer {
    pub provider: String,
    pub video_id: String,
}

#[derive(Serialize, FromRow)]
pub struct Language {
    pub language: String,
    pub interface: bool,
    pub audio: bool,
    pub subtitles: bool,
}

#[derive(Serialize)]
pub struct GameDetail {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub release_precision: String,
    pub cover_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub logo_url: Option<String>,
    pub time_to_beat_minutes: Option<i32>,
    pub pc_requirements: Option<serde_json::Value>,
    /// Ek paket ya da yeniden yapımın ana oyunu.
    pub parent: Option<GameRef>,
    pub series: Option<Named>,
    pub releases: Vec<Release>,
    pub genres: Vec<Named>,
    pub themes: Vec<Named>,
    pub modes: Vec<Named>,
    pub perspectives: Vec<Named>,
    pub developers: Vec<Named>,
    pub publishers: Vec<Named>,
    pub screenshots: Vec<String>,
    pub artworks: Vec<String>,
    pub trailers: Vec<Trailer>,
    pub languages: Vec<Language>,
    pub age_ratings: BTreeMap<String, String>,
    pub alt_titles: Vec<String>,
    pub external_ids: BTreeMap<String, String>,
    /// Bu oyunun ek paketleri, yeniden yapımları ve portları.
    pub related: Vec<GameRef>,
    /// Aynı serinin diğer oyunları, çıkış sırasıyla.
    pub series_games: Vec<GameRef>,
    /// En çok tür ve tema paylaşan oyunlar.
    pub similar: Vec<GameRef>,
}

/// (tür, görsel adresi, sağlayıcı, video kimliği)
type MediaRow = (String, Option<String>, Option<String>, Option<String>);

#[derive(FromRow)]
struct GameRow {
    id: Uuid,
    slug: String,
    title: String,
    kind: String,
    status: String,
    summary: Option<String>,
    description: Option<String>,
    release_date: Option<NaiveDate>,
    release_precision: String,
    cover_url: Option<String>,
    backdrop_url: Option<String>,
    logo_url: Option<String>,
    time_to_beat_minutes: Option<i32>,
    pc_requirements: Option<serde_json::Value>,
    parent_id: Option<Uuid>,
    series_id: Option<Uuid>,
}

/// Slug'a göre oyun. Eski bir slug verilirse oyun yine bulunur; dönen `slug` güncel olandır,
/// çağıran farkı görüp yönlendirir. `include_unpublished`: yetkililer taslakları da görür.
pub async fn detail(
    db: &PgPool,
    slug: &str,
    include_unpublished: bool,
) -> sqlx::Result<Option<GameDetail>> {
    let row: Option<GameRow> = sqlx::query_as(
        "SELECT g.id, g.slug::text AS slug, g.title, g.kind::text AS kind, g.status::text AS status,
                g.summary, g.description, g.release_date,
                g.release_precision::text AS release_precision, g.cover_url, g.backdrop_url,
                g.logo_url, g.time_to_beat_minutes, g.pc_requirements, g.parent_id, g.series_id
           FROM games g
          WHERE (g.slug = $1
                 OR g.id = (SELECT game_id FROM game_slug_history WHERE slug = $1))
            AND ($2 OR g.status = 'published')",
    )
    .bind(slug)
    .bind(include_unpublished)
    .fetch_optional(db)
    .await?;
    let Some(game) = row else { return Ok(None) };

    let parent: Option<GameRef> = match game.parent_id {
        Some(parent_id) => {
            sqlx::query_as(
                "SELECT slug::text AS slug, title, kind::text AS kind, cover_url
                   FROM games WHERE id = $1 AND status = 'published'",
            )
            .bind(parent_id)
            .fetch_optional(db)
            .await?
        }
        None => None,
    };
    let series: Option<Named> = match game.series_id {
        Some(series_id) => {
            sqlx::query_as("SELECT slug::text AS slug, name FROM series WHERE id = $1")
                .bind(series_id)
                .fetch_optional(db)
                .await?
        }
        None => None,
    };

    let releases: Vec<(String, String, String, String, Option<NaiveDate>, String)> =
        sqlx::query_as(
            "SELECT p.slug::text, p.name, p.family, r.region, r.release_date,
                    r.release_precision::text
               FROM game_releases r JOIN platforms p ON p.id = r.platform_id
              WHERE r.game_id = $1
              ORDER BY r.release_date NULLS LAST, p.position, r.region",
        )
        .bind(game.id)
        .fetch_all(db)
        .await?;

    let tags: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT t.kind::text, t.slug::text, t.name
           FROM game_tags gt JOIN tags t ON t.id = gt.tag_id
          WHERE gt.game_id = $1
          ORDER BY t.name",
    )
    .bind(game.id)
    .fetch_all(db)
    .await?;
    let tagged = |kind: &str| -> Vec<Named> {
        tags.iter()
            .filter(|(tag_kind, _, _)| tag_kind == kind)
            .map(|(_, slug, name)| Named {
                slug: slug.clone(),
                name: name.clone(),
            })
            .collect()
    };

    let companies: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT gc.role::text, c.slug::text, c.name
           FROM game_companies gc JOIN companies c ON c.id = gc.company_id
          WHERE gc.game_id = $1
          ORDER BY c.name",
    )
    .bind(game.id)
    .fetch_all(db)
    .await?;
    let in_role = |role: &str| -> Vec<Named> {
        companies
            .iter()
            .filter(|(company_role, _, _)| company_role == role)
            .map(|(_, slug, name)| Named {
                slug: slug.clone(),
                name: name.clone(),
            })
            .collect()
    };

    let media: Vec<MediaRow> = sqlx::query_as(
        "SELECT kind::text, url, provider, video_id
           FROM game_media WHERE game_id = $1
          ORDER BY kind, position",
    )
    .bind(game.id)
    .fetch_all(db)
    .await?;
    let images = |kind: &str| -> Vec<String> {
        media
            .iter()
            .filter(|(media_kind, ..)| media_kind == kind)
            .filter_map(|(_, url, ..)| url.clone())
            .collect()
    };
    let trailers = media
        .iter()
        .filter_map(|(_, _, provider, video_id)| {
            Some(Trailer {
                provider: provider.clone()?,
                video_id: video_id.clone()?,
            })
        })
        .collect();

    let languages: Vec<Language> = sqlx::query_as(
        "SELECT language, interface, audio, subtitles
           FROM game_languages WHERE game_id = $1
          ORDER BY language",
    )
    .bind(game.id)
    .fetch_all(db)
    .await?;

    let age_ratings: Vec<(String, String)> =
        sqlx::query_as("SELECT board, rating FROM game_age_ratings WHERE game_id = $1")
            .bind(game.id)
            .fetch_all(db)
            .await?;
    let alt_titles: Vec<String> =
        sqlx::query_scalar("SELECT title FROM game_alt_titles WHERE game_id = $1 ORDER BY title")
            .bind(game.id)
            .fetch_all(db)
            .await?;
    let external_ids: Vec<(String, String)> =
        sqlx::query_as("SELECT source, external_id FROM game_external_ids WHERE game_id = $1")
            .bind(game.id)
            .fetch_all(db)
            .await?;

    let related: Vec<GameRef> = sqlx::query_as(
        "SELECT slug::text AS slug, title, kind::text AS kind, cover_url
           FROM games
          WHERE parent_id = $1 AND status = 'published'
          ORDER BY release_date NULLS LAST, title",
    )
    .bind(game.id)
    .fetch_all(db)
    .await?;

    let series_games: Vec<GameRef> = match game.series_id {
        Some(series_id) => {
            sqlx::query_as(
                "SELECT slug::text AS slug, title, kind::text AS kind, cover_url
                   FROM games
                  WHERE series_id = $1 AND id <> $2 AND status = 'published' AND kind = 'game'
                  ORDER BY release_date NULLS LAST, title
                  LIMIT 24",
            )
            .bind(series_id)
            .bind(game.id)
            .fetch_all(db)
            .await?
        }
        None => Vec::new(),
    };

    // Ortak tür ve tema sayısına göre; eşitlikte daha yeni olan öne gelir.
    let similar: Vec<GameRef> = sqlx::query_as(
        "SELECT o.slug::text AS slug, o.title, o.kind::text AS kind, o.cover_url
           FROM game_tags mine
           JOIN tags t ON t.id = mine.tag_id AND t.kind IN ('genre', 'theme')
           JOIN game_tags theirs ON theirs.tag_id = mine.tag_id AND theirs.game_id <> mine.game_id
           JOIN games o ON o.id = theirs.game_id
          WHERE mine.game_id = $1
            AND o.status = 'published' AND o.kind = 'game'
            AND o.parent_id IS DISTINCT FROM $1
          GROUP BY o.id
          ORDER BY count(*) DESC, o.release_date DESC NULLS LAST, o.id
          LIMIT 12",
    )
    .bind(game.id)
    .fetch_all(db)
    .await?;

    Ok(Some(GameDetail {
        genres: tagged("genre"),
        themes: tagged("theme"),
        modes: tagged("mode"),
        perspectives: tagged("perspective"),
        developers: in_role("developer"),
        publishers: in_role("publisher"),
        screenshots: images("screenshot"),
        artworks: images("artwork"),
        trailers,
        releases: releases
            .into_iter()
            .map(
                |(slug, name, family, region, release_date, release_precision)| Release {
                    platform: Platform { slug, name, family },
                    region,
                    release_date,
                    release_precision,
                },
            )
            .collect(),
        id: game.id,
        slug: game.slug,
        title: game.title,
        kind: game.kind,
        status: game.status,
        summary: game.summary,
        description: game.description,
        release_date: game.release_date,
        release_precision: game.release_precision,
        cover_url: game.cover_url,
        backdrop_url: game.backdrop_url,
        logo_url: game.logo_url,
        time_to_beat_minutes: game.time_to_beat_minutes,
        pc_requirements: game.pc_requirements,
        parent,
        series,
        languages,
        age_ratings: age_ratings.into_iter().collect(),
        alt_titles,
        external_ids: external_ids.into_iter().collect(),
        related,
        series_games,
        similar,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_wildcards_are_escaped() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("back\\slash"), "back\\\\slash");
        assert_eq!(escape_like("plain"), "plain");
    }

    #[test]
    fn sort_parses_known_values_only() {
        assert_eq!(Sort::parse("title"), Some(Sort::Title));
        assert_eq!(Sort::parse("added"), Some(Sort::Added));
        assert_eq!(Sort::parse("newest"), Some(Sort::Newest));
        assert_eq!(Sort::parse("soonest"), Some(Sort::Soonest));
        assert_eq!(Sort::parse("popular"), None);
        assert_eq!(Window::parse("upcoming"), Some(Window::Upcoming));
        assert_eq!(Window::parse("someday"), None);
    }
}
