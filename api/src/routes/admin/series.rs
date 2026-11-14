//! Seri yönetimi: seri oluşturma, oyun ekleme ve önerilen oynama sırasını elle belirleme.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::role::{RequireAdmin, RequireModerator},
    catalog::slug::slugify,
    error::AppError,
    routes::guard::{TEN_MINUTES, rate_limit},
    state::AppState,
};

/// Bir seride en fazla bu kadar oyun olur (oyun sayfası da ilk 50'yi gösterir).
const MAX_MEMBERS: usize = 50;
const MAX_NAME: usize = 200;
const MAX_NOTE: usize = 200;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/series", get(list).post(create))
        .route("/series/{id}", get(detail).patch(rename).delete(remove))
        .route("/series/{id}/games", axum::routing::put(set_games))
}

fn write_limit(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    rate_limit(state, &format!("admin:write:{user_id}"), 120, TEN_MINUTES)
}

fn read_limit(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    rate_limit(state, &format!("admin:read:{user_id}"), 600, TEN_MINUTES)
}

/// Seri adı: kırpılmış, boş olmayan, slug üretebilen bir ad.
fn clean_name(raw: &str) -> Result<(String, String), AppError> {
    let name = raw.trim();
    let slug = slugify(name);
    if name.is_empty() || name.chars().count() > MAX_NAME || slug.is_empty() {
        return Err(AppError::Validation("invalid_series"));
    }
    Ok((name.to_owned(), slug))
}

fn unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &err
        && db.is_unique_violation()
    {
        return AppError::Conflict("series_taken");
    }
    AppError::from(err)
}

#[derive(Serialize, FromRow)]
struct SeriesRow {
    id: Uuid,
    slug: String,
    name: String,
    games: i64,
    /// Yönetici sıra verdiyse true; değilse oyunlar çıkış tarihiyle sıralanır.
    ordered: bool,
}

async fn list(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
) -> Result<Json<Vec<SeriesRow>>, AppError> {
    read_limit(&state, actor.user_id)?;
    let rows = sqlx::query_as(
        "SELECT s.id, s.slug::text AS slug, s.name, count(g.id) AS games,
                COALESCE(bool_or(g.series_position IS NOT NULL), false) AS ordered
           FROM series s LEFT JOIN games g ON g.series_id = s.id
          GROUP BY s.id
          ORDER BY s.name
          LIMIT 500",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct NewSeries {
    name: String,
}

#[derive(Serialize)]
struct Created {
    id: Uuid,
    slug: String,
}

async fn create(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Json(body): Json<NewSeries>,
) -> Result<(StatusCode, Json<Created>), AppError> {
    write_limit(&state, actor.user_id)?;
    let (name, slug) = clean_name(&body.name)?;
    let (id,): (Uuid,) =
        sqlx::query_as("INSERT INTO series (slug, name) VALUES ($1, $2) RETURNING id")
            .bind(&slug)
            .bind(&name)
            .fetch_one(&state.db)
            .await
            .map_err(unique_violation)?;
    tracing::info!(actor = %actor.user_id, series = %id, "seri oluşturuldu");
    Ok((StatusCode::CREATED, Json(Created { id, slug })))
}

#[derive(Serialize, FromRow)]
struct Member {
    id: Uuid,
    slug: String,
    title: String,
    kind: String,
    status: String,
    cover_url: Option<String>,
    release_date: Option<NaiveDate>,
    release_precision: String,
    note: Option<String>,
}

#[derive(Serialize)]
struct SeriesDetail {
    id: Uuid,
    slug: String,
    name: String,
    ordered: bool,
    games: Vec<Member>,
}

/// Seri ve oyunları: önerilen sıra varsa o sırayla, yoksa çıkış tarihiyle.
async fn detail(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Path(id): Path<Uuid>,
) -> Result<Json<SeriesDetail>, AppError> {
    read_limit(&state, actor.user_id)?;
    let series: Option<(Uuid, String, String)> =
        sqlx::query_as("SELECT id, slug::text, name FROM series WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let Some((id, slug, name)) = series else {
        return Err(AppError::NotFound("series_not_found"));
    };
    let games: Vec<Member> = sqlx::query_as(
        "SELECT id, slug::text AS slug, title, kind::text AS kind, status::text AS status,
                cover_url, release_date, release_precision::text AS release_precision,
                series_note AS note
           FROM games
          WHERE series_id = $1
          ORDER BY series_position NULLS LAST, release_date NULLS LAST, title
          LIMIT 500",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;
    let ordered: bool = sqlx::query_scalar(
        "SELECT COALESCE(bool_or(series_position IS NOT NULL), false) FROM games WHERE series_id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(SeriesDetail {
        id,
        slug,
        name,
        ordered,
        games,
    }))
}

