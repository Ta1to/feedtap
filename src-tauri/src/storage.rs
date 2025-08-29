use crate::types::SourceConfig;
use anyhow::Result;
use std::path::PathBuf;
use tauri::Manager;

pub struct Storage {
    app_handle: tauri::AppHandle,
}

impl Storage {
    pub fn new(app_handle: tauri::AppHandle) -> Self { Self { app_handle } }

    fn config_path(&self) -> Result<PathBuf> {
        let dir = self
            .app_handle
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("path error: {e}"))?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("sources.json"))
    }

    pub async fn list_sources(&self) -> Vec<SourceConfig> {
        match self.read_sources() {
            Ok(v) => v,
            Err(_) => vec![],
        }
    }

    pub async fn add_source(&mut self, src: SourceConfig) -> Result<()> {
        let mut list = self.read_sources().unwrap_or_default();
        if list.iter().any(|s| s.id == src.id) {
            // replace
            if let Some(pos) = list.iter().position(|s| s.id == src.id) {
                list[pos] = src;
            }
        } else {
            list.push(src);
        }
        self.write_sources(&list)
    }

    pub async fn remove_source(&mut self, id: &str) -> Result<()> {
        let mut list = self.read_sources().unwrap_or_default();
        list.retain(|s| s.id != id);
        self.write_sources(&list)
    }

    pub async fn ensure_default_sources(&mut self) -> Result<()> {
        let mut list = self.read_sources().unwrap_or_default();
        
        // Define all crypto sources
        let crypto_sources = vec![
            SourceConfig {
                id: "cryptopanic".into(),
                name: "CryptoPanic".into(),
                url: "https://cryptopanic.com/news/rss/".into(),
                kind: "rss".into(),
                interval_ms: 60_000,
            },
            SourceConfig {
                id: "coindesk".into(),
                name: "CoinDesk".into(),
                url: "https://www.coindesk.com/arc/outboundfeeds/rss/".into(),
                kind: "rss".into(),
                interval_ms: 300_000, // 5 minutes
            },
            SourceConfig {
                id: "cointelegraph".into(),
                name: "Cointelegraph".into(),
                url: "https://cointelegraph.com/rss".into(),
                kind: "rss".into(),
                interval_ms: 300_000,
            },
            SourceConfig {
                id: "decrypt".into(),
                name: "Decrypt".into(),
                url: "https://decrypt.co/feed".into(),
                kind: "rss".into(),
                interval_ms: 300_000,
            },
            SourceConfig {
                id: "theblock".into(),
                name: "The Block".into(),
                url: "https://www.theblock.co/rss.xml".into(),
                kind: "rss".into(),
                interval_ms: 240_000, // 4 minutes
            },
            SourceConfig {
                id: "bitcoinmagazine".into(),
                name: "Bitcoin Magazine".into(),
                url: "https://bitcoinmagazine.com/.rss/full/".into(),
                kind: "rss".into(),
                interval_ms: 360_000, // 6 minutes
            },
            SourceConfig {
                id: "coinjournal".into(),
                name: "Coin Journal".into(),
                url: "https://coinjournal.net/feed/".into(),
                kind: "rss".into(),
                interval_ms: 300_000,
            },
            SourceConfig {
                id: "defipulse".into(),
                name: "DeFi Pulse".into(),
                url: "https://defipulse.com/blog/feed".into(),
                kind: "rss".into(),
                interval_ms: 600_000, // 10 minutes
            },
            SourceConfig {
                id: "binance_blog".into(),
                name: "Binance Blog".into(),
                url: "https://www.binance.com/en/blog/rss".into(),
                kind: "rss".into(),
                interval_ms: 600_000,
            },
            SourceConfig {
                id: "cryptobriefing".into(),
                name: "Crypto Briefing".into(),
                url: "https://cryptobriefing.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: 300_000,
            },
            SourceConfig {
                id: "cryptoslate".into(),
                name: "CryptoSlate".into(),
                url: "https://cryptoslate.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: 300_000,
            },
            SourceConfig {
                id: "newsbtc".into(),
                name: "NewsBTC".into(),
                url: "https://www.newsbtc.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: 180_000, // 3 minutes
            },
            SourceConfig {
                id: "u_today".into(),
                name: "U.Today".into(),
                url: "https://u.today/rss".into(),
                kind: "rss".into(),
                interval_ms: 240_000,
            },
            SourceConfig {
                id: "cryptopotato".into(),
                name: "CryptoPotato".into(),
                url: "https://cryptopotato.com/feed/".into(),
                kind: "rss".into(),
                interval_ms: 300_000,
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

    fn read_sources(&self) -> Result<Vec<SourceConfig>> {
        let path = self.config_path()?;
        if !path.exists() {
            return Ok(vec![]);
        }
        let data = std::fs::read_to_string(path)?;
        let v: Vec<SourceConfig> = serde_json::from_str(&data)?;
        Ok(v)
    }

    fn write_sources(&self, list: &Vec<SourceConfig>) -> Result<()> {
        let path = self.config_path()?;
        let data = serde_json::to_string_pretty(list)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}
