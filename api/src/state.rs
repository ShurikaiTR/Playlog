use std::sync::Arc;

use sqlx::PgPool;

use crate::{mail::Mailer, ratelimit::RateLimiter};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub mailer: Mailer,
    pub code_secret: Arc<[u8]>,
    /// Var olmayan kullanıcıda da Argon2 çalıştırmak için (zamanlama sızıntısını önler).
    pub dummy_hash: Arc<str>,
    pub limiter: Arc<RateLimiter>,
}
