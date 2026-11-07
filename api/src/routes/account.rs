//! Hesap ayarları uçları: bilgiler, kişisel bilgiler, şifre, giriş yöntemleri, oturumlar ve
//! hesabı silme. Hepsi oturum ister ve yalnızca oturum sahibinin kayıtlarına dokunur.

use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

use super::{
    auth::{TEN_MINUTES, UserView, map_unique_violation, rate_limit},
    oauth::provider_of,
};
use crate::{
    auth::{extractor::CurrentUser, password, role::Role, validate},
    error::AppError,
    oauth::Provider,
    state::AppState,
};

const FIFTEEN_MINUTES: Duration = Duration::from_secs(900);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(account).delete(delete_account))
        .route("/profile", put(update_profile))
        .route("/password", put(set_password))
        .route("/sessions", get(sessions).delete(revoke_other_sessions))
        .route("/sessions/{id}", delete(revoke_session))
        .route("/sign-in-methods/{provider}", delete(unlink))
        .route("/sign-in-methods/{provider}/start", post(link_start))
        .route("/sign-in-methods/{provider}/callback", post(link_callback))
}

// ---------- Hesap bilgileri ----------

#[derive(Serialize)]
struct AccountView {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: String,
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

/// Bağlı sağlayıcılar. Eski 'steam' değeri artık desteklenmediği için listelenmez.
async fn linked_accounts(db: &sqlx::PgPool, user_id: Uuid) -> Result<Vec<LinkedAccount>, AppError> {
    Ok(sqlx::query_as(
        "SELECT provider::text AS provider, created_at
           FROM oauth_accounts
          WHERE user_id = $1 AND provider IN ('google', 'discord', 'apple')
          ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

// ---------- Kişisel bilgiler ----------

#[derive(Deserialize)]
struct ProfileUpdate {
    username: String,
    display_name: Option<String>,
}

async fn update_profile(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<ProfileUpdate>,
) -> Result<Json<UserView>, AppError> {
    let username = validate::username(&body.username)?;
    let display_name = validate::display_name(body.display_name.as_deref())?;
    if username != user.username {
        rate_limit(
            &state,
            &format!("account:username:{}", user.user_id),
            5,
            Duration::from_secs(60 * 60 * 24),
        )?;
    }

    sqlx::query(
        "UPDATE users SET username = $2, display_name = $3, updated_at = now() WHERE id = $1",
    )
    .bind(user.user_id)
    .bind(&username)
    .bind(&display_name)
    .execute(&state.db)
    .await
    .map_err(map_unique_violation)?;

    Ok(Json(UserView {
        id: user.user_id,
        username,
        display_name,
        email: user.email,
        role: user.role,
    }))
}

// ---------- Şifre ----------

#[derive(Deserialize)]
struct PasswordChange {
    /// Şifresi olan hesapta zorunludur; yalnızca sosyal girişi olan hesap ilk şifresini belirlerken boştur.
    current_password: Option<String>,
    new_password: String,
}

async fn set_password(
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

    let new_hash = password::hash(body.new_password).await?;
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO credentials (user_id, password_hash) VALUES ($1, $2)
         ON CONFLICT (user_id)
         DO UPDATE SET password_hash = EXCLUDED.password_hash, updated_at = now()",
    )
    .bind(user.user_id)
    .bind(new_hash)
    .execute(&mut *tx)
    .await?;
    // Şifre değişince bu cihaz dışındaki oturumlar kapanır (OWASP Session Management).
    delete_other_sessions(&mut tx, user.user_id, user.session_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- Oturumlar ----------

#[derive(Serialize, FromRow)]
struct SessionView {
    id: Uuid,
    user_agent: Option<String>,
    ip: Option<String>,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    current: bool,
}

async fn sessions(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<SessionView>>, AppError> {
    // Kimlik doğrulayıcıyla aynı geçerlilik koşulu: süresi dolanlar listelenmez.
    let sessions = sqlx::query_as(
        "SELECT id, user_agent, host(ip) AS ip, created_at, last_seen_at, id = $2 AS current
           FROM sessions
          WHERE user_id = $1
            AND expires_at > now()
            AND created_at > now() - interval '90 days'
          ORDER BY id = $2 DESC, last_seen_at DESC",
    )
    .bind(user.user_id)
    .bind(user.session_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(sessions))
}

/// Tek bir oturumu kapatır; bu cihazın oturumu da olabilir (çıkışla aynı).
async fn revoke_session(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound("session_not_found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn revoke_other_sessions(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db.acquire().await?;
    delete_other_sessions(&mut conn, user.user_id, user.session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_other_sessions(
    conn: &mut PgConnection,
    user_id: Uuid,
    keep: Uuid,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
        .bind(user_id)
        .bind(keep)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

// ---------- Giriş yöntemleri ----------

/// Bir sağlayıcı bağlantısını kaldırır. Hesaba girmenin son yolu kaldırılamaz.
async fn unlink(
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
struct LinkStart {
    authorization_url: String,
    state_token: String,
}

/// Bağlama akışını başlatır. State belirteci bu kullanıcıya bağlıdır; giriş akışında geçmez.
async fn link_start(
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
struct LinkCallback {
    code: String,
    state: String,
    state_token: String,
}

async fn link_callback(
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
        .finish(provider, body.code, &body.state, &body.state_token)
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

// ---------- Hesabı silme ----------

#[derive(Deserialize)]
struct DeleteAccount {
    /// Şifresi olan hesapta zorunlu.
    password: Option<String>,
    /// Şifresi olmayan (yalnızca sosyal girişli) hesapta kullanıcı adını yazarak onaylar.
    confirm_username: Option<String>,
}

async fn delete_account(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<DeleteAccount>,
) -> Result<StatusCode, AppError> {
    rate_limit(
        &state,
        &format!("account:delete:{}", user.user_id),
        5,
        FIFTEEN_MINUTES,
    )?;

    let stored: Option<String> =
        sqlx::query_scalar("SELECT password_hash FROM credentials WHERE user_id = $1")
            .bind(user.user_id)
            .fetch_optional(&state.db)
            .await?;
    match stored {
        Some(stored) => {
            let given = body
                .password
                .ok_or(AppError::Unauthorized("invalid_password"))?;
            if !password::verify(given, stored).await? {
                return Err(AppError::Unauthorized("invalid_password"));
            }
        }
        None => {
            let typed = body.confirm_username.unwrap_or_default();
            if typed.trim().to_ascii_lowercase() != user.username {
                return Err(AppError::Validation("confirmation_mismatch"));
            }
        }
    }

    let mut tx = state.db.begin().await?;
    // Son admin hesabını silerse yönetim paneline kimse giremez; önce yetkiyi devretmeli.
    if user.role == Role::Admin {
        let admins: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin' FOR UPDATE")
                .fetch_all(&mut *tx)
                .await?;
        if admins.len() <= 1 && admins.contains(&user.user_id) {
            return Err(AppError::Conflict("last_admin"));
        }
    }
    // Kimlik bilgileri, bağlı hesaplar ve oturumlar ON DELETE CASCADE ile birlikte silinir.
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    tracing::info!(user = %user.user_id, "hesap silindi");
    Ok(StatusCode::NO_CONTENT)
}
