use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::token, error::AppError, state::AppState};

/// `Authorization: Bearer <id>.<gizli parça>` başlığından doğrulanmış kullanıcı.
/// Başlığı SvelteKit sunucusu, tarayıcıdaki oturum çerezinden alıp ekler.
pub struct CurrentUser {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(FromRow)]
struct SessionRow {
    verifier_hash: Vec<u8>,
    user_id: Uuid,
    username: String,
    email: String,
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized = || AppError::Unauthorized("unauthenticated");

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| unauthorized())?;
        let parsed = token::parse(bearer.token()).ok_or_else(unauthorized)?;

        // Mutlak ömür 90 gün; boşta kalma süresi `expires_at` ile (30 gün) denetlenir.
        let row: SessionRow = sqlx::query_as(
            "SELECT s.verifier_hash, u.id AS user_id, u.username, u.email
               FROM sessions s
               JOIN users u ON u.id = s.user_id
              WHERE s.id = $1
                AND s.expires_at > now()
                AND s.created_at > now() - interval '90 days'",
        )
        .bind(parsed.id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(unauthorized)?;

        if !token::verifier_matches(parsed.secret, &row.verifier_hash) {
            return Err(unauthorized());
        }

        // Her istekte yazmamak için en çok 5 dakikada bir yenilenir.
        sqlx::query(
            "UPDATE sessions
                SET last_seen_at = now(), expires_at = now() + interval '30 days'
              WHERE id = $1 AND last_seen_at < now() - interval '5 minutes'",
        )
        .bind(parsed.id)
        .execute(&state.db)
        .await?;

        Ok(Self {
            session_id: parsed.id,
            user_id: row.user_id,
            username: row.username,
            email: row.email,
        })
    }
}
