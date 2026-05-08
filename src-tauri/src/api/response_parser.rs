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
    use super::{
        collect_section_content, format_sections, parse_and_normalize_back, parse_sections,
        skip_blank_lines,
    };

    // ─── skip_blank_lines ─────────────────────────────────────────────

    #[test]
    fn skip_blank_skips_empty_lines() {
        let lines = &["", "  ", "hello"];
        let mut idx = 0;
        skip_blank_lines(lines, &mut idx);
        assert_eq!(idx, 2);
    }

    #[test]
    fn skip_blank_stops_at_non_blank() {
        let lines = &["", "hello", ""];
        let mut idx = 0;
        skip_blank_lines(lines, &mut idx);
        assert_eq!(idx, 1);
    }

    #[test]
    fn skip_blank_stops_at_eof() {
        let lines: &[&str] = &[];
        let mut idx = 0;
        skip_blank_lines(lines, &mut idx);
        assert_eq!(idx, 0);
    }

    #[test]
    fn skip_blank_does_not_advance_on_non_blank() {
        let lines = &["hello"];
        let mut idx = 0;
        skip_blank_lines(lines, &mut idx);
        assert_eq!(idx, 0);
    }

    // ─── collect_section_content ──────────────────────────────────────

    #[test]
    fn collect_content_until_next_heading() {
        let lines = &["first line", "second line", "NEXT", "trailing"];
        let mut idx = 0;
        let content = collect_section_content(lines, &mut idx, Some("NEXT"))
            .expect("should collect");
        assert_eq!(content, "first line\nsecond line");
        assert_eq!(idx, 2);
    }

    #[test]
    fn collect_content_until_eof_when_no_next() {
        let lines = &["line a", "line b"];
        let mut idx = 0;
        let content = collect_section_content(lines, &mut idx, None)
            .expect("should collect");
        assert_eq!(content, "line a\nline b");
        assert_eq!(idx, 2);
    }

    #[test]
    fn collect_content_trims_outer_whitespace() {
        let lines = &["  hello  ", "  world  "];
        let mut idx = 0;
        let content = collect_section_content(lines, &mut idx, None)
            .expect("should collect");
        assert_eq!(content, "hello  \n  world");
    }

    #[test]
    fn collect_rejects_empty_content() {
        let lines = &["HEADER"];
        let mut idx = 1;
        let err = collect_section_content(lines, &mut idx, None)
            .expect_err("should reject empty");
        assert!(err.contains("seção vazia"));
    }

    #[test]
    fn collect_content_stops_at_exact_heading_match() {
        let lines = &["some text", "STOP", "more text"];
        let mut idx = 0;
        let content = collect_section_content(lines, &mut idx, Some("STOP"))
            .expect("should collect");
        assert_eq!(content, "some text");
        assert_eq!(idx, 1);
    }

    #[test]
    fn collect_content_ignores_heading_in_middle_of_line() {
        let lines = &["this line has STOP in it", "not actually a heading"];
        let mut idx = 0;
        let content = collect_section_content(lines, &mut idx, Some("STOP"))
            .expect("should collect");
        assert_eq!(content, "this line has STOP in it\nnot actually a heading");
    }

    // ─── format_sections ──────────────────────────────────────────────

    #[test]
    fn format_joins_section_contents_with_newlines() {
        let sections = vec![
            ("A".to_string(), "first".to_string()),
            ("B".to_string(), "second".to_string()),
        ];
        assert_eq!(format_sections(&sections), "first\nsecond");
    }

    #[test]
    fn format_trims_section_contents() {
        let sections = vec![
            ("A".to_string(), "  hello  ".to_string()),
        ];
        assert_eq!(format_sections(&sections), "hello");
    }

    #[test]
    fn format_returns_empty_for_empty_vec() {
        let sections: Vec<(String, String)> = vec![];
        assert_eq!(format_sections(&sections), "");
    }

    #[test]
    fn format_with_single_section() {
        let sections = vec![
            ("X".to_string(), "only one".to_string()),
        ];
        assert_eq!(format_sections(&sections), "only one");
    }

    // ─── parse_sections ───────────────────────────────────────────────

    #[test]
    fn parse_valid_sections() {
        let content = "A\nhello\n\nB\nworld";
        let result = parse_sections(content, &["A", "B"]).expect("should parse");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("A".to_string(), "hello".to_string()));
        assert_eq!(result[1], ("B".to_string(), "world".to_string()));
    }

    #[test]
    fn parse_rejects_missing_heading() {
        let content = "A\nhello";
        let err = parse_sections(content, &["A", "B"]).expect_err("should fail");
        assert!(err.contains("B"));
    }

    #[test]
    fn parse_rejects_content_before_first_heading() {
        let content = "junk\nA\nhello\nB\nworld";
        let err = parse_sections(content, &["A", "B"]).expect_err("should fail");
        assert!(err.contains("A"));
    }

    #[test]
    fn parse_rejects_wrong_heading_order() {
        let content = "B\nworld\nA\nhello";
        let err = parse_sections(content, &["A", "B"]).expect_err("should fail");
        assert!(err.contains("A"));
    }

    // ─── parse_and_normalize_back ──────────────────────────────────

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

    #[test]
    fn parses_content_with_trailing_blank_lines() {
        let result = parse_and_normalize_back(
            "iniciante",
            "TRADUÇÃO\nEla olhou.\n\nEQUIVALENTE\nolhou\n\n\n",
        )
        .expect("should parse trailing blanks");

        assert_eq!(result, "Ela olhou.\nolhou");
    }

    #[test]
    fn rewrites_section_content_with_newlines() {
        let result = parse_and_normalize_back(
            "avancado",
            "DEFINITION\nHard to\nunderstand.\n\nSUPPORT\nenigmatic, opaque",
        )
        .expect("should parse multiline");

        assert_eq!(result, "Hard to\nunderstand.\nenigmatic, opaque");
    }

    #[test]
    fn rejects_invalid_card_model() {
        let err = parse_and_normalize_back("invalid", "anything")
            .expect_err("should fail");

        assert_eq!(err, "Modelo invalido.");
    }
}
