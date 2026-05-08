use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
pub struct AnkiClient {
    base_url: String,
    client: reqwest::Client,
}

fn build_anki_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}")
}

impl AnkiClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: build_anki_url(host, port),
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_connection(&self) -> Result<u16, String> {
        let req = AnkiRequest::<serde_json::Value> {
            action: "version".to_string(),
            version: 6,
            params: None,
        };
        let resp: AnkiResponse<u16> = self.post(req).await?;
        resp.result.ok_or_else(|| resp.error.unwrap_or("Erro AnkiConnect.".to_string()))
    }

    pub async fn get_deck_names(&self) -> Result<Vec<String>, String> {
        let req = AnkiRequest::<serde_json::Value> {
            action: "deckNames".to_string(),
            version: 6,
            params: None,
        };
        let resp: AnkiResponse<Vec<String>> = self.post(req).await?;
        resp.result.ok_or_else(|| resp.error.unwrap_or("Erro AnkiConnect.".to_string()))
    }

    pub async fn get_model_names(&self) -> Result<Vec<String>, String> {
        let req = AnkiRequest::<serde_json::Value> {
            action: "modelNames".to_string(),
            version: 6,
            params: None,
        };
        let resp: AnkiResponse<Vec<String>> = self.post(req).await?;
        resp.result.ok_or_else(|| resp.error.unwrap_or("Erro AnkiConnect.".to_string()))
    }

    pub async fn get_model_field_names(&self, model: &str) -> Result<Vec<String>, String> {
        #[derive(Serialize)]
        struct ModelFieldParams {
            #[serde(rename = "modelName")]
            model_name: String,
        }
        let params = ModelFieldParams {
            model_name: model.to_string(),
        };
        let req = AnkiRequest {
            action: "modelFieldNames".to_string(),
            version: 6,
            params: Some(params),
        };
        let resp: AnkiResponse<Vec<String>> = self.post(req).await?;
        resp.result.ok_or_else(|| resp.error.unwrap_or("Erro AnkiConnect.".to_string()))
    }

    pub async fn add_note(
        &self,
        deck: &str,
        model: &str,
        fields: serde_json::Map<String, Value>,
        tags: &[String],
    ) -> Result<i64, String> {
        let note = Note {
            deck_name: deck.to_string(),
            model_name: model.to_string(),
            fields,
            tags: tags.to_vec(),
        };
        let params = AddNoteParams { note };
        let req = AnkiRequest {
            action: "addNote".to_string(),
            version: 6,
            params: Some(params),
        };
        let resp: AnkiResponse<i64> = self.post(req).await?;
        resp.result.ok_or_else(|| resp.error.unwrap_or("Erro AnkiConnect.".to_string()))
    }

    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        req: AnkiRequest<T>,
    ) -> Result<R, String> {
        self.client
            .post(format!("{}/", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<R>()
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Serialize)]
struct AnkiRequest<T: Serialize> {
    action: String,
    version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

#[derive(Deserialize)]
struct AnkiResponse<T> {
    result: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct AddNoteParams {
    note: Note,
}

#[derive(Serialize)]
struct Note {
    #[serde(rename = "deckName")]
    deck_name: String,
    #[serde(rename = "modelName")]
    model_name: String,
    fields: serde_json::Map<String, Value>,
    tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ─── build_anki_url ───────────────────────────────────────────────

    #[test]
    fn builds_url_from_host_and_port() {
        assert_eq!(build_anki_url("localhost", 8765), "http://localhost:8765");
        assert_eq!(build_anki_url("192.168.1.1", 8765), "http://192.168.1.1:8765");
    }

    #[test]
    fn builds_url_with_different_host() {
        assert_eq!(build_anki_url("anki-server.local", 8765), "http://anki-server.local:8765");
    }

    #[test]
    fn builds_url_with_custom_port() {
        assert_eq!(build_anki_url("localhost", 12345), "http://localhost:12345");
    }

    // ─── AnkiRequest serialization ────────────────────────────────────

    #[test]
    fn serializes_request_without_params() {
        let req = AnkiRequest::<serde_json::Value> {
            action: "version".to_string(),
            version: 6,
            params: None,
        };
        let json = serde_json::to_value(&req).expect("serialize");
        assert_eq!(json["action"], "version");
        assert_eq!(json["version"], 6);
        assert!(json.get("params").is_none());
    }

    #[test]
    fn serializes_request_with_params() {
        let params = json!({"modelName": "Basic"});
        let req = AnkiRequest {
            action: "modelFieldNames".to_string(),
            version: 6,
            params: Some(params),
        };
        let json = serde_json::to_value(&req).expect("serialize");
        assert_eq!(json["action"], "modelFieldNames");
        assert_eq!(json["version"], 6);
        assert_eq!(json["params"]["modelName"], "Basic");
    }

    // ─── AnkiResponse deserialization ─────────────────────────────────

    #[test]
    fn deserializes_successful_response() {
        let json = r#"{"result": 6, "error": null}"#;
        let resp: AnkiResponse<u16> = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.result, Some(6));
        assert!(resp.error.is_none());
    }

    #[test]
    fn deserializes_error_response() {
        let json = r#"{"result": null, "error": "deck not found"}"#;
        let resp: AnkiResponse<serde_json::Value> = serde_json::from_str(json).expect("deserialize");
        assert!(resp.result.is_none());
        assert_eq!(resp.error.unwrap(), "deck not found");
    }

    #[test]
    fn deserializes_array_result() {
        let json = r#"{"result": ["Default", "English"], "error": null}"#;
        let resp: AnkiResponse<Vec<String>> = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.result.unwrap(), vec!["Default".to_string(), "English".to_string()]);
    }

    #[test]
    fn deserializes_result_with_missing_error_field() {
        let json = r#"{"result": 42}"#;
        let resp: AnkiResponse<u16> = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.result, Some(42));
        assert!(resp.error.is_none());
    }

    // ─── Note serialization ───────────────────────────────────────────

    #[test]
    fn serializes_note_with_correct_field_names() {
        let mut fields = serde_json::Map::new();
        fields.insert("Front".to_string(), json!("Hello"));
        fields.insert("Back".to_string(), json!("World"));

        let note = Note {
            deck_name: "Default".to_string(),
            model_name: "Basic".to_string(),
            fields,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let json = serde_json::to_value(&note).expect("serialize");
        assert_eq!(json["deckName"], "Default");
        assert_eq!(json["modelName"], "Basic");
        assert_eq!(json["fields"]["Front"], "Hello");
        assert_eq!(json["fields"]["Back"], "World");
        assert_eq!(json["tags"], json!(["tag1", "tag2"]));
    }

    #[test]
    fn serializes_note_with_empty_tags() {
        let fields = serde_json::Map::new();
        let note = Note {
            deck_name: "MyDeck".to_string(),
            model_name: "Basic".to_string(),
            fields,
            tags: vec![],
        };

        let json = serde_json::to_value(&note).expect("serialize");
        assert_eq!(json["tags"], json!([]));
        assert_eq!(json["deckName"], "MyDeck");
    }

    // ─── AddNoteParams serialization ──────────────────────────────────

    #[test]
    fn serializes_add_note_params_with_note() {
        let mut fields = serde_json::Map::new();
        fields.insert("Front".to_string(), json!("front text"));

        let note = Note {
            deck_name: "Default".to_string(),
            model_name: "Basic".to_string(),
            fields,
            tags: vec!["sentenceminer".to_string()],
        };

        let params = AddNoteParams { note };
        let json = serde_json::to_value(&params).expect("serialize");

        assert_eq!(json["note"]["deckName"], "Default");
        assert_eq!(json["note"]["modelName"], "Basic");
        assert_eq!(json["note"]["fields"]["Front"], "front text");
        assert_eq!(json["note"]["tags"], json!(["sentenceminer"]));
    }

    // ─── Full request + params integration ────────────────────────────

    #[test]
    fn serializes_add_note_request() {
        let mut fields = serde_json::Map::new();
        fields.insert("Front".to_string(), json!("front"));
        fields.insert("Back".to_string(), json!("back"));

        let note = Note {
            deck_name: "JLPT".to_string(),
            model_name: "Basic".to_string(),
            fields,
            tags: vec!["sentenceminer".to_string()],
        };

        let req: AnkiRequest<AddNoteParams> = AnkiRequest {
            action: "addNote".to_string(),
            version: 6,
            params: Some(AddNoteParams { note }),
        };

        let json = serde_json::to_value(&req).expect("serialize");
        assert_eq!(json["action"], "addNote");
        assert_eq!(json["params"]["note"]["deckName"], "JLPT");
        assert_eq!(json["params"]["note"]["fields"]["Front"], "front");
    }
}
