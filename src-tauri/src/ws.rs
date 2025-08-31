use crate::types::{WsMessage, LogMessage};
use axum::{routing::get, Router};
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use moka::future::Cache;
use serde::{Serialize, Deserialize};
use regex::Regex;
use tower_http::cors::{Any, CorsLayer};
use url::Url;
#[derive(Clone)]
struct WsState {
    tx: broadcast::Sender<crate::types::NewsItem>,
    log_tx: broadcast::Sender<LogMessage>,
    preview_cache: Arc<Cache<String, String>>, // url -> sanitized preview html/text
}

pub fn start_ws_server(port: u16, mut rx: broadcast::Receiver<crate::types::NewsItem>) {
    let (tx_items, _) = broadcast::channel(1024); // Increased buffer for better performance
    let (log_tx, _) = broadcast::channel(256); // Channel for log messages
    let preview_cache: Arc<Cache<String, String>> = Arc::new(
        Cache::builder()
            .max_capacity(256)
            .time_to_live(std::time::Duration::from_secs(60 * 60)) // 1h TTL
            .build()
    );

    // pump from aggregator rx into ws tx_items
    let tx_forward = tx_items.clone();
    tauri::async_runtime::spawn(async move {
        while let Ok(item) = rx.recv().await {
            // Performance optimization: Only forward if there are subscribers
            if tx_forward.receiver_count() > 0 {
                tracing::debug!(
                    item_id = %item.id,
                    subscriber_count = tx_forward.receiver_count(),
                    "Forwarding item to WebSocket subscribers"
                );
                if let Err(_) = tx_forward.send(item) {
                    tracing::warn!("Failed to forward item to WebSocket subscribers");
                }
            } else {
                tracing::debug!(
                    item_id = %item.id,
                    "Skipping item - no WebSocket subscribers"
                );
            }
        }
    });

    let state = WsState { tx: tx_items.clone(), log_tx: log_tx.clone(), preview_cache: preview_cache.clone() };

    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/stream", get(ws_handler))
        .route("/preview", get(http_preview))
        .with_state(state)
        .layer(cors);

    tauri::async_runtime::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        tracing::info!(%addr, "WS server listening");
        axum::serve(
            tokio::net::TcpListener::bind(addr).await.expect("bind ws"),
            app
        ).await.expect("serve ws");
    });
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WsState>) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| {
        tracing::info!("New WebSocket client connected");
        ws_stream(socket, state)
    })
}

async fn ws_stream(mut socket: WebSocket, state: WsState) {
    // Send hello
    let hello = WsMessage::Hello { server_version: env!("CARGO_PKG_VERSION").to_string() };
    if socket.send(Message::Text(serde_json::to_string(&hello).unwrap())).await.is_err() {
        tracing::warn!("Failed to send hello message to WebSocket client");
        return;
    }
    tracing::info!(server_version = env!("CARGO_PKG_VERSION"), "Sent hello message to WebSocket client");

    let mut rx = state.tx.subscribe();
    let mut log_rx = state.log_tx.subscribe();
    let mut hb = interval(Duration::from_secs(30)); // Reduced heartbeat frequency for performance
    
    loop {
        tokio::select! {
            Ok(item) = rx.recv() => {
                tracing::debug!(
                    item_id = %item.id,
                    item_title = %item.title,
                    source_id = %item.source.id,
                    source_name = %item.source.name,
                    "Sending news item via WebSocket"
                );
                
                let msg = WsMessage::Item(item.clone());
                // Performance optimization: Pre-serialize JSON once
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        tracing::warn!("Failed to send WebSocket message, client disconnected");
                        break;
                    } else {
                        // Only log SENT when successfully transmitted to client
                        let log_msg = LogMessage {
                            level: "sent".to_string(),
                            message: format!("Sent '{}' from {}", 
                                           item.title.chars().take(40).collect::<String>(),
                                           item.source.name),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap().as_millis() as u64,
                        };
                        let _ = state.log_tx.send(log_msg);
                    }
                } else {
                    tracing::error!("Failed to serialize WebSocket message");
                }
            }
            Ok(log_msg) = log_rx.recv() => {
                let msg = WsMessage::Log(log_msg);
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        tracing::warn!("Failed to send log message, client disconnected");
                        break;
                    }
                }
            }
            _ = hb.tick() => {
                let msg = WsMessage::Heartbeat;
                if socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.is_err() {
                    tracing::info!("WebSocket client disconnected during heartbeat");
                    break;
                }
            }
            else => {
                tracing::info!("WebSocket client disconnected");
                break;
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct PreviewQuery { url: String }

#[derive(Debug, Serialize)]
struct PreviewResponse {
    url: String,
    content_preview: String,
}

async fn http_preview(Query(q): Query<PreviewQuery>, State(state): State<WsState>) -> impl IntoResponse {
    // validate URL
    let url = match Url::parse(&q.url) {
        Ok(u) if matches!(u.scheme(), "http" | "https") => u,
        _ => return (StatusCode::BAD_REQUEST, "invalid url").into_response(),
    };

    // cache hit
    if let Some(cached) = state.preview_cache.get(&q.url).await {
        let body = serde_json::to_string(&PreviewResponse { url: q.url.clone(), content_preview: cached }).unwrap();
        return (StatusCode::OK, body).into_response();
    }

    // fetch with timeout
    let client = reqwest::Client::builder()
        .user_agent("FeedTap/2.0 Preview (+https://github.com/Ta1to/feedtap)")
        .timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to build HTTP client for preview");
            return (StatusCode::INTERNAL_SERVER_ERROR, "client error").into_response();
        }
    };

    let resp = match client.get(url.clone()).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::info!(error = %e, link = %q.url, "Preview fetch failed");
            return (StatusCode::BAD_GATEWAY, "fetch failed").into_response();
        }
    };

    let status = resp.status();
    if !status.is_success() {
        tracing::info!(code = %status, link = %q.url, "Preview non-success status");
        return (StatusCode::BAD_GATEWAY, "upstream status").into_response();
    }

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            tracing::info!(error = %e, link = %q.url, "Preview read body failed");
            return (StatusCode::BAD_GATEWAY, "body failed").into_response();
        }
    };

    let html = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => {
            return (StatusCode::BAD_GATEWAY, "non-utf8 body").into_response();
        }
    };

    // Try extract main content
    let extracted = extract_main_content(html);
    
    // If extraction failed or returned script content, try to get meta description
    let final_content = if extracted.is_empty() || contains_script_content(&extracted) {
        let doc = scraper::Html::parse_document(html);
        if let Ok(meta_sel) = scraper::Selector::parse("meta[name=description], meta[property='og:description']") {
            if let Some(meta) = doc.select(&meta_sel).next() {
                if let Some(content) = meta.value().attr("content") {
                    if content.len() > 30 {
                        content.to_string()
                    } else {
                        "Preview not available for this content.".to_string()
                    }
                } else {
                    "Preview not available for this content.".to_string()
                }
            } else {
                "Preview not available for this content.".to_string()
            }
        } else {
            "Preview not available for this content.".to_string()
        }
    } else {
        extracted
    };

    let sanitized = ammonia::Builder::default()
        .add_tags(["p", "a", "strong", "em", "ul", "ol", "li", "blockquote", "h3", "h4"].into_iter())
        .url_relative(ammonia::UrlRelative::Deny)
        .clean(&final_content)
        .to_string();

    // Truncate to a safe size
    let preview = if sanitized.len() > 3000 {
        format!("{}…", &sanitized[..2999])
    } else { sanitized };

    state.preview_cache.insert(q.url.clone(), preview.clone()).await;

    let body = serde_json::to_string(&PreviewResponse { url: q.url, content_preview: preview }).unwrap();
    (StatusCode::OK, body).into_response()
}

