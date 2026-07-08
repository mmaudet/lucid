fn main() {
    // Numéro de build = hash git court, embarqué et affiché dans l'app.
    let build = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "dev".into());
    println!("cargo:rustc-env=LUCID_BUILD={build}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    // Ne déclenche la génération de contexte Tauri (qui exige ui/dist au compile-time)
    // que pour le build GUI. Le build headless (cargo test) reste intact.
    if std::env::var("CARGO_FEATURE_GUI").is_ok() {
        tauri_build::build();
    }
}
