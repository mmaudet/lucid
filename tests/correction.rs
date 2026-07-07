use lucid::backends::mock::MockBackend;
use lucid::correction::{correct, guardrails::Status};
use lucid::dictionary::Dictionary;
use lucid::openai::ChatRequest;

fn req(text: &str) -> ChatRequest {
    serde_json::from_value(serde_json::json!({
        "model": "lucid",
        "messages": [ { "role": "user", "content": text } ]
    }))
    .unwrap()
}

#[tokio::test]
async fn happy_path_renvoie_texte_corrige() {
    let backend = MockBackend::with_response("Michel-Marie Maudet");
    let dict = Dictionary::default();
    let cfg = lucid::config::Config::default().correction;
    let out = correct(&backend, &dict, &cfg, &req("michel marie mode")).await;
    assert_eq!(out.text, "Michel-Marie Maudet");
    assert_eq!(out.status, Status::Corrected);
}

#[tokio::test]
async fn backend_en_echec_declenche_failsafe() {
    let backend = MockBackend::failing();
    let dict = Dictionary::default();
    let cfg = lucid::config::Config::default().correction;
    let out = correct(&backend, &dict, &cfg, &req("phrase entrée")).await;
    assert_eq!(out.text, "phrase entrée");
    assert_eq!(out.status, Status::FailSafe);
}
