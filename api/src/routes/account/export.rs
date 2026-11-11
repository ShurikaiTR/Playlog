//! Verilerimi indir.

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use super::{LinkedAccount, linked_accounts};
use crate::{
    auth::{extractor::CurrentUser, role::Role},
    error::AppError,
    routes::guard::{FIFTEEN_MINUTES, rate_limit},
    state::AppState,
};

#[derive(Serialize)]
pub(super) struct Export {
    exported_at: DateTime<Utc>,
    account: ExportAccount,
    sign_in_methods: Vec<LinkedAccount>,
    sessions: Vec<ExportSession>,
}

#[derive(Serialize, FromRow)]
pub(super) struct ExportAccount {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    email: Option<String>,
    email_verified_at: Option<DateTime<Utc>>,
    avatar_url: Option<String>,
    role: Role,
    marketing_opt_in_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    has_password: bool,
}

#[derive(Serialize, FromRow)]
pub(super) struct ExportSession {
    user_agent: Option<String>,
    ip: Option<String>,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
}

/// Hesapta tutulan her şey tek bir JSON belgesi olarak (KVKK/GDPR veri taşınabilirliği).
/// Parola özeti, oturum belirteçleri ve sağlayıcı kimlik numaraları dahil edilmez.
pub(super) async fn export(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Export>, AppError> {
    rate_limit(
        &state,
        &format!("account:export:{}", user.user_id),
        10,
        FIFTEEN_MINUTES,
    )?;
    let account: ExportAccount = sqlx::query_as(
        "SELECT u.id, u.username, u.display_name, u.email, u.email_verified_at, u.avatar_url,
                u.role, u.marketing_opt_in_at, u.created_at, u.updated_at,
                EXISTS (SELECT 1 FROM credentials c WHERE c.user_id = u.id) AS has_password
           FROM users u
          WHERE u.id = $1",
    )
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await?;
    let sessions = sqlx::query_as(
        "SELECT user_agent, host(ip) AS ip, created_at, last_seen_at
           FROM sessions
          WHERE user_id = $1 AND expires_at > now()
          ORDER BY created_at",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(Export {
        exported_at: Utc::now(),
        account,
        sign_in_methods: linked_accounts(&state.db, user.user_id).await?,
        sessions,
    }))
}
