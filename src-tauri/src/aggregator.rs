use crate::storage::Storage;
use crate::taps::make_tap;
use crate::types::{NewsItem, SourceConfig};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration, Instant};

#[derive(Clone)]
pub struct AggregatorHandle {
    inner: Arc<Inner>,
}

struct Inner {
    tx: broadcast::Sender<NewsItem>,
    // map source id -> last seen ids set
    seen: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    storage: Arc<Mutex<Storage>>,
    last_poll: Arc<Mutex<HashMap<String, Instant>>>,
}

pub struct Aggregator;

impl Aggregator {
    pub fn start(storage: Arc<Mutex<Storage>>) -> (AggregatorHandle, broadcast::Receiver<NewsItem>) {
        let (tx, rx) = broadcast::channel(256);
    let seen = Arc::new(Mutex::new(HashMap::<String, HashSet<String>>::new()));
    let last_poll = Arc::new(Mutex::new(HashMap::<String, Instant>::new()));
        let inner = Arc::new(Inner { tx: tx.clone(), seen: seen.clone(), storage: storage.clone(), last_poll: last_poll.clone() });
        let handle = AggregatorHandle { inner: inner.clone() };

        // single scheduler loop
        let inner_clone = inner.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let now = Instant::now();
                let sources = {
                    let st = inner_clone.storage.lock().await;
                    st.list_sources().await
                };
                for (i, src) in sources.into_iter().enumerate() {
                    // slight stagger to avoid burst
                    let stagger = (i as u64) * 200;
                    let inner2 = inner_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        if stagger > 0 { sleep(Duration::from_millis(stagger)).await; }
                        let lp = inner2.last_poll.lock().await;
                        let due = match lp.get(&src.id) {
                            Some(&t) => now.duration_since(t) >= Duration::from_millis(src.interval_ms.max(10_000)),
                            None => true,
                        };
                        if due {
                            drop(lp);
                            if let Err(e) = poll_once(inner2.clone(), src.clone()).await {
                                tracing::warn!(?e, "poll_once failed");
                            } else {
                                let mut lp2 = inner2.last_poll.lock().await;
                                lp2.insert(src.id.clone(), Instant::now());
                            }
                        }
                    });
                }
                sleep(Duration::from_millis(1_000)).await;
            }
        });

        (handle, rx)
    }
}

impl AggregatorHandle {
    pub async fn trigger_refresh(&self, id: Option<String>) -> Result<()> {
        let sources = {
            let st = self.inner.storage.lock().await;
            let list = st.list_sources().await;
            if let Some(id) = id {
                list.into_iter().filter(|s| s.id == id).collect()
            } else {
                list
            }
        };

        for src in sources {
            let inner = self.inner.clone();
            tokio::spawn(async move {
                if let Err(e) = poll_once(inner, src).await {
                    tracing::warn!(?e, "manual poll failed");
                }
            });
        }
        Ok(())
    }
}

async fn poll_once(inner: Arc<Inner>, src: SourceConfig) -> Result<()> {
    let tap = make_tap(&src.kind);
    let items = tap.fetch(&src).await?;

    let mut seen = inner.seen.lock().await;
    let entry = seen.entry(src.id.clone()).or_default();
    for item in items {
        if entry.insert(item.id.clone()) { // new
            let _ = inner.tx.send(item);
        }
    }
    Ok(())
}