fn extract_main_content(html: &str) -> String {
    let doc = scraper::Html::parse_document(html);
    
    // Special handling for CryptoPanic pages - look for the actual article content or source link
    if html.contains("cryptopanic.com") {
        // Try to find the news summary/description in CryptoPanic's structure
        let cp_selectors = [
            ".news-summary",
            ".post-content", 
            ".news-description",
            "[data-news-summary]",
            ".article-content"
        ];
        
        for selector_str in &cp_selectors {
            if let Ok(sel) = scraper::Selector::parse(selector_str) {
                for node in doc.select(&sel) {
                    let text = extract_node_text(node);
                    if text.len() > 100 && !text.contains("window.App") && !text.contains("OneSignal") {
                        return text;
                    }
                }
            }
        }
        
        // Look for meta description as fallback for CryptoPanic
        if let Ok(meta_sel) = scraper::Selector::parse("meta[name=description]") {
            if let Some(meta) = doc.select(&meta_sel).next() {
                if let Some(content) = meta.value().attr("description") {
                    if content.len() > 50 {
                        return content.to_string();
                    }
                }
            }
        }
    }

    // Standard content extraction for other sites
    let content_selectors = [
        "article",
        "main", 
        "[role=main]",
        ".article",
        ".post",
        ".content",
        ".entry-content",
        ".post-content"
    ];
    
    for selector_str in &content_selectors {
        if let Ok(sel) = scraper::Selector::parse(selector_str) {
            for node in doc.select(&sel) {
                let text = extract_node_text(node);
                if text.len() > 200 && !contains_script_content(&text) {
                    return text;
                }
            }
        }
    }
    
    // Fallback: body text, but filter out script content
    if let Ok(sel_body) = scraper::Selector::parse("body") {
        if let Some(body) = doc.select(&sel_body).next() {
            let text = extract_node_text(body);
            if !text.is_empty() && !contains_script_content(&text) {
                return text;
            }
        }
    }
    
    // Last resort: strip tags naively
    let cleaned = naive_strip_html(html);
    if contains_script_content(&cleaned) {
        "Content could not be extracted properly.".to_string()
    } else {
        cleaned
    }
}

fn extract_node_text(node: scraper::ElementRef) -> String {
    let mut out = String::new();
    for child in node.text() {
        let text = child.trim();
        if !text.is_empty() {
            out.push_str(text);
            out.push(' ');
        }
    }
    out.trim().to_string()
}

fn contains_script_content(text: &str) -> bool {
    text.contains("window.App") || 
    text.contains("OneSignal") || 
    text.contains("VueComponents") ||
    text.contains("javascript:") ||
    text.contains("var ") ||
    text.contains("function(") ||
    text.contains("jQuery") ||
    text.len() < 50  // Too short to be meaningful content
}

fn naive_strip_html(html: &str) -> String {
    // Simple and fast tag removal, collapse whitespace
    let tag_re = Regex::new(r"<[^>]*>").unwrap();
    let ws_re = Regex::new(r"\s+").unwrap();
    let no_tags = tag_re.replace_all(html, " ");
    let collapsed = ws_re.replace_all(&no_tags, " ");
    collapsed.trim().to_string()
}
