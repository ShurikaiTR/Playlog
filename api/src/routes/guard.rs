//! Uçların ortak korumaları: istemci IP'si, hız sınırı ve Turnstile doğrulaması.

use std::time::Duration;

use axum::http::HeaderMap;

use crate::{error::AppError, state::AppState};

pub(super) const TEN_MINUTES: Duration = Duration::from_secs(600);
pub(super) const FIFTEEN_MINUTES: Duration = Duration::from_secs(900);

/// SvelteKit'in ilettiği gerçek istemci IP'si (API yalnızca iç ağdadır, başlığa güvenilir).
/// Başlık yoksa ortak bir "unknown" kovasına düşer; bu daha sıkı sınırlanır, güvenli taraf.
pub(super) fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-client-ip")
        .and_then(|v| v.to_str().ok())
        .filter(|ip| ip.parse::<std::net::IpAddr>().is_ok())
        .unwrap_or("unknown")
        .to_owned()
}

/// Turnstile açıksa belirteci doğrular; kapalıysa (yerel geliştirme) atlar.
pub(super) async fn verify_captcha(
    state: &AppState,
    headers: &HeaderMap,
    token: Option<&str>,
) -> Result<(), AppError> {
    match &state.turnstile {
        Some(turnstile) => turnstile.verify(token, &client_ip(headers)).await,
        None => Ok(()),
    }
}

pub(super) fn rate_limit(
    state: &AppState,
    key: &str,
    max: u32,
    window: Duration,
) -> Result<(), AppError> {
    state
        .limiter
        .hit(key, max, window)
        .map_err(|_| AppError::TooManyRequests("rate_limited"))
}
