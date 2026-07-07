use lucid::backends::BackendRequest;
use lucid::config::Config;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn req() -> BackendRequest {
    BackendRequest {
        messages: vec![lucid::openai::ChatMessage {
            role: "user".into(),
            content: "michel marie mode".into(),
        }],
        temperature: 0.15,
        top_p: 0.9,
        max_tokens: 128,
        model: "luciole".into(),
    }
}

#[tokio::test]
async fn complete_parse_le_contenu() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [ { "message": { "role": "assistant", "content": "Michel-Marie Maudet" } } ]
        })))
        .mount(&server)
        .await;

    let mut cfg = Config::default();
    cfg.backend.base_url = format!("{}/v1", server.uri());
    let backend = lucid::backends::from_config(&cfg.backend);

    let out = backend.complete(&req()).await.unwrap();
    assert_eq!(out, "Michel-Marie Maudet");
}

#[tokio::test]
async fn health_detecte_l_indisponibilite() {
    let mut cfg = Config::default();
    cfg.backend.base_url = "http://127.0.0.1:1/v1".into(); // port injoignable
    cfg.backend.health_timeout_ms = 300;
    let backend = lucid::backends::from_config(&cfg.backend);
    assert!(!backend.health().await.reachable);
}
