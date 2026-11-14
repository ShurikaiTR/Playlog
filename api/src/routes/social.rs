//! Sosyal uçlar: kullanıcı listeleri, puanlar ve incelemeler.
//!
//! Oyun başına toplamlar herkese açıktır; yazma uçları oturum ister ve yalnızca kendi kayıtlarına dokunur.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::guard::{TEN_MINUTES, rate_limit};
use crate::{
    auth::extractor::{CurrentUser, MaybeUser},
    error::AppError,
    state::AppState,
};

const MAX_LISTS: i64 = 100;
const MAX_LIST_GAMES: i64 = 5_000;
const MAX_REVIEWS: i64 = 50;

/// `/v1/games/{slug}/...` altına eklenir.
pub fn game_router() -> Router<AppState> {
    Router::new()
        .route("/{slug}/social", get(game_social))
        .route("/{slug}/rating", put(put_rating).delete(delete_rating))
        .route("/{slug}/review", put(put_review).delete(delete_review))
}

/// `/v1/me/lists`
pub fn me_router() -> Router<AppState> {
    Router::new()
        .route("/lists", get(my_lists).post(create_list))
        .route(
            "/lists/{id}",
            axum::routing::patch(update_list).delete(delete_list),
        )
        .route(
            "/lists/{id}/games/{slug}",
            put(add_to_list).delete(remove_from_list),
        )
}

/// Yazma uçlarında kullanıcı başına sınır; kötüye kullanımı ve döngüye girmiş istemcileri keser.
fn write_limit(state: &AppState, user: &CurrentUser) -> Result<(), AppError> {
    rate_limit(state, &format!("social:{}", user.user_id), 120, TEN_MINUTES)
}

async fn game_id(db: &PgPool, slug: &str) -> Result<Uuid, AppError> {
    sqlx::query_scalar("SELECT id FROM games WHERE slug = $1 AND status = 'published'")
        .bind(slug)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::NotFound("game_not_found"))
}

// ---------------------------------------------------------------------------------------------
// Oyun sayfası toplamları
// ---------------------------------------------------------------------------------------------

#[derive(FromRow)]
struct Totals {
    played: i64,
    play_later: i64,
    rating_avg: Option<f64>,
    rating_count: i64,
    review_count: i64,
}

#[derive(Serialize, FromRow)]
struct Person {
    username: String,
    name: String,
    avatar_url: Option<String>,
}

#[derive(Serialize, FromRow)]
struct Review {
    username: String,
    name: String,
    avatar_url: Option<String>,
    text: String,
    spoiler: bool,
    score: Option<i16>,
    created_at: DateTime<Utc>,
    mine: bool,
}

#[derive(Serialize)]
struct Mine {
    rating: Option<i16>,
    review: Option<MyReview>,
    /// Oyunun içinde bulunduğu listelerim.
    lists: Vec<Uuid>,
}

#[derive(Serialize)]
struct MyReview {
    text: String,
    spoiler: bool,
}

#[derive(Serialize)]
struct GameSocial {
    played: i64,
    play_later: i64,
    rating_avg: Option<f64>,
    rating_count: i64,
    review_count: i64,
    /// Oynadı listesini herkese açık tutan son birkaç oyuncu.
    players: Vec<Person>,
    reviews: Vec<Review>,
    mine: Option<Mine>,
}

