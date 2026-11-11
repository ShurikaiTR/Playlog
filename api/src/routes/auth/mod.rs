//! Kimlik doğrulama uçları. Tarayıcı bunlara doğrudan gitmez; SvelteKit sunucusu çağırır.

mod login;
mod password_reset;
mod signup;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{code, extractor::CurrentUser, role::Role},
    error::AppError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/signup/start", post(signup::start))
        .route("/signup/verify", post(signup::verify))
        .route("/login", post(login::password))
        .route("/login-code/start", post(login::code_start))
        .route("/login-code/verify", post(login::code_verify))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/password-reset/start", post(password_reset::start))
        .route("/password-reset/confirm", post(password_reset::confirm))
}

// ---------- Ortak istek / yanıt tipleri ----------

/// E-postaya gelen kodla yapılan doğrulama (kayıt ve kodla giriş).
#[derive(Deserialize)]
struct VerifyCode {
    email: String,
    code: String,
}

#[derive(Serialize)]
struct CodeSent {
    status: &'static str,
    expires_in: i64,
}

impl CodeSent {
    fn new() -> Json<Self> {
        Json(Self {
            status: "code_sent",
            expires_in: code::TTL_MINUTES * 60,
        })
    }
}

#[derive(Serialize)]
pub(super) struct UserView {
    pub(super) id: Uuid,
    pub(super) username: String,
    pub(super) display_name: Option<String>,
    /// Steam ile açılan hesapta e-posta yoktur (sonradan eklenebilir).
    pub(super) email: Option<String>,
    pub(super) role: Role,
}

#[derive(Serialize)]
pub(super) struct SessionResponse {
    /// SvelteKit bunu `__Host-` önekli, HttpOnly çereze yazar.
    pub(super) token: String,
    pub(super) user: UserView,
}

async fn logout(State(state): State<AppState>, user: CurrentUser) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(user.session_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(user: CurrentUser) -> Json<UserView> {
    Json(UserView {
        id: user.user_id,
        username: user.username,
        display_name: user.display_name,
        email: user.email,
        role: user.role,
    })
}
