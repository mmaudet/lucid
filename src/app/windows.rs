//! Fenêtre principale UNIQUE (barre latérale) + navigation vers une section.

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const MAIN: &str = "main";

#[derive(Clone, Copy, Debug)]
pub enum View {
    Dictionary,
    Settings,
    Journal,
    Stats,
}

impl View {
    pub fn label(self) -> &'static str {
        match self {
            View::Dictionary => "dictionary",
            View::Settings => "settings",
            View::Journal => "journal",
            View::Stats => "stats",
        }
    }
}

/// Crée-ou-refocalise la fenêtre principale ; `section` = section à afficher (hash à la
/// création, événement `navigate` si la fenêtre existe déjà).
fn open(app: &AppHandle, section: Option<&str>) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window(MAIN) {
        win.show()?;
        win.set_focus()?;
        if let Some(s) = section {
            let _ = app.emit("navigate", s);
        }
        return Ok(());
    }
    let url = match section {
        Some(s) => format!("index.html#{s}"),
        None => "index.html".to_string(),
    };
    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App(url.into()))
        .title("Lucid")
        .inner_size(940.0, 640.0)
        .min_inner_size(760.0, 480.0)
        .build()?;
    Ok(())
}

/// Ouvre la fenêtre et navigue vers une section précise.
pub fn open_view(app: &AppHandle, view: View) -> tauri::Result<()> {
    open(app, Some(view.label()))
}

/// Ouvre la fenêtre principale (accueil).
pub fn open_main(app: &AppHandle) -> tauri::Result<()> {
    open(app, None)
}
