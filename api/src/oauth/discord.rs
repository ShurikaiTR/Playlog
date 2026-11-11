//! Discord girişi (OAuth 2.0, PKCE ile). Kimlik, erişim belirteciyle `/users/@me` uç noktasından okunur.

use anyhow::{Context, anyhow};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, Scope, TokenResponse as _, TokenUrl, basic::BasicClient,
};
use serde::Deserialize;

use super::{Authorization, OAuth, Provider, ProviderIdentity};

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URL: &str = "https://discord.com/api/users/@me";

impl OAuth {
    pub(super) fn discord_authorize(
        &self,
        challenge: PkceCodeChallenge,
    ) -> anyhow::Result<Authorization> {
        let client = self.discord_client()?;
        let (url, csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".into()))
            .add_scope(Scope::new("email".into()))
            .set_pkce_challenge(challenge)
            .url();
        Ok(Authorization {
            url,
            state: csrf.secret().clone(),
            nonce: String::new(),
        })
    }

    pub(super) async fn discord_identity(
        &self,
        code: String,
        verifier: PkceCodeVerifier,
    ) -> anyhow::Result<ProviderIdentity> {
        let client = self.discord_client()?;
        let tokens = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(verifier)
            .request_async(&self.http)
            .await
            .context("Discord kod değişimi başarısız")?;
        self.discord_user(tokens.access_token().secret()).await
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
}

#[cfg(test)]
mod tests {
    use super::{
        super::testing::{oauth, url_query},
        *,
    };

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
}
