use crate::types::{NewsItem, SourceConfig, SourceInfo};
use anyhow::Result;
use feed_rs::model::Entry;
use regex::Regex;
use std::sync::OnceLock;

pub struct RssTap;

#[async_trait::async_trait]
impl super::Tap for RssTap {
    async fn fetch(&self, src: &SourceConfig) -> Result<Vec<NewsItem>> {
        // Enhanced HTTP client with better headers and error handling
        let client = reqwest::Client::builder()
            .user_agent("FeedTap/2.0 (+https://github.com/Ta1to/feedtap)")
            .timeout(std::time::Duration::from_secs(15)) // Reduced timeout for better performance
            .build()?;
            
        let response = client.get(&src.url).send().await?;
        let body = response.bytes().await?;
        let feed = feed_rs::parser::parse(&body[..])?;
        
        let src_info = SourceInfo {
            id: src.id.clone(),
            name: src.name.clone(),
        };
        
        let mut items: Vec<NewsItem> = feed
            .entries
            .into_iter()
            .map(|e| entry_to_item(&src_info, e))
            .collect();
            
        // Sort by published date (newest first) to ensure consistent ordering
        items.sort_by(|a, b| {
            match (&b.published_at, &a.published_at) {
                (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        
        Ok(items)
    }
}

fn entry_to_item(source: &SourceInfo, e: Entry) -> NewsItem {
    let title = e.title.clone().map(|t| t.content).unwrap_or_else(|| "(no title)".into());
    let link = e
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("alternate"))
        .or_else(|| e.links.first())
        .map(|l| l.href.clone())
        .unwrap_or_default();

    // Enhanced summary extraction with HTML cleaning and crypto-specific content
    let summary = extract_enhanced_summary(&e);
    
    let published_at = e.published.or(e.updated).map(|d| d.to_string());
    let id_source = if e.id.is_empty() { 
        format!("{}:{}", source.id, title) 
    } else { 
        e.id.clone() 
    };
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

pub fn entry_to_crypto_item(source: &SourceInfo, e: Entry, source_id: &str) -> NewsItem {
    let raw_title = e.title.clone().map(|t| t.content).unwrap_or_else(|| "(no title)".into());
    let title = super::crypto_feeds::crypto_utils::clean_crypto_title(&raw_title);
    
    let link = e
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("alternate"))
        .or_else(|| e.links.first())
        .map(|l| l.href.clone())
        .unwrap_or_default();

    // Enhanced summary extraction with crypto-specific processing
    let summary = extract_crypto_enhanced_summary(&e, source_id);
    
    let published_at = e.published.or(e.updated).map(|d| d.to_string());
    let id_source = if e.id.is_empty() { 
        format!("{}:{}", source.id, title) 
    } else { 
        e.id.clone() 
    };
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

fn extract_enhanced_summary(entry: &Entry) -> Option<String> {
    // Priority order: content -> summary
    let raw_content = entry
        .content
        .as_ref()
        .and_then(|c| c.body.as_ref().map(|b| b.as_str()))
        .or_else(|| entry.summary.as_ref().map(|s| s.content.as_str()));

    raw_content.map(|content| {
        // Clean HTML tags and entities
        let cleaned = clean_html_content(content);
        
        // Truncate to reasonable length for crypto news summaries
        if cleaned.len() > 500 {
            format!("{}...", &cleaned[..497])
        } else {
            cleaned
        }
    })
}

fn extract_crypto_enhanced_summary(entry: &Entry, source_id: &str) -> Option<String> {
    // Priority order: content -> summary
    let raw_content = entry
        .content
        .as_ref()
        .and_then(|c| c.body.as_ref().map(|b| b.as_str()))
        .or_else(|| entry.summary.as_ref().map(|s| s.content.as_str()));

    raw_content.map(|content| {
        // Clean HTML tags and entities
        let cleaned = clean_html_content(content);
        
        // Extract price mentions for crypto relevance
        let price_mentions = super::crypto_feeds::crypto_utils::extract_price_mentions(&cleaned);
        
        // Truncate and enhance for crypto content
        let truncated = if cleaned.len() > 400 {
            format!("{}...", &cleaned[..397])
        } else {
            cleaned
        };

        // Add price context if found
        if !price_mentions.is_empty() && source_id.contains("bitcoin") {
            format!("{} [Prices: {}]", truncated, price_mentions.join(", "))
        } else {
            truncated
        }
    })
}

fn clean_html_content(html: &str) -> String {
    static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();
    static HTML_ENTITY_REGEX: OnceLock<Regex> = OnceLock::new();
    static WHITESPACE_REGEX: OnceLock<Regex> = OnceLock::new();
    
    let tag_regex = HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
    let entity_regex = HTML_ENTITY_REGEX.get_or_init(|| Regex::new(r"&[a-zA-Z]+;|&#\d+;").unwrap());
    let ws_regex = WHITESPACE_REGEX.get_or_init(|| Regex::new(r"\s+").unwrap());
    
    // Remove HTML tags
    let no_tags = tag_regex.replace_all(html, " ");
    
    // Convert common HTML entities
    let no_entities = entity_regex.replace_all(&no_tags, |caps: &regex::Captures| {
        match caps.get(0).unwrap().as_str() {
            "&amp;" => "&",
            "&lt;" => "<",
            "&gt;" => ">",
            "&quot;" => "\"",
            "&apos;" => "'",
            "&nbsp;" => " ",
            _ => " ",
        }
    });
    
    // Normalize whitespace
    let normalized = ws_regex.replace_all(&no_entities, " ");
    
    normalized.trim().to_string()
}
