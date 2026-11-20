use unicode_segmentation::UnicodeSegmentation;

/// Extracts the first `sentences_count` sentences from text/content using Unicode segmentation.
pub fn extract_summary(content: &str, sentences_count: usize) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let sentences: Vec<&str> = trimmed
        .unicode_sentences()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .take(sentences_count)
        .collect();

    sentences.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractive_summary_returns_first_two_sentences() {
        let text = "Rust is a systems programming language. It is designed for memory safety and concurrency. It also has great tooling! Many developers love it.";
        let summary = extract_summary(text, 2);
        assert_eq!(
            summary,
            "Rust is a systems programming language. It is designed for memory safety and concurrency."
        );

        let short_text = "Single sentence here.";
        assert_eq!(extract_summary(short_text, 2), "Single sentence here.");
    }
}
