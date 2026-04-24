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
    use super::build_prompt;

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
    }

    #[test]
    fn rejects_invalid_model() {
        let err = build_prompt("foo", "English", "Portuguese", "test", "word")
            .expect_err("invalid model should fail");

        assert_eq!(err, "Modelo invalido.");
    }
}
