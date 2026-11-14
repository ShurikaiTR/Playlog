//! Katalog içe aktarma (yalnızca admin): ada ya da Steam bağlantısına göre arama, önizleme ve
//! arka planda çalışan toplu iş. Kaynaklar Wikidata ve Steam'dir; ayrıntı `catalog::source`.

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    auth::role::RequireAdmin,
    catalog::{
        job::{Params, View},
        source::{self, Composed, ListFilter, SourceError},
    },
    error::AppError,
    routes::guard::{TEN_MINUTES, rate_limit},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/import/search", get(search))
        .route("/import/preview", get(preview))
        .route(
            "/import/jobs/current",
            get(current_job).post(start_job).delete(cancel_job),
        )
}

fn limit(state: &AppState, admin: uuid::Uuid) -> Result<(), AppError> {
    rate_limit(state, &format!("admin:import:{admin}"), 120, TEN_MINUTES)
}

fn source_error(err: SourceError) -> AppError {
    match err {
        SourceError::NotFound => AppError::NotFound(err.code()),
        SourceError::NotAGame => AppError::Validation(err.code()),
        SourceError::Unavailable => AppError::Unavailable(err.code()),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
struct HitView {
    source: &'static str,
    id: String,
    name: String,
    description: Option<String>,
    image: Option<String>,
    /// Oyun katalogda zaten varsa adresi.
    existing_slug: Option<String>,
}

#[derive(Serialize)]
struct SearchResult {
    hits: Vec<HitView>,
}

async fn existing(state: &AppState, source: &str, id: &str) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar(
        "SELECT g.slug::text FROM game_external_ids e JOIN games g ON g.id = e.game_id
          WHERE e.source = $1 AND e.external_id = $2",
    )
    .bind(source)
    .bind(id)
    .fetch_optional(&state.db)
    .await?)
}

