//! Katalog yönetimi: toplu içe aktarma, yayın durumu ve taslak önizleme.

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, FromRequestParts, Path, State},
    http::{header::AUTHORIZATION, request::Parts},
    routing::{get, patch, post},
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{
    auth::role::{RequireAdmin, RequireModerator},
    catalog::{
        import::{GameInput, ImportError, import_game},
        read::{self, GameDetail},
    },
    error::AppError,
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
        .route("/games/{slug}", get(preview))
        .route("/games/{id}/status", patch(set_status))
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

#[derive(Deserialize)]
struct ImportBody {
    games: Vec<GameInput>,
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
    for (index, game) in body.games.into_iter().enumerate() {
        let title = game.title.clone();
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
