pub fn normalize_lemma(value: &str) -> String {
    normalize_context(value).to_lowercase()
}

pub fn normalize_context(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn dedupe_key(lemma: &str, source_language: &str) -> String {
    format!(
        "{}|{}",
        normalize_lemma(lemma),
        source_language.trim().to_lowercase()
    )
}
