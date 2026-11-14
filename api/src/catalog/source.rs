//! Dış kaynaklardan oyun verisi.
//!
//! * Wikidata: ana liste ve kimlik. Ad, çıkış tarihi, platformlar, seri, tür, geliştirici/yayıncı
//!   ve mağaza kimlikleri (Steam, GOG, PlayStation, Xbox, Nintendo).
//! * Steam (oyunun Steam kimliği varsa): açıklama, tür, tarih, diller, görseller (kapak, arka plan,
//!   logo), yaş sınırı, sistem gereksinimleri.
//!
//! Yalnızca yapısal kaynaklar kullanılır; hiçbir şey üretilmez. Çıktı `GameInput` biçiminde bir JSON'dur.

use std::{collections::HashMap, time::Duration};

use chrono::{Datelike, NaiveDate};
use serde_json::{Map, Value, json};

const USER_AGENT: &str = "playthru-catalog-importer/0.1 (https://playthru.app)";
const STEAM_ASSETS: &str = "https://shared.akamai.steamstatic.com/store_item_assets/";
/// Steam mağaza API'si yaklaşık 5 dakikada 200 isteğe izin verir; toplu işlemde oyunlar arası bekleme.
pub const STEAM_DELAY: Duration = Duration::from_millis(1600);
/// Wikidata sorguları bir isteğin bütçesinden uzun sürebilir; arayüzden gelenlerde kısa tutulur.
pub const QUICK: Duration = Duration::from_secs(8);
pub const PATIENT: Duration = Duration::from_secs(55);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceError {
    /// Steam'de böyle bir uygulama yok ya da mağazadan kalkmış.
    NotFound,
    /// Uygulama var ama oyun değil (DLC, araç, film...).
    NotAGame,
    /// Kaynak yanıt vermedi ya da istek sınırına takıldı.
    Unavailable,
}

impl SourceError {
    pub fn code(self) -> &'static str {
        match self {
            Self::NotFound => "steam_app_not_found",
            Self::NotAGame => "steam_app_not_a_game",
            Self::Unavailable => "source_unavailable",
        }
    }
}

impl From<reqwest::Error> for SourceError {
    fn from(err: reqwest::Error) -> Self {
        tracing::warn!(error = %err, "katalog kaynağına ulaşılamadı");
        Self::Unavailable
    }
}

