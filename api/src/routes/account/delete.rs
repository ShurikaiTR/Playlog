//! Hesabı silme. Şifre ya da yakın zamanda açılmış bir oturum ister; son admin silinemez.

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{extractor::CurrentUser, password, role::Role},
    error::AppError,
    routes::guard::{FIFTEEN_MINUTES, rate_limit},
    state::AppState,
};

#[derive(Deserialize, Default)]
pub(super) struct DeleteAccount {
    /// Şifresi olan hesapta zorunlu.
    password: Option<String>,
}

/// Şifresi olmayan hesap silinirken oturum en fazla bu kadar önce açılmış olmalı (yeniden giriş).
const FRESH_SESSION: &str = "1 day";

pub(super) async fn delete_account(
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
        // Yalnızca sosyal girişli hesap: kimliği yakın zamanda sağlayıcıda doğrulanmış olmalı.
        None => {
            let fresh: bool = sqlx::query_scalar(
                "SELECT created_at > now() - $2::interval FROM sessions WHERE id = $1",
            )
            .bind(user.session_id)
            .bind(FRESH_SESSION)
            .fetch_one(&state.db)
            .await?;
            if !fresh {
                return Err(AppError::Unauthorized("session_not_fresh"));
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
