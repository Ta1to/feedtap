use crate::types::{NewsItem, SourceConfig, SourceInfo};
use anyhow::Result;
use feed_rs::model::Entry;

pub struct RssTap;

#[async_trait::async_trait]
impl super::Tap for RssTap {
    async fn fetch(&self, src: &SourceConfig) -> Result<Vec<NewsItem>> {
        let body = reqwest::get(&src.url).await?.bytes().await?;
        let feed = feed_rs::parser::parse(&body[..])?;
        let src_info = SourceInfo {
            id: src.id.clone(),
            name: src.name.clone(),
        };
        let items = feed
            .entries
            .into_iter()
            .map(|e| entry_to_item(&src_info, e))
            .collect();
        Ok(items)
    }
}

fn entry_to_item(source: &SourceInfo, e: Entry) -> NewsItem {
    let title = e.title.map(|t| t.content).unwrap_or_else(|| "(no title)".into());
    let link = e
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("alternate"))
    .or_else(|| e.links.first())
    .map(|l| l.href.clone())
        .unwrap_or_default();

    let summary = e.summary.map(|s| s.content);
    let published_at = e.published.or(e.updated).map(|d| d.to_string());
    let id_source = if e.id.is_empty() { format!("{}:{}", source.id, title) } else { e.id.clone() };
    let stable = format!("{}|{}|{}", id_source, link, published_at.clone().unwrap_or_default());
    let id = format!("{:x}", seahash::hash(stable.as_bytes()));

    NewsItem {
        id,
        title,
        link,
        summary,
        published_at,
        source: source.clone(),
    }
}
