const INICIANTE_TEMPLATE: &str = include_str!("prompts/iniciante.txt");
const INTERMEDIARIO_TEMPLATE: &str = include_str!("prompts/intermediario.txt");
const AVANCADO_TEMPLATE: &str = include_str!("prompts/avancado.txt");

pub fn build_prompt(
    card_model: &str,
    source_language: &str,
    target_language: &str,
    sentence: &str,
    term: &str,
) -> Result<String, String> {
    let template = match card_model {
        "iniciante" => INICIANTE_TEMPLATE,
        "intermediario" => INTERMEDIARIO_TEMPLATE,
        "avancado" => AVANCADO_TEMPLATE,
        _ => return Err("Modelo invalido.".to_string()),
    };

    Ok(render_template(
        template,
        source_language,
        target_language,
        sentence,
        term,
    ))
}

fn render_template(
    template: &str,
    source_language: &str,
    target_language: &str,
    sentence: &str,
    term: &str,
) -> String {
    template
        .replace("{{source_language}}", source_language)
        .replace("{{target_language}}", target_language)
        .replace("{{sentence}}", sentence)
        .replace("{{term}}", term)
}

#[cfg(test)]
mod tests {
    use super::{build_prompt, render_template};

    // ─── render_template ─────────────────────────────────────────────

    #[test]
    fn renders_all_placeholders_in_template() {
        let template = "{{source_language}} {{target_language}} {{sentence}} {{term}}";
        let result = render_template(template, "English", "Portuguese", "Hello world", "world");

        assert_eq!(result, "English Portuguese Hello world world");
    }

    #[test]
    fn renders_with_empty_values() {
        let template = "[{{source_language}}][{{target_language}}][{{sentence}}][{{term}}]";
        let result = render_template(template, "", "", "", "");

        assert_eq!(result, "[][][][]");
    }

    #[test]
    fn renders_with_special_characters() {
        let template = "{{sentence}} - {{term}}";
        let result = render_template(
            template,
            "English",
            "Portuguese",
            "It's \"simple\", isn't it?",
            "isn't",
        );

        assert!(result.contains("\"simple\""));
        assert!(result.contains("isn't"));
    }

    #[test]
    fn does_not_replace_unknown_placeholders() {
        let template = "{{source_language}}{{unknown}}{{term}}";
        let result = render_template(template, "EN", "PT", "sentence", "word");

        assert_eq!(result, "EN{{unknown}}word");
    }

    #[test]
    fn renders_repeated_placeholders() {
        let template = "{{term}},{{term}}";
        let result = render_template(template, "EN", "PT", "sentence", "word");

        assert_eq!(result, "word,word");
    }

    // ─── build_prompt ─────────────────────────────────────────────────

    #[test]
    fn renders_beginner_prompt_from_template() {
        let prompt = build_prompt(
            "iniciante",
            "English",
            "Brazilian Portuguese",
            "She looked at him with an inscrutable expression.",
            "inscrutable",
        )
        .expect("prompt should render");

        assert!(prompt.contains("Source language: English"));
        assert!(prompt.contains("Target language: Brazilian Portuguese"));
        assert!(prompt.contains("Unknown term: \"inscrutable\""));
        assert!(prompt.contains("TRADUÇÃO"));
        assert!(prompt.contains("EQUIVALENTE"));
        assert!(prompt.contains("Card model: beginner"));
    }

    #[test]
    fn renders_intermediate_prompt_with_correct_sections() {
        let prompt = build_prompt(
            "intermediario",
            "English",
            "Brazilian Portuguese",
            "The view was breathtaking.",
            "breathtaking",
        )
        .expect("prompt should render");

        assert!(prompt.contains("Source language: English"));
        assert!(prompt.contains("Target language: Brazilian Portuguese"));
        assert!(prompt.contains("Unknown term: \"breathtaking\""));
        assert!(prompt.contains("DEFINIÇÃO"));
        assert!(prompt.contains("SUPPORT"));
        assert!(prompt.contains("Card model: intermediate"));
    }

    #[test]
    fn renders_advanced_prompt_with_consistent_sections() {
        let prompt = build_prompt(
            "avancado",
            "English",
            "Brazilian Portuguese",
            "The result was negligible.",
            "negligible",
        )
        .expect("prompt should render");

        assert!(prompt.contains("Card model: advanced"));
        assert!(prompt.contains("DEFINITION"));
        assert!(prompt.contains("SUPPORT"));
        assert!(prompt.contains("Source language: English"));
    }

    #[test]
    fn renders_prompt_with_empty_sentence() {
        let prompt = build_prompt("iniciante", "English", "Portuguese", "", "word")
            .expect("prompt should render");

        assert!(prompt.contains("Sentence: \"\""));
        assert!(prompt.contains("Unknown term: \"word\""));
    }

    #[test]
    fn renders_prompt_with_empty_term() {
        let prompt = build_prompt("iniciante", "English", "Portuguese", "A sentence.", "")
            .expect("prompt should render");

        assert!(prompt.contains("Sentence: \"A sentence.\""));
        assert!(prompt.contains("Unknown term: \"\""));
    }

    #[test]
    fn rejects_invalid_model() {
        let err = build_prompt("foo", "English", "Portuguese", "test", "word")
            .expect_err("invalid model should fail");

        assert_eq!(err, "Modelo invalido.");
    }
}
