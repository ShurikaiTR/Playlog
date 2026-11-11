//! Oturumlar (cihazlar): listeleme ve kapatma.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::{extractor::CurrentUser, session},
    error::AppError,
    state::AppState,
};

#[derive(Serialize, FromRow)]
pub(super) struct SessionView {
    id: Uuid,
    user_agent: Option<String>,
    ip: Option<String>,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    /// Boşta kalma süresiyle 90 günlük mutlak sınırdan hangisi önce gelirse.
    expires_at: DateTime<Utc>,
    current: bool,
}

pub(super) async fn list(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<SessionView>>, AppError> {
    // Kimlik doğrulayıcıyla aynı geçerlilik koşulu: süresi dolanlar listelenmez.
    let sessions = sqlx::query_as(
        "SELECT id, user_agent, host(ip) AS ip, created_at, last_seen_at,
                LEAST(expires_at, created_at + interval '90 days') AS expires_at, id = $2 AS current
           FROM sessions
          WHERE user_id = $1
            AND expires_at > now()
            AND created_at > now() - interval '90 days'
          ORDER BY id = $2 DESC, last_seen_at DESC",
    )
    .bind(user.user_id)
    .bind(user.session_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(sessions))
}

/// Tek bir oturumu kapatır; bu cihazın oturumu da olabilir (çıkışla aynı).
pub(super) async fn revoke(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound("session_not_found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn revoke_others(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db.acquire().await?;
    session::delete_others(&mut conn, user.user_id, user.session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
