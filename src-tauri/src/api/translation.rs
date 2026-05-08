use serde::{Deserialize, Serialize};

use super::{prompts, response_parser};

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

fn build_chat_request(model: &str, prompt: &str) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        temperature: 0.3,
        max_tokens: 300,
    }
}

fn build_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
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

    let req = build_chat_request(model, &prompt);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_seconds))
        .build()
        .map_err(|e| e.to_string())?;

    let url = build_url(base_url);
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

    let body_text = resp.text().await.map_err(|e| e.to_string())?;
    parse_api_response(&body_text, card_model)
}

fn parse_api_response(body_text: &str, card_model: &str) -> Result<String, String> {
    let body: ChatResponse = serde_json::from_str(body_text).map_err(|e| e.to_string())?;
    let content = body
        .choices
        .get(0)
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Resposta vazia da API.".to_string())?;

    response_parser::parse_and_normalize_back(card_model, &content)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── build_chat_request ───────────────────────────────────────────

    #[test]
    fn builds_correct_request_payload() {
        let prompt = "Test prompt for translation";
        let req = build_chat_request("llama3-70b-8192", prompt);

        assert_eq!(req.model, "llama3-70b-8192");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
        assert_eq!(req.messages[0].content, prompt);
        assert_eq!(req.temperature, 0.3);
        assert_eq!(req.max_tokens, 300);
    }

    #[test]
    fn builds_request_with_different_model() {
        let req = build_chat_request("gpt-4", "Some prompt");

        assert_eq!(req.model, "gpt-4");
        assert_eq!(req.messages[0].content, "Some prompt");
    }

    #[test]
    fn request_has_single_user_message() {
        let req = build_chat_request("model", "prompt");

        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
    }

    // ─── build_url ────────────────────────────────────────────────────

    #[test]
    fn url_without_trailing_slash() {
        let url = build_url("https://api.groq.com/openai/v1");
        assert_eq!(url, "https://api.groq.com/openai/v1/chat/completions");
    }

    #[test]
    fn url_with_trailing_slash() {
        let url = build_url("https://api.groq.com/openai/v1/");
        assert_eq!(url, "https://api.groq.com/openai/v1/chat/completions");
    }

    #[test]
    fn url_with_long_base_url() {
        let url = build_url("https://custom-api.example.com/v1/openai");
        assert_eq!(url, "https://custom-api.example.com/v1/openai/chat/completions");
    }

    // ─── parse_api_response ───────────────────────────────────────────

    fn sample_beginner_response() -> String {
        r#"{"choices":[{"message":{"content":"TRADUÇÃO\nEla olhou.\n\nEQUIVALENTE\nolhou"}}]}"#.to_string()
    }

    #[test]
    fn parses_valid_api_response() {
        let result = parse_api_response(&sample_beginner_response(), "iniciante")
            .expect("should parse");
        assert_eq!(result, "Ela olhou.\nolhou");
    }

    #[test]
    fn parses_response_with_extra_whitespace_in_content() {
        let json = r#"{"choices":[{"message":{"content":"  TRADUÇÃO\n  Olá.\n\nEQUIVALENTE\n  olá  "}}]}"#;
        let result = parse_api_response(json, "iniciante").expect("should parse");
        assert_eq!(result, "Olá.\nolá");
    }

    #[test]
    fn rejects_empty_choices_array() {
        let json = r#"{"choices":[]}"#;
        let err = parse_api_response(json, "iniciante").expect_err("should fail");
        assert!(err.contains("vazia"));
    }

    #[test]
    fn rejects_malformed_json() {
        let json = r#"{"choices":[{"message":{"content":null}}]}"#;
        let err = parse_api_response(json, "iniciante").expect_err("should fail");
        assert!(err.contains("invalid type"));
    }

    #[test]
    fn rejects_response_without_choices_field() {
        let json = r#"{"not_choices":[]}"#;
        let err = parse_api_response(json, "iniciante").expect_err("should fail");
        assert!(err.contains("missing field"));
    }

    #[test]
    fn rejects_advanced_model_with_malformed_response() {
        let json = r#"{invalid json here"#;
        let err = parse_api_response(json, "avancado").expect_err("should fail");
        assert!(err.contains("key must be a string"));
    }
}
