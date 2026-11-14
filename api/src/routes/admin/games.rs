//! Katalog yönetimi: toplu içe aktarma, yayın durumu ve taslak önizleme.

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, FromRequestParts, Path, Query, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    routing::{get, patch, post},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{
    auth::role::{RequireAdmin, RequireModerator},
    catalog::{
        import::{GameInput, ImportError, create_game, import_game, update_game_by_id},
        read::{self, GameDetail},
    },
    error::AppError,
    routes::guard::{TEN_MINUTES, rate_limit},
    state::AppState,
};

/// Tek istekte en fazla bu kadar oyun; daha büyük yüklemeler parçalara bölünür.
const MAX_BATCH: usize = 500;
/// Açıklamalar ve görsel listeleriyle 500 oyun birkaç megabayt tutabilir.
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/games/import",
            post(import).layer(DefaultBodyLimit::max(MAX_BODY_BYTES)),
        )
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
    let status = match query.status.as_deref() {
        None | Some("") | Some("all") => None,
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
                g.release_precision::text AS release_precision, g.cover_url, g.updated_at,
                count(*) OVER () AS total
           FROM games g
          WHERE ($1::text IS NULL OR g.status = $1::publish_status)
            AND ($2::text IS NULL OR g.title ILIKE $2 OR g.slug::text ILIKE $2)
          ORDER BY g.updated_at DESC, g.id
          LIMIT $3 OFFSET $4",
    )
    .bind(status)
    .bind(pattern)
    .bind(limit)
    .bind(offset)
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

/// İçe aktarmayı kimin yaptığı: admin oturumu ya da betiğin makine anahtarı.
/// Anahtar yalnızca bu uçta geçer; başka hiçbir yönetim ucunu açmaz.
struct Importer(String);

impl FromRequestParts<AppState> for Importer {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let bearer = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));
        if let (Some(given), Some(expected)) = (bearer, state.catalog_import_token.as_deref())
            && bool::from(given.as_bytes().ct_eq(expected.as_bytes()))
        {
            return Ok(Self("import-token".into()));
        }
        let RequireAdmin(admin) = RequireAdmin::from_request_parts(parts, state).await?;
        Ok(Self(admin.user_id.to_string()))
    }
}

/// Oyunlar burada ham JSON olarak alınır ve tek tek çözülür: biçimi bozuk bir kayıt (örn. geçersiz
/// tarih) yalnızca kendisini düşürür, gruptaki diğer oyunları değil.
#[derive(Deserialize)]
struct ImportBody {
    games: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct ImportReport {
    created: usize,
    updated: usize,
    failed: usize,
    results: Vec<ImportResult>,
}

#[derive(Serialize)]
struct ImportResult {
    /// Gönderilen listedeki sıra.
    index: usize,
    title: String,
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'static str>,
    /// Biçim hatasında hangi alanın neden reddedildiği.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

/// Oyunları ekler ya da günceller. Her oyun kendi transaction'ında işlenir: biri hatalıysa
/// diğerleri yine yazılır ve rapor hangisinin neden reddedildiğini söyler.
async fn import(
    State(state): State<AppState>,
    importer: Importer,
    Json(body): Json<ImportBody>,
) -> Result<Json<ImportReport>, AppError> {
    if body.games.is_empty() || body.games.len() > MAX_BATCH {
        return Err(AppError::Validation("invalid_batch_size"));
    }

    let mut report = ImportReport {
        created: 0,
        updated: 0,
        failed: 0,
        results: Vec::with_capacity(body.games.len()),
    };
    for (index, raw) in body.games.into_iter().enumerate() {
        let title = raw
            .get("title")
            .and_then(|title| title.as_str())
            .unwrap_or_default()
            .chars()
            .take(300)
            .collect::<String>();
        let game: GameInput = match serde_json::from_value(raw) {
            Ok(game) => game,
            Err(err) => {
                report.failed += 1;
                report.results.push(ImportResult {
                    index,
                    title,
                    outcome: "failed",
                    id: None,
                    slug: None,
                    error: Some("invalid_format"),
                    detail: Some(err.to_string()),
                });
                continue;
            }
        };
        let mut tx = state.db.begin().await?;
        let result = match import_game(&mut tx, game).await {
            Ok(imported) => {
                tx.commit().await?;
                if imported.created {
                    report.created += 1;
                } else {
                    report.updated += 1;
                }
                ImportResult {
                    index,
                    title,
                    outcome: if imported.created {
                        "created"
                    } else {
                        "updated"
                    },
                    id: Some(imported.id),
                    slug: Some(imported.slug),
                    error: None,
                    detail: None,
                }
            }
            Err(err) => {
                // Transaction bırakılınca geri alınır; bu oyundan hiçbir şey yazılmaz.
                drop(tx);
                report.failed += 1;
                let code = match err {
                    ImportError::Invalid(code) => code,
                    ImportError::Db(err) => {
                        tracing::error!(error = ?err, index, title, "oyun içe aktarılamadı");
                        "internal"
                    }
                };
                ImportResult {
                    index,
                    title,
                    outcome: "failed",
                    id: None,
                    slug: None,
                    error: Some(code),
                    detail: None,
                }
            }
        };
        report.results.push(result);
    }

    tracing::info!(
        actor = %importer.0,
        created = report.created,
        updated = report.updated,
        failed = report.failed,
        "katalog içe aktarma"
    );
    Ok(Json(report))
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
