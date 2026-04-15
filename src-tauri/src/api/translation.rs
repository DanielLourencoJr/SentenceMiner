use serde::{Deserialize, Serialize};

use super::prompts;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

pub async fn generate_back(
    base_url: &str,
    api_key: &str,
    model: &str,
    source_language: &str,
    target_language: &str,
    sentence: &str,
    term: &str,
    card_model: &str,
    timeout_seconds: u64,
) -> Result<String, String> {
    let prompt = prompts::build_prompt(
        card_model,
        source_language,
        target_language,
        sentence,
        term,
    )?;

    let req = ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        temperature: 0.3,
        max_tokens: 300,
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_seconds))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Erro HTTP: {}", resp.status()));
    }

    let body: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    let content = body
        .choices
        .get(0)
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Resposta vazia da API.".to_string())?;

    Ok(content)
}
