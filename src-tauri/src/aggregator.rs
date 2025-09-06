use crate::storage::Storage;
use crate::taps::make_tap;
use crate::types::{NewsItem, SourceConfig};
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration, Instant};

// Performance optimization: Limit memory usage
const MAX_SEEN_ITEMS_PER_SOURCE: usize = 1000;
const BATCH_SIZE: usize = 3;

#[derive(Clone)]
pub struct AggregatorHandle {
    inner: Arc<Inner>,
}

struct Inner {
    tx: broadcast::Sender<NewsItem>,
    // Performance optimized: VecDeque with size limit instead of growing HashSet
    seen: Arc<Mutex<HashMap<String, VecDeque<String>>>>,
    storage: Storage,
    last_poll: Arc<Mutex<HashMap<String, Instant>>>,
}

pub struct Aggregator;

impl Aggregator {
    pub fn start(storage: Storage) -> (AggregatorHandle, broadcast::Receiver<NewsItem>) {
        let (tx, rx) = broadcast::channel(512); // Increased buffer for better performance
        let seen = Arc::new(Mutex::new(HashMap::<String, VecDeque<String>>::new()));
        let last_poll = Arc::new(Mutex::new(HashMap::<String, Instant>::new()));
        let inner = Arc::new(Inner { tx: tx.clone(), seen: seen.clone(), storage, last_poll: last_poll.clone() });
        let handle = AggregatorHandle { inner: inner.clone() };

        // Optimized scheduler loop - batch processing and pre-filtering
        let inner_clone = inner.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let now = Instant::now();
                let sources = {
                    inner_clone.storage.list_sources()
                };

                // Pre-filter sources that need polling to reduce spawned tasks
                let mut due_sources = Vec::new();
                {
                    let lp = inner_clone.last_poll.lock().await;
                    for src in sources {
                        let due = match lp.get(&src.id) {
                            Some(&t) => now.duration_since(t) >= Duration::from_millis(src.interval_ms.max(10_000)),
                            None => true,
                        };
                        if due {
                            due_sources.push(src);
                        }
                    }
                }

                // Process in batches to reduce system load
                for chunk in due_sources.chunks(BATCH_SIZE) {
                    for (i, src) in chunk.iter().enumerate() {
                        let stagger = (i as u64) * 200;
                        let inner2 = inner_clone.clone();
                        let src_clone = src.clone();
                        
                        tauri::async_runtime::spawn(async move {
                            if stagger > 0 { 
                                sleep(Duration::from_millis(stagger)).await; 
                            }
                            
                            if let Err(e) = poll_once(inner2.clone(), src_clone.clone()).await {
                                tracing::warn!(?e, source_id = %src_clone.id, "poll_once failed");
                            } else {
                                let mut lp = inner2.last_poll.lock().await;
                                lp.insert(src_clone.id.clone(), Instant::now());
                            }
                        });
                    }
                    
                    // Small delay between batches to prevent overwhelming the system
                    if chunk.len() == BATCH_SIZE {
                        sleep(Duration::from_millis(1000)).await;
                    }
                }

                // Wait longer between scheduler runs for better resource usage
                sleep(Duration::from_millis(2_000)).await;
            }
        });

        (handle, rx)
    }
}

impl AggregatorHandle {
    pub async fn trigger_refresh(&self, id: Option<String>) -> Result<()> {
        let sources = {
            let list = self.inner.storage.list_sources();
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

    // Performance optimization: Use VecDeque with size limit instead of growing HashSet
    let mut seen = inner.seen.lock().await;
    let entry = seen.entry(src.id.clone()).or_insert_with(VecDeque::new);
    
    let mut new_items = 0;
    for item in items {
        if !entry.contains(&item.id) {
            // Add new item
            entry.push_back(item.id.clone());
            
            // Limit memory usage - remove old items if exceeding limit
            if entry.len() > MAX_SEEN_ITEMS_PER_SOURCE {
                entry.pop_front();
            }
            
            // Send the item
            if inner.tx.send(item.clone()).is_ok() {
                new_items += 1;
                tracing::debug!(
                    item_id = %item.id,
                    source_id = %src.id,
                    "News item added to broadcast channel"
                );
            } else {
                tracing::warn!(
                    item_id = %item.id,
                    source_id = %src.id,
                    "Failed to send news item to broadcast channel"
                );
            }
        }
    }

    if new_items > 0 {
        tracing::debug!(source_id = %src.id, new_items, "fetched new items");
    }

    Ok(())
}
