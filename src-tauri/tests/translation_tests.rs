use app_lib::api::translation::generate_back;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn generate_back_returns_parsed_content() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{
                "message": {
                    "content": "TRADUÇÃO\nEla olhou.\n\nEQUIVALENTE\nolhou"
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let result = generate_back(
        &mock_server.uri(),
        "test-key",
        "test-model",
        "English",
        "Brazilian Portuguese",
        "She looked.",
        "looked",
        "iniciante",
        30,
    )
    .await
    .expect("should generate back");

    assert_eq!(result, "Ela olhou.\nolhou");
}

#[tokio::test]
async fn generate_back_returns_error_on_http_failure() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let err = generate_back(
        &mock_server.uri(),
        "bad-key",
        "test-model",
        "English",
        "Brazilian Portuguese",
        "Hello.",
        "Hello",
        "iniciante",
        30,
    )
    .await
    .expect_err("should fail");

    assert!(err.contains("401"));
}

#[tokio::test]
async fn generate_back_returns_error_on_empty_choices() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": []
        })))
        .mount(&mock_server)
        .await;

    let err = generate_back(
        &mock_server.uri(),
        "test-key",
        "test-model",
        "English",
        "Brazilian Portuguese",
        "Hello.",
        "Hello",
        "iniciante",
        30,
    )
    .await
    .expect_err("should fail");

    assert!(err.contains("vazia"));
}

#[tokio::test]
async fn generate_back_timeout_returns_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    let err = generate_back(
        &mock_server.uri(),
        "test-key",
        "test-model",
        "English",
        "Brazilian Portuguese",
        "Hello.",
        "Hello",
        "iniciante",
        1,
    )
    .await
    .expect_err("should timeout");

    // The error message is platform-dependent but the key is:
    // generate_back returns Err on timeout, not a panic
    assert!(!err.is_empty(), "expected error message on timeout");
}
