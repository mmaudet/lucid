//! Infra fenêtres : ouvre-ou-refocalise une webview Svelte par label.
//! Socle réutilisé par les 4 fenêtres (Dictionnaire / Réglages / Journal / Stats).

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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

    pub fn title(self) -> &'static str {
        match self {
            View::Dictionary => "Lucid — Dictionnaire",
            View::Settings => "Lucid — Réglages",
            View::Journal => "Lucid — Journal",
            View::Stats => "Lucid — Statistiques",
        }
    }

    pub fn html(self) -> &'static str {
        match self {
            View::Dictionary => "dictionary.html",
            View::Settings => "settings.html",
            View::Journal => "journal.html",
            View::Stats => "stats.html",
        }
    }
}

/// Affiche+focus la fenêtre si elle existe, sinon la crée.
pub fn open_view(app: &AppHandle, view: View) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window(view.label()) {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, view.label(), WebviewUrl::App(view.html().into()))
        .title(view.title())
        .inner_size(900.0, 640.0)
        .min_inner_size(560.0, 400.0)
        .build()?;
    Ok(())
}
