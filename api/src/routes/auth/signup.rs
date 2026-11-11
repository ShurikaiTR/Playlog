//! E-postayla kayıt: bilgiler alınır, adrese kod gider; doğru kod hesabı açar ve oturum başlatır.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use uuid::Uuid;

use super::{CodeSent, SessionResponse, UserView, VerifyCode};
use crate::{
    auth::{
        email_code::{self, Purpose},
        password,
        role::Role,
        session, validate,
    },
    error::{AppError, map_unique_violation},
    routes::guard::{TEN_MINUTES, client_ip, rate_limit, verify_captcha},
    state::AppState,
};

#[derive(Deserialize)]
pub(super) struct SignupStart {
    /// Turnstile yanıt belirteci (cf-turnstile-response).
    #[serde(default)]
    turnstile_token: Option<String>,
    email: String,
    username: String,
    password: String,
    display_name: Option<String>,
    #[serde(default)]
    marketing_opt_in: bool,
}

pub(super) async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SignupStart>,
) -> Result<(StatusCode, Json<CodeSent>), AppError> {
    rate_limit(
        &state,
        &format!("signup:ip:{}", client_ip(&headers)),
        10,
        TEN_MINUTES,
    )?;
    verify_captcha(&state, &headers, body.turnstile_token.as_deref()).await?;
    let email = validate::email(&body.email)?;
    let username = validate::username(&body.username)?;
    let display_name = validate::display_name(body.display_name.as_deref())?;
    validate::password(&body.password)?;

    let username_taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)")
            .bind(&username)
            .fetch_one(&state.db)
            .await?;
    if username_taken {
        return Err(AppError::Conflict("username_taken"));
    }

    email_code::ensure_cooldown(&state, Purpose::Signup, &email).await?;

    let email_taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
            .bind(&email)
            .fetch_one(&state.db)
            .await?;
    if email_taken {
        // Hesap sayımını sızdırmamak için aynı yanıt dönülür; sahibine bilgi e-postası gider.
        state
            .mailer
            .send(
                &email,
                "Playthru: bu e-posta zaten kayıtlı",
                "Bu e-posta adresiyle zaten bir Playthru hesabı var. Giriş yapabilir ya da \
                 şifreni sıfırlayabilirsin. Bu isteği sen yapmadıysan bu e-postayı yok sayabilirsin.",
                None,
            )
            .await?;
        return Ok((StatusCode::ACCEPTED, CodeSent::new()));
    }

    let password_hash = password::hash(body.password).await?;
    let pending = serde_json::json!({
        "username": username,
        "display_name": display_name,
        "marketing_opt_in": body.marketing_opt_in,
        "password_hash": password_hash,
    });
    email_code::issue(&state, Purpose::Signup, &email, Some(pending)).await?;
    Ok((StatusCode::ACCEPTED, CodeSent::new()))
}

pub(super) async fn verify(
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

    let mut tx = state.db.begin().await?;
    let Some(pending) =
        email_code::consume(&state, &mut tx, Purpose::Signup, &email, code_input).await?
    else {
        // Yanlış kod: sayaç artırıldı, değişiklik kalıcı olmalı.
        tx.commit().await?;
        return Err(AppError::Unauthorized("invalid_code"));
    };

    let pending: serde_json::Value = pending.ok_or(AppError::Unauthorized("invalid_code"))?;
    let username = pending["username"].as_str().ok_or_else(corrupt)?.to_owned();
    let password_hash = pending["password_hash"].as_str().ok_or_else(corrupt)?;
    let display_name = pending["display_name"].as_str().map(str::to_owned);
    let marketing_opt_in = pending["marketing_opt_in"].as_bool().unwrap_or(false);

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, email, email_verified_at, display_name, marketing_opt_in_at)
         VALUES ($1, $2, now(), $3, CASE WHEN $4 THEN now() END)
         RETURNING id",
    )
    .bind(&username)
    .bind(&email)
    .bind(&display_name)
    .bind(marketing_opt_in)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_unique_violation)?;

    sqlx::query("INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(password_hash)
        .execute(&mut *tx)
        .await?;

    let token = session::create(&mut tx, user_id, &headers).await?;
    tx.commit().await?;

    Ok(Json(SessionResponse {
        token,
        user: UserView {
            id: user_id,
            username,
            display_name,
            email: Some(email),
            // Yeni hesap her zaman sütunun varsayılanı olan 'user' rolüyle açılır.
            role: Role::User,
        },
    }))
}

fn corrupt() -> AppError {
    AppError::Internal(anyhow::anyhow!("bekleyen kayıt verisi bozuk"))
}
