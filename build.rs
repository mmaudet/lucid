fn main() {
    // Ne déclenche la génération de contexte Tauri (qui exige ui/dist au compile-time)
    // que pour le build GUI. Le build headless (cargo test) reste intact.
    if std::env::var("CARGO_FEATURE_GUI").is_ok() {
        tauri_build::build();
    }
}
