//! Google ve Apple (OpenID Connect), Discord (OAuth 2.0) ve Steam (OpenID 2.0) girişi.
//!
//! Akış durumsuzdur: `start` çağrısı state, nonce ve PKCE doğrulayıcısını HMAC ile
//! imzalı kısa ömürlü bir belirtece koyar; SvelteKit bunu tarayıcıda httpOnly çerezde
//! saklar ve dönüşte API'ye geri verir. Sunucuda geçici kayıt tutulmaz.

use std::{collections::HashMap, sync::Arc, time::Duration};

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
const STEAM_OPENID_URL: &str = "https://steamcommunity.com/openid/login";
const STEAM_OPENID_NS: &str = "http://specs.openid.net/auth/2.0";
const STEAM_IDENTIFIER_SELECT: &str = "http://specs.openid.net/auth/2.0/identifier_select";
const STEAM_ID_PREFIX: &str = "https://steamcommunity.com/openid/id/";
const STEAM_SUMMARY_URL: &str = "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/";
/// Steam imzalı yanıtta bunların hepsi bulunmalı; yoksa doğrulama anlamsız olur.
const STEAM_REQUIRED_SIGNED: &[&str] = &[
    "op_endpoint",
    "claimed_id",
    "identity",
    "return_to",
    "response_nonce",
    "assoc_handle",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    Google,
    Discord,
    Apple,
    Steam,
}

impl Provider {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "google" => Some(Self::Google),
            "discord" => Some(Self::Discord),
            "apple" => Some(Self::Apple),
            "steam" => Some(Self::Steam),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Discord => "discord",
            Self::Apple => "apple",
            Self::Steam => "steam",
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
    pub avatar_url: Option<String>,
}

