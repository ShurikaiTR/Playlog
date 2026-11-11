//! HMAC ile imzalı, kısa ömürlü belirteçler: akış durumu (`state`) ve kullanıcı adı bekleyen
//! yeni hesap (`pending`). Sunucuda kayıt tutmamak için tarayıcıya verilir, dönüşte doğrulanır.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use super::{OAuth, now};

const PENDING_TTL_SECONDS: i64 = 900;
/// İmzaya karışan bağlam: bir türün belirteci diğerinin yerine kullanılamaz.
pub(super) const STATE_CONTEXT: &str = "oauth-state";
const PENDING_CONTEXT: &str = "oauth-pending";

#[derive(Serialize, Deserialize)]
pub(super) struct StateClaims {
    pub(super) provider: String,
    pub(super) state: String,
    pub(super) nonce: String,
    pub(super) verifier: String,
    pub(super) exp: i64,
    /// Hesap ayarlarından başlatılan bağlama akışında, bağlanacak kullanıcı.
    /// Giriş akışında yoktur; böylece bir akışın belirteci diğerinde kullanılamaz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) link_user: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct PendingClaims {
    pub provider: String,
    pub subject: String,
    /// Steam e-posta vermez; o hesap e-postasız açılır.
    #[serde(default)]
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub exp: i64,
}

impl OAuth {
    pub(super) fn sign<T: Serialize>(&self, context: &str, claims: &T) -> anyhow::Result<String> {
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims)?);
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&self.sign_key)?;
        mac.update(context.as_bytes());
        mac.update(b"\0");
        mac.update(payload.as_bytes());
        let tag = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        Ok(format!("{payload}.{tag}"))
    }

    pub(super) fn verify<T: DeserializeOwned>(&self, context: &str, token: &str) -> Option<T> {
        let (payload, tag) = token.split_once('.')?;
        let tag = URL_SAFE_NO_PAD.decode(tag).ok()?;
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&self.sign_key).ok()?;
        mac.update(context.as_bytes());
        mac.update(b"\0");
        mac.update(payload.as_bytes());
        let expected = mac.finalize().into_bytes();
        if !bool::from(expected.as_slice().ct_eq(&tag)) {
            return None;
        }
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()
    }

    pub fn sign_pending(&self, claims: &PendingClaims) -> anyhow::Result<String> {
        self.sign(PENDING_CONTEXT, claims)
    }

    pub fn verify_pending(&self, token: &str) -> Option<PendingClaims> {
        let claims: PendingClaims = self.verify(PENDING_CONTEXT, token)?;
        (claims.exp > now()).then_some(claims)
    }

    pub fn pending_ttl() -> i64 {
        PENDING_TTL_SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::{super::testing::oauth, *};

    #[test]
    fn pending_token_roundtrip_and_tamper() {
        let oauth = oauth();
        let token = oauth
            .sign_pending(&PendingClaims {
                provider: "discord".into(),
                subject: "42".into(),
                email: Some("a@b.co".into()),
                display_name: None,
                avatar_url: None,
                exp: now() + 60,
            })
            .unwrap();
        assert_eq!(oauth.verify_pending(&token).unwrap().subject, "42");

        let (payload, tag) = token.split_once('.').unwrap();
        let forged_payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&serde_json::json!({
                "provider": "discord", "subject": "99", "email": "a@b.co",
                "display_name": null, "exp": now() + 60
            }))
            .unwrap(),
        );
        assert_ne!(payload, forged_payload);
        assert!(
            oauth
                .verify_pending(&format!("{forged_payload}.{tag}"))
                .is_none()
        );
        assert!(oauth.verify_pending("garbage").is_none());
    }

    #[test]
    fn expired_pending_token_is_rejected() {
        let oauth = oauth();
        let token = oauth
            .sign_pending(&PendingClaims {
                provider: "discord".into(),
                subject: "1".into(),
                email: Some("a@b.co".into()),
                display_name: None,
                avatar_url: None,
                exp: now() - 1,
            })
            .unwrap();
        assert!(oauth.verify_pending(&token).is_none());
    }

    #[test]
    fn state_token_is_context_bound() {
        let oauth = oauth();
        let token = oauth
            .sign(
                STATE_CONTEXT,
                &StateClaims {
                    provider: "discord".into(),
                    state: "s".into(),
                    nonce: String::new(),
                    verifier: "v".into(),
                    exp: now() + 60,
                    link_user: None,
                },
            )
            .unwrap();
        // Aynı imza başka bağlamda (pending) geçerli olmamalı.
        assert!(
            oauth
                .verify::<StateClaims>(PENDING_CONTEXT, &token)
                .is_none()
        );
        assert!(oauth.verify::<StateClaims>(STATE_CONTEXT, &token).is_some());
    }

    #[test]
    fn link_user_survives_state_token_and_defaults_to_none() {
        let oauth = oauth();
        let user = uuid::Uuid::now_v7();
        let claims = |link_user| StateClaims {
            provider: "discord".into(),
            state: "s".into(),
            nonce: String::new(),
            verifier: "v".into(),
            exp: now() + 60,
            link_user,
        };
        let linked = oauth.sign(STATE_CONTEXT, &claims(Some(user))).unwrap();
        let parsed: StateClaims = oauth.verify(STATE_CONTEXT, &linked).unwrap();
        assert_eq!(parsed.link_user, Some(user));
        // Giriş akışının belirtecinde alan hiç yazılmaz, okununca `None` olur.
        let sign_in = oauth.sign(STATE_CONTEXT, &claims(None)).unwrap();
        let payload = sign_in.split_once('.').unwrap().0;
        let json = String::from_utf8(URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
        assert!(!json.contains("link_user"));
        let parsed: StateClaims = oauth.verify(STATE_CONTEXT, &sign_in).unwrap();
        assert_eq!(parsed.link_user, None);
    }

    #[test]
    fn pending_token_without_email_round_trips() {
        let oauth = oauth();
        let token = oauth
            .sign_pending(&PendingClaims {
                provider: "steam".into(),
                subject: "76561198000000000".into(),
                email: None,
                display_name: Some("Gaben".into()),
                avatar_url: None,
                exp: now() + 60,
            })
            .unwrap();
        let claims = oauth.verify_pending(&token).unwrap();
        assert_eq!(claims.email, None);
        assert_eq!(claims.display_name.as_deref(), Some("Gaben"));
    }
}
