//! Kişisel bilgiler: kullanıcı adı ve görünen ad.

use std::time::Duration;

use axum::{Json, extract::State};
use serde::Deserialize;

use crate::{
    auth::{extractor::CurrentUser, validate},
    error::{AppError, map_unique_violation},
    routes::{auth::UserView, guard::rate_limit},
    state::AppState,
};

#[derive(Deserialize)]
pub(super) struct ProfileUpdate {
    username: String,
    display_name: Option<String>,
}

pub(super) async fn update(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<ProfileUpdate>,
) -> Result<Json<UserView>, AppError> {
    let username = validate::username(&body.username)?;
    let display_name = validate::display_name(body.display_name.as_deref())?;
    if username != user.username {
        rate_limit(
            &state,
            &format!("account:username:{}", user.user_id),
            5,
            Duration::from_secs(60 * 60 * 24),
        )?;
    }

    sqlx::query(
        "UPDATE users SET username = $2, display_name = $3, updated_at = now() WHERE id = $1",
    )
    .bind(user.user_id)
    .bind(&username)
    .bind(&display_name)
    .execute(&state.db)
    .await
    .map_err(map_unique_violation)?;

    Ok(Json(UserView {
        id: user.user_id,
        username,
        display_name,
        email: user.email,
        role: user.role,
    }))
}
