//! Giriş: şifreyle ya da e-postaya gelen tek kullanımlık kodla.

use std::time::Duration;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

use super::{CodeSent, SessionResponse, UserView, VerifyCode};
use crate::{
    auth::{
        code,
        email_code::{self, Purpose},
        password::verify as verify_password,
        role::Role,
        session, validate,
    },
    error::AppError,
    routes::guard::{FIFTEEN_MINUTES, TEN_MINUTES, client_ip, rate_limit, verify_captcha},
    state::AppState,
};

// ---------- Şifreyle giriş ----------

#[derive(Deserialize)]
pub(super) struct Login {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    /// E-posta ya da kullanıcı adı.
    identifier: String,
    password: String,
}

#[derive(FromRow)]
pub(super) struct LoginRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: Option<String>,
    password_hash: Option<String>,
    role: Role,
}

pub(super) async fn password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Login>,
) -> Result<Json<SessionResponse>, AppError> {
    let identifier = body.identifier.trim().to_lowercase();
    // Hem IP hem hesap başına: tek IP'den çok hesap ve çok IP'den tek hesap denemeleri sınırlanır.
    let ip_key = format!("login:ip:{}", client_ip(&headers));
    let id_key = format!("login:id:{identifier}");
    rate_limit(&state, &ip_key, 20, FIFTEEN_MINUTES)?;
    rate_limit(&state, &id_key, 8, FIFTEEN_MINUTES)?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    // Sabit sorgular: dinamik SQL kullanılmaz, her ikisi de benzersiz indeksten çalışır.
    let query = if identifier.contains('@') {
        "SELECT u.id, u.username, u.display_name, u.email, u.role, c.password_hash
           FROM users u LEFT JOIN credentials c ON c.user_id = u.id
          WHERE u.email = $1"
    } else {
        "SELECT u.id, u.username, u.display_name, u.email, u.role, c.password_hash
           FROM users u LEFT JOIN credentials c ON c.user_id = u.id
          WHERE u.username = $1"
    };

    let row: Option<LoginRow> = sqlx::query_as(query)
        .bind(&identifier)
        .fetch_optional(&state.db)
        .await?;

    // Kullanıcı yoksa ya da parolası yoksa (yalnızca Google/Discord) da Argon2 çalışır.
    let stored = row
        .as_ref()
        .and_then(|r| r.password_hash.clone())
        .unwrap_or_else(|| state.dummy_hash.to_string());
    let password_ok = verify_password(body.password, stored).await?;

    let Some(row) = row.filter(|r| password_ok && r.password_hash.is_some()) else {
        return Err(AppError::Unauthorized("invalid_credentials"));
    };

    state.limiter.reset(&id_key);
    let mut conn = state.db.acquire().await?;
    let token = session::create(&mut conn, row.id, &headers).await?;
    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            email: row.email,
            role: row.role,
        },
    }))
}

// ---------- E-posta koduyla giriş ----------

#[derive(Deserialize)]
pub(super) struct LoginCodeStart {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    email: String,
}

/// Giriş kodunu yollar. Hesap olsun ya da olmasın yanıt (ve hız sınırı davranışı) aynıdır;
/// sınırlar veritabanına değil bellekteki sayaca bakar, böylece hesap sayımı sızmaz.
pub(super) async fn code_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginCodeStart>,
) -> Result<(StatusCode, Json<CodeSent>), AppError> {
    rate_limit(
        &state,
        &format!("logincode:ip:{}", client_ip(&headers)),
        10,
        TEN_MINUTES,
    )?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    let email = validate::email(&body.email)?;
    // Aynı adrese dakikada bir kod: istemcinin geri sayımıyla aynı süre.
    state
        .limiter
        .hit(
            &format!("logincode:email:{email}"),
            1,
            Duration::from_secs(code::RESEND_COOLDOWN_SECONDS as u64),
        )
        .map_err(|_| AppError::TooManyRequests("code_cooldown"))?;

    let eligible: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1 AND email_verified_at IS NOT NULL)",
    )
    .bind(&email)
    .fetch_one(&state.db)
    .await?;
    if eligible {
        email_code::issue(&state, Purpose::LoginCode, &email, None).await?;
    }
    Ok((StatusCode::ACCEPTED, CodeSent::new()))
}

#[derive(FromRow)]
pub(super) struct CodeLoginRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: Option<String>,
    role: Role,
}

/// Kodu doğrular (en fazla `MAX_ATTEMPTS` deneme, tek kullanımlık) ve parolalı girişle
/// aynı biçimde oturum açar.
pub(super) async fn code_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyCode>,
) -> Result<Json<SessionResponse>, AppError> {
    rate_limit(
        &state,
        &format!("verify:ip:{}", client_ip(&headers)),
        30,
        TEN_MINUTES,
    )?;
    let email = validate::email(&body.email)?;
    let code_input = validate::code(&body.code)?;
    rate_limit(
        &state,
        &format!("logincode:verify:{email}"),
        10,
        FIFTEEN_MINUTES,
    )?;

    let mut tx = state.db.begin().await?;
    if email_code::consume(&state, &mut tx, Purpose::LoginCode, &email, code_input)
        .await?
        .is_none()
    {
        // Yanlış kod: sayaç artırıldı, değişiklik kalıcı olmalı.
        tx.commit().await?;
        return Err(AppError::Unauthorized("invalid_code"));
    }

    let row: CodeLoginRow = sqlx::query_as(
        "SELECT id, username, display_name, email, role
           FROM users WHERE email = $1 AND email_verified_at IS NOT NULL",
    )
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized("invalid_code"))?;

    let token = session::create(&mut tx, row.id, &headers).await?;
    tx.commit().await?;
    state.limiter.reset(&format!("logincode:verify:{email}"));

    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            email: row.email,
            role: row.role,
        },
    }))
}
