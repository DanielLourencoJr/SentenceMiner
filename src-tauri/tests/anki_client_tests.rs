use app_lib::anki::client::AnkiClient;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn check_connection_returns_version() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 6,
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());
    let version = client.check_connection().await.expect("should connect");

    assert_eq!(version, 6);
}

#[tokio::test]
async fn get_deck_names_returns_list() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": ["Default", "English", "JLPT"],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());
    let decks = client.get_deck_names().await.expect("should get decks");

    assert_eq!(decks, vec!["Default", "English", "JLPT"]);
}

#[tokio::test]
async fn get_model_names_returns_list() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": ["Basic", "Basic (and reversed)"],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());
    let models = client.get_model_names().await.expect("should get models");

    assert_eq!(models, vec!["Basic", "Basic (and reversed)"]);
}

#[tokio::test]
async fn get_model_field_names_returns_fields() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": ["Front", "Back"],
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());
    let fields = client
        .get_model_field_names("Basic")
        .await
        .expect("should get fields");

    assert_eq!(fields, vec!["Front", "Back"]);
}

#[tokio::test]
async fn add_note_returns_note_id() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 1234567890,
            "error": null
        })))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());

    let mut fields = serde_json::Map::new();
    fields.insert("Front".to_string(), json!("hello world"));
    fields.insert("Back".to_string(), json!("olá mundo"));

    let note_id = client
        .add_note("Default", "Basic", fields, &["sentenceminer".to_string()])
        .await
        .expect("should add note");

    assert_eq!(note_id, 1234567890);
}

#[tokio::test]
async fn returns_error_when_anki_returns_error_field() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": null,
            "error": "deck not found"
        })))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());
    let err = client.get_deck_names().await.expect_err("should fail");

    assert!(err.contains("deck not found"));
}

#[tokio::test]
async fn returns_error_when_server_returns_http_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = AnkiClient::new("localhost", mock_server.address().port());
    let err = client.check_connection().await.expect_err("should fail");

    assert!(err.contains("500") || err.contains("error") || err.contains("status"));
}
