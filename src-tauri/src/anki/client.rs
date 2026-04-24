use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
pub struct AnkiClient {
    base_url: String,
    client: reqwest::Client,
}

impl AnkiClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{host}:{port}"),
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