async fn game_social(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(slug): Path<String>,
) -> Result<Json<GameSocial>, AppError> {
    let db = &state.db;
    let game = game_id(db, &slug).await?;
    let viewer_id = viewer.as_ref().map(|user| user.user_id);

    let totals: Totals = sqlx::query_as(
        "SELECT
            (SELECT count(*) FROM user_list_games lg JOIN user_lists l ON l.id = lg.list_id
              WHERE lg.game_id = $1 AND l.kind = 'played') AS played,
            (SELECT count(*) FROM user_list_games lg JOIN user_lists l ON l.id = lg.list_id
              WHERE lg.game_id = $1 AND l.kind = 'play_later') AS play_later,
            (SELECT avg(score)::float8 FROM game_ratings WHERE game_id = $1) AS rating_avg,
            (SELECT count(*) FROM game_ratings WHERE game_id = $1) AS rating_count,
            (SELECT count(*) FROM game_reviews WHERE game_id = $1) AS review_count",
    )
    .bind(game)
    .fetch_one(db)
    .await?;

    let players: Vec<Person> = sqlx::query_as(
        "SELECT u.username, COALESCE(u.display_name, u.username) AS name, u.avatar_url
           FROM user_list_games lg
           JOIN user_lists l ON l.id = lg.list_id AND l.kind = 'played' AND l.visibility = 'public'
           JOIN users u ON u.id = l.user_id
          WHERE lg.game_id = $1
          ORDER BY lg.added_at DESC
          LIMIT 4",
    )
    .bind(game)
    .fetch_all(db)
    .await?;

    let reviews: Vec<Review> = sqlx::query_as(
        "SELECT u.username, COALESCE(u.display_name, u.username) AS name, u.avatar_url,
                r.body AS text, r.spoiler, gr.score, r.created_at,
                (r.user_id IS NOT DISTINCT FROM $2) AS mine
           FROM game_reviews r
           JOIN users u ON u.id = r.user_id
           LEFT JOIN game_ratings gr ON gr.user_id = r.user_id AND gr.game_id = r.game_id
          WHERE r.game_id = $1
          ORDER BY (r.user_id IS NOT DISTINCT FROM $2) DESC, r.created_at DESC
          LIMIT $3",
    )
    .bind(game)
    .bind(viewer_id)
    .bind(MAX_REVIEWS)
    .fetch_all(db)
    .await?;

    let mine = match viewer_id {
        None => None,
        Some(user_id) => {
            let rating: Option<i16> = sqlx::query_scalar(
                "SELECT score FROM game_ratings WHERE user_id = $1 AND game_id = $2",
            )
            .bind(user_id)
            .bind(game)
            .fetch_optional(db)
            .await?;
            let review: Option<(String, bool)> = sqlx::query_as(
                "SELECT body, spoiler FROM game_reviews WHERE user_id = $1 AND game_id = $2",
            )
            .bind(user_id)
            .bind(game)
            .fetch_optional(db)
            .await?;
            let lists: Vec<Uuid> = sqlx::query_scalar(
                "SELECT l.id FROM user_list_games lg JOIN user_lists l ON l.id = lg.list_id
                  WHERE l.user_id = $1 AND lg.game_id = $2",
            )
            .bind(user_id)
            .bind(game)
            .fetch_all(db)
            .await?;
            Some(Mine {
                rating,
                review: review.map(|(text, spoiler)| MyReview { text, spoiler }),
                lists,
            })
        }
    };

    Ok(Json(GameSocial {
        played: totals.played,
        play_later: totals.play_later,
        rating_avg: totals.rating_avg,
        rating_count: totals.rating_count,
        review_count: totals.review_count,
        players,
        reviews,
        mine,
    }))
}

// ---------------------------------------------------------------------------------------------
// Puan ve inceleme
// ---------------------------------------------------------------------------------------------

#[derive(Deserialize)]
struct RatingBody {
    score: i16,
}

