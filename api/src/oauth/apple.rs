//! Apple ile giriş (OpenID Connect). İstemci sırrı sabit değildir: her kod değişiminde .p8
//! anahtarıyla imzalanan kısa ömürlü bir JWT üretilir.

use anyhow::Context;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use oauth2::{AuthType, AuthorizationCode, CsrfToken, Scope};
use openidconnect::{Nonce, core::CoreAuthenticationFlow};
use p256::{
    ecdsa::{Signature, SigningKey, signature::Signer},
    pkcs8::DecodePrivateKey,
};

use super::{
    Authorization, OAuth, Provider, ProviderIdentity, now,
    oidc::{OidcClient, identity_from_claims},
};

const APPLE_ISSUER: &str = "https://appleid.apple.com";
/// Apple istemci sırrı (JWT) bu kadar geçerli; her kod değişiminde yenisi üretilir.
const APPLE_SECRET_TTL_SECONDS: i64 = 300;

/// Apple Developer hesabından: Services ID (`client_id`), Team ID, Sign in with Apple anahtarının
/// kimliği ve .p8 dosyasının içeriği.
pub struct AppleConfig {
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key_pem: String,
}

pub(super) struct Apple {
    client_id: String,
    team_id: String,
    key_id: String,
    key: SigningKey,
}

impl Apple {
    pub(super) fn from_config(config: AppleConfig) -> anyhow::Result<Self> {
        Ok(Self {
            key: SigningKey::from_pkcs8_pem(config.private_key_pem.trim())
                .context("APPLE_PRIVATE_KEY geçerli bir .p8 (PKCS#8 PEM) anahtarı değil")?,
            client_id: config.client_id,
            team_id: config.team_id,
            key_id: config.key_id,
        })
    }
}

impl OAuth {
    pub(super) async fn apple_authorize(&self) -> anyhow::Result<Authorization> {
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
        Ok(Authorization {
            url,
            state: csrf.secret().clone(),
            nonce: nonce.secret().clone(),
        })
    }

    /// Kodu belirteçle değiştirir ve kimlik jetonunu (imza, nonce) doğrular.
    pub(super) async fn apple_identity(
        &self,
        code: String,
        nonce: String,
    ) -> anyhow::Result<ProviderIdentity> {
        let apple = self.apple.as_ref().context("Apple yapılandırılmamış")?;
        let client = self
            .apple_client(apple, Some(apple_client_secret(apple, now())?))
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
            .claims(&client.id_token_verifier(), &Nonce::new(nonce))
            .context("Apple kimlik jetonu doğrulanamadı")?;
        // Apple adı kimlik jetonuna koymaz; yalnızca ilk girişte form alanında gönderir.
        Ok(identity_from_claims(id_claims, false))
    }

    async fn apple_client(
        &self,
        apple: &Apple,
        client_secret: Option<String>,
    ) -> anyhow::Result<OidcClient> {
        let metadata = self.metadata(APPLE_ISSUER, &self.apple_metadata).await?;
        self.oidc_client(metadata, &apple.client_id, client_secret, Provider::Apple)
    }
}

/// Apple'ın istediği istemci sırrı: Team ID'nin Services ID adına ES256 ile imzaladığı kısa ömürlü JWT.
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

#[cfg(test)]
mod tests {
    use super::{
        super::testing::{apple_oauth, url_query},
        *,
    };

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
}
