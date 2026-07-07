//! Non-régression FR contre le modèle réel.
//! Prérequis : `scripts/setup-model.sh` lancé (llama-server), puis pointez
//! Lucid/le test dessus via LUCID_BACKEND__BASE_URL si le port n'est pas 8080.
//! Exécuter : `cargo test --test nonreg_reel -- --ignored --nocapture`

use lucid::backends::from_config;
use lucid::config::Config;
use lucid::correction::correct;
use lucid::dictionary::Dictionary;
use lucid::openai::ChatRequest;
use std::path::Path;

#[derive(serde::Deserialize)]
struct Cas {
    r#in: String,
    expect_contains: String,
}

#[tokio::test]
#[ignore]
async fn non_regression_fr() {
    // Utilise la config (dont backend.base_url via env LUCID_BACKEND__BASE_URL).
    let cfg = Config::from_figment(None).expect("config");
    let backend = from_config(&cfg.backend);
    let dict = Dictionary::load(Path::new("tests/fixtures/dictionary.json"));
    let cas: Vec<Cas> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/fr_noms_propres.json").unwrap(),
    )
    .unwrap();

    let mut echecs = Vec::new();
    for c in &cas {
        let req: ChatRequest = serde_json::from_value(serde_json::json!({
            "messages": [ { "role": "user", "content": c.r#in } ]
        }))
        .unwrap();
        let out = correct(&*backend, &dict, &cfg.correction, &req).await;
        println!("{:?}\n  -> {:?}\n", c.r#in, out.text);
        if !out.text.contains(&c.expect_contains) {
            echecs.push(format!(
                "{:?} n'a pas produit {:?} (obtenu {:?})",
                c.r#in, c.expect_contains, out.text
            ));
        }
    }
    assert!(
        echecs.is_empty(),
        "Échecs de non-régression (attendu : meilleur avec Q8_0) :\n{}",
        echecs.join("\n")
    );
}