/// Ada göre arama; Steam bağlantısı ya da uygulama kimliği doğrudan tek sonuç verir.
async fn search(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResult>, AppError> {
    limit(&state, actor.user_id)?;
    let term = query.q.trim();
    if term.chars().count() < 2 || term.chars().count() > 200 {
        return Err(AppError::Validation("invalid_query"));
    }
    let found = match source::parse_steam_ref(term) {
        Some(appid) => {
            let steam = state.source.steam(appid).await.map_err(source_error)?;
            vec![source::Hit {
                source: "steam",
                id: appid.to_string(),
                name: steam.details["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                description: None,
                image: steam.details["header_image"].as_str().map(str::to_owned),
            }]
        }
        None => state.source.search(term).await,
    };
    let mut hits = Vec::with_capacity(found.len());
    for hit in found {
        let existing_slug = existing(&state, hit.source, &hit.id).await?;
        hits.push(HitView {
            source: hit.source,
            id: hit.id,
            name: hit.name,
            description: hit.description,
            image: hit.image,
            existing_slug,
        });
    }
    Ok(Json(SearchResult { hits }))
}

#[derive(Deserialize)]
struct PreviewQuery {
    steam: Option<String>,
    wikidata: Option<String>,
}

#[derive(Serialize)]
struct Preview {
    /// `GameInput` biçiminde; panel formu bununla doldurur.
    game: Value,
    review: Option<&'static str>,
    wikidata: bool,
    steam: bool,
    existing_slug: Option<String>,
}

/// Seçilen oyunun kaynaklardan derlenmiş hali; hiçbir şey yazılmaz.
async fn preview(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Query(query): Query<PreviewQuery>,
) -> Result<Json<Preview>, AppError> {
    limit(&state, actor.user_id)?;
    let source = &state.source;

    let (wiki, steam) = match (query.steam.as_deref(), query.wikidata.as_deref()) {
        (Some(reference), _) => {
            let appid = source::parse_steam_ref(reference)
                .ok_or(AppError::Validation("invalid_steam_reference"))?;
            let steam = source.steam(appid).await.map_err(source_error)?;
            let qid = source.qid_for_app(appid, source::QUICK).await;
            let wiki = match qid {
                Some(qid) => source
                    .wiki(std::slice::from_ref(&qid), source::QUICK)
                    .await
                    .remove(&qid),
                None => None,
            };
            (wiki, Some(steam))
        }
        (None, Some(qid)) => {
            let qid = qid.trim().to_owned();
            let wiki = source
                .wiki(std::slice::from_ref(&qid), source::QUICK)
                .await
                .remove(&qid)
                .ok_or(AppError::NotFound("wikidata_item_not_found"))?;
            // Steam'de olmayan ya da kalkmış oyun Wikidata verisiyle devam eder.
            let steam = match wiki.steam_app() {
                Some(appid) => match source.steam(appid).await {
                    Ok(steam) => Some(steam),
                    Err(SourceError::NotAGame) => {
                        return Err(source_error(SourceError::NotAGame));
                    }
                    Err(_) => None,
                },
                None => None,
            };
            (Some(wiki), steam)
        }
        (None, None) => return Err(AppError::Validation("invalid_query")),
    };

    let Composed { input, review } = source::compose(wiki.as_ref(), steam.as_ref(), "draft")
        .ok_or(AppError::NotFound("wikidata_item_not_found"))?;
    let existing_slug = match (&steam, &wiki) {
        (Some(steam), _) => existing(&state, "steam", &steam.appid.to_string()).await?,
        (None, Some(wiki)) => existing(&state, "wikidata", &wiki.qid).await?,
        (None, None) => None,
    };
    Ok(Json(Preview {
        game: input,
        review,
        wikidata: wiki.is_some(),
        steam: steam.is_some(),
        existing_slug,
    }))
}

#[derive(Deserialize)]
struct JobBody {
    limit: u32,
    #[serde(default)]
    offset: u32,
    platform: Option<String>,
    year_from: Option<i32>,
    year_to: Option<i32>,
    status: String,
    #[serde(default)]
    update_existing: bool,
}

async fn start_job(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Json(body): Json<JobBody>,
) -> Result<(StatusCode, Json<View>), AppError> {
    limit(&state, actor.user_id)?;
    if !(1..=500).contains(&body.limit) || body.offset > 5000 {
        return Err(AppError::Validation("invalid_batch_size"));
    }
    if !matches!(body.status.as_str(), "draft" | "published") {
        return Err(AppError::Validation("invalid_status"));
    }
    for year in [body.year_from, body.year_to].into_iter().flatten() {
        if !(1950..=2100).contains(&year) {
            return Err(AppError::Validation("invalid_year"));
        }
    }
    let platform = body
        .platform
        .filter(|slug| !slug.is_empty() && slug != "all");
    if let Some(slug) = &platform {
        let known: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM platforms WHERE slug = $1)")
                .bind(slug)
                .fetch_one(&state.db)
                .await?;
        if !known {
            return Err(AppError::Validation("unknown_platform"));
        }
    }
    let params = Params {
        limit: body.limit,
        offset: body.offset,
        filter: ListFilter {
            platform,
            year_from: body.year_from,
            year_to: body.year_to,
        },
        status: body.status,
        update_existing: body.update_existing,
    };
    if !state
        .import_job
        .start(state.db.clone(), state.source.clone(), params)
    {
        return Err(AppError::Conflict("import_running"));
    }
    tracing::info!(actor = %actor.user_id, limit = body.limit, "toplu içe aktarma başladı");
    let view = state
        .import_job
        .view()
        .ok_or(AppError::NotFound("no_import_job"))?;
    Ok((StatusCode::ACCEPTED, Json(view)))
}

async fn current_job(
    State(state): State<AppState>,
    RequireAdmin(_): RequireAdmin,
) -> Result<Json<Option<View>>, AppError> {
    Ok(Json(state.import_job.view()))
}

async fn cancel_job(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
) -> Result<StatusCode, AppError> {
    if !state.import_job.cancel() {
        return Err(AppError::NotFound("no_import_job"));
    }
    tracing::info!(actor = %actor.user_id, "toplu içe aktarma durduruldu");
    Ok(StatusCode::NO_CONTENT)
}