async fn put_rating(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(slug): Path<String>,
    Json(body): Json<RatingBody>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    if !(1..=10).contains(&body.score) {
        return Err(AppError::Validation("invalid_score"));
    }
    let game = game_id(&state.db, &slug).await?;
    sqlx::query(
        "INSERT INTO game_ratings (user_id, game_id, score) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, game_id) DO UPDATE SET score = EXCLUDED.score, updated_at = now()",
    )
    .bind(user.user_id)
    .bind(game)
    .bind(body.score)
    .execute(&state.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_rating(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    let game = game_id(&state.db, &slug).await?;
    sqlx::query("DELETE FROM game_ratings WHERE user_id = $1 AND game_id = $2")
        .bind(user.user_id)
        .bind(game)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ReviewBody {
    text: String,
    #[serde(default)]
    spoiler: bool,
}

async fn put_review(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(slug): Path<String>,
    Json(body): Json<ReviewBody>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    let text = body.text.trim();
    if !(10..=5000).contains(&text.chars().count()) {
        return Err(AppError::Validation("invalid_review"));
    }
    let game = game_id(&state.db, &slug).await?;
    sqlx::query(
        "INSERT INTO game_reviews (user_id, game_id, body, spoiler) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, game_id)
         DO UPDATE SET body = EXCLUDED.body, spoiler = EXCLUDED.spoiler, updated_at = now()",
    )
    .bind(user.user_id)
    .bind(game)
    .bind(text)
    .bind(body.spoiler)
    .execute(&state.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_review(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    let game = game_id(&state.db, &slug).await?;
    sqlx::query("DELETE FROM game_reviews WHERE user_id = $1 AND game_id = $2")
        .bind(user.user_id)
        .bind(game)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------------------------
// Listeler
// ---------------------------------------------------------------------------------------------

#[derive(Serialize)]
struct ListItem {
    slug: String,
    title: String,
    cover_url: Option<String>,
    backdrop_url: Option<String>,
    release_date: Option<NaiveDate>,
    added_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct ListOut {
    id: Uuid,
    kind: String,
    name: String,
    description: String,
    visibility: String,
    /// Kapak olarak seçilen oyun; yoksa arayüz listedeki ilk oyunu gösterir.
    cover_slug: Option<String>,
    items: Vec<ListItem>,
}

#[derive(FromRow)]
struct ListRow {
    id: Uuid,
    kind: String,
    name: String,
    description: String,
    visibility: String,
    cover_slug: Option<String>,
}

#[derive(FromRow)]
struct ItemRow {
    list_id: Uuid,
    slug: String,
    title: String,
    cover_url: Option<String>,
    backdrop_url: Option<String>,
    release_date: Option<NaiveDate>,
    added_at: DateTime<Utc>,
}

/// İki sistem listesi ilk kullanımda oluşur; "Oynadım" herkese açık, "Sonra oyna" gizli başlar.
async fn ensure_system_lists(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO user_lists (user_id, kind, name, description, visibility) VALUES
            ($1, 'played', 'Played', 'The games you have played.', 'public'),
            ($1, 'play_later', 'Play later', 'The games waiting for their turn.', 'private')
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .execute(db)
    .await?;
    Ok(())
}

async fn my_lists(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<ListOut>>, AppError> {
    let db = &state.db;
    ensure_system_lists(db, user.user_id).await?;

    let rows: Vec<ListRow> = sqlx::query_as(
        "SELECT l.id, l.kind::text AS kind, l.name, l.description, l.visibility::text AS visibility,
                g.slug::text AS cover_slug
           FROM user_lists l
           LEFT JOIN games g ON g.id = l.cover_game_id AND g.status = 'published'
          WHERE l.user_id = $1
          ORDER BY CASE l.kind WHEN 'played' THEN 0 WHEN 'play_later' THEN 1 ELSE 2 END,
                   l.created_at, l.id",
    )
    .bind(user.user_id)
    .fetch_all(db)
    .await?;

    let items: Vec<ItemRow> = sqlx::query_as(
        "SELECT lg.list_id, g.slug::text AS slug, g.title, g.cover_url, g.backdrop_url, g.release_date,
                lg.added_at
           FROM user_list_games lg
           JOIN user_lists l ON l.id = lg.list_id
           JOIN games g ON g.id = lg.game_id AND g.status = 'published'
          WHERE l.user_id = $1
          ORDER BY lg.added_at DESC
          LIMIT $2",
    )
    .bind(user.user_id)
    .bind(MAX_LIST_GAMES)
    .fetch_all(db)
    .await?;

    let lists = rows
        .into_iter()
        .map(|row| ListOut {
            items: items
                .iter()
                .filter(|item| item.list_id == row.id)
                .map(|item| ListItem {
                    slug: item.slug.clone(),
                    title: item.title.clone(),
                    cover_url: item.cover_url.clone(),
                    backdrop_url: item.backdrop_url.clone(),
                    release_date: item.release_date,
                    added_at: item.added_at,
                })
                .collect(),
            id: row.id,
            kind: row.kind,
            name: row.name,
            description: row.description,
            visibility: row.visibility,
            cover_slug: row.cover_slug,
        })
        .collect();
    Ok(Json(lists))
}

#[derive(Deserialize)]
struct CreateList {
    name: String,
    /// Liste oluşturulurken eklenecek oyun (oyun sayfasındaki "New list" akışı).
    game_slug: Option<String>,
}

#[derive(Serialize)]
struct Created {
    id: Uuid,
}

fn clean_name(raw: &str) -> Result<String, AppError> {
    let name = raw.trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(AppError::Validation("invalid_list_name"));
    }
    Ok(name.to_owned())
}

async fn create_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<CreateList>,
) -> Result<(StatusCode, Json<Created>), AppError> {
    write_limit(&state, &user)?;
    let name = clean_name(&body.name)?;
    let game = match &body.game_slug {
        Some(slug) => Some(game_id(&state.db, slug).await?),
        None => None,
    };

    let mut tx = state.db.begin().await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM user_lists WHERE user_id = $1 AND kind = 'custom'",
    )
    .bind(user.user_id)
    .fetch_one(&mut *tx)
    .await?;
    if count >= MAX_LISTS {
        return Err(AppError::Conflict("too_many_lists"));
    }
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO user_lists (user_id, kind, name) VALUES ($1, 'custom', $2) RETURNING id",
    )
    .bind(user.user_id)
    .bind(name)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(game) = game {
        sqlx::query("INSERT INTO user_list_games (list_id, game_id) VALUES ($1, $2)")
            .bind(id)
            .bind(game)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(Created { id })))
}

#[derive(Deserialize)]
struct UpdateList {
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
    /// Kapak olarak listedeki bir oyunun adresi; boş metin varsayılana döner.
    cover_slug: Option<String>,
}

async fn update_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateList>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    let name = body.name.as_deref().map(clean_name).transpose()?;
    let description = body.description.as_deref().map(str::trim);
    if description.is_some_and(|text| text.chars().count() > 500) {
        return Err(AppError::Validation("invalid_description"));
    }
    let visibility = match body.visibility.as_deref() {
        None => None,
        Some(v @ ("private" | "public")) => Some(v),
        Some(_) => return Err(AppError::Validation("invalid_visibility")),
    };

    // Sistem listelerinin adı sabittir; yalnızca özel listeler yeniden adlandırılır.
    let result = sqlx::query(
        "UPDATE user_lists
            SET name = CASE WHEN kind = 'custom' THEN COALESCE($3, name) ELSE name END,
                description = COALESCE($4, description),
                visibility = COALESCE($5::list_visibility, visibility)
          WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.user_id)
    .bind(name)
    .bind(description)
    .bind(visibility)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("list_not_found"));
    }

    if let Some(slug) = body.cover_slug.as_deref() {
        // Kapak yalnızca listenin kendi oyunlarından seçilir.
        let cover: Option<Uuid> = if slug.is_empty() {
            None
        } else {
            Some(
                sqlx::query_scalar(
                    "SELECT g.id FROM user_list_games lg JOIN games g ON g.id = lg.game_id
                      WHERE lg.list_id = $1 AND g.slug = $2",
                )
                .bind(id)
                .bind(slug)
                .fetch_optional(&state.db)
                .await?
                .ok_or(AppError::Validation("cover_not_in_list"))?,
            )
        };
        sqlx::query("UPDATE user_lists SET cover_game_id = $2 WHERE id = $1 AND user_id = $3")
            .bind(id)
            .bind(cover)
            .bind(user.user_id)
            .execute(&state.db)
            .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    let result =
        sqlx::query("DELETE FROM user_lists WHERE id = $1 AND user_id = $2 AND kind = 'custom'")
            .bind(id)
            .bind(user.user_id)
            .execute(&state.db)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("list_not_found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn owned_list(db: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1 FROM user_lists WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .map(|_| ())
        .ok_or(AppError::NotFound("list_not_found"))
}

async fn add_to_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, slug)): Path<(Uuid, String)>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    owned_list(&state.db, id, user.user_id).await?;
    let game = game_id(&state.db, &slug).await?;
    let size: i64 = sqlx::query_scalar("SELECT count(*) FROM user_list_games WHERE list_id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    if size >= MAX_LIST_GAMES {
        return Err(AppError::Conflict("list_full"));
    }
    sqlx::query(
        "INSERT INTO user_list_games (list_id, game_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(game)
    .execute(&state.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_from_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, slug)): Path<(Uuid, String)>,
) -> Result<StatusCode, AppError> {
    write_limit(&state, &user)?;
    owned_list(&state.db, id, user.user_id).await?;
    let game = game_id(&state.db, &slug).await?;
    sqlx::query("DELETE FROM user_list_games WHERE list_id = $1 AND game_id = $2")
        .bind(id)
        .bind(game)
        .execute(&state.db)
        .await?;
    // Kapak olan oyun çıkarıldıysa kapak varsayılana döner.
    sqlx::query("UPDATE user_lists SET cover_game_id = NULL WHERE id = $1 AND cover_game_id = $2")
        .bind(id)
        .bind(game)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
