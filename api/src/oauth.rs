//! Google (OpenID Connect) ve Discord (OAuth 2.0) ile giriş.
//!
//! Akış durumsuzdur: `start` çağrısı state, nonce ve PKCE doğrulayıcısını HMAC ile
//! imzalı kısa ömürlü bir belirtece koyar; SvelteKit bunu tarayıcıda httpOnly çerezde
//! saklar ve dönüşte API'ye geri verir. Sunucuda geçici kayıt tutulmaz.

use std::{sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse as _, TokenUrl, basic::BasicClient,
};
use openidconnect::{
    IssuerUrl, Nonce,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

const STATE_TTL_SECONDS: i64 = 600;
const PENDING_TTL_SECONDS: i64 = 900;
const METADATA_TTL: Duration = Duration::from_secs(3600);
const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URL: &str = "https://discord.com/api/users/@me";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    Google,
    Discord,
}

impl Provider {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "google" => Some(Self::Google),
            "discord" => Some(Self::Discord),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Discord => "discord",
        }
    }
}

#[derive(Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Dışarıdan gelen kimlik: sağlayıcının doğruladığı bilgiler.
pub struct ProviderIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
}

pub struct OAuth {
    public_url: String,
    google: Option<Credentials>,
    discord: Option<Credentials>,
    /// Yönlendirme izlemeyen istemci (OIDC/OAuth2 belgesi bunu şart koşar).
    http: openidconnect::reqwest::Client,
    api_http: reqwest::Client,
    sign_key: Vec<u8>,
    google_metadata: RwLock<Option<(std::time::Instant, Arc<CoreProviderMetadata>)>>,
}

