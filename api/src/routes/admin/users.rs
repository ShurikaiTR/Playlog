//! Kullanıcı listesi (yalnızca admin). Rol değişikliği `staff.rs` içindedir.

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::role::{RequireAdmin, Role},
    error::AppError,
    routes::guard::{TEN_MINUTES, rate_limit},
    state::AppState,
};

const MAX_PAGE: i64 = 100;

pub fn router() -> Router<AppState> {
    Router::new().route("/users", get(list))
}

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    role: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize, FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: Option<String>,
    role: Role,
    created_at: DateTime<Utc>,
    #[serde(skip)]
    total: i64,
}

#[derive(Serialize)]
struct UserPage {
    total: i64,
    items: Vec<UserRow>,
}

/// Kullanıcılar, en yeni önce. Ad, kullanıcı adı ya da e-postaya göre aranabilir.
async fn list(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Query(query): Query<ListQuery>,
) -> Result<Json<UserPage>, AppError> {
    rate_limit(
        &state,
        &format!("admin:read:{}", actor.user_id),
        600,
        TEN_MINUTES,
    )?;
    let role = match query.role.as_deref() {
        None | Some("") | Some("all") => None,
        Some(role @ ("user" | "moderator" | "admin")) => Some(role),
        Some(_) => return Err(AppError::Validation("invalid_role")),
    };
    let pattern = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty() && q.chars().count() <= 100)
        .map(|q| {
            let escaped = q
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            format!("%{escaped}%")
        });
    let limit = query.limit.unwrap_or(25).clamp(1, MAX_PAGE);
    let offset = query.offset.unwrap_or(0).max(0);

    let items: Vec<UserRow> = sqlx::query_as(
        "SELECT id, username, display_name, email, role, created_at, count(*) OVER () AS total
           FROM users
          WHERE ($1::text IS NULL OR role = $1::user_role)
            AND ($2::text IS NULL OR username ILIKE $2 OR display_name ILIKE $2 OR email ILIKE $2)
          ORDER BY created_at DESC, id
          LIMIT $3 OFFSET $4",
    )
    .bind(role)
    .bind(pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;
    let total = items.first().map_or(0, |row| row.total);
    Ok(Json(UserPage { total, items }))
}
