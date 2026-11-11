//! Steam girişi (OpenID 2.0). Kod değişimi yoktur: dönüşteki `openid.*` alanları Steam'e
//! doğrulatılır. Steam e-posta vermez; ad ve avatar Steam Web API'den alınır.

use std::collections::HashMap;

use anyhow::{Context, anyhow};
use oauth2::CsrfToken;
use serde::Deserialize;

use super::{Authorization, OAuth, Provider, ProviderIdentity};

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

impl OAuth {
    pub(super) fn steam_authorize(&self) -> anyhow::Result<Authorization> {
        let state = CsrfToken::new_random().secret().clone();
        let url = self.steam_login_url(&state)?;
        Ok(Authorization {
            url,
            state,
            nonce: String::new(),
        })
    }

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
    pub(super) async fn steam_verify(
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
    pub(super) async fn steam_identity(
        &self,
        steam_id: String,
    ) -> anyhow::Result<ProviderIdentity> {
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

#[cfg(test)]
mod tests {
    use super::{
        super::{
            signed::{STATE_CONTEXT, StateClaims},
            testing::oauth,
        },
        *,
    };

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
        let claims: StateClaims = oauth.verify(STATE_CONTEXT, &started.state_token).unwrap();
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
}
