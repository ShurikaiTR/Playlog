//! Katalog yönetimi: oyun listesi, ekleme, düzenleme, yayın durumu ve taslak önizleme.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::role::{RequireAdmin, RequireModerator},
    catalog::{
        import::{GameInput, ImportError, create_game, update_game_by_id},
        read::{self, GameDetail},
    },
    error::AppError,
    routes::guard::{TEN_MINUTES, rate_limit},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/games", get(list).post(create))
        .route("/games/{key}", get(preview).put(update).delete(remove))
        .route("/games/{key}/status", patch(set_status))
        .route("/platforms", get(platforms))
}

/// Yazma uçları yönetici başına sınırlanır; ele geçirilmiş bir oturum kataloğu toplu bozamasın.
fn write_limit(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    rate_limit(state, &format!("admin:write:{user_id}"), 120, TEN_MINUTES)
}

fn read_limit(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    rate_limit(state, &format!("admin:read:{user_id}"), 600, TEN_MINUTES)
}

/// Girdi hatasını istemci koduna çevirir; slug çakışması 409 olur.
fn import_error(err: ImportError) -> AppError {
    match err {
        ImportError::Invalid("slug_taken") => AppError::Conflict("slug_taken"),
        ImportError::Invalid(code) => AppError::Validation(code),
        ImportError::Db(err) => AppError::from(err),
    }
}

const MAX_PAGE: i64 = 100;

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize, FromRow)]
struct AdminGameRow {
    id: Uuid,
    slug: String,
    title: String,
    kind: String,
    status: String,
    release_date: Option<NaiveDate>,
    release_precision: String,
    cover_url: Option<String>,
    /// İçe aktarma bu kaydı elle bakılacak diye işaretlediyse nedeni.
    review_reason: Option<String>,
    updated_at: DateTime<Utc>,
    #[serde(skip)]
    total: i64,
}

#[derive(Serialize)]
struct AdminGamePage {
    total: i64,
    items: Vec<AdminGameRow>,
}

/// Yayında olmayanlar dahil tüm oyunlar; başlığa göre aranabilir, durumuna göre süzülebilir.
async fn list(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Query(query): Query<ListQuery>,
) -> Result<Json<AdminGamePage>, AppError> {
    read_limit(&state, actor.user_id)?;
    // "review": içe aktarmanın elle bakılacak diye işaretlediği oyunlar.
    let review_only = query.status.as_deref() == Some("review");
    let status = match query.status.as_deref() {
        None | Some("") | Some("all") | Some("review") => None,
        Some(status @ ("draft" | "published" | "hidden")) => Some(status),
        Some(_) => return Err(AppError::Validation("invalid_status")),
    };
    let text = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty() && q.chars().count() <= 100);
    let pattern = text.map(|q| {
        let escaped = q
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        format!("%{escaped}%")
    });
    let limit = query.limit.unwrap_or(25).clamp(1, MAX_PAGE);
    let offset = query.offset.unwrap_or(0).max(0);

    let items: Vec<AdminGameRow> = sqlx::query_as(
        "SELECT g.id, g.slug::text AS slug, g.title, g.kind::text AS kind,
                g.status::text AS status, g.release_date,
                g.release_precision::text AS release_precision, g.cover_url, g.review_reason, g.updated_at,
                count(*) OVER () AS total
           FROM games g
          WHERE ($1::text IS NULL OR g.status = $1::publish_status)
            AND (NOT $5 OR g.review_reason IS NOT NULL)
            AND ($2::text IS NULL OR g.title ILIKE $2 OR g.slug::text ILIKE $2)
          ORDER BY g.updated_at DESC, g.id
          LIMIT $3 OFFSET $4",
    )
    .bind(status)
    .bind(pattern)
    .bind(limit)
    .bind(offset)
    .bind(review_only)
    .fetch_all(&state.db)
    .await?;
    let total = items.first().map_or(0, |row| row.total);
    Ok(Json(AdminGamePage { total, items }))
}

#[derive(Serialize)]
struct Saved {
    id: Uuid,
    slug: String,
}

async fn create(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Json(input): Json<GameInput>,
) -> Result<(StatusCode, Json<Saved>), AppError> {
    write_limit(&state, actor.user_id)?;
    let mut tx = state.db.begin().await?;
    let saved = create_game(&mut tx, input).await.map_err(import_error)?;
    tx.commit().await?;
    tracing::info!(actor = %actor.user_id, game = %saved.id, "oyun oluşturuldu");
    Ok((
        StatusCode::CREATED,
        Json(Saved {
            id: saved.id,
            slug: saved.slug,
        }),
    ))
}

/// Oyunun tüm alanlarını ve ilişkilerini gönderilenle değiştirir (PUT).
async fn update(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Path(id): Path<Uuid>,
    Json(input): Json<GameInput>,
) -> Result<Json<Saved>, AppError> {
    write_limit(&state, actor.user_id)?;
    let mut tx = state.db.begin().await?;
    let saved = update_game_by_id(&mut tx, id, input)
        .await
        .map_err(import_error)?
        .ok_or(AppError::NotFound("game_not_found"))?;
    tx.commit().await?;
    tracing::info!(actor = %actor.user_id, game = %id, "oyun düzenlendi");
    Ok(Json(Saved {
        id: saved.id,
        slug: saved.slug,
    }))
}

/// Oyunu ve ona bağlı her şeyi (kütüphane kayıtları, puanlar, incelemeler) siler; yalnızca admin.
async fn remove(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, actor.user_id)?;
    let deleted = sqlx::query("DELETE FROM games WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound("game_not_found"));
    }
    tracing::info!(actor = %actor.user_id, game = %id, "oyun silindi");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, FromRow)]
struct PlatformOption {
    slug: String,
    name: String,
    family: String,
}

async fn platforms(
    State(state): State<AppState>,
    _: RequireModerator,
) -> Result<Json<Vec<PlatformOption>>, AppError> {
    let rows = sqlx::query_as(
        "SELECT slug::text AS slug, name, family FROM platforms ORDER BY position, name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

/// Yayında olmayanlar dahil oyunun tamamı; yayına almadan önce gözden geçirmek için.
async fn preview(
    State(state): State<AppState>,
    _: RequireModerator,
    Path(slug): Path<String>,
) -> Result<Json<GameDetail>, AppError> {
    read::detail(&state.db, &slug, true)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound("game_not_found"))
}

#[derive(Deserialize)]
struct StatusChange {
    status: String,
}

#[derive(Serialize)]
struct StatusView {
    id: Uuid,
    slug: String,
    status: String,
}

async fn set_status(
    State(state): State<AppState>,
    RequireModerator(actor): RequireModerator,
    Path(id): Path<Uuid>,
    Json(body): Json<StatusChange>,
) -> Result<Json<StatusView>, AppError> {
    write_limit(&state, actor.user_id)?;
    if !matches!(body.status.as_str(), "draft" | "published" | "hidden") {
        return Err(AppError::Validation("invalid_status"));
    }
    let updated: Option<(String,)> = sqlx::query_as(
        "UPDATE games SET status = $2::publish_status, updated_at = now()
          WHERE id = $1
          RETURNING slug::text",
    )
    .bind(id)
    .bind(&body.status)
    .fetch_optional(&state.db)
    .await?;
    let Some((slug,)) = updated else {
        return Err(AppError::NotFound("game_not_found"));
    };
    tracing::info!(actor = %actor.user_id, game = %id, status = body.status, "oyun durumu değişti");
    Ok(Json(StatusView {
        id,
        slug,
        status: body.status,
    }))
}
