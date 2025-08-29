use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default = "default_kind")]
    pub kind: String, // "rss" for MVP
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
}

fn default_kind() -> String { "rss".into() }
fn default_interval_ms() -> u64 { 60_000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub link: String,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub source: SourceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "lowercase")]
pub enum WsMessage {
    Hello { server_version: String },
    Item(NewsItem),
    Heartbeat,
    Error { message: String },
}
