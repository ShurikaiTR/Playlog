//! Cloudflare Turnstile sunucu tarafı doğrulaması (siteverify).
//! Belge: https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
//!
//! `TURNSTILE_SECRET` tanımlıysa doğrulama zorunludur ve her hata isteği reddeder (fail closed);
//! tanımlı değilse (yerel geliştirme) hiç kurulmaz ve kontrol atlanır.

use std::time::Duration;

use serde::Deserialize;

use crate::error::AppError;

const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
/// Cloudflare belgesi: yanıt belirteci en fazla 2048 karakterdir.
const MAX_TOKEN_LEN: usize = 2048;

pub struct Turnstile {
    client: reqwest::Client,
    secret: String,
    /// Küçük harfe çevrilmiş izinli alan adları.
    hostnames: Vec<String>,
}

#[derive(Deserialize)]
struct SiteVerify {
    success: bool,
    hostname: Option<String>,
}

impl Turnstile {
    pub fn new(secret: String, hostnames: Vec<String>) -> anyhow::Result<Self> {
        if hostnames.is_empty() {
            tracing::warn!(
                "TURNSTILE_HOSTNAMES boş: alan adı denetimi yapılmayacak, yalnızca belirteç doğrulanacak"
            );
        }
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?,
            secret,
            hostnames: hostnames.into_iter().map(|h| h.to_lowercase()).collect(),
        })
    }

    /// Belirteci siteverify ile doğrular. Eksik/geçersiz belirteç, hatalı alan adı, ağ hatası
    /// ve zaman aşımı dahil her başarısızlık isteği reddeder.
    pub async fn verify(&self, token: Option<&str>, remote_ip: &str) -> Result<(), AppError> {
        let token = token
            .map(str::trim)
            .filter(|t| !t.is_empty() && t.len() <= MAX_TOKEN_LEN)
            .ok_or(AppError::Validation("captcha_failed"))?;

        let mut form = vec![("secret", self.secret.as_str()), ("response", token)];
        if remote_ip.parse::<std::net::IpAddr>().is_ok() {
            form.push(("remoteip", remote_ip));
        }

        let response: SiteVerify = self
            .client
            .post(SITEVERIFY_URL)
            .form(&form)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|err| AppError::Internal(err.into()))?
            .json()
            .await
            .map_err(|err| AppError::Internal(err.into()))?;

        if self.accepts(&response) {
            Ok(())
        } else {
            Err(AppError::Validation("captcha_failed"))
        }
    }

    fn accepts(&self, response: &SiteVerify) -> bool {
        response.success
            && (self.hostnames.is_empty()
                || response
                    .hostname
                    .as_deref()
                    .is_some_and(|h| self.hostnames.contains(&h.to_lowercase())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turnstile(hostnames: &[&str]) -> Turnstile {
        Turnstile::new(
            "secret".into(),
            hostnames.iter().map(|h| h.to_string()).collect(),
        )
        .unwrap()
    }

    fn response(success: bool, hostname: Option<&str>) -> SiteVerify {
        SiteVerify {
            success,
            hostname: hostname.map(str::to_owned),
        }
    }

    #[test]
    fn rejects_failed_verification() {
        assert!(!turnstile(&["playthru.app"]).accepts(&response(false, Some("playthru.app"))));
    }

    #[test]
    fn requires_hostname_on_the_allowlist() {
        let t = turnstile(&["playthru.app", "www.playthru.app"]);
        assert!(t.accepts(&response(true, Some("playthru.app"))));
        assert!(t.accepts(&response(true, Some("WWW.Playthru.app"))));
        assert!(!t.accepts(&response(true, Some("evil.example"))));
        assert!(!t.accepts(&response(true, None)));
    }

    #[test]
    fn empty_allowlist_skips_the_hostname_check() {
        assert!(turnstile(&[]).accepts(&response(true, Some("anything.example"))));
    }

    #[tokio::test]
    async fn missing_or_oversized_token_is_rejected_without_a_request() {
        let t = turnstile(&["playthru.app"]);
        assert!(t.verify(None, "203.0.113.7").await.is_err());
        assert!(t.verify(Some("  "), "203.0.113.7").await.is_err());
        let long = "x".repeat(MAX_TOKEN_LEN + 1);
        assert!(t.verify(Some(&long), "203.0.113.7").await.is_err());
    }
}
