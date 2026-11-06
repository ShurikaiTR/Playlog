//! Google / Discord girişi. SvelteKit yönlendirmeleri yapar, bu uçlar yalnızca iç ağdan çağrılır.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::post,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::auth::{
    SessionResponse, TEN_MINUTES, UserView, client_ip, create_session, map_unique_violation,
    rate_limit,
};
use crate::{
    auth::validate,
    error::AppError,
    oauth::{PendingClaims, Provider, now},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/complete", post(complete))
        .route("/{provider}/start", post(start))
        .route("/{provider}/callback", post(callback))
}

#[derive(Serialize)]
struct StartResponse {
    authorization_url: String,
    state_token: String,
}

#[derive(Deserialize)]
struct CallbackBody {
    code: String,
    state: String,
    state_token: String,
}

#[derive(Deserialize)]
struct CompleteBody {
    pending_token: String,
    username: String,
    #[serde(default)]
    marketing_opt_in: bool,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum CallbackResponse {
    SignedIn(SessionResponse),
    UsernameRequired {
        pending_token: String,
        email: String,
        suggested_username: String,
        display_name: Option<String>,
    },
}

fn provider_of(state: &AppState, raw: &str) -> Result<Provider, AppError> {
    Provider::parse(raw)
        .filter(|p| state.oauth.enabled(*p))
        .ok_or(AppError::Validation("unknown_provider"))
}

async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> Result<Json<StartResponse>, AppError> {
    let provider = provider_of(&state, &provider)?;
    rate_limit(
        &state,
        &format!("oauth:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let started = state.oauth.start(provider).await?;
    Ok(Json(StartResponse {
        authorization_url: started.authorization_url,
        state_token: started.state_token,
    }))
}

#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: String,
}

impl From<UserRow> for UserView {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            email: row.email,
        }
    }
}

async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(body): Json<CallbackBody>,
) -> Result<Json<CallbackResponse>, AppError> {
    let provider = provider_of(&state, &provider)?;
    rate_limit(
        &state,
        &format!("oauth:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;

    let identity = match state
        .oauth
        .finish(provider, body.code, &body.state, &body.state_token)
        .await
    {
        Ok(Some(identity)) => identity,
        Ok(None) => return Err(AppError::Unauthorized("invalid_state")),
        Err(err) => {
            tracing::warn!(error = ?err, provider = provider.as_str(), "OAuth girişi başarısız");
            return Err(AppError::Unauthorized("oauth_failed"));
        }
    };

    let mut tx = state.db.begin().await?;

    // 1) Bu sağlayıcı hesabı zaten bağlı mı?
    let linked: Option<UserRow> = sqlx::query_as(
        "SELECT u.id, u.username, u.display_name, u.email
           FROM oauth_accounts o JOIN users u ON u.id = o.user_id
          WHERE o.provider = $1::oauth_provider AND o.provider_account_id = $2",
    )
    .bind(provider.as_str())
    .bind(&identity.subject)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(user) = linked {
        let token = create_session(&mut tx, user.id, &headers).await?;
        tx.commit().await?;
        return Ok(Json(CallbackResponse::SignedIn(SessionResponse {
            token,
            user: user.into(),
        })));
    }

    // Kayıt ve bağlama yalnızca sağlayıcının doğruladığı e-postayla yapılır.
    let email = match (identity.email, identity.email_verified) {
        (Some(email), true) => validate::email(&email)?,
        _ => return Err(AppError::Conflict("email_unverified")),
    };

    // 2) Aynı e-postayla hesap varsa bağla (e-posta doğrulandığı için güvenli).
    let existing: Option<UserRow> =
        sqlx::query_as("SELECT id, username, display_name, email FROM users WHERE email = $1")
            .bind(&email)
            .fetch_optional(&mut *tx)
            .await?;
    if let Some(user) = existing {
        sqlx::query(
            "INSERT INTO oauth_accounts (user_id, provider, provider_account_id)
             VALUES ($1, $2::oauth_provider, $3)
             ON CONFLICT (user_id, provider) DO NOTHING",
        )
        .bind(user.id)
        .bind(provider.as_str())
        .bind(&identity.subject)
        .execute(&mut *tx)
        .await?;
        let token = create_session(&mut tx, user.id, &headers).await?;
        tx.commit().await?;
        return Ok(Json(CallbackResponse::SignedIn(SessionResponse {
            token,
            user: user.into(),
        })));
    }
    tx.commit().await?;

    // 3) Yeni kullanıcı: kullanıcı adını seçmesi gerekir.
    let display_name = clean_display_name(identity.display_name.as_deref());
    let suggested_username = suggest_username(&state, display_name.as_deref(), &email).await?;
    let pending_token = state.oauth.sign_pending(&PendingClaims {
        provider: provider.as_str().into(),
        subject: identity.subject,
        email: email.clone(),
        display_name: display_name.clone(),
        exp: now() + crate::oauth::OAuth::pending_ttl(),
    })?;
    Ok(Json(CallbackResponse::UsernameRequired {
        pending_token,
        email,
        suggested_username,
        display_name,
    }))
}

async fn complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CompleteBody>,
) -> Result<Json<SessionResponse>, AppError> {
    rate_limit(
        &state,
        &format!("oauth:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let pending = state
        .oauth
        .verify_pending(&body.pending_token)
        .ok_or(AppError::Unauthorized("invalid_pending"))?;
    let username = validate::username(&body.username)?;

    let mut tx = state.db.begin().await?;
    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, email, email_verified_at, display_name, marketing_opt_in_at)
         VALUES ($1, $2, now(), $3, CASE WHEN $4 THEN now() END)
         RETURNING id",
    )
    .bind(&username)
    .bind(&pending.email)
    .bind(&pending.display_name)
    .bind(body.marketing_opt_in)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_unique_violation)?;

    sqlx::query(
        "INSERT INTO oauth_accounts (user_id, provider, provider_account_id)
         VALUES ($1, $2::oauth_provider, $3)",
    )
    .bind(user_id)
    .bind(&pending.provider)
    .bind(&pending.subject)
    .execute(&mut *tx)
    .await
    .map_err(map_unique_violation)?;

    let token = create_session(&mut tx, user_id, &headers).await?;
    tx.commit().await?;

    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: user_id,
            username,
            display_name: pending.display_name,
            email: pending.email,
        },
    }))
}

fn clean_display_name(raw: Option<&str>) -> Option<String> {
    let name: String = raw?.chars().filter(|c| !c.is_control()).take(40).collect();
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// Sağlayıcıdaki ada ya da e-postanın yerel kısmına göre uygun, boşta bir kullanıcı adı önerir.
async fn suggest_username(
    state: &AppState,
    display_name: Option<&str>,
    email: &str,
) -> Result<String, AppError> {
    let source = display_name.unwrap_or_else(|| email.split('@').next().unwrap_or("oyuncu"));
    let mut base: String = source
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(16)
        .collect();
    if base.len() < 3 {
        base = format!("oyuncu{base}");
    }

    for attempt in 0..6 {
        let candidate = if attempt == 0 {
            base.clone()
        } else {
            let mut suffix = [0u8; 2];
            getrandom::fill(&mut suffix).map_err(anyhow::Error::from)?;
            format!("{base}{:04}", u16::from_le_bytes(suffix) % 10_000)
        };
        if validate::username(&candidate).is_err() {
            continue;
        }
        let taken: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)")
                .bind(&candidate)
                .fetch_one(&state.db)
                .await?;
        if !taken {
            return Ok(candidate);
        }
    }
    Ok(String::new())
}
