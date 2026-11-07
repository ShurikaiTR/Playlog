//! Google ve Apple (OpenID Connect) ile Discord (OAuth 2.0) girişi.
//!
//! Akış durumsuzdur: `start` çağrısı state, nonce ve PKCE doğrulayıcısını HMAC ile
//! imzalı kısa ömürlü bir belirtece koyar; SvelteKit bunu tarayıcıda httpOnly çerezde
//! saklar ve dönüşte API'ye geri verir. Sunucuda geçici kayıt tutulmaz.

use std::{sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse as _, TokenUrl, basic::BasicClient,
};
use openidconnect::{
    IssuerUrl, Nonce,
    core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenClaims, CoreProviderMetadata},
};
use p256::{
    ecdsa::{Signature, SigningKey, signature::Signer},
    pkcs8::DecodePrivateKey,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

const STATE_TTL_SECONDS: i64 = 600;
const PENDING_TTL_SECONDS: i64 = 900;
const METADATA_TTL: Duration = Duration::from_secs(3600);
const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const APPLE_ISSUER: &str = "https://appleid.apple.com";
/// Apple istemci sırrı (JWT) bu kadar geçerli; her kod değişiminde yenisi üretilir.
const APPLE_SECRET_TTL_SECONDS: i64 = 300;
const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URL: &str = "https://discord.com/api/users/@me";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    Google,
    Discord,
    Apple,
}

impl Provider {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "google" => Some(Self::Google),
            "discord" => Some(Self::Discord),
            "apple" => Some(Self::Apple),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Discord => "discord",
            Self::Apple => "apple",
        }
    }
}

#[derive(Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Apple Developer hesabından: Services ID (`client_id`), Team ID, Sign in with Apple anahtarının
/// kimliği ve .p8 dosyasının içeriği.
pub struct AppleConfig {
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key_pem: String,
}

struct Apple {
    client_id: String,
    team_id: String,
    key_id: String,
    key: SigningKey,
}

type MetadataCache = RwLock<Option<(std::time::Instant, Arc<CoreProviderMetadata>)>>;

/// Keşif belgesinden kurulan, yönlendirme adresi ayarlı OIDC istemcisi.
type OidcClient = CoreClient<
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;

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
    apple: Option<Apple>,
    /// Yönlendirme izlemeyen istemci (OIDC/OAuth2 belgesi bunu şart koşar).
    http: openidconnect::reqwest::Client,
    api_http: reqwest::Client,
    sign_key: Vec<u8>,
    google_metadata: MetadataCache,
    apple_metadata: MetadataCache,
}

#[derive(Serialize, Deserialize)]
struct StateClaims {
    provider: String,
    state: String,
    nonce: String,
    verifier: String,
    exp: i64,
    /// Hesap ayarlarından başlatılan bağlama akışında, bağlanacak kullanıcı.
    /// Giriş akışında yoktur; böylece bir akışın belirteci diğerinde kullanılamaz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    link_user: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct PendingClaims {
    pub provider: String,
    pub subject: String,
    pub email: String,
    pub display_name: Option<String>,
    pub exp: i64,
}

