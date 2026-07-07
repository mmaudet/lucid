//! Socle de l'app barre de menus (Tauri v2, feature `gui`). Vague 1 : minimal.
//!
//! État managé UNIQUE (`AppRuntime`) détenant le `ServerManager`. Les fenêtres
//! Svelte et la couche de commandes viendront en Vague 2 ; ici on pose le socle
//! (tray + démarrage du serveur in-process) sur le runtime Tokio de Tauri.

use crate::config::Config;
use crate::runtime::ServerManager;
use std::sync::Arc;
use tauri::tray::TrayIconBuilder;
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
        .manage(app_runtime)
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

            // Icône barre de menus (titre texte pour l'instant ; icône template en M8).
            TrayIconBuilder::with_id("lucid-tray")
                .title("Lucid")
                .tooltip("Lucid — correcteur de dictée FR (local)")
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