#[derive(Serialize, Deserialize)]
struct StateClaims {
    provider: String,
    state: String,
    nonce: String,
    verifier: String,
    exp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct PendingClaims {
    pub provider: String,
    pub subject: String,
    pub email: String,
    pub display_name: Option<String>,
    pub exp: i64,
}

pub struct Started {
    pub authorization_url: String,
    pub state_token: String,
}

impl OAuth {
    pub fn new(
        public_url: Option<String>,
        google: Option<Credentials>,
        discord: Option<Credentials>,
        sign_key: &[u8],
    ) -> anyhow::Result<Self> {
        let public_url = public_url
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_owned();
        if (google.is_some() || discord.is_some()) && public_url.is_empty() {
            anyhow::bail!(
                "Google/Discord için PUBLIC_URL tanımlı olmalı (örn. https://playthru.app)"
            );
        }
        let http = openidconnect::reqwest::ClientBuilder::new()
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()?;
        let api_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            public_url,
            google,
            discord,
            http,
            api_http,
            sign_key: sign_key.to_vec(),
            google_metadata: RwLock::new(None),
        })
    }

    pub fn enabled(&self, provider: Provider) -> bool {
        match provider {
            Provider::Google => self.google.is_some(),
            Provider::Discord => self.discord.is_some(),
        }
    }

    fn redirect_url(&self, provider: Provider) -> anyhow::Result<RedirectUrl> {
        Ok(RedirectUrl::new(format!(
            "{}/auth/{}/callback",
            self.public_url,
            provider.as_str()
        ))?)
    }

    // ---------- İmzalı belirteçler ----------

    fn sign<T: Serialize>(&self, context: &str, claims: &T) -> anyhow::Result<String> {
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims)?);
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&self.sign_key)?;
        mac.update(context.as_bytes());
        mac.update(b"\0");
        mac.update(payload.as_bytes());
        let tag = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        Ok(format!("{payload}.{tag}"))
    }

    fn verify<T: DeserializeOwned>(&self, context: &str, token: &str) -> Option<T> {
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
        self.sign("oauth-pending", claims)
    }

    pub fn verify_pending(&self, token: &str) -> Option<PendingClaims> {
        let claims: PendingClaims = self.verify("oauth-pending", token)?;
        (claims.exp > now()).then_some(claims)
    }

    pub fn pending_ttl() -> i64 {
        PENDING_TTL_SECONDS
    }

    // ---------- Akış ----------

    pub async fn start(&self, provider: Provider) -> anyhow::Result<Started> {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = match provider {
            Provider::Google => {
                let creds = self.google.as_ref().context("Google yapılandırılmamış")?;
                let client = self.google_client(creds).await?;
                let (url, csrf, nonce) = client
                    .authorize_url(
                        CoreAuthenticationFlow::AuthorizationCode,
                        CsrfToken::new_random,
                        Nonce::new_random,
                    )
                    .add_scope(Scope::new("email".into()))
                    .add_scope(Scope::new("profile".into()))
                    .set_pkce_challenge(challenge)
                    .url();
                (url, csrf.secret().clone(), nonce.secret().clone())
            }
            Provider::Discord => {
                let client = self.discord_client()?;
                let (url, csrf) = client
                    .authorize_url(CsrfToken::new_random)
                    .add_scope(Scope::new("identify".into()))
                    .add_scope(Scope::new("email".into()))
                    .set_pkce_challenge(challenge)
                    .url();
                (url, csrf.secret().clone(), String::new())
            }
        };

        let state_token = self.sign(
            "oauth-state",
            &StateClaims {
                provider: provider.as_str().into(),
                state,
                nonce,
                verifier: verifier.secret().clone(),
                exp: now() + STATE_TTL_SECONDS,
            },
        )?;
        Ok(Started {
            authorization_url: url.to_string(),
            state_token,
        })
    }

    /// Geri dönüşü doğrular, kodu belirteçle değiştirir, sağlayıcı kimliğini döner.
    /// `None`: state geçersiz/süresi dolmuş. `Err`: sağlayıcı reddetti ya da ağ hatası.
    pub async fn finish(
        &self,
        provider: Provider,
        code: String,
        state: &str,
        state_token: &str,
    ) -> anyhow::Result<Option<ProviderIdentity>> {
        let Some(claims) = self.verify::<StateClaims>("oauth-state", state_token) else {
            return Ok(None);
        };
        let state_matches = claims.state.as_bytes().ct_eq(state.as_bytes());
        if claims.provider != provider.as_str() || claims.exp <= now() || !bool::from(state_matches)
        {
            return Ok(None);
        }
        let verifier = PkceCodeVerifier::new(claims.verifier);

        let identity = match provider {
            Provider::Google => {
                let creds = self.google.as_ref().context("Google yapılandırılmamış")?;
                let client = self.google_client(creds).await?;
                let tokens = client
                    .exchange_code(AuthorizationCode::new(code))?
                    .set_pkce_verifier(verifier)
                    .request_async(&self.http)
                    .await
                    .context("Google kod değişimi başarısız")?;
                let id_token = openidconnect::TokenResponse::id_token(&tokens)
                    .context("Google kimlik jetonu dönmedi")?;
                let id_claims = id_token
                    .claims(&client.id_token_verifier(), &Nonce::new(claims.nonce))
                    .context("Google kimlik jetonu doğrulanamadı")?;
                ProviderIdentity {
                    subject: id_claims.subject().as_str().to_owned(),
                    email: id_claims.email().map(|e| e.as_str().to_owned()),
                    email_verified: id_claims.email_verified().unwrap_or(false),
                    display_name: id_claims
                        .name()
                        .and_then(|n| n.get(None))
                        .map(|n| n.as_str().to_owned()),
                }
            }
            Provider::Discord => {
                let client = self.discord_client()?;
                let tokens = client
                    .exchange_code(AuthorizationCode::new(code))
                    .set_pkce_verifier(verifier)
                    .request_async(&self.http)
                    .await
                    .context("Discord kod değişimi başarısız")?;
                self.discord_user(tokens.access_token().secret()).await?
            }
        };
        Ok(Some(identity))
    }

    async fn google_client(
        &self,
        creds: &Credentials,
    ) -> anyhow::Result<
        CoreClient<
            openidconnect::EndpointSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointMaybeSet,
            openidconnect::EndpointMaybeSet,
        >,
    > {
        let metadata = self.google_metadata().await?;
        Ok(CoreClient::from_provider_metadata(
            (*metadata).clone(),
            openidconnect::ClientId::new(creds.client_id.clone()),
            Some(openidconnect::ClientSecret::new(
                creds.client_secret.clone(),
            )),
        )
        .set_redirect_uri(openidconnect::RedirectUrl::new(
            self.redirect_url(Provider::Google)?.to_string(),
        )?))
    }

    /// Google'ın anahtar kümesi dönüşümlü değişir; bu yüzden keşif belgesi saatte bir yenilenir.
    async fn google_metadata(&self) -> anyhow::Result<Arc<CoreProviderMetadata>> {
        if let Some((fetched, metadata)) = &*self.google_metadata.read().await
            && fetched.elapsed() < METADATA_TTL
        {
            return Ok(metadata.clone());
        }
        let metadata = Arc::new(
            CoreProviderMetadata::discover_async(IssuerUrl::new(GOOGLE_ISSUER.into())?, &self.http)
                .await
                .context("Google keşif belgesi alınamadı")?,
        );
        *self.google_metadata.write().await = Some((std::time::Instant::now(), metadata.clone()));
        Ok(metadata)
    }

    fn discord_client(
        &self,
    ) -> anyhow::Result<
        BasicClient<
            oauth2::EndpointSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointSet,
        >,
    > {
        let creds = self.discord.as_ref().context("Discord yapılandırılmamış")?;
        Ok(BasicClient::new(ClientId::new(creds.client_id.clone()))
            .set_client_secret(ClientSecret::new(creds.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(DISCORD_AUTH_URL.into())?)
            .set_token_uri(TokenUrl::new(DISCORD_TOKEN_URL.into())?)
            .set_redirect_uri(self.redirect_url(Provider::Discord)?))
    }

    async fn discord_user(&self, access_token: &str) -> anyhow::Result<ProviderIdentity> {
        #[derive(Deserialize)]
        struct DiscordUser {
            id: String,
            username: String,
            global_name: Option<String>,
            email: Option<String>,
            #[serde(default)]
            verified: bool,
        }
        let user: DiscordUser = self
            .api_http
            .get(DISCORD_USER_URL)
            .bearer_auth(access_token)
            // Discord, geçerli bir User-Agent ister: "DiscordBot ($url, $versionNumber)".
            .header(
                "User-Agent",
                format!("DiscordBot ({}, 1.0)", self.public_url),
            )
            .send()
            .await
            .context("Discord kullanıcı isteği başarısız")?
            .error_for_status()
            .map_err(|e| anyhow!("Discord kullanıcı isteği hata döndürdü: {e}"))?
            .json()
            .await?;
        Ok(ProviderIdentity {
            subject: user.id,
            email: user.email.map(|e| e.to_lowercase()),
            email_verified: user.verified,
            display_name: user.global_name.or(Some(user.username)),
        })
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth() -> OAuth {
        OAuth::new(
            Some("https://playthru.app/".into()),
            None,
            Some(Credentials {
                client_id: "id".into(),
                client_secret: "secret".into(),
            }),
            b"0123456789abcdef0123456789abcdef",
        )
        .unwrap()
    }

    #[test]
    fn pending_token_roundtrip_and_tamper() {
        let oauth = oauth();
        let token = oauth
            .sign_pending(&PendingClaims {
                provider: "discord".into(),
                subject: "42".into(),
                email: "a@b.co".into(),
                display_name: None,
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
                email: "a@b.co".into(),
                display_name: None,
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
                "oauth-state",
                &StateClaims {
                    provider: "discord".into(),
                    state: "s".into(),
                    nonce: String::new(),
                    verifier: "v".into(),
                    exp: now() + 60,
                },
            )
            .unwrap();
        // Aynı imza başka bağlamda (pending) geçerli olmamalı.
        assert!(
            oauth
                .verify::<StateClaims>("oauth-pending", &token)
                .is_none()
        );
        assert!(oauth.verify::<StateClaims>("oauth-state", &token).is_some());
    }

    #[tokio::test]
    async fn discord_start_builds_pkce_url() {
        let started = oauth().start(Provider::Discord).await.unwrap();
        let url = url_query(&started.authorization_url);
        assert!(
            started
                .authorization_url
                .starts_with("https://discord.com/oauth2/authorize?")
        );
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(
            url.contains("redirect_uri=https%3A%2F%2Fplaythru.app%2Fauth%2Fdiscord%2Fcallback")
        );
        assert!(url.contains("scope=identify+email"));
    }

    fn url_query(url: &str) -> String {
        url.split_once('?')
            .map(|(_, q)| q.to_owned())
            .unwrap_or_default()
    }
}
