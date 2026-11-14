//! Yönetim paneli özeti: kayıt sayıları.

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use sqlx::FromRow;

use crate::{auth::role::RequireModerator, error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/stats", get(stats))
}

#[derive(Serialize, FromRow)]
struct Stats {
    games: i64,
    games_published: i64,
    games_draft: i64,
    users: i64,
    reviews: i64,
    lists: i64,
}

async fn stats(
    State(state): State<AppState>,
    _: RequireModerator,
) -> Result<Json<Stats>, AppError> {
    let stats = sqlx::query_as(
        "SELECT (SELECT count(*) FROM games) AS games,
                (SELECT count(*) FROM games WHERE status = 'published') AS games_published,
                (SELECT count(*) FROM games WHERE status = 'draft') AS games_draft,
                (SELECT count(*) FROM users) AS users,
                (SELECT count(*) FROM game_reviews) AS reviews,
                (SELECT count(*) FROM user_lists) AS lists",
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(stats))
}
