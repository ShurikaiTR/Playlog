//! Katalog uçları: oyun listesi, oyun ayrıntısı ve etiketler. Yalnızca yayındaki oyunlar döner.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    catalog::read::{
        self, DEFAULT_PAGE, GameDetail, GamePage, ListFilter, MAX_PAGE, Sort, TagCount, Window,
    },
    error::AppError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/{slug}", get(detail))
        .merge(super::social::game_router())
}

/// `/v1/tags`: süzgeçlerde ve tür sayfasında gösterilen etiketler.
pub fn tags_router() -> Router<AppState> {
    Router::new().route("/", get(tags))
}

/// Çok değerli süzgeçler virgülle ayrılır: `genres=rpg,action`.
#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    genres: Option<String>,
    platforms: Option<String>,
    modes: Option<String>,
    year_min: Option<i32>,
    year_max: Option<i32>,
    window: Option<String>,
    sort: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// Süzgeç başına en fazla bu kadar değer.
const MAX_VALUES: usize = 20;

fn values(raw: Option<String>) -> Result<Vec<String>, AppError> {
    let values: Vec<String> = raw
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect();
    if values.len() > MAX_VALUES || values.iter().any(|value| value.len() > 120) {
        return Err(AppError::Validation("invalid_filter"));
    }
    Ok(values)
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<GamePage>, AppError> {
    let sort = match query.sort.as_deref() {
        Some(raw) => Sort::parse(raw).ok_or(AppError::Validation("invalid_sort"))?,
        None => Sort::default(),
    };
    let window = query
        .window
        .as_deref()
        .map(|raw| Window::parse(raw).ok_or(AppError::Validation("invalid_window")))
        .transpose()?;
    let year_ok = |year: Option<i32>| year.is_none_or(|year| (1950..=2100).contains(&year));
    if !year_ok(query.year_min) || !year_ok(query.year_max) {
        return Err(AppError::Validation("invalid_year"));
    }
    // Boş arama yok sayılır; aşırı uzun olan reddedilir.
    let search = query
        .q
        .map(|q| q.trim().to_owned())
        .filter(|q| !q.is_empty());
    if search.as_deref().is_some_and(|q| q.chars().count() > 100) {
        return Err(AppError::Validation("invalid_query"));
    }

    let filter = ListFilter {
        query: search,
        genres: values(query.genres)?,
        platforms: values(query.platforms)?,
        modes: values(query.modes)?,
        year_min: query.year_min,
        year_max: query.year_max,
        window,
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

#[derive(Deserialize)]
struct TagsQuery {
    kind: Option<String>,
}

async fn tags(
    State(state): State<AppState>,
    Query(query): Query<TagsQuery>,
) -> Result<Json<Vec<TagCount>>, AppError> {
    let kind = query.kind.as_deref().unwrap_or("genre");
    if !matches!(kind, "genre" | "theme" | "mode" | "perspective") {
        return Err(AppError::Validation("invalid_kind"));
    }
    Ok(Json(read::tags(&state.db, kind).await?))
}
