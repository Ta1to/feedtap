use crate::types::SourceConfig;
use anyhow::Result;
use std::path::PathBuf;
use tauri::Manager;

// Interval constants for feed polling (in milliseconds)
const FAST_INTERVAL: u64 = 60_000;    // 1 minute
const NORMAL_INTERVAL: u64 = 300_000; // 5 minutes
const SLOW_INTERVAL: u64 = 600_000;   // 10 minutes
const MEDIUM_INTERVAL: u64 = 240_000; // 4 minutes
const SLOWER_INTERVAL: u64 = 360_000; // 6 minutes
const FASTEST_INTERVAL: u64 = 180_000; // 3 minutes

/// Storage manager for RSS feed sources configuration
/// Handles reading, writing, and managing source configurations in JSON format
#[derive(Clone)]
pub struct Storage {
    app_handle: tauri::AppHandle,
}

impl Storage {
    /// Creates a new Storage instance with the given app handle
    pub fn new(app_handle: tauri::AppHandle) -> Self { Self { app_handle } }

    /// Gets the configuration file path for storing sources
    fn config_path(&self) -> Result<PathBuf> {
        let dir = self
            .app_handle
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("path error: {e}"))?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("sources.json"))
    }

    /// Loads all configured sources from storage
    /// Returns empty vector if no sources are configured or on error
    pub fn list_sources(&self) -> Vec<SourceConfig> {
        match self.read_sources() {
            Ok(v) => v,
            Err(_) => vec![],
        }
    }

    /// Adds a new source or updates an existing one with the same ID
    pub fn add_source(&self, src: SourceConfig) -> Result<()> {
        let mut list = self.read_sources().unwrap_or_default();
        
        // Find existing source or add new one
        if let Some(pos) = list.iter().position(|s| s.id == src.id) {
            // Replace existing source
            list[pos] = src;
        } else {
            // Add new source
            list.push(src);
        }
        
        self.write_sources(&list)
    }

    /// Removes a source by its ID
    pub fn remove_source(&self, id: &str) -> Result<()> {
        let mut list = self.read_sources().unwrap_or_default();
        list.retain(|s| s.id != id);
        self.write_sources(&list)
    }

    /// Ensures default crypto news sources are configured
    /// Adds missing default sources without overwriting existing ones
    pub fn ensure_default_sources(&self) -> Result<()> {
        let mut list = self.read_sources().unwrap_or_default();
        
        // Define all crypto sources with named intervals
        let crypto_sources = vec![
            SourceConfig {
                id: "cryptopanic".into(),
                name: "CryptoPanic".into(),
                url: "https://cryptopanic.com/news/rss/".into(),
                kind: "rss".into(),
                interval_ms: FAST_INTERVAL,
            },
            SourceConfig {
                id: "coindesk".into(),
                name: "CoinDesk".into(),
                url: "https://www.coindesk.com/arc/outboundfeeds/rss/".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
            SourceConfig {
                id: "cointelegraph".into(),
                name: "Cointelegraph".into(),
                url: "https://cointelegraph.com/rss".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
            SourceConfig {
                id: "decrypt".into(),
                name: "Decrypt".into(),
                url: "https://decrypt.co/feed".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
            SourceConfig {
                id: "theblock".into(),
                name: "The Block".into(),
                url: "https://www.theblock.co/rss.xml".into(),
                kind: "rss".into(),
                interval_ms: MEDIUM_INTERVAL,
            },
            SourceConfig {
                id: "bitcoinmagazine".into(),
                name: "Bitcoin Magazine".into(),
                url: "https://bitcoinmagazine.com/.rss/full/".into(),
                kind: "rss".into(),
                interval_ms: SLOWER_INTERVAL,
            },
            SourceConfig {
                id: "coinjournal".into(),
                name: "Coin Journal".into(),
                url: "https://coinjournal.net/feed/".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
            SourceConfig {
                id: "defipulse".into(),
                name: "DeFi Pulse".into(),
                url: "https://defipulse.com/blog/feed".into(),
                kind: "rss".into(),
                interval_ms: SLOW_INTERVAL,
            },
            SourceConfig {
                id: "binance_blog".into(),
                name: "Binance Blog".into(),
                url: "https://www.binance.com/en/blog/rss".into(),
                kind: "rss".into(),
                interval_ms: SLOW_INTERVAL,
            },
            SourceConfig {
                id: "cryptobriefing".into(),
                name: "Crypto Briefing".into(),
                url: "https://cryptobriefing.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
            SourceConfig {
                id: "cryptoslate".into(),
                name: "CryptoSlate".into(),
                url: "https://cryptoslate.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
            SourceConfig {
                id: "newsbtc".into(),
                name: "NewsBTC".into(),
                url: "https://www.newsbtc.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: FASTEST_INTERVAL,
            },
            SourceConfig {
                id: "u_today".into(),
                name: "U.Today".into(),
                url: "https://u.today/rss".into(),
                kind: "rss".into(),
                interval_ms: MEDIUM_INTERVAL,
            },
            SourceConfig {
                id: "cryptopotato".into(),
                name: "CryptoPotato".into(),
                url: "https://cryptopotato.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: NORMAL_INTERVAL,
            },
        ];
        
        // Check which sources are missing and add them
        let mut added_new_sources = false;
        for new_source in crypto_sources {
            if !list.iter().any(|existing| existing.id == new_source.id) {
                list.push(new_source);
                added_new_sources = true;
            }
        }
        
        // Save if we added new sources
        if added_new_sources || list.is_empty() {
            self.write_sources(&list)?;
        }
        
        Ok(())
    }

    /// Reads source configurations from the JSON file
    fn read_sources(&self) -> Result<Vec<SourceConfig>> {
        let path = self.config_path()?;
        if !path.exists() {
            return Ok(vec![]);
        }
        let data = std::fs::read_to_string(path)?;
        let v: Vec<SourceConfig> = serde_json::from_str(&data)?;
        Ok(v)
    }

    /// Writes source configurations to the JSON file
    fn write_sources(&self, list: &Vec<SourceConfig>) -> Result<()> {
        let path = self.config_path()?;
        let data = serde_json::to_string_pretty(list)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}
