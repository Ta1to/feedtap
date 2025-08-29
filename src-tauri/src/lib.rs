mod types;
mod taps;
mod aggregator;
mod ws;
mod storage;

use std::sync::Arc;
use aggregator::AggregatorHandle;
use tokio::sync::Mutex;
use types::SourceConfig;
use tauri::Manager;

// Tauri commands for managing sources and triggering refresh
#[tauri::command]
async fn list_sources(state: tauri::State<'_, AppState>) -> Result<Vec<SourceConfig>, String> {
    Ok(state.storage.lock().await.list_sources().await)
}

#[tauri::command]
async fn add_source(state: tauri::State<'_, AppState>, src: SourceConfig) -> Result<(), String> {
    state
        .storage
        .lock()
        .await
        .add_source(src)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_source(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .storage
        .lock()
        .await
        .remove_source(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_now(state: tauri::State<'_, AppState>, id: Option<String>) -> Result<(), String> {
    state
        .aggregator
        .trigger_refresh(id)
        .await
        .map_err(|e| e.to_string())
}

struct AppState {
    aggregator: AggregatorHandle,
    storage: Arc<Mutex<storage::Storage>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let port = 8787u16;

            let storage = Arc::new(Mutex::new(storage::Storage::new(app_handle.clone())));
            let storage_clone = storage.clone();

            // ensure default sources asynchronously
            tauri::async_runtime::spawn({
                let storage = storage_clone.clone();
                async move {
                    let mut st = storage.lock().await;
                    let _ = st.ensure_default_sources().await;
                }
            });

            // start aggregator and ws
            let (aggregator, rx) = aggregator::Aggregator::start(storage_clone.clone());
            ws::start_ws_server(port, rx);

            app.manage(AppState { aggregator, storage: storage_clone });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_sources,
            add_source,
            remove_source,
            refresh_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
