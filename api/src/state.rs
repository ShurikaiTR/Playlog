use std::sync::Arc;

use sqlx::PgPool;

use crate::{mail::Mailer, oauth::OAuth, ratelimit::RateLimiter, turnstile::Turnstile};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub mailer: Mailer,
    pub code_secret: Arc<[u8]>,
    /// Var olmayan kullanıcıda da Argon2 çalıştırmak için (zamanlama sızıntısını önler).
    pub dummy_hash: Arc<str>,
    pub limiter: Arc<RateLimiter>,
    pub oauth: Arc<OAuth>,
    /// Yalnızca TURNSTILE_SECRET tanımlıysa var; yoksa bot doğrulaması atlanır (yerel geliştirme).
    pub turnstile: Option<Arc<Turnstile>>,
}
