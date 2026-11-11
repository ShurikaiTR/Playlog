//! Oturum kayıtları: açma ve toplu kapatma. Belirtecin kendisi `token` modülündedir;
//! istekten oturumu çözen `extractor::CurrentUser`'dır.

use axum::http::HeaderMap;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{auth::token, error::AppError};

/// Yeni oturum açar, istemciye verilecek belirteci döner.
/// IP ve tarayıcı bilgisi SvelteKit'in ilettiği başlıklardan gelir (API yalnızca iç ağdadır).
pub async fn create(
    conn: &mut PgConnection,
    user_id: Uuid,
    headers: &HeaderMap,
) -> Result<String, AppError> {
    let new = token::generate()?;
    let header = |name: &str, max: usize| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.chars().take(max).collect::<String>())
    };

    sqlx::query(
        "INSERT INTO sessions (id, user_id, verifier_hash, user_agent, ip, expires_at)
         VALUES ($1, $2, $3, $4, $5::text::inet, now() + interval '30 days')",
    )
    .bind(new.id)
    .bind(user_id)
    .bind(new.verifier_hash)
    .bind(header("x-client-user-agent", 300))
    .bind(header("x-client-ip", 45).filter(|ip| ip.parse::<std::net::IpAddr>().is_ok()))
    .execute(&mut *conn)
    .await?;
    Ok(new.token)
}

/// `keep` dışındaki tüm oturumları kapatır (örn. şifre değişince diğer cihazlar).
pub async fn delete_others(
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
