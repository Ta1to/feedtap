use crate::types::{NewsItem, SourceConfig, SourceInfo};
use anyhow::Result;
use reqwest::Client;
use feed_rs::parser;
use std::time::Duration;

pub struct CryptoFeedTap;

#[async_trait::async_trait]
impl super::Tap for CryptoFeedTap {
    async fn fetch(&self, src: &SourceConfig) -> Result<Vec<NewsItem>> {
        let client = Client::builder()
            .user_agent("FeedTap/2.0 Crypto Feed Reader (+https://github.com/Ta1to/feedtap)")
            .timeout(Duration::from_secs(45))
            .build()?;

        // Handle special cases for different crypto sources
        let response = match src.id.as_str() {
            "coindesk" => fetch_coindesk(&client, &src.url).await?,
            "cointelegraph" => fetch_cointelegraph(&client, &src.url).await?,
            "theblock" => fetch_theblock(&client, &src.url).await?,
            "bitcoinmagazine" => fetch_bitcoin_magazine(&client, &src.url).await?,
            "decrypt" => fetch_decrypt(&client, &src.url).await?,
            _ => client.get(&src.url).send().await?
        };

        let body = response.bytes().await?;
        let feed = parser::parse(&body[..])?;
        
        let src_info = SourceInfo {
            id: src.id.clone(),
            name: src.name.clone(),
        };
        
        let mut items: Vec<NewsItem> = feed
            .entries
            .into_iter()
            .map(|e| super::rss::entry_to_crypto_item(&src_info, e, &src.id))
            .collect();

        // Sort by published date (newest first)
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

async fn fetch_coindesk(client: &Client, url: &str) -> Result<reqwest::Response> {
    // CoinDesk sometimes requires specific headers
    Ok(client
        .get(url)
        .header("Accept", "application/rss+xml, application/xml, text/xml")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?)
}

async fn fetch_cointelegraph(client: &Client, url: &str) -> Result<reqwest::Response> {
    // Cointelegraph feed handling
    Ok(client
        .get(url)
        .header("Accept", "application/rss+xml, application/xml")
        .header("Cache-Control", "no-cache")
        .send()
        .await?)
}

async fn fetch_theblock(client: &Client, url: &str) -> Result<reqwest::Response> {
    // The Block feed handling
    Ok(client
        .get(url)
        .header("Accept", "application/rss+xml")
        .header("User-Agent", "Mozilla/5.0 (compatible; FeedTap/2.0; +https://feedtap.io)")
        .send()
        .await?)
}

async fn fetch_bitcoin_magazine(client: &Client, url: &str) -> Result<reqwest::Response> {
    // Bitcoin Magazine specific handling
    Ok(client
        .get(url)
        .header("Accept", "application/rss+xml, application/atom+xml")
        .send()
        .await?)
}

async fn fetch_decrypt(client: &Client, url: &str) -> Result<reqwest::Response> {
    // Decrypt feed handling
    Ok(client
        .get(url)
        .header("Accept", "application/rss+xml")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await?)
}

// Crypto-specific RSS parser utilities
pub mod crypto_utils {
    use regex::Regex;
    use std::sync::OnceLock;

    pub fn extract_crypto_keywords(text: &str) -> Vec<String> {
        static CRYPTO_REGEX: OnceLock<Regex> = OnceLock::new();
        
        let regex = CRYPTO_REGEX.get_or_init(|| {
            Regex::new(r"(?i)\b(bitcoin|btc|ethereum|eth|crypto|blockchain|defi|nft|altcoin|stablecoin|mining|trading|hodl|bull|bear|market|price|pump|dump|moon|dip|correction|rally)\b").unwrap()
        });

        regex.find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect()
    }

    pub fn calculate_crypto_relevance_score(title: &str, summary: Option<&str>) -> u32 {
        let combined_text = format!("{} {}", title, summary.unwrap_or(""));
        let keywords = extract_crypto_keywords(&combined_text);
        
        // Weight different keyword types
        let mut score = 0u32;
        for keyword in keywords {
            score += match keyword.as_str() {
                "bitcoin" | "btc" | "ethereum" | "eth" => 5,
                "crypto" | "blockchain" | "defi" => 4,
                "trading" | "market" | "price" => 3,
                "nft" | "altcoin" | "mining" => 3,
                "bull" | "bear" | "pump" | "dump" => 2,
                _ => 1,
            };
        }

        score
    }

    pub fn clean_crypto_title(title: &str) -> String {
        // Remove common prefixes/suffixes from crypto news titles
        let binding = title
            .replace("BREAKING:", "")
            .replace("UPDATE:", "")
            .replace("EXCLUSIVE:", "");
        let cleaned = binding.trim();

        // Capitalize first letter
        if let Some(first_char) = cleaned.chars().next() {
            first_char.to_uppercase().collect::<String>() + &cleaned[first_char.len_utf8()..]
        } else {
            cleaned.to_string()
        }
    }

    pub fn extract_price_mentions(text: &str) -> Vec<String> {
        static PRICE_REGEX: OnceLock<Regex> = OnceLock::new();
        
        let regex = PRICE_REGEX.get_or_init(|| {
            Regex::new(r"(?i)\$[\d,]+\.?\d*|\d+\.?\d*\s*(?:usd|btc|eth|usdt|usdc)").unwrap()
        });

        regex.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}