/// Doğrulanmış dönüş: sağlayıcı kimliği ve akış bağlama içinse bağlanacak kullanıcı.
pub struct Finished {
    pub identity: ProviderIdentity,
    pub link_user: Option<uuid::Uuid>,
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
        apple: Option<AppleConfig>,
        sign_key: &[u8],
    ) -> anyhow::Result<Self> {
        let public_url = public_url
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_owned();
        if (google.is_some() || discord.is_some() || apple.is_some()) && public_url.is_empty() {
            anyhow::bail!(
                "Google/Discord/Apple için PUBLIC_URL tanımlı olmalı (örn. https://playthru.app)"
            );
        }
        // Anahtar açılışta bir kez okunur: hatalı .p8 ilk girişte değil, başlarken fark edilir.
        let apple = apple
            .map(|config| -> anyhow::Result<Apple> {
                Ok(Apple {
                    key: SigningKey::from_pkcs8_pem(config.private_key_pem.trim())
                        .context("APPLE_PRIVATE_KEY geçerli bir .p8 (PKCS#8 PEM) anahtarı değil")?,
                    client_id: config.client_id,
                    team_id: config.team_id,
                    key_id: config.key_id,
                })
            })
            .transpose()?;
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
            apple,
            http,
            api_http,
            sign_key: sign_key.to_vec(),
            google_metadata: RwLock::new(None),
            apple_metadata: RwLock::new(None),
        })
    }

    pub fn enabled(&self, provider: Provider) -> bool {
        match provider {
            Provider::Google => self.google.is_some(),
            Provider::Discord => self.discord.is_some(),
            Provider::Apple => self.apple.is_some(),
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

    /// `link_user`: hesap ayarlarından bağlama akışı başlatan kullanıcı; girişte `None`.
    pub async fn start(
        &self,
        provider: Provider,
        link_user: Option<uuid::Uuid>,
    ) -> anyhow::Result<Started> {
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
            Provider::Apple => {
                let apple = self.apple.as_ref().context("Apple yapılandırılmamış")?;
                let client = self.apple_client(apple, None).await?;
                // Ad ve e-posta istendiğinde Apple dönüşü yalnızca form POST ile yapar.
                // Apple PKCE'yi belgelemiyor; kod enjeksiyonuna karşı nonce + gizli istemci sırrı kullanılır.
                let (url, csrf, nonce) = client
                    .authorize_url(
                        CoreAuthenticationFlow::AuthorizationCode,
                        CsrfToken::new_random,
                        Nonce::new_random,
                    )
                    .add_scope(Scope::new("name".into()))
                    .add_scope(Scope::new("email".into()))
                    .add_extra_param("response_mode", "form_post")
                    .url();
                (url, csrf.secret().clone(), nonce.secret().clone())
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
                link_user,
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
    ) -> anyhow::Result<Option<Finished>> {
        let Some(claims) = self.verify::<StateClaims>("oauth-state", state_token) else {
            return Ok(None);
        };
        let state_matches = claims.state.as_bytes().ct_eq(state.as_bytes());
        if claims.provider != provider.as_str() || claims.exp <= now() || !bool::from(state_matches)
        {
            return Ok(None);
        }
        let verifier = PkceCodeVerifier::new(claims.verifier);
        let link_user = claims.link_user;

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
                identity_from_claims(id_claims, true)
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
            Provider::Apple => {
                let apple = self.apple.as_ref().context("Apple yapılandırılmamış")?;
                let client = self
                    .apple_client(apple, Some(self.apple_client_secret(apple)?))
                    .await?
                    // Apple istemci kimliğini ve sırrını gövdede ister (client_secret_post).
                    .set_auth_type(AuthType::RequestBody);
                let tokens = client
                    .exchange_code(AuthorizationCode::new(code))?
                    .request_async(&self.http)
                    .await
                    .context("Apple kod değişimi başarısız")?;
                let id_token = openidconnect::TokenResponse::id_token(&tokens)
                    .context("Apple kimlik jetonu dönmedi")?;
                let id_claims = id_token
                    .claims(&client.id_token_verifier(), &Nonce::new(claims.nonce))
                    .context("Apple kimlik jetonu doğrulanamadı")?;
                // Apple adı kimlik jetonuna koymaz; yalnızca ilk girişte form alanında gönderir.
                identity_from_claims(id_claims, false)
            }
        };
        Ok(Some(Finished {
            identity,
            link_user,
        }))
    }

    async fn google_client(&self, creds: &Credentials) -> anyhow::Result<OidcClient> {
        let metadata = self.metadata(GOOGLE_ISSUER, &self.google_metadata).await?;
        self.oidc_client(
            metadata,
            &creds.client_id,
            Some(creds.client_secret.clone()),
            Provider::Google,
        )
    }

    async fn apple_client(
        &self,
        apple: &Apple,
        client_secret: Option<String>,
    ) -> anyhow::Result<OidcClient> {
        let metadata = self.metadata(APPLE_ISSUER, &self.apple_metadata).await?;
        self.oidc_client(metadata, &apple.client_id, client_secret, Provider::Apple)
    }

    fn oidc_client(
        &self,
        metadata: Arc<CoreProviderMetadata>,
        client_id: &str,
        client_secret: Option<String>,
        provider: Provider,
    ) -> anyhow::Result<OidcClient> {
        Ok(CoreClient::from_provider_metadata(
            (*metadata).clone(),
            openidconnect::ClientId::new(client_id.to_owned()),
            client_secret.map(openidconnect::ClientSecret::new),
        )
        .set_redirect_uri(openidconnect::RedirectUrl::new(
            self.redirect_url(provider)?.to_string(),
        )?))
    }

    /// Sağlayıcıların anahtar kümesi dönüşümlü değişir; bu yüzden keşif belgesi saatte bir yenilenir.
    async fn metadata(
        &self,
        issuer: &str,
        cache: &MetadataCache,
    ) -> anyhow::Result<Arc<CoreProviderMetadata>> {
        if let Some((fetched, metadata)) = &*cache.read().await
            && fetched.elapsed() < METADATA_TTL
        {
            return Ok(metadata.clone());
        }
        let metadata = Arc::new(
            CoreProviderMetadata::discover_async(IssuerUrl::new(issuer.into())?, &self.http)
                .await
                .with_context(|| format!("{issuer} keşif belgesi alınamadı"))?,
        );
        *cache.write().await = Some((std::time::Instant::now(), metadata.clone()));
        Ok(metadata)
    }

    /// Apple'ın istediği istemci sırrı: Team ID'nin Services ID adına ES256 ile imzaladığı kısa ömürlü JWT.
    fn apple_client_secret(&self, apple: &Apple) -> anyhow::Result<String> {
        apple_client_secret(apple, now())
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

/// Doğrulanmış kimlik jetonundan ortak kimlik. Apple'da ad jetonda yer almaz (`with_name = false`).
fn identity_from_claims(claims: &CoreIdTokenClaims, with_name: bool) -> ProviderIdentity {
    ProviderIdentity {
        subject: claims.subject().as_str().to_owned(),
        email: claims.email().map(|e| e.as_str().to_lowercase()),
        email_verified: claims.email_verified().unwrap_or(false),
        display_name: with_name
            .then(|| claims.name().and_then(|n| n.get(None)))
            .flatten()
            .map(|n| n.as_str().to_owned()),
    }
}

/// `header.claims.imza`; ES256 imzası JWS'e göre ham 64 baytlık r||s'dir (DER değil).
fn apple_client_secret(apple: &Apple, issued_at: i64) -> anyhow::Result<String> {
    let header = serde_json::json!({ "alg": "ES256", "kid": apple.key_id, "typ": "JWT" });
    let claims = serde_json::json!({
        "iss": apple.team_id,
        "iat": issued_at,
        "exp": issued_at + APPLE_SECRET_TTL_SECONDS,
        "aud": APPLE_ISSUER,
        "sub": apple.client_id,
    });
    let signing_input = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?),
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims)?)
    );
    let signature: Signature = apple.key.sign(signing_input.as_bytes());
    Ok(format!(
        "{signing_input}.{}",
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    ))
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
            None,
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
                    link_user: None,
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
        let linked = oauth.sign("oauth-state", &claims(Some(user))).unwrap();
        let parsed: StateClaims = oauth.verify("oauth-state", &linked).unwrap();
        assert_eq!(parsed.link_user, Some(user));
        // Giriş akışının belirtecinde alan hiç yazılmaz, okununca `None` olur.
        let sign_in = oauth.sign("oauth-state", &claims(None)).unwrap();
        let payload = sign_in.split_once('.').unwrap().0;
        let json = String::from_utf8(URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
        assert!(!json.contains("link_user"));
        let parsed: StateClaims = oauth.verify("oauth-state", &sign_in).unwrap();
        assert_eq!(parsed.link_user, None);
    }

    #[tokio::test]
    async fn discord_start_builds_pkce_url() {
        let started = oauth().start(Provider::Discord, None).await.unwrap();
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

    /// Yalnızca test için üretilmiş P-256 anahtarı (hiçbir hesaba bağlı değil).
    const TEST_APPLE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgGPlxYySc5HprtN7V
e2n6TOBrOL33Vsw+aWK9v+FH+LyhRANCAAR7DduJib/86+d3N/RybH/xwpn/syCr
4PeV4IxNLczN1rLkY6NTc02paCmR6KjscWJdL31GaPOm2l3gfNhzFC72
-----END PRIVATE KEY-----";

    fn apple_oauth() -> OAuth {
        OAuth::new(
            Some("https://playthru.app".into()),
            None,
            None,
            Some(AppleConfig {
                client_id: "app.playthru.web".into(),
                team_id: "TEAM123456".into(),
                key_id: "KEY1234567".into(),
                private_key_pem: TEST_APPLE_KEY.into(),
            }),
            b"0123456789abcdef0123456789abcdef",
        )
        .unwrap()
    }

    #[test]
    fn apple_client_secret_is_a_valid_es256_jwt() {
        use p256::ecdsa::{VerifyingKey, signature::Verifier};

        let oauth = apple_oauth();
        let apple = oauth.apple.as_ref().unwrap();
        let jwt = apple_client_secret(apple, 1_800_000_000).unwrap();
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3);

        let decode = |part: &str| -> serde_json::Value {
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part).unwrap()).unwrap()
        };
        let header = decode(parts[0]);
        assert_eq!(header["alg"], "ES256");
        assert_eq!(header["kid"], "KEY1234567");
        let claims = decode(parts[1]);
        assert_eq!(claims["iss"], "TEAM123456");
        assert_eq!(claims["sub"], "app.playthru.web");
        assert_eq!(claims["aud"], "https://appleid.apple.com");
        assert_eq!(claims["iat"], 1_800_000_000);
        assert_eq!(claims["exp"], 1_800_000_000 + APPLE_SECRET_TTL_SECONDS);

        let signature_bytes = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();
        assert_eq!(
            signature_bytes.len(),
            64,
            "JWS ES256 imzası ham r||s olmalı"
        );
        let signature = Signature::from_slice(&signature_bytes).unwrap();
        let verifying_key = VerifyingKey::from(&apple.key);
        assert!(
            verifying_key
                .verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &signature)
                .is_ok()
        );
    }

    #[test]
    fn apple_rejects_a_bad_private_key() {
        let result = OAuth::new(
            Some("https://playthru.app".into()),
            None,
            None,
            Some(AppleConfig {
                client_id: "a".into(),
                team_id: "b".into(),
                key_id: "c".into(),
                private_key_pem:
                    "-----BEGIN PRIVATE KEY-----\nbm90IGEga2V5\n-----END PRIVATE KEY-----".into(),
            }),
            b"0123456789abcdef0123456789abcdef",
        );
        assert!(result.is_err());
    }

    /// Apple'ın gerçek keşif belgesine bağlanır: `cargo test -- --ignored apple_start`.
    #[tokio::test]
    #[ignore = "ağ erişimi ister"]
    async fn apple_start_builds_form_post_url() {
        let started = apple_oauth().start(Provider::Apple, None).await.unwrap();
        assert!(
            started
                .authorization_url
                .starts_with("https://appleid.apple.com/auth/authorize?")
        );
        let query = url_query(&started.authorization_url);
        assert!(query.contains("response_type=code"));
        assert!(query.contains("response_mode=form_post"));
        assert!(query.contains("client_id=app.playthru.web"));
        assert!(query.contains("nonce="));
        assert!(
            query.contains("redirect_uri=https%3A%2F%2Fplaythru.app%2Fauth%2Fapple%2Fcallback")
        );
        assert!(query.contains("scope=openid+name+email"));
    }

    fn url_query(url: &str) -> String {
        url.split_once('?')
            .map(|(_, q)| q.to_owned())
            .unwrap_or_default()
    }
}