async fn rename(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Path(id): Path<Uuid>,
    Json(body): Json<NewSeries>,
) -> Result<Json<Created>, AppError> {
    write_limit(&state, actor.user_id)?;
    let (name, slug) = clean_name(&body.name)?;
    let updated: Option<(Uuid,)> =
        sqlx::query_as("UPDATE series SET name = $2, slug = $3 WHERE id = $1 RETURNING id")
            .bind(id)
            .bind(&name)
            .bind(&slug)
            .fetch_optional(&state.db)
            .await
            .map_err(unique_violation)?;
    let (id,) = updated.ok_or(AppError::NotFound("series_not_found"))?;
    tracing::info!(actor = %actor.user_id, series = %id, "seri yeniden adlandırıldı");
    Ok(Json(Created { id, slug }))
}

/// Seriyi siler; oyunlar silinmez, yalnızca seriden çıkar.
async fn remove(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, actor.user_id)?;
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE games SET series_id = NULL, series_position = NULL, series_note = NULL,
                updated_at = now()
          WHERE series_id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    let deleted = sqlx::query("DELETE FROM series WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound("series_not_found"));
    }
    tx.commit().await?;
    tracing::info!(actor = %actor.user_id, series = %id, "seri silindi");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Entry {
    game_id: Uuid,
    note: Option<String>,
}

#[derive(Deserialize)]
struct SeriesGames {
    /// Gönderilen sıra önerilen oynama sırası mı? Değilse oyunlar çıkış tarihiyle gösterilir.
    ordered: bool,
    games: Vec<Entry>,
}

/// Serinin üyelerini ve sırasını gönderilenle değiştirir (PUT): listede olmayan oyun seriden çıkar,
/// başka bir seriden gelen oyun buraya taşınır. DLC ve ek paketler seriye alınmaz.
async fn set_games(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Path(id): Path<Uuid>,
    Json(body): Json<SeriesGames>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, actor.user_id)?;
    if body.games.len() > MAX_MEMBERS {
        return Err(AppError::Validation("too_many_games"));
    }
    let mut ids = Vec::with_capacity(body.games.len());
    let mut notes = Vec::with_capacity(body.games.len());
    for entry in &body.games {
        if ids.contains(&entry.game_id) {
            return Err(AppError::Validation("duplicate_game"));
        }
        let note = entry
            .note
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty());
        if note.is_some_and(|n| n.chars().count() > MAX_NOTE) {
            return Err(AppError::Validation("invalid_note"));
        }
        ids.push(entry.game_id);
        notes.push(note.map(str::to_owned));
    }

    let mut tx = state.db.begin().await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM series WHERE id = $1)")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if !exists {
        return Err(AppError::NotFound("series_not_found"));
    }
    let found: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM games
          WHERE id = ANY($1) AND kind NOT IN ('dlc', 'expansion', 'bundle')",
    )
    .bind(&ids)
    .fetch_one(&mut *tx)
    .await?;
    if found != ids.len() as i64 {
        return Err(AppError::Validation("invalid_game"));
    }

    // Listede olmayan eski üyeler seriden çıkar.
    sqlx::query(
        "UPDATE games SET series_id = NULL, series_position = NULL, series_note = NULL,
                updated_at = now()
          WHERE series_id = $1 AND NOT (id = ANY($2))",
    )
    .bind(id)
    .bind(&ids)
    .execute(&mut *tx)
    .await?;
    // Listedekiler gönderilen sırayla yerleşir; sıra verilmediyse yalnızca üyelik yazılır.
    sqlx::query(
        "UPDATE games g
            SET series_id = $1,
                series_position = CASE WHEN $4 THEN u.position::integer END,
                series_note = u.note,
                updated_at = now()
           FROM unnest($2::uuid[], $3::text[]) WITH ORDINALITY AS u(game_id, note, position)
          WHERE g.id = u.game_id",
    )
    .bind(id)
    .bind(&ids)
    .bind(&notes)
    .bind(body.ordered)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    tracing::info!(actor = %actor.user_id, series = %id, games = ids.len(), "seri oyunları kaydedildi");
    Ok(StatusCode::NO_CONTENT)
}
