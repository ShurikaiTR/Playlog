//! Şifre belirleme/değiştirme ve giriş e-postasını doğrulama kodu.

use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{
    auth::{
        email_code::{self, Purpose},
        extractor::CurrentUser,
        password, session, validate,
    },
    error::{AppError, map_unique_violation},
    routes::guard::{FIFTEEN_MINUTES, rate_limit},
    state::AppState,
};

#[derive(Deserialize)]
pub(super) struct PasswordChange {
    /// Şifresi olan hesapta zorunludur; yalnızca sosyal girişi olan hesap ilk şifresini belirlerken boştur.
    current_password: Option<String>,
    new_password: String,
    /// Şifreyle birlikte giriş e-postası da değişecekse yeni adres ve ona gelen kod.
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    code: Option<String>,
    /// Bu cihaz dışındaki oturumlar kapatılsın mı (varsayılan: evet).
    #[serde(default = "yes")]
    sign_out_others: bool,
}

fn yes() -> bool {
    true
}

pub(super) async fn set(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<PasswordChange>,
) -> Result<StatusCode, AppError> {
    rate_limit(
        &state,
        &format!("account:password:{}", user.user_id),
        5,
        FIFTEEN_MINUTES,
    )?;
    validate::password(&body.new_password)?;

    let stored: Option<String> =
        sqlx::query_scalar("SELECT password_hash FROM credentials WHERE user_id = $1")
            .bind(user.user_id)
            .fetch_optional(&state.db)
            .await?;
    if let Some(stored) = stored {
        let current = body
            .current_password
            .ok_or(AppError::Unauthorized("invalid_password"))?;
        if !password::verify(current, stored).await? {
            return Err(AppError::Unauthorized("invalid_password"));
        }
    }

    let new_email = match body.email.as_deref() {
        Some(raw) => Some(validate::email(raw)?),
        None => None,
    };
    // E-postası olmayan hesap (Steam) e-posta ile giriş açarken adresini de vermelidir.
    if new_email.is_none() && user.email.is_none() {
        return Err(AppError::Validation("invalid_email"));
    }
    let new_hash = password::hash(body.new_password).await?;
    let mut tx = state.db.begin().await?;
    if let Some(email) = new_email {
        let current: Option<String> =
            sqlx::query_scalar("SELECT email FROM users WHERE id = $1 FOR UPDATE")
                .bind(user.user_id)
                .fetch_one(&mut *tx)
                .await?;
        // Adres değişmediyse (sağlayıcıdan gelen e-posta) kod gerekmez; değiştiyse (ya da hiç yoksa)
        // sahibi olduğu kanıtlanmalı.
        if current.as_deref() != Some(email.as_str()) {
            let code_input = validate::code(body.code.as_deref().unwrap_or_default())?;
            let pending =
                email_code::consume(&state, &mut tx, Purpose::EmailChange, &email, code_input)
                    .await?;
            let valid = pending
                .flatten()
                .and_then(|p| p.get("user_id").and_then(|v| v.as_str().map(str::to_owned)))
                .is_some_and(|id| id == user.user_id.to_string());
            if !valid {
                // Yanlış kod: deneme sayacı kalıcı olsun.
                tx.commit().await?;
                return Err(AppError::Unauthorized("invalid_code"));
            }
            sqlx::query(
                "UPDATE users SET email = $2, email_verified_at = now(), updated_at = now() WHERE id = $1",
            )
            .bind(user.user_id)
            .bind(&email)
            .execute(&mut *tx)
            .await
            .map_err(map_unique_violation)?;
        }
    }
    sqlx::query(
        "INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)
         ON CONFLICT (user_id)
         DO UPDATE SET password_hash = EXCLUDED.password_hash, updated_at = now()",
    )
    .bind(user.user_id)
    .bind(new_hash)
    .execute(&mut *tx)
    .await?;
    // Önerilen ve varsayılan: şifre değişince bu cihaz dışındaki oturumlar kapanır (OWASP Session Management).
    if body.sign_out_others {
        session::delete_others(&mut tx, user.user_id, user.session_id).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub(super) struct EmailCodeRequest {
    email: String,
}

/// Giriş e-postası olarak seçilen yeni adrese doğrulama kodu yollar. Başka hesabın adresi reddedilir.
pub(super) async fn email_code(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<EmailCodeRequest>,
) -> Result<StatusCode, AppError> {
    rate_limit(
        &state,
        &format!("account:email-code:{}", user.user_id),
        5,
        FIFTEEN_MINUTES,
    )?;
    let email = validate::email(&body.email)?;
    state
        .limiter
        .hit(
            &format!("account:email-code:addr:{email}"),
            1,
            Duration::from_secs(60),
        )
        .map_err(|_| AppError::TooManyRequests("code_cooldown"))?;
    let taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1 AND id <> $2)")
            .bind(&email)
            .bind(user.user_id)
            .fetch_one(&state.db)
            .await?;
    if taken {
        return Err(AppError::Conflict("email_taken"));
    }
    let pending = serde_json::json!({ "user_id": user.user_id });
    email_code::issue(&state, Purpose::EmailChange, &email, Some(pending)).await?;
    Ok(StatusCode::NO_CONTENT)
}
