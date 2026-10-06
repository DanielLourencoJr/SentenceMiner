//! Term inference, layer 1: pick the rarest word of a sentence.
//!
//! Scoring uses an embedded top-10k English frequency list
//! (`google-10000-english`, rank order = common first). No network, no
//! Anki access: unknown words (typos excluded by the caller, if any)
//! count as the rarest. Proper nouns, numbers, acronyms and contractions
//! are skipped; only a light lowercase normalization is applied, so
//! inflected forms may not match their base form (documented limitation,
//! to be improved by the Anki-vocabulary layer).

use std::collections::HashMap;
use std::sync::OnceLock;

// Ranked word list, most common first. Source: google-10000-english
// (https://github.com/first20hours/google-10000-english).
const FREQUENCY_LIST: &str = include_str!("frequency.txt");

// Words never suggested, however rare they rank.
const STOPWORDS: &[&str] = &[
    "a",
    "about",
    "above",
    "after",
    "again",
    "against",
    "all",
    "am",
    "an",
    "and",
    "any",
    "are",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "but",
    "by",
    "can",
    "did",
    "do",
    "does",
    "doing",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "has",
    "have",
    "having",
    "he",
    "her",
    "here",
    "hers",
    "herself",
    "him",
    "himself",
    "his",
    "how",
    "i",
    "if",
    "in",
    "into",
    "is",
    "it",
    "its",
    "itself",
    "just",
    "me",
    "more",
    "most",
    "my",
    "myself",
    "no",
    "nor",
    "not",
    "now",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "ought",
    "our",
    "ours",
    "ourselves",
    "out",
    "over",
    "own",
    "same",
    "she",
    "should",
    "so",
    "some",
    "such",
    "than",
    "that",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "these",
    "they",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "we",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "who",
    "whom",
    "why",
    "will",
    "with",
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
];

fn rank_table() -> &'static HashMap<String, usize> {
    static TABLE: OnceLock<HashMap<String, usize>> = OnceLock::new();
    TABLE.get_or_init(|| {
        FREQUENCY_LIST
            .lines()
            .enumerate()
            .map(|(rank, word)| (word.to_string(), rank))
            .collect()
    })
}

fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

fn all_digits(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_digit())
}

/// Split a sentence into `(lookup key, surface form)` tokens.
/// The key is lowercase alphanumeric; the surface form keeps the
/// original casing for display/highlighting.
fn tokenize(sentence: &str) -> Vec<(String, String)> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for c in sentence.chars() {
        // Apostrophes stay inside the token so contractions ("don't")
        // survive as one unit and can be skipped whole.
        if c.is_alphanumeric() || c == '\'' {
            current.push(c);
        } else if !current.is_empty() {
            tokens.push((current.to_lowercase(), current.clone()));
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push((current.to_lowercase(), current.clone()));
    }
    tokens
}

struct ScoredToken {
    score: usize,
    len: usize,
    position: usize,
    surface: String,
}

fn eligible(key: &str, surface: &str, is_first_token: bool) -> bool {
    if key.len() < 2 || is_stopword(key) || all_digits(key) {
        return false;
    }
    // Contractions ("don't") split oddly and are always common: skip.
    if surface.contains('\'') {
        return false;
    }
    // Acronyms and proper nouns are not vocabulary to mine.
    let first = surface.chars().next().unwrap_or(' ');
    if first.is_uppercase()
        && (surface.chars().skip(1).all(|c| c.is_uppercase()) || !is_first_token)
    {
        return false;
    }
    true
}

/// Suggest the rarest eligible word, in surface form.
/// Returns an empty string when nothing qualifies.
pub fn infer_term(sentence: &str) -> String {
    let table = rank_table();
    let mut best: Option<ScoredToken> = None;

    for (position, (key, surface)) in tokenize(sentence).iter().enumerate() {
        if !eligible(key, surface, position == 0) {
            continue;
        }
        // Unknown words rank past the whole list: most interesting.
        let score = table.get(key).copied().unwrap_or(usize::MAX);
        let candidate = ScoredToken {
            score,
            len: key.len(),
            position,
            surface: surface.clone(),
        };
        // Higher rank = rarer = more interesting. Unknown words rank
        // past the whole list. Ties break toward longer, then earlier.
        let better = match &best {
            None => true,
            Some(current) => {
                (
                    std::cmp::Reverse(candidate.score),
                    std::cmp::Reverse(candidate.len),
                    candidate.position,
                ) < (
                    std::cmp::Reverse(current.score),
                    std::cmp::Reverse(current.len),
                    current.position,
                )
            }
        };
        if better {
            best = Some(candidate);
        }
    }

    best.map(|c| c.surface).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_rarest_word() {
        assert_eq!(
            infer_term("She looked at him with an inscrutable expression."),
            "inscrutable"
        );
    }

    #[test]
    fn ignores_stopwords_and_common_words() {
        assert_eq!(infer_term("The cat sat on the mat."), "mat");
    }

    #[test]
    fn skips_proper_nouns_mid_sentence() {
        // "Paris" is capitalized mid-sentence: skipped, the unknown
        // "breathtaking" (past the whole list) wins.
        assert_eq!(
            infer_term("I visited Paris and it was breathtaking."),
            "breathtaking"
        );
    }

    #[test]
    fn skips_numbers_and_acronyms() {
        assert_eq!(
            infer_term("In 2024 the NASA report was mindblowing."),
            "mindblowing"
        );
    }

    #[test]
    fn skips_contractions() {
        // "don't" must not surface as "don".
        let result = infer_term("I don't understand this word: floccinaucinihilipilification.");
        assert_eq!(result, "floccinaucinihilipilification");
    }

    #[test]
    fn returns_surface_form() {
        // Capitalized sentence-initial word keeps its surface casing.
        assert_eq!(infer_term("Breathtaking views everywhere."), "Breathtaking");
    }

    #[test]
    fn empty_and_common_only_yield_nothing() {
        assert_eq!(infer_term(""), "");
        assert_eq!(infer_term("   "), "");
        assert_eq!(infer_term("I am."), "");
    }

    #[test]
    fn tokenize_keeps_surfaces() {
        let tokens = tokenize("Hello, beautiful World!");
        assert_eq!(
            tokens,
            vec![
                ("hello".to_string(), "Hello".to_string()),
                ("beautiful".to_string(), "beautiful".to_string()),
                ("world".to_string(), "World".to_string()),
            ]
        );
    }
}
