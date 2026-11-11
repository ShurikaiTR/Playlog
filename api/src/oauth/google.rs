//! Google girişi (OpenID Connect, PKCE ile).

use anyhow::Context;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope};
use openidconnect::{Nonce, core::CoreAuthenticationFlow};

use super::{
    Authorization, OAuth, Provider, ProviderIdentity,
    oidc::{OidcClient, identity_from_claims},
};

const GOOGLE_ISSUER: &str = "https://accounts.google.com";

impl OAuth {
    pub(super) async fn google_authorize(
        &self,
        challenge: PkceCodeChallenge,
    ) -> anyhow::Result<Authorization> {
        let client = self.google_client().await?;
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
        Ok(Authorization {
            url,
            state: csrf.secret().clone(),
            nonce: nonce.secret().clone(),
        })
    }

    /// Kodu belirteçle değiştirir ve kimlik jetonunu (imza, nonce) doğrular.
    pub(super) async fn google_identity(
        &self,
        code: String,
        verifier: PkceCodeVerifier,
        nonce: String,
    ) -> anyhow::Result<ProviderIdentity> {
        let client = self.google_client().await?;
        let tokens = client
            .exchange_code(AuthorizationCode::new(code))?
            .set_pkce_verifier(verifier)
            .request_async(&self.http)
            .await
            .context("Google kod değişimi başarısız")?;
        let id_token = openidconnect::TokenResponse::id_token(&tokens)
            .context("Google kimlik jetonu dönmedi")?;
        let id_claims = id_token
            .claims(&client.id_token_verifier(), &Nonce::new(nonce))
            .context("Google kimlik jetonu doğrulanamadı")?;
        Ok(identity_from_claims(id_claims, true))
    }

    async fn google_client(&self) -> anyhow::Result<OidcClient> {
        let creds = self.google.as_ref().context("Google yapılandırılmamış")?;
        let metadata = self.metadata(GOOGLE_ISSUER, &self.google_metadata).await?;
        self.oidc_client(
            metadata,
            &creds.client_id,
            Some(creds.client_secret.clone()),
            Provider::Google,
        )
    }
}
