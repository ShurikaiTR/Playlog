//! Hesap ayarları uçları: bilgiler, kişisel bilgiler, şifre, giriş yöntemleri, oturumlar ve
//! hesabı silme. Hepsi oturum ister ve yalnızca oturum sahibinin kayıtlarına dokunur.

mod delete;
mod export;
mod password;
mod profile;
mod sessions;
mod sign_in_methods;

use axum::{
    Json, Router,
    extract::State,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::{extractor::CurrentUser, role::Role},
    error::AppError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(account).delete(delete::delete_account))
        .route("/export", get(export::export))
        .route("/profile", put(profile::update))
        .route("/password", put(password::set))
        .route("/email-code", post(password::email_code))
        .route(
            "/sessions",
            get(sessions::list).delete(sessions::revoke_others),
        )
        .route("/sessions/{id}", delete(sessions::revoke))
        .route(
            "/sign-in-methods/{provider}",
            delete(sign_in_methods::unlink),
        )
        .route(
            "/sign-in-methods/{provider}/start",
            post(sign_in_methods::link_start),
        )
        .route(
            "/sign-in-methods/{provider}/callback",
            post(sign_in_methods::link_callback),
        )
}

#[derive(Serialize)]
struct AccountView {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    role: Role,
    created_at: DateTime<Utc>,
    /// E-posta ve şifreyle girişi var mı.
    has_password: bool,
    sign_in_methods: Vec<LinkedAccount>,
}

#[derive(Serialize, FromRow)]
struct LinkedAccount {
    provider: String,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct AccountRow {
    avatar_url: Option<String>,
    created_at: DateTime<Utc>,
    has_password: bool,
}

async fn account(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<AccountView>, AppError> {
    let row: AccountRow = sqlx::query_as(
        "SELECT u.avatar_url, u.created_at,
                EXISTS (SELECT 1 FROM credentials c WHERE c.user_id = u.id) AS has_password
           FROM users u
          WHERE u.id = $1",
    )
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await?;
    let sign_in_methods = linked_accounts(&state.db, user.user_id).await?;

    Ok(Json(AccountView {
        id: user.user_id,
        username: user.username,
        display_name: user.display_name,
        email: user.email,
        avatar_url: row.avatar_url,
        role: user.role,
        created_at: row.created_at,
        has_password: row.has_password,
        sign_in_methods,
    }))
}

/// Bağlı sağlayıcılar.
async fn linked_accounts(db: &sqlx::PgPool, user_id: Uuid) -> Result<Vec<LinkedAccount>, AppError> {
    Ok(sqlx::query_as(
        "SELECT provider::text AS provider, created_at
           FROM oauth_accounts
          WHERE user_id = $1 AND provider IN ('google', 'discord', 'apple', 'steam')
          ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}
