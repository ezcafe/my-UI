use unicode_segmentation::UnicodeSegmentation;

/// Strips HTML tags and common HTML entities, returning normalized whitespace text.
pub fn strip_html_tags(content: &str) -> String {
    let mut in_tag = false;
    let mut output = String::with_capacity(content.len());
    for c in content.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(c),
            _ => {}
        }
    }
    let unescaped = output
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    let normalized = unescaped
        .replace(" .", ".")
        .replace(" ,", ",")
        .replace(" !", "!")
        .replace(" ?", "?")
        .replace(" :", ":")
        .replace(" ;", ";");

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts the first `sentences_count` sentences from text/content using Unicode segmentation.
pub fn extract_summary(content: &str, sentences_count: usize) -> String {
    let clean = strip_html_tags(content);
    let trimmed = clean.trim();
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

    #[test]
    fn test_extractive_summary_returns_three_sentences() {
        let text = "First sentence. Second sentence. Third sentence. Fourth sentence.";
        let summary = extract_summary(text, 3);
        assert_eq!(
            summary,
            "First sentence. Second sentence. Third sentence."
        );
    }

    #[test]
    fn test_summary_extractor_strips_html_tags() {
        let html_text = "<p>First sentence with <b>bold</b> and <a href=\"https://example.com\">link</a>. Second sentence contains &amp; entity.</p><p>Third sentence.</p>";
        let summary = extract_summary(html_text, 2);
        assert_eq!(
            summary,
            "First sentence with bold and link. Second sentence contains & entity."
        );
    }
}
