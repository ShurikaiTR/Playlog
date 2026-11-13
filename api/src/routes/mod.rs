mod account;
mod admin;
mod auth;
mod games;
mod guard;
mod oauth;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/v1/auth", auth::router())
        .nest("/v1/auth/oauth", oauth::router())
        .nest("/v1/account", account::router())
        .nest("/v1/admin", admin::router())
        .nest("/v1/games", games::router())
        .nest("/v1/tags", games::tags_router())
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

/// Süreç ayakta mı ve veritabanına ulaşabiliyor mu?
async fn health(State(state): State<AppState>) -> Result<Json<Health>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!(%err, "sağlık kontrolü: veritabanına ulaşılamadı");
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    Ok(Json(Health { status: "ok" }))
}
