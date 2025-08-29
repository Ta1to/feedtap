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
        if list.is_empty() {
            list.push(SourceConfig {
                id: "cryptopanic".into(),
                name: "CryptoPanic".into(),
                url: "https://cryptopanic.com/news/rss/".into(),
                kind: "rss".into(),
                interval_ms: 60_000,
            });
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
