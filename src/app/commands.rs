//! Couche de commandes Tauri appelées par le frontend Svelte.
//! Un état managé UNIQUE (`AppRuntime`) → source de vérité unique.

use tauri::State;

use super::windows::{open_view, View};
use super::AppRuntime;
use crate::api_info::{api_info, ApiInfo};

#[tauri::command]
pub async fn server_status(state: State<'_, AppRuntime>) -> Result<serde_json::Value, String> {
    let mgr = state.server.lock().await;
    let running = mgr.is_running();
    let reachable = mgr.backend_health().await.reachable;
    Ok(serde_json::json!({ "running": running, "backend_reachable": reachable }))
}

#[tauri::command]
pub async fn endpoint_info(state: State<'_, AppRuntime>) -> Result<ApiInfo, String> {
    let mgr = state.server.lock().await;
    Ok(api_info(mgr.config()))
}

#[tauri::command]
pub async fn start_server(state: State<'_, AppRuntime>) -> Result<(), String> {
    state
        .server
        .lock()
        .await
        .start()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_server(state: State<'_, AppRuntime>) -> Result<(), String> {
    state.server.lock().await.stop().await;
    Ok(())
}

#[tauri::command]
pub fn open_window(app: tauri::AppHandle, view: String) -> Result<(), String> {
    let v = match view.as_str() {
        "dictionary" => View::Dictionary,
        "settings" => View::Settings,
        "journal" => View::Journal,
        "stats" => View::Stats,
        other => return Err(format!("vue inconnue : {other}")),
    };
    open_view(&app, v).map_err(|e| e.to_string())
}