pub struct OAuth {
    public_url: String,
    google: Option<Credentials>,
    discord: Option<Credentials>,
    apple: Option<Apple>,
    /// Steam Web API anahtarı (STEAM_API_KEY); tanımlıysa Steam girişi açıktır.
    steam_api_key: Option<String>,
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
    /// Steam e-posta vermez; o hesap e-postasız açılır.
    #[serde(default)]
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
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
        steam_api_key: Option<String>,
        sign_key: &[u8],
    ) -> anyhow::Result<Self> {
        let public_url = public_url
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_owned();
        if (google.is_some() || discord.is_some() || apple.is_some() || steam_api_key.is_some())
            && public_url.is_empty()
        {
            anyhow::bail!(
                "Google/Discord/Apple/Steam için PUBLIC_URL tanımlı olmalı (örn. https://playthru.app)"
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
            steam_api_key,
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
            Provider::Steam => self.steam_api_key.is_some(),
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
            Provider::Steam => {
                let state = CsrfToken::new_random().secret().clone();
                let url = self.steam_login_url(&state)?;
                (url, state, String::new())
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
        openid: &HashMap<String, String>,
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
            Provider::Steam => {
                // Steam'de kod yoktur; dönüş `openid.*` alanlarıdır ve Steam'e doğrulatılır.
                let steam_id = self.steam_verify(state, openid).await?;
                self.steam_identity(steam_id).await?
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
            avatar_url: None,
        })
    }

    // ---------- Steam (OpenID 2.0) ----------

    /// Steam giriş sayfası adresi. `return_to` içine state konur; Steam bu adresi imzalı yanıtta geri verir.
    fn steam_login_url(&self, state: &str) -> anyhow::Result<reqwest::Url> {
        let return_to = steam_return_to(&self.redirect_url(Provider::Steam)?.to_string(), state);
        Ok(reqwest::Url::parse_with_params(
            STEAM_OPENID_URL,
            [
                ("openid.ns", STEAM_OPENID_NS),
                ("openid.mode", "checkid_setup"),
                ("openid.return_to", return_to.as_str()),
                ("openid.realm", self.public_url.as_str()),
                ("openid.identity", STEAM_IDENTIFIER_SELECT),
                ("openid.claimed_id", STEAM_IDENTIFIER_SELECT),
            ],
        )?)
    }

    /// Dönüşü doğrular ve SteamID64'ü döner: yerel kontroller, sonra Steam'e `check_authentication`.
    async fn steam_verify(
        &self,
        state: &str,
        params: &HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let return_to = steam_return_to(&self.redirect_url(Provider::Steam)?.to_string(), state);
        let steam_id = steam_check_response(params, &return_to)?;

        let mut form: Vec<(&str, &str)> = params
            .iter()
            .filter(|(key, _)| key.starts_with("openid."))
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        form.retain(|(key, _)| *key != "openid.mode");
        form.push(("openid.mode", "check_authentication"));

        let body = self
            .api_http
            .post(STEAM_OPENID_URL)
            .form(&form)
            .send()
            .await
            .context("Steam doğrulama isteği başarısız")?
            .error_for_status()
            .map_err(|e| anyhow!("Steam doğrulaması hata döndürdü: {e}"))?
            .text()
            .await?;
        if !steam_is_valid(&body) {
            return Err(anyhow!("Steam yanıtı doğrulanamadı"));
        }
        Ok(steam_id)
    }

    /// Görünen ad ve avatar Steam Web API'den alınır; alınamazsa giriş yine de sürer.
    async fn steam_identity(&self, steam_id: String) -> anyhow::Result<ProviderIdentity> {
        let profile = match self.steam_api_key.as_deref() {
            Some(key) => self
                .steam_profile(key, &steam_id)
                .await
                .inspect_err(|err| tracing::warn!(error = %err, "Steam profili alınamadı"))
                .ok()
                .flatten(),
            None => None,
        };
        let (display_name, avatar_url) = profile.unwrap_or_default();
        Ok(ProviderIdentity {
            subject: steam_id,
            email: None,
            email_verified: false,
            display_name,
            avatar_url,
        })
    }

    async fn steam_profile(
        &self,
        key: &str,
        steam_id: &str,
    ) -> anyhow::Result<Option<(Option<String>, Option<String>)>> {
        #[derive(Deserialize)]
        struct Summaries {
            response: Players,
        }
        #[derive(Deserialize)]
        struct Players {
            #[serde(default)]
            players: Vec<Player>,
        }
        #[derive(Deserialize)]
        struct Player {
            steamid: String,
            personaname: Option<String>,
            avatarfull: Option<String>,
        }
        // Anahtar adreste gider; hata mesajında görünmesin diye adres hatadan çıkarılır.
        let summaries: Summaries = self
            .api_http
            .get(reqwest::Url::parse_with_params(
                STEAM_SUMMARY_URL,
                [("key", key), ("steamids", steam_id)],
            )?)
            .send()
            .await
            .map_err(|e| anyhow!("Steam profil isteği başarısız: {}", e.without_url()))?
            .error_for_status()
            .map_err(|e| anyhow!("Steam profil isteği hata döndürdü: {}", e.without_url()))?
            .json()
            .await
            .map_err(|e| anyhow!("Steam profil yanıtı okunamadı: {}", e.without_url()))?;
        Ok(summaries
            .response
            .players
            .into_iter()
            .find(|player| player.steamid == steam_id)
            .map(|player| {
                (
                    player.personaname,
                    player.avatarfull.filter(|url| steam_avatar_ok(url)),
                )
            }))
    }
}

/// Steam'in yönlendireceği adres; state buraya gömülür ve imzalı yanıtla doğrulanır.
fn steam_return_to(redirect_url: &str, state: &str) -> String {
    format!("{redirect_url}?state={state}")
}

/// Yerel kontroller: yanıt bir onay (`id_res`), bizim adresimize ve Steam'e ait, gerekli alanlar imzalı.
/// Geçerliyse SteamID64'ü döner. İmzanın kendisini yalnızca Steam doğrulayabilir.
fn steam_check_response(
    params: &HashMap<String, String>,
    expected_return_to: &str,
) -> anyhow::Result<String> {
    let get = |key: &str| params.get(key).map(String::as_str);
    if get("openid.ns") != Some(STEAM_OPENID_NS) || get("openid.mode") != Some("id_res") {
        return Err(anyhow!("Steam yanıtı bir onay değil"));
    }
    if get("openid.op_endpoint") != Some(STEAM_OPENID_URL) {
        return Err(anyhow!("Steam yanıtı beklenen uçtan gelmedi"));
    }
    if get("openid.return_to") != Some(expected_return_to) {
        return Err(anyhow!("Steam yanıtının return_to değeri uyuşmuyor"));
    }
    let signed: Vec<&str> = get("openid.signed")
        .unwrap_or_default()
        .split(',')
        .collect();
    if let Some(missing) = STEAM_REQUIRED_SIGNED.iter().find(|f| !signed.contains(f)) {
        return Err(anyhow!("Steam yanıtında {missing} imzalı değil"));
    }
    let claimed = get("openid.claimed_id").unwrap_or_default();
    if get("openid.identity") != Some(claimed) {
        return Err(anyhow!("Steam kimlik alanları uyuşmuyor"));
    }
    let id = claimed
        .strip_prefix(STEAM_ID_PREFIX)
        .filter(|id| (1..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit()))
        .ok_or_else(|| anyhow!("Steam kimliği geçersiz"))?;
    Ok(id.to_owned())
}

/// `check_authentication` yanıtı satır satır `anahtar:değer`'dir; kabul için `is_valid:true` gerekir.
fn steam_is_valid(body: &str) -> bool {
    body.lines().any(|line| line.trim() == "is_valid:true")
}

/// Avatar adresi veritabanına ve sayfalara girer: yalnızca Steam'in CDN'inden https adresleri kabul edilir.
fn steam_avatar_ok(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|url| {
        url.scheme() == "https"
            && url
                .host_str()
                .is_some_and(|host| host == "steamstatic.com" || host.ends_with(".steamstatic.com"))
    })
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
        avatar_url: None,
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
            Some("steamkey".into()),
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
            None,
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
            None,
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

    #[tokio::test]
    async fn steam_start_builds_openid_url() {
        let oauth = oauth();
        assert!(oauth.enabled(Provider::Steam));
        let started = oauth.start(Provider::Steam, None).await.unwrap();
        assert!(
            started
                .authorization_url
                .starts_with("https://steamcommunity.com/openid/login?")
        );
        let url = reqwest::Url::parse(&started.authorization_url).unwrap();
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["openid.mode"], "checkid_setup");
        assert_eq!(query["openid.ns"], STEAM_OPENID_NS);
        assert_eq!(query["openid.realm"], "https://playthru.app");
        assert_eq!(query["openid.identity"], STEAM_IDENTIFIER_SELECT);
        let return_to = &query["openid.return_to"];
        assert!(return_to.starts_with("https://playthru.app/auth/steam/callback?state="));
        // State, imzalı belirteçtekiyle aynı olmalı.
        let claims: StateClaims = oauth.verify("oauth-state", &started.state_token).unwrap();
        assert_eq!(
            return_to,
            &steam_return_to("https://playthru.app/auth/steam/callback", &claims.state)
        );
    }

    #[test]
    fn steam_is_disabled_without_an_api_key() {
        let oauth = OAuth::new(
            Some("https://playthru.app".into()),
            None,
            None,
            None,
            None,
            b"0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        assert!(!oauth.enabled(Provider::Steam));
    }

    const RETURN_TO: &str = "https://playthru.app/auth/steam/callback?state=abc";

    fn steam_params() -> HashMap<String, String> {
        [
            ("openid.ns", STEAM_OPENID_NS),
            ("openid.mode", "id_res"),
            ("openid.op_endpoint", STEAM_OPENID_URL),
            (
                "openid.claimed_id",
                "https://steamcommunity.com/openid/id/76561198000000000",
            ),
            (
                "openid.identity",
                "https://steamcommunity.com/openid/id/76561198000000000",
            ),
            ("openid.return_to", RETURN_TO),
            ("openid.response_nonce", "2026-11-09T16:00:00Zabc"),
            ("openid.assoc_handle", "1234567890"),
            (
                "openid.signed",
                "signed,op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle",
            ),
            ("openid.sig", "sig"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
    }

    #[test]
    fn steam_response_checks_pass_for_a_genuine_shape() {
        assert_eq!(
            steam_check_response(&steam_params(), RETURN_TO).unwrap(),
            "76561198000000000"
        );
    }

    #[test]
    fn steam_response_checks_reject_tampering() {
        let tamper = |key: &str, value: &str| {
            let mut params = steam_params();
            params.insert(key.into(), value.into());
            steam_check_response(&params, RETURN_TO).is_err()
        };
        // Başka bir state'e (başka bir akışa) ait yanıt.
        assert!(
            steam_check_response(
                &steam_params(),
                "https://playthru.app/auth/steam/callback?state=zzz"
            )
            .is_err()
        );
        assert!(tamper("openid.mode", "cancel"));
        assert!(tamper(
            "openid.op_endpoint",
            "https://evil.example/openid/login"
        ));
        assert!(tamper(
            "openid.claimed_id",
            "https://evil.example/openid/id/76561198000000000"
        ));
        assert!(tamper(
            "openid.claimed_id",
            "https://steamcommunity.com/openid/id/12abc"
        ));
        // identity ile claimed_id aynı olmalı.
        assert!(tamper(
            "openid.identity",
            "https://steamcommunity.com/openid/id/1"
        ));
        // return_to imzalı değilse state'e güvenilemez.
        assert!(tamper(
            "openid.signed",
            "signed,op_endpoint,claimed_id,identity"
        ));
        let mut missing = steam_params();
        missing.remove("openid.ns");
        assert!(steam_check_response(&missing, RETURN_TO).is_err());
    }

    #[test]
    fn steam_is_valid_needs_the_exact_line() {
        assert!(steam_is_valid(
            "ns:http://specs.openid.net/auth/2.0\nis_valid:true\n"
        ));
        assert!(!steam_is_valid(
            "ns:http://specs.openid.net/auth/2.0\nis_valid:false\n"
        ));
        assert!(!steam_is_valid(""));
    }

    #[test]
    fn steam_avatar_must_be_https_on_steams_cdn() {
        assert!(steam_avatar_ok(
            "https://avatars.steamstatic.com/abc_full.jpg"
        ));
        assert!(steam_avatar_ok(
            "https://avatars.fastly.steamstatic.com/abc_full.jpg"
        ));
        assert!(!steam_avatar_ok(
            "http://avatars.steamstatic.com/abc_full.jpg"
        ));
        assert!(!steam_avatar_ok("https://evilsteamstatic.com/abc.jpg"));
        assert!(!steam_avatar_ok(
            "https://steamstatic.com.evil.example/abc.jpg"
        ));
        assert!(!steam_avatar_ok("javascript:alert(1)"));
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

    fn url_query(url: &str) -> String {
        url.split_once('?')
            .map(|(_, q)| q.to_owned())
            .unwrap_or_default()
    }
}
