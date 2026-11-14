//! Yönetim paneli özeti: kayıt sayıları.

use axum::{Json, Router, extract::State, routing::get};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::FromRow;

use crate::{auth::role::RequireModerator, error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/stats", get(stats))
}

/// Gün başına yeni kayıt; grafikte gösterilir.
#[derive(Serialize, FromRow)]
struct Day {
    date: NaiveDate,
    users: i64,
    games: i64,
}

#[derive(Serialize)]
struct Report {
    #[serde(flatten)]
    totals: Stats,
    /// Son 90 gün, eskiden yeniye; kaydı olmayan günler de sıfırla yer alır.
    activity: Vec<Day>,
}

#[derive(Serialize, FromRow)]
struct Stats {
    games: i64,
    games_published: i64,
    games_draft: i64,
    users: i64,
    reviews: i64,
    lists: i64,
    /// Son 30 günde eklenenler ve ondan önceki 30 günde eklenenler; rozetteki değişim bunlardan.
    games_new: i64,
    games_prev: i64,
    users_new: i64,
    users_prev: i64,
    reviews_new: i64,
    reviews_prev: i64,
    lists_new: i64,
    lists_prev: i64,
}

async fn stats(
    State(state): State<AppState>,
    _: RequireModerator,
) -> Result<Json<Report>, AppError> {
    let totals = sqlx::query_as(
        "SELECT (SELECT count(*) FROM games) AS games,
                (SELECT count(*) FROM games WHERE status = 'published') AS games_published,
                (SELECT count(*) FROM games WHERE status = 'draft') AS games_draft,
                (SELECT count(*) FROM users) AS users,
                (SELECT count(*) FROM game_reviews) AS reviews,
                (SELECT count(*) FROM user_lists) AS lists,
                (SELECT count(*) FROM games WHERE created_at >= now() - interval '30 days') AS games_new,
                (SELECT count(*) FROM games WHERE created_at >= now() - interval '60 days'
                    AND created_at < now() - interval '30 days') AS games_prev,
                (SELECT count(*) FROM users WHERE created_at >= now() - interval '30 days') AS users_new,
                (SELECT count(*) FROM users WHERE created_at >= now() - interval '60 days'
                    AND created_at < now() - interval '30 days') AS users_prev,
                (SELECT count(*) FROM game_reviews WHERE created_at >= now() - interval '30 days') AS reviews_new,
                (SELECT count(*) FROM game_reviews WHERE created_at >= now() - interval '60 days'
                    AND created_at < now() - interval '30 days') AS reviews_prev,
                (SELECT count(*) FROM user_lists WHERE created_at >= now() - interval '30 days') AS lists_new,
                (SELECT count(*) FROM user_lists WHERE created_at >= now() - interval '60 days'
                    AND created_at < now() - interval '30 days') AS lists_prev",
    )
    .fetch_one(&state.db)
    .await?;
    let activity = sqlx::query_as(
        "WITH days AS (SELECT current_date - g AS day FROM generate_series(0, 89) AS g),
              new_users AS (SELECT created_at::date AS day, count(*) AS n FROM users
                             WHERE created_at >= current_date - 89 GROUP BY 1),
              new_games AS (SELECT created_at::date AS day, count(*) AS n FROM games
                             WHERE created_at >= current_date - 89 GROUP BY 1)
         SELECT d.day AS date, COALESCE(u.n, 0) AS users, COALESCE(g.n, 0) AS games
           FROM days d
           LEFT JOIN new_users u ON u.day = d.day
           LEFT JOIN new_games g ON g.day = d.day
          ORDER BY d.day",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(Report { totals, activity }))
}
