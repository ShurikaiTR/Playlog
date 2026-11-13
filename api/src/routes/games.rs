//! Katalog uçları: oyun listesi ve oyun ayrıntısı. Yalnızca yayındaki oyunlar döner.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    catalog::read::{self, DEFAULT_PAGE, GameDetail, GamePage, ListFilter, MAX_PAGE, Sort},
    error::AppError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/{slug}", get(detail))
}

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    genre: Option<String>,
    platform: Option<String>,
    year: Option<i32>,
    sort: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<GamePage>, AppError> {
    let sort = match query.sort.as_deref() {
        Some(raw) => Sort::parse(raw).ok_or(AppError::Validation("invalid_sort"))?,
        None => Sort::default(),
    };
    if query
        .year
        .is_some_and(|year| !(1950..=2100).contains(&year))
    {
        return Err(AppError::Validation("invalid_year"));
    }
    // Boş ya da aşırı uzun arama yok sayılır / reddedilir; trigram tek karakterde anlamsızdır.
    let search = query
        .q
        .map(|q| q.trim().to_owned())
        .filter(|q| !q.is_empty());
    if search.as_deref().is_some_and(|q| q.chars().count() > 100) {
        return Err(AppError::Validation("invalid_query"));
    }

    let filter = ListFilter {
        query: search,
        genre: query.genre.filter(|genre| !genre.is_empty()),
        platform: query.platform.filter(|platform| !platform.is_empty()),
        year: query.year,
        sort,
        limit: query.limit.unwrap_or(DEFAULT_PAGE).clamp(1, MAX_PAGE),
        offset: query.offset.unwrap_or(0).clamp(0, 100_000),
    };
    Ok(Json(read::list(&state.db, &filter).await?))
}

async fn detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<GameDetail>, AppError> {
    read::detail(&state.db, &slug, false)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound("game_not_found"))
}
