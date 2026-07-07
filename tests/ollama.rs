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
        model: "lucid".into(),
        stop: vec![],
    }
}

#[tokio::test]
async fn ollama_backend_complete_parse_le_contenu() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [ { "message": { "role": "assistant", "content": "Michel-Marie Maudet" } } ]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })))
        .mount(&server)
        .await;

    let mut cfg = Config::default();
    cfg.backend.kind = "ollama".into();
    cfg.backend.base_url = format!("{}/v1", server.uri());
    let backend = lucid::backends::from_config(&cfg.backend);

    assert_eq!(backend.complete(&req()).await.unwrap(), "Michel-Marie Maudet");
    assert!(backend.health().await.reachable);
}
