//! Google ve Apple (OpenID Connect), Discord (OAuth 2.0) ve Steam (OpenID 2.0) girişi.
//!
//! Akış durumsuzdur: `start` çağrısı state, nonce ve PKCE doğrulayıcısını HMAC ile
//! imzalı kısa ömürlü bir belirtece koyar; SvelteKit bunu tarayıcıda httpOnly çerezde
//! saklar ve dönüşte API'ye geri verir. Sunucuda geçici kayıt tutulmaz.

mod apple;
mod discord;
mod google;
mod oidc;
mod signed;
mod steam;
#[cfg(test)]
mod testing;

use std::{collections::HashMap, time::Duration};

use oauth2::{PkceCodeChallenge, PkceCodeVerifier, RedirectUrl};
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

use self::{
    apple::Apple,
    oidc::MetadataCache,
    signed::{STATE_CONTEXT, StateClaims},
};
pub use self::{apple::AppleConfig, signed::PendingClaims};

const STATE_TTL_SECONDS: i64 = 600;

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

/// Doğrulanmış dönüş: sağlayıcı kimliği ve akış bağlama içinse bağlanacak kullanıcı.
pub struct Finished {
    pub identity: ProviderIdentity,
    pub link_user: Option<uuid::Uuid>,
}

pub struct Started {
    pub authorization_url: String,
    pub state_token: String,
}

/// Bir sağlayıcının giriş sayfası adresi ve akışı dönüşte doğrulamak için saklanacak değerler.
/// `nonce` yalnızca OpenID Connect sağlayıcılarında (Google, Apple) doludur.
struct Authorization {
    url: reqwest::Url,
    state: String,
    nonce: String,
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
        let apple = apple.map(Apple::from_config).transpose()?;
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

    /// `link_user`: hesap ayarlarından bağlama akışı başlatan kullanıcı; girişte `None`.
    pub async fn start(
        &self,
        provider: Provider,
        link_user: Option<uuid::Uuid>,
    ) -> anyhow::Result<Started> {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let authorization = match provider {
            Provider::Google => self.google_authorize(challenge).await?,
            Provider::Discord => self.discord_authorize(challenge)?,
            Provider::Apple => self.apple_authorize().await?,
            Provider::Steam => self.steam_authorize()?,
        };

        let state_token = self.sign(
            STATE_CONTEXT,
            &StateClaims {
                provider: provider.as_str().into(),
                state: authorization.state,
                nonce: authorization.nonce,
                verifier: verifier.secret().clone(),
                exp: now() + STATE_TTL_SECONDS,
                link_user,
            },
        )?;
        Ok(Started {
            authorization_url: authorization.url.to_string(),
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
        let Some(claims) = self.verify::<StateClaims>(STATE_CONTEXT, state_token) else {
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
            Provider::Google => self.google_identity(code, verifier, claims.nonce).await?,
            Provider::Discord => self.discord_identity(code, verifier).await?,
            Provider::Apple => self.apple_identity(code, claims.nonce).await?,
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
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
