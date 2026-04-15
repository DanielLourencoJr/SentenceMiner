use serde::{Deserialize, Serialize};

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
    let prompt = build_prompt(
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

fn build_prompt(
    card_model: &str,
    source_language: &str,
    target_language: &str,
    sentence: &str,
    term: &str,
) -> Result<String, String> {
    match card_model {
        "iniciante" => Ok(format!(
            "You are a language flashcard assistant.\n\nSource language: {source_language}\nTarget language: {target_language}\nSentence: \"{sentence}\"\nUnknown term: \"{term}\"\n\nGenerate the back of a flashcard. Respond in {target_language}. Include:\n1. A natural translation of the full sentence.\n2. On a new line, only the {target_language} equivalent of the unknown term — nothing else.\n\nRespond only with the flashcard content. No labels, no preamble, no formatting."
        )),
        "intermediario" => Ok(format!(
            "You are a language flashcard assistant.\n\nSource language: {source_language}\nTarget language: {target_language}\nSentence: \"{sentence}\"\nUnknown term: \"{term}\"\n\nGenerate the back of a flashcard. Respond in {target_language}. Include:\n1. A concise definition of the unknown term in {target_language}.\n2. Up to three synonyms in {source_language}.\n\nRespond only with the flashcard content. No labels, no preamble, no formatting."
        )),
        "avancado" => Ok(format!(
            "You are a language flashcard assistant.\n\nSource language: {source_language}\nSentence: \"{sentence}\"\nUnknown term: \"{term}\"\n\nGenerate the back of a flashcard entirely in {source_language}. Include only a concise definition of the unknown term.\n\nRespond only with the flashcard content. No labels, no preamble, no formatting."
        )),
        _ => Err("Modelo invalido.".to_string()),
    }
}