/// Wikidata'dan bir oyun için gelenler.
#[derive(Default, Clone, Debug)]
pub struct Wiki {
    pub qid: String,
    pub label: Option<String>,
    pub released: Option<NaiveDate>,
    /// `platforms` tablosundaki slug'lar.
    pub platforms: Vec<String>,
    pub series: Option<String>,
    pub genres: Vec<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    /// steam, gog, psn, xbox, nintendo -> kimlik.
    pub store_ids: Vec<(&'static str, String)>,
}

impl Wiki {
    pub fn steam_app(&self) -> Option<u32> {
        self.store_ids
            .iter()
            .find(|(source, _)| *source == "steam")
            .and_then(|(_, id)| id.parse().ok())
    }
}

pub struct Steam {
    pub appid: u32,
    pub details: Value,
    pub assets: Option<Value>,
    /// Steam'in kütüphane logosu; her oyunda yoktur.
    pub logo: Option<String>,
}

pub struct Hit {
    /// `steam` ya da `wikidata`.
    pub source: &'static str,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct ListFilter {
    pub platform: Option<String>,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
}

pub struct Composed {
    pub input: Value,
    /// Elle bakılması gereken bir şey varsa nedeni (tarih yok, kaynaklar çelişiyor).
    pub review: Option<&'static str>,
}

pub struct Source {
    http: reqwest::Client,
}

impl Source {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(Duration::from_secs(10))
                .build()?,
        })
    }

    async fn get_json(
        &self,
        url: &str,
        query: &[(&str, &str)],
        accept: &str,
        timeout: Duration,
    ) -> Result<Value, SourceError> {
        let url =
            reqwest::Url::parse_with_params(url, query).map_err(|_| SourceError::Unavailable)?;
        let response = self
            .http
            .get(url)
            .header("accept", accept)
            .timeout(timeout)
            .send()
            .await?;
        if !response.status().is_success() {
            tracing::warn!(status = %response.status(), "katalog kaynağı hata döndürdü");
            return Err(SourceError::Unavailable);
        }
        Ok(response.json().await?)
    }

    // ---------- Arama ----------

    /// Ada göre arama: önce Steam mağazası, sonra Wikidata (Steam'de olmayan oyunlar için).
    pub async fn search(&self, term: &str) -> Vec<Hit> {
        let (steam, wiki) = tokio::join!(self.search_steam(term), self.search_wikidata(term));
        let mut hits = steam.unwrap_or_default();
        hits.extend(wiki.unwrap_or_default());
        hits
    }

    async fn search_steam(&self, term: &str) -> Result<Vec<Hit>, SourceError> {
        let data = self
            .get_json(
                "https://store.steampowered.com/api/storesearch/",
                &[("term", term), ("l", "english"), ("cc", "us")],
                "application/json",
                QUICK,
            )
            .await?;
        Ok(data["items"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item["type"] == "app")
                    .filter_map(|item| {
                        Some(Hit {
                            source: "steam",
                            id: item["id"].as_u64()?.to_string(),
                            name: item["name"].as_str()?.to_owned(),
                            description: None,
                            image: item["tiny_image"].as_str().map(str::to_owned),
                        })
                    })
                    .take(8)
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn search_wikidata(&self, term: &str) -> Result<Vec<Hit>, SourceError> {
        let data = self
            .get_json(
                "https://www.wikidata.org/w/api.php",
                &[
                    ("action", "wbsearchentities"),
                    ("search", term),
                    ("language", "en"),
                    ("type", "item"),
                    ("limit", "15"),
                    ("format", "json"),
                ],
                "application/json",
                QUICK,
            )
            .await?;
        Ok(data["search"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| {
                        item["description"]
                            .as_str()
                            .is_some_and(|d| d.to_lowercase().contains("video game"))
                    })
                    .filter_map(|item| {
                        let id = item["id"].as_str().filter(|id| is_qid(id))?;
                        Some(Hit {
                            source: "wikidata",
                            id: id.to_owned(),
                            name: item["label"].as_str()?.to_owned(),
                            description: item["description"].as_str().map(str::to_owned),
                            image: None,
                        })
                    })
                    .take(6)
                    .collect()
            })
            .unwrap_or_default())
    }

    // ---------- Steam ----------

    /// Bir Steam uygulamasının mağaza kaydı ve görselleri; oyun değilse hata.
    pub async fn steam(&self, appid: u32) -> Result<Steam, SourceError> {
        let id = appid.to_string();
        let data = self
            .get_json(
                "https://store.steampowered.com/api/appdetails",
                &[("appids", &id), ("l", "english"), ("cc", "us")],
                "application/json",
                QUICK,
            )
            .await?;
        let entry = &data[&id];
        if entry["success"] != true {
            return Err(SourceError::NotFound);
        }
        if entry["data"]["type"] != "game" {
            return Err(SourceError::NotAGame);
        }
        let (assets, logo) = tokio::join!(self.steam_assets(appid), self.steam_logo(appid));
        Ok(Steam {
            appid,
            details: entry["data"].clone(),
            assets,
            logo,
        })
    }

    /// `logo.png` yalnızca varsa döner (varlık denetimi için HEAD isteği).
    async fn steam_logo(&self, appid: u32) -> Option<String> {
        let url = format!("{STEAM_ASSETS}steam/apps/{appid}/logo.png");
        let response = self.http.head(&url).timeout(QUICK).send().await.ok()?;
        response.status().is_success().then_some(url)
    }

    /// Görsel dosya adları; alınamazsa Steam'in kendi başlık görseline düşülür.
    async fn steam_assets(&self, appid: u32) -> Option<Value> {
        let input = json!({
            "ids": [{ "appid": appid }],
            "context": { "language": "english", "country_code": "US" },
            "data_request": { "include_assets": true },
        })
        .to_string();
        let data = self
            .get_json(
                "https://api.steampowered.com/IStoreBrowseService/GetItems/v1",
                &[("input_json", &input)],
                "application/json",
                QUICK,
            )
            .await
            .ok()?;
        let assets = data["response"]["store_items"][0]["assets"].clone();
        assets.is_object().then_some(assets)
    }

    // ---------- Wikidata ----------

    async fn sparql(
        &self,
        query: &str,
        timeout: Duration,
    ) -> Result<Vec<HashMap<String, String>>, SourceError> {
        let data = self
            .get_json(
                "https://query.wikidata.org/sparql",
                &[("query", query)],
                "application/sparql-results+json",
                timeout,
            )
            .await?;
        Ok(data["results"]["bindings"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(Value::as_object)
                    .map(|row| {
                        row.iter()
                            .filter_map(|(key, cell)| {
                                Some((key.clone(), cell["value"].as_str()?.to_owned()))
                            })
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Steam kimliğine bağlı Wikidata kaydının kimliği (birden çoksa en çok dilde yer alan).
    pub async fn qid_for_app(&self, appid: u32, timeout: Duration) -> Option<String> {
        let query = format!(
            r#"SELECT ?item WHERE {{ ?item wdt:P1733 "{appid}"; wikibase:sitelinks ?links. }}
               ORDER BY DESC(?links) LIMIT 1"#
        );
        let rows = self.sparql(&query, timeout).await.ok()?;
        let uri = rows.first()?.get("item")?;
        uri.rsplit('/').next().map(str::to_owned)
    }

    /// Wikipedia'nın en çok dilde yer verdiği oyunlar (popülerlik göstergesi), süzgece uyanlar.
    pub async fn list(
        &self,
        filter: &ListFilter,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<(String, Option<String>)>, SourceError> {
        let mut conditions = String::new();
        if let Some(slug) = &filter.platform
            && let Some((qid, _)) = PLATFORM_IDS.iter().find(|(_, s)| s == slug)
        {
            conditions.push_str(&format!("?item wdt:P400 wd:{qid}.\n"));
        }
        if filter.year_from.is_some() || filter.year_to.is_some() {
            let from = filter.year_from.unwrap_or(1950).clamp(1950, 2100);
            let to = filter.year_to.unwrap_or(2100).clamp(1950, 2100);
            conditions.push_str(&format!(
                "?item wdt:P577 ?date. FILTER(isLiteral(?date) && YEAR(?date) >= {from} && YEAR(?date) <= {to})\n"
            ));
        }
        let query = format!(
            r#"SELECT ?item ?itemLabel ?links WHERE {{
                ?item wdt:P31 wd:Q7889; wikibase:sitelinks ?links.
                {conditions}
                SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
            }} GROUP BY ?item ?itemLabel ?links ORDER BY DESC(?links) ?item LIMIT {limit} OFFSET {offset}"#
        );
        Ok(self
            .sparql(&query, PATIENT)
            .await?
            .into_iter()
            .filter_map(|row| {
                let qid = row.get("item")?.rsplit('/').next()?.to_owned();
                is_qid(&qid).then(|| {
                    let label = row.get("itemLabel").cloned().filter(|l| !is_qid(l));
                    (qid, label)
                })
            })
            .collect())
    }

    /// Wikidata kayıtlarının ayrıntıları. İki sorgu: tarih/platform/seri/kimlikler ile
    /// tür/şirketler (birlikte tek sorguda satırlar çarpılıp zaman aşımına uğrar).
    pub async fn wiki(&self, qids: &[String], timeout: Duration) -> HashMap<String, Wiki> {
        let qids: Vec<&String> = qids.iter().filter(|q| is_qid(q)).collect();
        if qids.is_empty() {
            return HashMap::new();
        }
        let values = qids
            .iter()
            .map(|q| format!("wd:{q}"))
            .collect::<Vec<_>>()
            .join(" ");
        let concat = |var: &str, name: &str| {
            format!(r#"(GROUP_CONCAT(DISTINCT {var}; separator="|") AS ?{name})"#)
        };
        let base = format!(
            r#"SELECT ?item ?itemLabel (MIN(?date) AS ?released)
                {} {} {} {} {} {} {} WHERE {{
                VALUES ?item {{ {values} }}
                OPTIONAL {{ ?item rdfs:label ?itemLabel. FILTER(LANG(?itemLabel) = "en") }}
                # "Bilinmeyen değer" ifadeleri boş düğüm olarak gelir ve MIN() ile gerçek tarihin önüne geçerdi.
                OPTIONAL {{ ?item wdt:P577 ?date. FILTER(isLiteral(?date)) }}
                OPTIONAL {{ ?item wdt:P400 ?platform. }}
                OPTIONAL {{ ?item wdt:P179 ?s. ?s rdfs:label ?seriesLabel. FILTER(LANG(?seriesLabel) = "en") }}
                OPTIONAL {{ ?item wdt:P1733 ?steamId. }}
                OPTIONAL {{ ?item wdt:P2725 ?gogId. }}
                OPTIONAL {{ ?item wdt:P12332 ?psnId. }}
                OPTIONAL {{ ?item wdt:P5885 ?xboxId. }}
                OPTIONAL {{ ?item wdt:P8084 ?nintendoId. }}
            }} GROUP BY ?item ?itemLabel"#,
            concat("STR(?platform)", "platformUris"),
            concat("?seriesLabel", "series"),
            concat("?steamId", "steam"),
            concat("?gogId", "gog"),
            concat("?psnId", "psn"),
            concat("?xboxId", "xbox"),
            concat("?nintendoId", "nintendo"),
        );
        let tags = format!(
            r#"SELECT ?item {} {} {} WHERE {{
                VALUES ?item {{ {values} }}
                OPTIONAL {{ ?item wdt:P136 ?g. ?g rdfs:label ?genreLabel. FILTER(LANG(?genreLabel) = "en") }}
                OPTIONAL {{ ?item wdt:P178 ?d. ?d rdfs:label ?devLabel. FILTER(LANG(?devLabel) = "en") }}
                OPTIONAL {{ ?item wdt:P123 ?p. ?p rdfs:label ?pubLabel. FILTER(LANG(?pubLabel) = "en") }}
            }} GROUP BY ?item"#,
            concat("?genreLabel", "genres"),
            concat("?devLabel", "devs"),
            concat("?pubLabel", "pubs"),
        );

        let (base, tags) = tokio::join!(self.sparql(&base, timeout), self.sparql(&tags, timeout));
        let mut result: HashMap<String, Wiki> = base
            .unwrap_or_default()
            .into_iter()
            .filter_map(wiki_from_row)
            .map(|wiki| (wiki.qid.clone(), wiki))
            .collect();
        for row in tags.unwrap_or_default() {
            let Some(qid) = row.get("item").and_then(|u| u.rsplit('/').next()) else {
                continue;
            };
            if let Some(wiki) = result.get_mut(qid) {
                wiki.genres = split(&row, "genres");
                wiki.developers = split(&row, "devs");
                wiki.publishers = split(&row, "pubs");
            }
        }
        result
    }
}

fn is_qid(value: &str) -> bool {
    value.len() > 1 && value.starts_with('Q') && value[1..].bytes().all(|b| b.is_ascii_digit())
}

fn split(row: &HashMap<String, String>, key: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for part in row.get(key).map(String::as_str).unwrap_or("").split('|') {
        let part = part.trim();
        if !part.is_empty() && !seen.iter().any(|s| s == part) {
            seen.push(part.to_owned());
        }
    }
    seen
}

fn wiki_from_row(row: HashMap<String, String>) -> Option<Wiki> {
    let qid = row.get("item")?.rsplit('/').next()?.to_owned();
    if !is_qid(&qid) {
        return None;
    }
    let platforms = split(&row, "platformUris")
        .iter()
        .filter_map(|uri| uri.rsplit('/').next())
        .filter_map(|id| {
            PLATFORM_IDS
                .iter()
                .find(|(q, _)| *q == id)
                .map(|(_, s)| (*s).to_owned())
        })
        .collect::<Vec<_>>();
    let mut store_ids = Vec::new();
    for source in ["steam", "gog", "psn", "xbox", "nintendo"] {
        // Birden çok değer varsa ilki; sıralama kararlı olsun diye en küçüğü.
        if let Some(id) = split(&row, source).into_iter().min() {
            store_ids.push((source, id));
        }
    }
    Some(Wiki {
        label: row.get("itemLabel").cloned().filter(|label| !is_qid(label)),
        released: row.get("released").and_then(|value| iso_date(value)),
        platforms: dedupe(platforms),
        series: split(&row, "series").into_iter().next(),
        qid,
        store_ids,
        ..Wiki::default()
    })
}

// Wikidata platformları -> `platforms` tablosundaki slug'lar; listede olmayan atlanır.
const PLATFORM_IDS: &[(&str, &str)] = &[
    ("Q1406", "pc"),
    ("Q14116", "mac"),
    ("Q388", "linux"),
    ("Q63184502", "ps5"),
    ("Q5014725", "ps4"),
    ("Q10683", "ps3"),
    ("Q98973368", "xbox-series"),
    ("Q13361286", "xbox-one"),
    ("Q48263", "xbox-360"),
    ("Q19610114", "switch"),
    ("Q56942", "wii-u"),
    ("Q48493", "ios"),
    ("Q94", "android"),
];

// Steam'in dil adları -> BCP 47.
const LANGUAGES: &[(&str, &str)] = &[
    ("English", "en"),
    ("French", "fr"),
    ("Italian", "it"),
    ("German", "de"),
    ("Spanish - Spain", "es"),
    ("Spanish - Latin America", "es-419"),
    ("Arabic", "ar"),
    ("Bulgarian", "bg"),
    ("Czech", "cs"),
    ("Danish", "da"),
    ("Dutch", "nl"),
    ("Finnish", "fi"),
    ("Greek", "el"),
    ("Hungarian", "hu"),
    ("Indonesian", "id"),
    ("Japanese", "ja"),
    ("Korean", "ko"),
    ("Norwegian", "no"),
    ("Polish", "pl"),
    ("Portuguese", "pt"),
    ("Portuguese - Portugal", "pt"),
    ("Portuguese - Brazil", "pt-BR"),
    ("Romanian", "ro"),
    ("Russian", "ru"),
    ("Simplified Chinese", "zh-Hans"),
    ("Traditional Chinese", "zh-Hant"),
    ("Swedish", "sv"),
    ("Thai", "th"),
    ("Turkish", "tr"),
    ("Ukrainian", "uk"),
    ("Vietnamese", "vi"),
];

// Steam bunları tür sayar ama oyunun ne olduğunu değil nasıl satıldığını anlatır.
const NOT_GENRES: &[&str] = &["Free To Play", "Early Access", "Game Demo"];

const MODES: &[(&str, &str)] = &[
    ("Single-player", "Single player"),
    ("Multi-player", "Multiplayer"),
    ("Co-op", "Co-op"),
    ("PvP", "PvP"),
    ("Online PvP", "PvP"),
    ("Online Co-op", "Co-op"),
    ("Shared/Split Screen", "Split screen"),
];

fn lookup(table: &[(&str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(name, _)| *name == key).map(|(_, v)| *v)
}

fn dedupe<I: IntoIterator<Item = String>>(values: I) -> Vec<String> {
    let mut seen = Vec::new();
    for value in values.into_iter().filter(|v| !v.is_empty()) {
        if !seen.contains(&value) {
            seen.push(value);
        }
    }
    seen
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn plausible(date: NaiveDate) -> bool {
    (1950..=2100).contains(&date.year())
}

/// "2015-05-19T00:00:00Z" -> günün tarihi; makul olmayan her şey (1950 öncesi, yer tutucular) `None`.
pub fn iso_date(value: &str) -> Option<NaiveDate> {
    let day = value.get(..10)?;
    if !matches!(value.get(10..11), None | Some("T")) {
        return None;
    }
    NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .ok()
        .filter(|date| plausible(*date))
}

/// Steam'in yazdığı tarih: "Nov 16, 2018", "16 Nov, 2018", "Nov 2018", "Q1 2024", "2024".
/// Gün dışındaki kesinlikler ayın/çeyreğin/yılın ilk gününe oturur ve kesinlik olarak döner.
pub fn steam_date(text: &str) -> Option<(NaiveDate, &'static str)> {
    let text = text.trim();
    for format in ["%b %d, %Y", "%d %b, %Y", "%d %b %Y"] {
        if let Ok(date) = NaiveDate::parse_from_str(text, format)
            && plausible(date)
        {
            return Some((date, "day"));
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(&format!("01 {text}"), "%d %b %Y") {
        return plausible(date).then_some((date, "month"));
    }
    let (quarter, year) = text
        .strip_prefix(['Q', 'q'])
        .and_then(|rest| rest.split_once(' '))?;
    let quarter: u32 = quarter.parse().ok().filter(|q| (1..=4).contains(q))?;
    let date = NaiveDate::from_ymd_opt(year.trim().parse().ok()?, quarter * 3 - 2, 1)?;
    plausible(date).then_some((date, "quarter"))
}

fn steam_year(text: &str) -> Option<(NaiveDate, &'static str)> {
    let text = text.trim();
    if text.len() == 4 && text.bytes().all(|b| b.is_ascii_digit()) {
        let date = NaiveDate::from_ymd_opt(text.parse().ok()?, 1, 1)?;
        return plausible(date).then_some((date, "year"));
    }
    None
}

/// Steam metni HTML yazar; katalog düz metin tutar, paragraflar arasında boş satır bırakır.
pub fn plain_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        text.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('>') else {
            // Kapanmayan etiket: kalanı atılır.
            rest = "";
            break;
        };
        let tag = rest[start + 1..start + end].trim().to_ascii_lowercase();
        let closing = tag.starts_with('/');
        let name = tag
            .trim_start_matches('/')
            .split(|c: char| c.is_whitespace() || c == '/')
            .next()
            .unwrap_or("");
        let heading =
            name.len() == 2 && name.starts_with('h') && name.as_bytes()[1].is_ascii_digit();
        if name == "br" || (closing && matches!(name, "p" | "li")) || (closing && heading) {
            text.push('\n');
        } else if name == "li" && !closing {
            text.push_str("• ");
        }
        rest = &rest[start + end + 1..];
    }
    text.push_str(rest);
    let text = text
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    // Tab ve satır sonu dışındaki denetim karakterleri gürültüdür; PostgreSQL NUL'u reddeder.
    let text: String = text
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();
    let mut lines: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let collapsed = line
            .split([' ', '\t'])
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        // Üç ve daha çok satır sonu iki satır sonuna iner.
        if collapsed.is_empty() && lines.last().is_some_and(String::is_empty) {
            continue;
        }
        lines.push(collapsed);
    }
    lines.join("\n").trim().to_owned()
}

/// "English<strong>*</strong>, French, …<br>…": Steam yalnızca arayüz ve sesi ayırır; altyazı bilinmez (false).
fn languages(html: &str) -> Vec<Value> {
    let list = html.split("<br").next().unwrap_or("");
    let mut seen: Vec<&str> = Vec::new();
    let mut result = Vec::new();
    for part in list.split(',') {
        let audio = part.contains('*');
        let name = plain_text(part).replace('*', "");
        let Some(code) = lookup(LANGUAGES, name.trim()) else {
            continue;
        };
        if !seen.contains(&code) {
            seen.push(code);
            result.push(
                json!({ "language": code, "interface": true, "audio": audio, "subtitles": false }),
            );
        }
    }
    result
}

/// "<strong>OS:</strong> Windows 10<br>" satırları -> { os, cpu, gpu, ram, storage }.
fn requirements(html: Option<&str>) -> Option<Value> {
    let html = html?;
    let mut result = Map::new();
    let html = html.replace("<li", "<br");
    for (index, piece) in html.split("<br").enumerate() {
        // İlk parça dışındakiler bir etiketin ortasından başlar ("<br>" ya da "<li class=…>").
        let piece = if index == 0 {
            piece
        } else {
            piece.split_once('>').map_or("", |(_, after)| after)
        };
        let Some((head, rest)) = piece.split_once("</strong>") else {
            continue;
        };
        let key = plain_text(head)
            .trim_end_matches(':')
            .trim()
            .trim_end_matches('*')
            .trim()
            .to_lowercase();
        let key = match key.as_str() {
            "os" => "os",
            "processor" => "cpu",
            "memory" => "ram",
            "graphics" => "gpu",
            "storage" => "storage",
            _ => continue,
        };
        let value = plain_text(rest);
        let value = value.trim_start_matches(':').trim().to_owned();
        if !value.is_empty() && !result.contains_key(key) {
            result.insert(key.to_owned(), Value::String(value));
        }
    }
    (!result.is_empty()).then_some(Value::Object(result))
}

fn asset_url(assets: Option<&Value>, name: &str) -> Option<String> {
    let assets = assets?;
    let file = assets[name].as_str()?;
    let format = assets["asset_url_format"].as_str()?;
    Some(format!(
        "{STEAM_ASSETS}{}",
        format.replace("${FILENAME}", file)
    ))
}

/// Kısa açıklama; kararlı bir yer tutucu olarak (asıl metinleri yönetici ayrıca yazar).
fn summary_of(text: &str) -> Option<String> {
    let clean = plain_text(text);
    if clean.is_empty() {
        return None;
    }
    Some(if clean.chars().count() <= 500 {
        clean
    } else {
        let cut: String = clean.chars().take(497).collect();
        format!("{}…", cut.trim_end())
    })
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// Wikidata ve Steam'i tek `GameInput` kaydında birleştirir.
///
/// Öncelik: tarih Steam -> Wikidata; tür, geliştirici, yayıncı Steam -> Wikidata; görseller ve
/// açıklama yalnızca Steam; platformlar ikisinin birleşimi. Tarih yoksa ya da iki kaynak bir
/// yıldan fazla ayrışıyorsa kayıt "gözden geçirilecek" diye işaretlenir.
pub fn compose(wiki: Option<&Wiki>, steam: Option<&Steam>, status: &str) -> Option<Composed> {
    if wiki.is_none() && steam.is_none() {
        return None;
    }
    let details = steam.map(|s| &s.details);
    let assets = steam.and_then(|s| s.assets.as_ref());
    let get = |key: &str| details.map(|d| &d[key]).unwrap_or(&Value::Null);

    // Tarih.
    let steam_release = get("release_date")["date"]
        .as_str()
        .and_then(|text| steam_date(text).or_else(|| steam_year(text)));
    let wiki_release = wiki.and_then(|w| w.released);
    let (release, precision, review) = match (steam_release, wiki_release) {
        (Some((date, precision)), other) => {
            let differs = other.is_some_and(|o| (o - date).num_days().abs() > 365);
            (
                Some(date),
                Some(precision),
                differs.then_some("Steam and Wikidata release dates differ"),
            )
        }
        (None, Some(date)) => (Some(date), Some("day"), None),
        (None, None) => (None, None, Some("No release date found")),
    };

    let mut platforms = wiki.map(|w| w.platforms.clone()).unwrap_or_default();
    for (key, slug) in [("windows", "pc"), ("mac", "mac"), ("linux", "linux")] {
        if get("platforms")[key] == true {
            platforms.push(slug.to_owned());
        }
    }
    let platforms = dedupe(platforms);

    let screenshots: Vec<String> = get("screenshots")
        .as_array()
        .map(|shots| {
            shots
                .iter()
                .take(12)
                .filter_map(|shot| shot["path_full"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    let minimum = requirements(get("pc_requirements")["minimum"].as_str());
    let recommended = requirements(get("pc_requirements")["recommended"].as_str());

    let mut ratings = Map::new();
    for board in ["pegi", "esrb"] {
        if let Some(rating) = get("ratings")[board]["rating"]
            .as_str()
            .filter(|r| !r.is_empty())
        {
            ratings.insert(board.to_owned(), Value::String(rating.to_uppercase()));
        }
    }

    let described = |key: &str| -> Vec<String> {
        get(key)
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item["description"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let steam_genres = dedupe(
        described("genres")
            .into_iter()
            .filter(|g| !NOT_GENRES.contains(&g.as_str())),
    );
    let genres = if steam_genres.is_empty() {
        dedupe(
            wiki.map(|w| w.genres.iter().map(|g| capitalize(g)).collect::<Vec<_>>())
                .unwrap_or_default(),
        )
    } else {
        steam_genres
    };
    let steam_developers = dedupe(strings(get("developers")));
    let steam_publishers = dedupe(strings(get("publishers")));
    let developers = if steam_developers.is_empty() {
        wiki.map(|w| w.developers.clone()).unwrap_or_default()
    } else {
        steam_developers
    };
    let publishers = if steam_publishers.is_empty() {
        wiki.map(|w| w.publishers.clone()).unwrap_or_default()
    } else {
        steam_publishers
    };

    let steam_name = get("name").as_str().map(str::to_owned);
    let title = wiki
        .and_then(|w| w.label.clone())
        .or_else(|| steam_name.clone())?;
    let alt_titles: Vec<String> = steam_name.into_iter().filter(|n| *n != title).collect();

    let header = get("header_image").as_str().map(str::to_owned);
    let cover = asset_url(assets, "library_capsule").or_else(|| header.clone());
    let backdrop = asset_url(assets, "library_hero")
        .or_else(|| screenshots.first().cloned())
        .or(header);
    let logo = steam.and_then(|s| s.logo.clone());

    let mut external_ids = Map::new();
    if let Some(steam) = steam {
        external_ids.insert("steam".into(), Value::String(steam.appid.to_string()));
    }
    if let Some(wiki) = wiki {
        external_ids.insert("wikidata".into(), Value::String(wiki.qid.clone()));
        for (source, id) in &wiki.store_ids {
            external_ids
                .entry((*source).to_owned())
                .or_insert_with(|| Value::String(id.clone()));
        }
    }

    let input = json!({
        "title": title,
        "status": status,
        "series": wiki.and_then(|w| w.series.clone()),
        "summary": get("short_description").as_str().and_then(summary_of),
        "release_date": release.map(|d| d.format("%Y-%m-%d").to_string()),
        "release_precision": precision,
        "cover_url": cover,
        "backdrop_url": backdrop,
        "logo_url": logo,
        "pc_requirements": (minimum.is_some() || recommended.is_some())
            .then(|| json!({ "minimum": minimum, "recommended": recommended })),
        // Yalnızca ilk çıkış tarihi biliniyor; platform başına tarihler boş bırakılır.
        "releases": platforms.iter().map(|platform| json!({ "platform": platform })).collect::<Vec<_>>(),
        "genres": genres,
        "modes": dedupe(described("categories").iter().filter_map(|c| lookup(MODES, c)).map(str::to_owned)),
        "developers": developers,
        "publishers": publishers,
        "external_ids": external_ids,
        "screenshots": screenshots,
        "languages": get("supported_languages").as_str().map(languages).unwrap_or_default(),
        "age_ratings": ratings,
        "alt_titles": alt_titles,
        "review_reason": review,
    });
    Some(Composed { input, review })
}

/// Steam bağlantısından ya da yalnızca sayıdan uygulama kimliği: `https://store.steampowered.com/app/570/Dota_2/`.
pub fn parse_steam_ref(input: &str) -> Option<u32> {
    let input = input.trim();
    if !input.is_empty() && input.len() <= 10 && input.bytes().all(|b| b.is_ascii_digit()) {
        return input.parse().ok().filter(|id| *id > 0);
    }
    let lower = input.to_ascii_lowercase();
    if !(lower.contains("steampowered.com/") || lower.contains("steamcommunity.com/")) {
        return None;
    }
    let after = lower.split("/app/").nth(1)?;
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok().filter(|id| *id > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn steam_refs() {
        assert_eq!(parse_steam_ref("570"), Some(570));
        assert_eq!(
            parse_steam_ref(" https://store.steampowered.com/app/292030/The_Witcher_3/ "),
            Some(292030)
        );
        assert_eq!(parse_steam_ref("store.steampowered.com/app/10"), Some(10));
        assert_eq!(parse_steam_ref("https://example.com/app/570"), None);
        assert_eq!(parse_steam_ref("elden ring"), None);
        assert_eq!(parse_steam_ref("0"), None);
    }

    #[test]
    fn wikidata_dates() {
        assert_eq!(iso_date("2015-05-19T00:00:00Z"), Some(day(2015, 5, 19)));
        assert_eq!(iso_date("1949-12-31T00:00:00Z"), None);
        assert_eq!(iso_date("2101-01-01T00:00:00Z"), None);
        assert_eq!(iso_date("2015-02-30T00:00:00Z"), None);
        assert_eq!(iso_date("-0300-01-01T00:00:00Z"), None);
        assert_eq!(iso_date("http://www"), None);
    }

    #[test]
    fn steam_dates() {
        assert_eq!(steam_date("Nov 16, 2018"), Some((day(2018, 11, 16), "day")));
        assert_eq!(steam_date("16 Nov, 2018"), Some((day(2018, 11, 16), "day")));
        assert_eq!(steam_date("Nov 2018"), Some((day(2018, 11, 1), "month")));
        assert_eq!(steam_date("Q2 2024"), Some((day(2024, 4, 1), "quarter")));
        assert_eq!(steam_date("To be announced"), None);
        assert_eq!(steam_date("Coming soon"), None);
        assert_eq!(steam_year("2024"), Some((day(2024, 1, 1), "year")));
    }

    #[test]
    fn text_cleanup() {
        assert_eq!(
            plain_text("<p>A &amp; B</p><ul><li>one</li><li>two</li></ul>x\u{0}y"),
            "A & B\n• one\n• two\nxy"
        );
        assert_eq!(plain_text("a<br><br><br><br>b"), "a\n\nb");
    }

    #[test]
    fn requirement_lines() {
        let html = "<strong>Minimum:</strong><br><ul class=\"bb_ul\"><li><strong>OS *:</strong> Windows 10<br></li><li><strong>Memory:</strong> 8 GB RAM<br></li></ul>";
        let parsed = requirements(Some(html)).unwrap();
        assert_eq!(parsed["os"], "Windows 10");
        assert_eq!(parsed["ram"], "8 GB RAM");
    }

    #[test]
    fn language_list() {
        let html = "English<strong>*</strong>, French, Turkish<br><strong>*</strong>languages with full audio support";
        let list = languages(html);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0]["audio"], true);
        assert_eq!(list[1]["audio"], false);
    }

    #[test]
    fn dates_disagree_flags_review() {
        let wiki = Wiki {
            qid: "Q1".into(),
            label: Some("Test".into()),
            released: Some(day(2010, 1, 1)),
            ..Wiki::default()
        };
        let steam = Steam {
            appid: 1,
            details: json!({ "name": "Test", "release_date": { "date": "Nov 16, 2018" } }),
            assets: None,
            logo: None,
        };
        let composed = compose(Some(&wiki), Some(&steam), "draft").unwrap();
        assert_eq!(composed.input["release_date"], "2018-11-16");
        assert!(composed.review.is_some());
        let none = compose(
            Some(&Wiki {
                released: None,
                ..wiki
            }),
            None,
            "draft",
        )
        .unwrap();
        assert_eq!(none.review, Some("No release date found"));
    }
}

/// Canlı kaynaklara bağlanır; `cargo test -- --ignored live --nocapture` ile elle çalıştırılır.
#[cfg(test)]
mod live {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn live() {
        let source = Source::new().unwrap();
        let steam = source.steam(949230).await.unwrap();
        let qid = source.qid_for_app(949230, PATIENT).await;
        eprintln!("qid {qid:?}");
        let wiki = source
            .wiki(&qid.clone().into_iter().collect::<Vec<_>>(), PATIENT)
            .await;
        let composed = compose(wiki.values().next(), Some(&steam), "draft").unwrap();
        eprintln!("{}", serde_json::to_string_pretty(&composed.input).unwrap());
        eprintln!("review {:?}", composed.review);
        let list = source
            .list(
                &ListFilter {
                    platform: Some("switch".into()),
                    year_from: Some(2023),
                    year_to: Some(2023),
                },
                0,
                5,
            )
            .await
            .unwrap();
        eprintln!("{list:?}");
        let qids: Vec<String> = list.iter().map(|(q, _)| q.clone()).collect();
        for (q, w) in source.wiki(&qids, PATIENT).await {
            eprintln!("{q} {:?}", w);
        }
        eprintln!(
            "{:?}",
            source
                .search("elden ring")
                .await
                .iter()
                .map(|h| (h.source, &h.id, &h.name))
                .collect::<Vec<_>>()
        );
    }
}
