use crate::types::{WsMessage, LogMessage};
use axum::{routing::get, Router};
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::extract::State;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
#[derive(Clone)]
struct WsState {
    tx: broadcast::Sender<crate::types::NewsItem>,
    log_tx: broadcast::Sender<LogMessage>,
}

pub fn start_ws_server(port: u16, mut rx: broadcast::Receiver<crate::types::NewsItem>) {
    let (tx_items, _) = broadcast::channel(1024); // Increased buffer for better performance
    let (log_tx, _) = broadcast::channel(256); // Channel for log messages

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

    let state = WsState { tx: tx_items.clone(), log_tx: log_tx.clone() };
    let app = Router::new()
        .route("/stream", get(ws_handler))
        .with_state(state);

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
