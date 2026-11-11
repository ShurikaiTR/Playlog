//! Testlerin ortak kurulumu.

use super::{AppleConfig, Credentials, OAuth};

/// Discord ve Steam açık.
pub(super) fn oauth() -> OAuth {
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

/// Yalnızca test için üretilmiş P-256 anahtarı (hiçbir hesaba bağlı değil).
const TEST_APPLE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgGPlxYySc5HprtN7V
e2n6TOBrOL33Vsw+aWK9v+FH+LyhRANCAAR7DduJib/86+d3N/RybH/xwpn/syCr
4PeV4IxNLczN1rLkY6NTc02paCmR6KjscWJdL31GaPOm2l3gfNhzFC72
-----END PRIVATE KEY-----";

pub(super) fn apple_oauth() -> OAuth {
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

pub(super) fn url_query(url: &str) -> String {
    url.split_once('?')
        .map(|(_, q)| q.to_owned())
        .unwrap_or_default()
}
