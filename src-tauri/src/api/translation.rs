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

#[derive(Deserialize)]
struct ProviderErrorEnvelope {
    error: ProviderErrorBody,
}

#[derive(Deserialize)]
struct ProviderErrorBody {
    #[serde(default)]
    message: String,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    code: Option<String>,
}

fn provider_error_message(status: reqwest::StatusCode, body: &str) -> String {
    let parsed: Result<ProviderErrorEnvelope, _> = serde_json::from_str(body);
    let (api_type, api_code, api_msg) = match parsed {
        Ok(env) => (
            env.error.r#type,
            env.error.code.unwrap_or_default(),
            env.error.message,
        ),
        Err(_) => (String::new(), String::new(), String::new()),
    };
    let combined = format!("{api_type} {api_code} {api_msg}").to_lowercase();
    let looks_like_model_error = api_code == "model_decommissioned"
        || api_code == "model_not_found"
        || combined.contains("decommission")
        || combined.contains("model_not_found")
        || combined.contains("does not exist");

    if looks_like_model_error {
        let detail = if api_msg.is_empty() {
            "no provider details".to_string()
        } else {
            api_msg
        };
        return format!(
            "The configured model was retired by the provider ({detail}). \
             Update the `model` field in ~/.config/sentenceminer/config.toml \
             (e.g. \"openai/gpt-oss-120b\") and restart the app."
        );
    }

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return "API key rejected (401). Check the `api_key` field in \
                ~/.config/sentenceminer/config.toml."
            .to_string();
    }

    if api_msg.is_empty() {
        format!("HTTP error: {status}")
    } else {
        format!("HTTP error: {status} — {api_msg}")
    }
}

// Nine args mirror 1:1 the config fields + Tauri command inputs;
// grouping them in a struct would just move verbosity to the call site.
#[allow(clippy::too_many_arguments)]
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
    let prompt =
        prompts::build_prompt(card_model, source_language, target_language, sentence, term)?;

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
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(provider_error_message(status, &body));
    }

    let body_text = resp.text().await.map_err(|e| e.to_string())?;
    parse_api_response(&body_text, card_model)
}

fn parse_api_response(body_text: &str, card_model: &str) -> Result<String, String> {
    let body: ChatResponse = serde_json::from_str(body_text).map_err(|e| e.to_string())?;
    let content = body
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Empty API response.".to_string())?;

    response_parser::parse_and_normalize_back(card_model, &content)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── build_chat_request ───────────────────────────────────────────

    #[test]
    fn builds_correct_request_payload() {
        let prompt = "Test prompt for translation";
        let req = build_chat_request("openai/gpt-oss-120b", prompt);

        assert_eq!(req.model, "openai/gpt-oss-120b");
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
        assert_eq!(
            url,
            "https://custom-api.example.com/v1/openai/chat/completions"
        );
    }

    // ─── parse_api_response ───────────────────────────────────────────

    fn sample_beginner_response() -> String {
        r#"{"choices":[{"message":{"content":"TRADUÇÃO\nShe looked.\n\nEQUIVALENTE\nlooked"}}]}"#
            .to_string()
    }

    #[test]
    fn parses_valid_api_response() {
        let result =
            parse_api_response(&sample_beginner_response(), "beginner").expect("should parse");
        assert_eq!(result, "She looked.\nlooked");
    }

    #[test]
    fn parses_response_with_extra_whitespace_in_content() {
        let json = r#"{"choices":[{"message":{"content":"  TRADUÇÃO\n  Hello.\n\nEQUIVALENTE\n  hello  "}}]}"#;
        let result = parse_api_response(json, "beginner").expect("should parse");
        assert_eq!(result, "Hello.\nhello");
    }

    #[test]
    fn rejects_empty_choices_array() {
        let json = r#"{"choices":[]}"#;
        let err = parse_api_response(json, "beginner").expect_err("should fail");
        assert!(err.contains("Empty API response"));
    }

    #[test]
    fn rejects_malformed_json() {
        let json = r#"{"choices":[{"message":{"content":null}}]}"#;
        let err = parse_api_response(json, "beginner").expect_err("should fail");
        assert!(err.contains("invalid type"));
    }

    #[test]
    fn rejects_response_without_choices_field() {
        let json = r#"{"not_choices":[]}"#;
        let err = parse_api_response(json, "beginner").expect_err("should fail");
        assert!(err.contains("missing field"));
    }

    // ─── provider_error_message ───────────────────────────────────────

    #[test]
    fn decommissioned_model_gives_actionable_message() {
        let body = r#"{"error":{"message":"The model llama3-70b-8192 has been decommissioned.","type":"invalid_request_error","code":"model_decommissioned"}}"#;
        let msg = provider_error_message(reqwest::StatusCode::BAD_REQUEST, body);
        assert!(msg.contains("retired"), "got: {msg}");
        assert!(msg.contains("config.toml"), "got: {msg}");
    }

    #[test]
    fn model_not_found_gives_actionable_message() {
        let body = r#"{"error":{"message":"The model `foo` does not exist.","type":"not_found","code":"model_not_found"}}"#;
        let msg = provider_error_message(reqwest::StatusCode::NOT_FOUND, body);
        assert!(msg.contains("retired"), "got: {msg}");
    }

    #[test]
    fn unauthorized_points_to_api_key() {
        let msg = provider_error_message(reqwest::StatusCode::UNAUTHORIZED, "");
        assert!(msg.contains("api_key"), "got: {msg}");
    }

    #[test]
    fn other_provider_error_includes_message() {
        let body = r#"{"error":{"message":"Rate limit reached.","type":"rate_limit","code":"rate_limit_exceeded"}}"#;
        let msg = provider_error_message(reqwest::StatusCode::TOO_MANY_REQUESTS, body);
        assert!(msg.contains("429"), "got: {msg}");
        assert!(msg.contains("Rate limit"), "got: {msg}");
    }

    #[test]
    fn non_json_error_falls_back_to_status() {
        let msg = provider_error_message(reqwest::StatusCode::BAD_GATEWAY, "<html>oops</html>");
        assert_eq!(msg, "HTTP error: 502 Bad Gateway");
    }
}
