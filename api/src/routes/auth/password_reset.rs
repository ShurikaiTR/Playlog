//! Şifre sıfırlama: adrese kod gider; doğru kodla yeni şifre kaydedilir ve tüm oturumlar kapanır.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use uuid::Uuid;

use super::CodeSent;
use crate::{
    auth::{
        email_code::{self, Purpose},
        password, validate,
    },
    error::AppError,
    routes::guard::{TEN_MINUTES, client_ip, rate_limit, verify_captcha},
    state::AppState,
};

#[derive(Deserialize)]
pub(super) struct PasswordResetStart {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    email: String,
}

#[derive(Deserialize)]
pub(super) struct PasswordResetConfirm {
    email: String,
    code: String,
    new_password: String,
}

pub(super) async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PasswordResetStart>,
) -> Result<(StatusCode, Json<CodeSent>), AppError> {
    rate_limit(
        &state,
        &format!("reset:ip:{}", client_ip(&headers)),
        10,
        TEN_MINUTES,
    )?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    let email = validate::email(&body.email)?;
    email_code::ensure_cooldown(&state, Purpose::PasswordReset, &email).await?;

    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
        .bind(&email)
        .fetch_one(&state.db)
        .await?;
    // Hesap yoksa da aynı yanıt dönülür (hesap sayımı sızmasın).
    if exists {
        email_code::issue(&state, Purpose::PasswordReset, &email, None).await?;
    }
    Ok((StatusCode::ACCEPTED, CodeSent::new()))
}

pub(super) async fn confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PasswordResetConfirm>,
) -> Result<StatusCode, AppError> {
    rate_limit(
        &state,
        &format!("verify:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let email = validate::email(&body.email)?;
    let code_input = validate::code(&body.code)?;
    validate::password(&body.new_password)?;

    let new_hash = password::hash(body.new_password).await?;

    let mut tx = state.db.begin().await?;
    if email_code::consume(&state, &mut tx, Purpose::PasswordReset, &email, code_input)
        .await?
        .is_none()
    {
        tx.commit().await?;
        return Err(AppError::Unauthorized("invalid_code"));
    }

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::Unauthorized("invalid_code"))?;

    sqlx::query(
        "INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)
         ON CONFLICT (user_id)
         DO UPDATE SET password_hash = EXCLUDED.password_hash, updated_at = now()",
    )
    .bind(user_id)
    .bind(new_hash)
    .execute(&mut *tx)
    .await?;

    // Parola değişince tüm oturumlar kapanır.
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
