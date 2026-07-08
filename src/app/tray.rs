//! Icône barre de menus + menu + gestion des événements.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::windows::{open_view, View};
use super::AppRuntime;
use crate::api_info::{api_info, ApiInfo};

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    // Icône barre de menus : carré blanc + « L » noir (couleurs conservées, non-template).
    // include_image! traite le PNG au build ; chemin relatif à la racine de la crate.
    TrayIconBuilder::with_id("lucid-tray")
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(false)
        .tooltip("Lucid — correcteur de dictée FR (local)")
        .menu(&menu)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .build(app)?;
    Ok(())
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let toggle = MenuItem::with_id(app, "toggle", "Arrêter le service", true, None::<&str>)?;
    let dict = MenuItem::with_id(app, "open_dictionary", "Dictionnaire…", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "open_settings", "Réglages…", true, None::<&str>)?;
    let journal = MenuItem::with_id(app, "open_journal", "Journal…", true, None::<&str>)?;
    let stats = MenuItem::with_id(app, "open_stats", "Statistiques…", true, None::<&str>)?;
    let copy_url = MenuItem::with_id(app, "copy_url", "Copier l'URL de l'endpoint", true, None::<&str>)?;
    let copy_token = MenuItem::with_id(app, "copy_token", "Copier le token bearer", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter Lucid", true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &toggle,
            &PredefinedMenuItem::separator(app)?,
            &dict,
            &settings,
            &journal,
            &stats,
            &PredefinedMenuItem::separator(app)?,
            &copy_url,
            &copy_token,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )
}

fn on_menu_event(app: &AppHandle, id: &str) {
    match id {
        "toggle" => toggle_server(app),
        "open_dictionary" => {
            let _ = open_view(app, View::Dictionary);
        }
        "open_settings" => {
            let _ = open_view(app, View::Settings);
        }
        "open_journal" => {
            let _ = open_view(app, View::Journal);
        }
        "open_stats" => {
            let _ = open_view(app, View::Stats);
        }
        "copy_url" => copy_text(app, |i| i.base_url),
        "copy_token" => copy_text(app, |i| i.api_key),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn toggle_server(app: &AppHandle) {
    let server = app.state::<AppRuntime>().server.clone();
    tauri::async_runtime::spawn(async move {
        let mut mgr = server.lock().await;
        if mgr.is_running() {
            mgr.stop().await;
        } else {
            let _ = mgr.start().await;
        }
    });
}

/// Copie une valeur d'ApiInfo (URL ou token) dans le presse-papiers.
fn copy_text(app: &AppHandle, pick: fn(ApiInfo) -> String) {
    let app = app.clone();
    let server = app.state::<AppRuntime>().server.clone();
    tauri::async_runtime::spawn(async move {
        let text = {
            let mgr = server.lock().await;
            pick(api_info(mgr.config()))
        };
        let _ = app.clipboard().write_text(text);
    });
}
