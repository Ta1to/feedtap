use crate::types::{NewsItem, SourceConfig};
use anyhow::Result;

pub mod rss;

#[async_trait::async_trait]
pub trait Tap: Send + Sync {
    async fn fetch(&self, src: &SourceConfig) -> Result<Vec<NewsItem>>;
}

pub fn make_tap(kind: &str) -> Box<dyn Tap> {
    match kind {
        "rss" | _ => Box::new(rss::RssTap),
    }
}
