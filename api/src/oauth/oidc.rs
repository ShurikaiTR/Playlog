//! Google ve Apple'ın ortak OpenID Connect parçaları: keşif belgesi, istemci ve kimlik jetonu.

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use openidconnect::{
    IssuerUrl,
    core::{CoreClient, CoreIdTokenClaims, CoreProviderMetadata},
};
use tokio::sync::RwLock;

use super::{OAuth, Provider, ProviderIdentity};

const METADATA_TTL: Duration = Duration::from_secs(3600);

pub(super) type MetadataCache = RwLock<Option<(std::time::Instant, Arc<CoreProviderMetadata>)>>;

/// Keşif belgesinden kurulan, yönlendirme adresi ayarlı OIDC istemcisi.
pub(super) type OidcClient = CoreClient<
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;

impl OAuth {
    pub(super) fn oidc_client(
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
    pub(super) async fn metadata(
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
}

/// Doğrulanmış kimlik jetonundan ortak kimlik. Apple'da ad jetonda yer almaz (`with_name = false`).
pub(super) fn identity_from_claims(
    claims: &CoreIdTokenClaims,
    with_name: bool,
) -> ProviderIdentity {
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
