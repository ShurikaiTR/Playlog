//! Giriş yöntemleri: sağlayıcı hesabı bağlama ve bağlantıyı kaldırma.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::extractor::CurrentUser,
    error::AppError,
    oauth::Provider,
    routes::{
        guard::{TEN_MINUTES, rate_limit},
        oauth::provider_of,
    },
    state::AppState,
};

/// Bir sağlayıcı bağlantısını kaldırır. Hesaba girmenin son yolu kaldırılamaz.
pub(super) async fn unlink(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(provider): Path<String>,
) -> Result<StatusCode, AppError> {
    // Anahtarı kaldırılmış bir sağlayıcının eski bağlantısı da silinebilsin.
    let provider = Provider::parse(&provider).ok_or(AppError::Validation("unknown_provider"))?;

    let mut tx = state.db.begin().await?;
    // Kullanıcı satırını kilitlemek eşzamanlı kaldırma isteklerini sıraya sokar; iki istek
    // aynı anda son iki yöntemi silip hesabı girişsiz bırakamaz.
    sqlx::query("SELECT 1 FROM users WHERE id = $1 FOR UPDATE")
        .bind(user.user_id)
        .execute(&mut *tx)
        .await?;
    let methods: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM credentials WHERE user_id = $1)
              + (SELECT count(*) FROM oauth_accounts WHERE user_id = $1)",
    )
    .bind(user.user_id)
    .fetch_one(&mut *tx)
    .await?;

    let linked: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM oauth_accounts
                         WHERE user_id = $1 AND provider = $2::oauth_provider)",
    )
    .bind(user.user_id)
    .bind(provider.as_str())
    .fetch_one(&mut *tx)
    .await?;
    if !linked {
        return Err(AppError::NotFound("not_linked"));
    }
    if methods <= 1 {
        return Err(AppError::Conflict("last_sign_in_method"));
    }

    sqlx::query("DELETE FROM oauth_accounts WHERE user_id = $1 AND provider = $2::oauth_provider")
        .bind(user.user_id)
        .bind(provider.as_str())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub(super) struct LinkStart {
    authorization_url: String,
    state_token: String,
}

/// Bağlama akışını başlatır. State belirteci bu kullanıcıya bağlıdır; giriş akışında geçmez.
pub(super) async fn link_start(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(provider): Path<String>,
) -> Result<Json<LinkStart>, AppError> {
    let provider = provider_of(&state, &provider)?;
    rate_limit(
        &state,
        &format!("account:link:{}", user.user_id),
        30,
        TEN_MINUTES,
    )?;
    let started = state.oauth.start(provider, Some(user.user_id)).await?;
    Ok(Json(LinkStart {
        authorization_url: started.authorization_url,
        state_token: started.state_token,
    }))
}

#[derive(Deserialize)]
pub(super) struct LinkCallback {
    #[serde(default)]
    code: String,
    state: String,
    state_token: String,
    /// Yalnızca Steam: dönüş adresindeki `openid.*` alanları.
    #[serde(default)]
    openid: HashMap<String, String>,
}

pub(super) async fn link_callback(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(provider): Path<String>,
    Json(body): Json<LinkCallback>,
) -> Result<StatusCode, AppError> {
    let provider = provider_of(&state, &provider)?;
    rate_limit(
        &state,
        &format!("account:link:{}", user.user_id),
        30,
        TEN_MINUTES,
    )?;

    let identity = match state
        .oauth
        .finish(
            provider,
            body.code,
            &body.state,
            &body.state_token,
            &body.openid,
        )
        .await
    {
        // Akışı bu kullanıcı başlatmış olmalı; giriş akışının belirteci burada geçmez.
        Ok(Some(finished)) if finished.link_user == Some(user.user_id) => finished.identity,
        Ok(_) => return Err(AppError::Unauthorized("invalid_state")),
        Err(err) => {
            tracing::warn!(error = ?err, provider = provider.as_str(), "OAuth bağlama başarısız");
            return Err(AppError::Unauthorized("oauth_failed"));
        }
    };

    // Oturum sahibi kendisi bağladığı için sağlayıcıdaki e-postanın hesabınkiyle aynı olması gerekmez.
    let inserted = sqlx::query(
        "INSERT INTO oauth_accounts (user_id, provider, provider_account_id)
         VALUES ($1, $2::oauth_provider, $3)",
    )
    .bind(user.user_id)
    .bind(provider.as_str())
    .bind(&identity.subject)
    .execute(&state.db)
    .await;
    match inserted {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            Err(match db.constraint() {
                Some("oauth_accounts_user_id_provider_key") => {
                    AppError::Conflict("provider_already_linked")
                }
                // Bu sağlayıcı hesabı başka bir Playthru hesabına bağlı.
                _ => AppError::Conflict("provider_linked_elsewhere"),
            })
        }
        Err(err) => Err(err.into()),
    }
}
