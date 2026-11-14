//! Toplu içe aktarma: arka planda çalışan tek bir iş. Wikidata'dan süzgece uyan oyunları sırayla
//! çeker, her birini Steam'le zenginleştirip kataloğa yazar ve oyun başına sonucu tutar.
//! İş bellekte yaşar (yeniden başlatmada kaybolur); aynı anda yalnızca bir iş çalışır.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tokio::time::sleep;

use super::{
    import::{GameInput, ImportError, import_game},
    source::{self, ListFilter, STEAM_DELAY, Source, SourceError},
};

/// Wikidata sayfa sayfa sorulur; tek sorgu bu kadar oyunu geçmez.
const PAGE: u32 = 50;
/// Steam hız sınırına takılınca beklenen süre ve deneme sayısı.
const RATE_LIMIT_WAIT: Duration = Duration::from_secs(65);
const STEAM_TRIES: u32 = 3;

pub struct Params {
    pub limit: u32,
    pub offset: u32,
    pub filter: ListFilter,
    /// Yeni oyunların durumu: `draft` ya da `published`.
    pub status: String,
    /// Katalogda zaten olan oyunlar da yeniden işlenir (kilitli alanlar korunur).
    pub update_existing: bool,
}

#[derive(Serialize, Clone)]
pub struct Item {
    pub title: String,
    /// created, updated, skipped, failed.
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Hata ya da atlama kodu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Elle bakılması gerekiyorsa nedeni.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct View {
    /// running, done, cancelled, failed.
    pub state: &'static str,
    pub limit: u32,
    pub processed: u32,
    pub created: u32,
    pub updated: u32,
    pub skipped: u32,
    pub failed: u32,
    pub review: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'static str>,
    pub started_at: DateTime<Utc>,
    pub items: Vec<Item>,
}

pub struct Job {
    view: Mutex<View>,
    cancel: AtomicBool,
}

impl Job {
    fn update(&self, change: impl FnOnce(&mut View)) {
        if let Ok(mut view) = self.view.lock() {
            change(&mut view);
        }
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    fn push(&self, item: Item) {
        self.update(|view| {
            view.processed += 1;
            match item.outcome {
                "created" => view.created += 1,
                "updated" => view.updated += 1,
                "skipped" => view.skipped += 1,
                _ => view.failed += 1,
            }
            if item.review.is_some() {
                view.review += 1;
            }
            view.items.push(item);
        });
    }
}

#[derive(Default)]
pub struct JobSlot {
    current: Mutex<Option<Arc<Job>>>,
}

impl JobSlot {
    pub fn view(&self) -> Option<View> {
        let job = self.current.lock().ok()?.clone()?;
        job.view.lock().ok().map(|view| view.clone())
    }

    pub fn cancel(&self) -> bool {
        match self.current.lock().ok().and_then(|slot| slot.clone()) {
            Some(job) => {
                job.cancel.store(true, Ordering::Relaxed);
                true
            }
            None => false,
        }
    }

