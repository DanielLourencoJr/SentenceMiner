pub fn parse_and_normalize_back(card_model: &str, content: &str) -> Result<String, String> {
    let expected_sections = match card_model {
        "iniciante" => &["TRADUÇÃO", "EQUIVALENTE"][..],
        "intermediario" => &["DEFINIÇÃO", "SUPPORT"][..],
        "avancado" => &["DEFINITION", "SUPPORT"][..],
        _ => return Err("Modelo invalido.".to_string()),
    };

    let sections = parse_sections(content, expected_sections)?;
    Ok(format_sections(&sections))
}

fn parse_sections(content: &str, expected_sections: &[&str]) -> Result<Vec<(String, String)>, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0usize;
    let mut parsed = Vec::with_capacity(expected_sections.len());

    for (section_index, heading) in expected_sections.iter().enumerate() {
        skip_blank_lines(&lines, &mut idx);

        let line = lines
            .get(idx)
            .ok_or_else(|| format!("Resposta fora do formato esperado: faltando seção {heading}."))?;
        if line.trim() != *heading {
            return Err(format!(
                "Resposta fora do formato esperado: seção {heading} ausente ou fora de ordem."
            ));
        }
        idx += 1;

        let next_heading = expected_sections.get(section_index + 1).copied();
        let section_content = collect_section_content(&lines, &mut idx, next_heading)?;
        parsed.push(((*heading).to_string(), section_content));
    }

    skip_blank_lines(&lines, &mut idx);
    if idx < lines.len() {
        return Err("Resposta fora do formato esperado: conteúdo extra inválido.".to_string());
    }

    Ok(parsed)
}

fn collect_section_content(
    lines: &[&str],
    idx: &mut usize,
    next_heading: Option<&str>,
) -> Result<String, String> {
    let mut collected = Vec::new();

    while *idx < lines.len() {
        let line = lines[*idx];
        if let Some(heading) = next_heading {
            if line.trim() == heading {
                break;
            }
        }
        collected.push(line);
        *idx += 1;
    }

    let content = collected.join("\n").trim().to_string();
    if content.is_empty() {
        return Err("Resposta fora do formato esperado: seção vazia.".to_string());
    }

    Ok(content)
}

fn format_sections(sections: &[(String, String)]) -> String {
    sections
        .iter()
        .map(|(_, content)| content.trim().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn skip_blank_lines(lines: &[&str], idx: &mut usize) {
    while *idx < lines.len() && lines[*idx].trim().is_empty() {
        *idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::parse_and_normalize_back;

    #[test]
    fn parses_beginner_response() {
        let parsed = parse_and_normalize_back(
            "iniciante",
            "TRADUÇÃO\nEla olhou para ele com uma expressão inescrutável.\n\nEQUIVALENTE\ninescrutável",
        )
        .expect("response should parse");

        assert_eq!(
            parsed,
            "Ela olhou para ele com uma expressão inescrutável.\ninescrutável"
        );
    }

    #[test]
    fn normalizes_extra_blank_lines() {
        let parsed = parse_and_normalize_back(
            "intermediario",
            "\n\nDEFINIÇÃO\nDifícil de entender.\n\n\nSUPPORT\nenigmatic, unreadable\n\n",
        )
        .expect("response should parse");

        assert_eq!(
            parsed,
            "Difícil de entender.\nenigmatic, unreadable"
        );
    }

    #[test]
    fn rejects_missing_section() {
        let err = parse_and_normalize_back("avancado", "DEFINITION\nHard to understand.")
            .expect_err("missing section should fail");

        assert!(err.contains("SUPPORT"));
    }

    #[test]
    fn rejects_wrong_order() {
        let err = parse_and_normalize_back(
            "iniciante",
            "EQUIVALENTE\ninescrutável\n\nTRADUÇÃO\nEla olhou para ele.",
        )
        .expect_err("wrong order should fail");

        assert!(err.contains("TRADUÇÃO"));
    }

    #[test]
    fn rejects_empty_section() {
        let err = parse_and_normalize_back("avancado", "DEFINITION\n\nSUPPORT\nnone")
            .expect_err("empty section should fail");

        assert!(err.contains("seção vazia"));
    }
}
