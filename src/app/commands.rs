//! Couche de commandes Tauri appelées par le frontend Svelte.
//! Un état managé UNIQUE (`AppRuntime`) → source de vérité unique.

use tauri::State;

use super::windows::{open_view, View};
use super::AppRuntime;
use crate::api_info::{api_info, ApiInfo};
use crate::config::Config;
use crate::dictionary::{Dictionary, Term};
use crate::store::{JournalRow, StatsSummary};

// ---------- Service / général ----------

#[tauri::command]
pub async fn server_status(state: State<'_, AppRuntime>) -> Result<serde_json::Value, String> {
    let mgr = state.server.lock().await;
    let running = mgr.is_running();
    let model = mgr.config().backend.model.clone();
    let reachable = mgr.backend_health().await.reachable;
    Ok(serde_json::json!({ "running": running, "backend_reachable": reachable, "model": model }))
}

#[tauri::command]
pub async fn endpoint_info(state: State<'_, AppRuntime>) -> Result<ApiInfo, String> {
    let mgr = state.server.lock().await;
    Ok(api_info(mgr.config()))
}

#[tauri::command]
pub fn app_build() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build": env!("LUCID_BUILD"),
    })
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
    let v = parse_view(&view)?;
    open_view(&app, v).map_err(|e| e.to_string())
}

fn parse_view(view: &str) -> Result<View, String> {
    match view {
        "dictionary" => Ok(View::Dictionary),
        "settings" => Ok(View::Settings),
        "journal" => Ok(View::Journal),
        "stats" => Ok(View::Stats),
        other => Err(format!("vue inconnue : {other}")),
    }
}

// ---------- Dictionnaire (M4) ----------

#[tauri::command]
pub async fn dict_list(state: State<'_, AppRuntime>) -> Result<Dictionary, String> {
    let mgr = state.server.lock().await;
    Ok((*mgr.dictionary().snapshot()).clone())
}

#[tauri::command]
pub async fn dict_save(state: State<'_, AppRuntime>, dict: Dictionary) -> Result<(), String> {
    let risky = crate::correction::risky_aliases(&dict);
    if !risky.is_empty() {
        let list = risky
            .iter()
            .map(|(c, a)| format!("« {a} » (→ {c})"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Alias trop proche(s) d'un mot français courant, refusé(s) — risque de fausses \
             corrections : {list}. Utilisez la faute de transcription exacte, ou un alias multi-mots."
        ));
    }
    let mgr = state.server.lock().await;
    mgr.dictionary().replace(dict).map_err(|e| e.to_string())
}

/// Ajoute un terme (ou un alias à un terme existant) — utilisé par « ajouter au dictionnaire ».
#[tauri::command]
pub async fn dict_add_term(
    state: State<'_, AppRuntime>,
    canonical: String,
    alias: Option<String>,
) -> Result<(), String> {
    let alias = alias.filter(|a| !a.trim().is_empty());
    if let Some(a) = &alias {
        if !a.contains(char::is_whitespace) && crate::correction::common_words::is_common_word(a) {
            return Err(format!(
                "L'alias « {a} » est un mot français courant — refusé (risque de fausses \
                 corrections). Choisissez une variante plus distinctive."
            ));
        }
    }
    let mgr = state.server.lock().await;
    let mut dict = (*mgr.dictionary().snapshot()).clone();
    match dict.terms.iter_mut().find(|t| t.canonical == canonical) {
        Some(t) => {
            if let Some(a) = alias {
                if !t.aliases.contains(&a) {
                    t.aliases.push(a);
                }
            }
        }
        None => dict.terms.push(Term {
            canonical,
            aliases: alias.into_iter().collect(),
        }),
    }
    mgr.dictionary().replace(dict).map_err(|e| e.to_string())
}

// ---------- Journal (M5) ----------

#[tauri::command]
pub async fn journal_list(
    state: State<'_, AppRuntime>,
    limit: Option<i64>,
) -> Result<Vec<JournalRow>, String> {
    let store = { state.server.lock().await.store().clone() };
    Ok(store.recent(limit.unwrap_or(200)).await)
}

#[tauri::command]
pub async fn journal_clear(state: State<'_, AppRuntime>) -> Result<(), String> {
    let store = { state.server.lock().await.store().clone() };
    store.clear().await;
    Ok(())
}

// ---------- Statistiques (M6) ----------

#[tauri::command]
pub async fn stats_summary(state: State<'_, AppRuntime>) -> Result<StatsSummary, String> {
    let store = { state.server.lock().await.store().clone() };
    Ok(store.stats_summary().await)
}

// ---------- Réglages (M7) ----------

#[tauri::command]
pub async fn config_get(state: State<'_, AppRuntime>) -> Result<Config, String> {
    Ok(state.server.lock().await.config().clone())
}

/// Persiste la config et l'applique (redémarre le serveur : host/port/backend).
#[tauri::command]
pub async fn config_save(state: State<'_, AppRuntime>, config: Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    let mut mgr = state.server.lock().await;
    mgr.apply_config(config)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}