    /// Yeni işi başlatır; çalışan bir iş varsa `false`.
    pub fn start(&self, db: PgPool, source: Arc<Source>, params: Params) -> bool {
        let Ok(mut slot) = self.current.lock() else {
            return false;
        };
        if let Some(job) = slot.as_ref()
            && job.view.lock().is_ok_and(|view| view.state == "running")
        {
            return false;
        }
        let job = Arc::new(Job {
            view: Mutex::new(View {
                state: "running",
                limit: params.limit,
                processed: 0,
                created: 0,
                updated: 0,
                skipped: 0,
                failed: 0,
                review: 0,
                error: None,
                started_at: Utc::now(),
                items: Vec::new(),
            }),
            cancel: AtomicBool::new(false),
        });
        *slot = Some(job.clone());
        tokio::spawn(async move {
            let outcome = run(&db, &source, &job, &params).await;
            job.update(|view| {
                view.state = match (&outcome, job.cancelled()) {
                    (Err(code), _) => {
                        view.error = Some(code);
                        "failed"
                    }
                    (Ok(()), true) => "cancelled",
                    (Ok(()), false) => "done",
                };
            });
        });
        true
    }
}

async fn run(db: &PgPool, source: &Source, job: &Job, params: &Params) -> Result<(), &'static str> {
    let mut offset = params.offset;
    let mut remaining = params.limit;
    while remaining > 0 && !job.cancelled() {
        let page = remaining.min(PAGE);
        let list = match source.list(&params.filter, offset, page).await {
            Ok(list) => list,
            // Wikidata anlık yanıt vermeyebilir; bir kez daha denenir.
            Err(_) => {
                sleep(Duration::from_secs(20)).await;
                source
                    .list(&params.filter, offset, page)
                    .await
                    .map_err(|_| "source_unavailable")?
            }
        };
        if list.is_empty() {
            break;
        }
        offset += list.len() as u32;
        remaining = remaining.saturating_sub(list.len() as u32);

        let qids: Vec<String> = list.iter().map(|(qid, _)| qid.clone()).collect();
        let wikis = source.wiki(&qids, source::PATIENT).await;
        for (qid, label) in list {
            if job.cancelled() {
                break;
            }
            let title = wikis
                .get(&qid)
                .and_then(|wiki| wiki.label.clone())
                .or(label)
                .unwrap_or_else(|| qid.clone());
            let item = match wikis.get(&qid) {
                Some(wiki) => {
                    // Etiketi olmayan kayıtlar için liste sorgusunun etiketi.
                    let mut wiki = wiki.clone();
                    wiki.label = wiki
                        .label
                        .or_else(|| Some(title.clone()).filter(|t| *t != qid));
                    one(db, source, job, params, &wiki, title).await
                }
                // Ayrıntı sorgusu yanıt vermedi: eksik veriyle kayıt açılmaz.
                None => failed(title, "source_unavailable"),
            };
            job.push(item);
        }
        sleep(Duration::from_secs(2)).await;
    }
    Ok(())
}

fn failed(title: String, reason: &str) -> Item {
    Item {
        title,
        outcome: "failed",
        slug: None,
        reason: Some(reason.to_owned()),
        review: None,
    }
}

fn skipped(title: String, reason: &str) -> Item {
    Item {
        outcome: "skipped",
        ..failed(title, reason)
    }
}

async fn one(
    db: &PgPool,
    source: &Source,
    job: &Job,
    params: &Params,
    wiki: &source::Wiki,
    title: String,
) -> Item {
    if !params.update_existing {
        let steam = wiki.steam_app().map(|app| app.to_string());
        let exists: Result<bool, _> = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM game_external_ids
                             WHERE (source = 'wikidata' AND external_id = $1)
                                OR (source = 'steam' AND external_id = $2))",
        )
        .bind(&wiki.qid)
        .bind(steam)
        .fetch_one(db)
        .await;
        match exists {
            Ok(true) => return skipped(title, "already_in_catalog"),
            Ok(false) => {}
            Err(err) => {
                tracing::error!(error = ?err, "katalog sorgusu başarısız");
                return failed(title, "internal");
            }
        }
    }

    let mut steam = None;
    if let Some(appid) = wiki.steam_app() {
        let mut tries = 0;
        loop {
            sleep(STEAM_DELAY).await;
            match source.steam(appid).await {
                Ok(found) => {
                    steam = Some(found);
                    break;
                }
                // Mağazadan kalkmış: Wikidata verisiyle devam edilir.
                Err(SourceError::NotFound) => break,
                Err(SourceError::NotAGame) => return skipped(title, "steam_app_not_a_game"),
                Err(SourceError::Unavailable) => {
                    tries += 1;
                    if tries >= STEAM_TRIES || job.cancelled() {
                        return failed(title, "source_unavailable");
                    }
                    sleep(RATE_LIMIT_WAIT).await;
                }
            }
        }
    }

    let Some(composed) = source::compose(Some(wiki), steam.as_ref(), &params.status) else {
        return failed(title, "no_data");
    };
    let review = composed.review.map(str::to_owned);
    let title = composed.input["title"]
        .as_str()
        .map_or(title, str::to_owned);
    let input: GameInput = match serde_json::from_value(composed.input) {
        Ok(input) => input,
        Err(err) => {
            tracing::warn!(error = %err, title, "içe aktarma kaydı çözülemedi");
            return failed(title, "invalid_format");
        }
    };
    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(_) => return failed(title, "internal"),
    };
    match import_game(&mut tx, input).await {
        Ok(imported) => {
            if tx.commit().await.is_err() {
                return failed(title, "internal");
            }
            Item {
                title,
                outcome: if imported.created {
                    "created"
                } else {
                    "updated"
                },
                slug: Some(imported.slug),
                reason: None,
                review,
            }
        }
        Err(err) => {
            drop(tx);
            let code = match err {
                ImportError::Invalid(code) => code,
                ImportError::Db(err) => {
                    tracing::error!(error = ?err, title, "oyun içe aktarılamadı");
                    "internal"
                }
            };
            failed(title, code)
        }
    }
}

/// Canlı kaynaklar ve gerçek bir PostgreSQL ister:
/// `TEST_DATABASE_URL=… cargo test -- --ignored job --nocapture`.
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "ağ ve veritabanı ister"]
    async fn job_runs_end_to_end() {
        let db = PgPool::connect(&std::env::var("TEST_DATABASE_URL").unwrap())
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        let slot = JobSlot::default();
        let params = || Params {
            limit: 4,
            offset: 0,
            filter: ListFilter {
                platform: Some("switch".into()),
                year_from: Some(2023),
                year_to: Some(2023),
            },
            status: "draft".into(),
            update_existing: false,
        };
        assert!(slot.start(db.clone(), Arc::new(Source::new().unwrap()), params()));
        assert!(!slot.start(db.clone(), Arc::new(Source::new().unwrap()), params()));
        loop {
            sleep(Duration::from_secs(2)).await;
            if slot.view().unwrap().state != "running" {
                break;
            }
        }
        let view = slot.view().unwrap();
        eprintln!("{}", serde_json::to_string_pretty(&view).unwrap());
        assert_eq!(view.state, "done");
        // İkinci çalıştırma hepsini atlar.
        assert!(slot.start(db.clone(), Arc::new(Source::new().unwrap()), params()));
        loop {
            sleep(Duration::from_secs(2)).await;
            if slot.view().unwrap().state != "running" {
                break;
            }
        }
        let again = slot.view().unwrap();
        assert_eq!(again.skipped as usize, again.items.len());
    }
}
