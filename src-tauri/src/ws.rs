use crate::types::WsMessage;
use axum::{routing::get, Router};
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::extract::State;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
#[derive(Clone)]
struct WsState {
    tx: broadcast::Sender<crate::types::NewsItem>,
}

pub fn start_ws_server(port: u16, mut rx: broadcast::Receiver<crate::types::NewsItem>) {
    let (tx_items, _) = broadcast::channel(1024); // Increased buffer for better performance

    // pump from aggregator rx into ws tx_items
    let tx_forward = tx_items.clone();
    tauri::async_runtime::spawn(async move {
        while let Ok(item) = rx.recv().await {
            // Performance optimization: Only forward if there are subscribers
            if tx_forward.receiver_count() > 0 {
                let _ = tx_forward.send(item);
            }
        }
    });

    let state = WsState { tx: tx_items.clone() };
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
    ws.on_upgrade(move |socket| ws_stream(socket, state))
}

async fn ws_stream(mut socket: WebSocket, state: WsState) {
    // Send hello
    let hello = WsMessage::Hello { server_version: env!("CARGO_PKG_VERSION").to_string() };
    if socket.send(Message::Text(serde_json::to_string(&hello).unwrap())).await.is_err() {
        return;
    }

    let mut rx = state.tx.subscribe();
    let mut hb = interval(Duration::from_secs(30)); // Reduced heartbeat frequency for performance
    
    loop {
        tokio::select! {
            Ok(item) = rx.recv() => {
                let msg = WsMessage::Item(item);
                // Performance optimization: Pre-serialize JSON once
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            _ = hb.tick() => {
                let msg = WsMessage::Heartbeat;
                if socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.is_err() {
                    break;
                }
            }
            else => break,
        }
    }
}
