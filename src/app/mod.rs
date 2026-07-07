//! Socle de l'app barre de menus (Tauri v2, feature `gui`).
//!
//! État managé UNIQUE (`AppRuntime`) détenant le `ServerManager`. La couche de
//! commandes (`commands`) et les fenêtres (`windows`) sont partagées avec le menu
//! du tray (`tray`). Le serveur axum tourne dans le runtime Tokio de Tauri.

mod commands;
mod tray;
mod windows;

use crate::config::Config;
use crate::runtime::ServerManager;
use std::sync::Arc;
use tauri::{ActivationPolicy, Manager};
use tokio::sync::Mutex;

/// État managé unique : détient le propriétaire du serveur.
pub struct AppRuntime {
    pub server: Arc<Mutex<ServerManager>>,
}

pub fn run(config: Config) -> anyhow::Result<()> {
    let app_runtime = AppRuntime {
        server: Arc::new(Mutex::new(ServerManager::new(config))),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_runtime)
        .invoke_handler(tauri::generate_handler![
            commands::server_status,
            commands::endpoint_info,
            commands::start_server,
            commands::stop_server,
            commands::open_window,
            commands::dict_list,
            commands::dict_save,
            commands::dict_add_term,
            commands::journal_list,
            commands::journal_clear,
            commands::stats_summary,
            commands::config_get,
            commands::config_save,
        ])
        .setup(|app| {
            // App barre de menus : pas d'icône dans le Dock.
            app.set_activation_policy(ActivationPolicy::Accessory);

            // Démarre le serveur HTTP local au lancement (runtime Tokio de Tauri).
            let server = app.state::<AppRuntime>().server.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = server.lock().await.start().await {
                    eprintln!("Lucid : échec du démarrage du serveur : {e}");
                }
            });

            tray::build_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
