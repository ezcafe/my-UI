use ammonia::Builder;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

#[cfg(test)]
static RE_ANCHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a(\s+[^>]*)?>"#).expect("Valid anchor regex"));
#[cfg(test)]
static RE_TARGET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"target="[^"]*""#).expect("Valid target regex"));
static RE_BLOCK_BOUNDARY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)<\s*(?:br\s*/?|p(?:\s+[^>]*)?|/\s*(?:p|div|li|blockquote|tr|td|th|h[1-6]|section|article))\s*>"#,
    )
    .expect("Valid block boundary regex")
});

#[cfg(test)]
pub fn sanitize_html(input: &str) -> String {
    let mut builder = Builder::default();
    builder.link_rel(Some("noopener noreferrer"));

    let mut tags = HashSet::new();
    tags.insert("p");
    tags.insert("b");
    tags.insert("i");
    tags.insert("strong");
    tags.insert("em");
    tags.insert("a");
    tags.insert("ul");
    tags.insert("ol");
    tags.insert("li");
    tags.insert("blockquote");
    tags.insert("pre");
    tags.insert("code");
    tags.insert("img");
    tags.insert("h1");
    tags.insert("h2");
    tags.insert("h3");
    tags.insert("h4");
    tags.insert("div");
    tags.insert("span");
    builder.tags(tags);

    builder.add_tag_attributes("a", &["target", "href"]);
    builder.add_tag_attributes("img", &["src", "alt", "title", "loading"]);

    let cleaned = builder.clean(input).to_string();

    // Ensure all <a> tags have target="_blank"
    RE_ANCHOR
        .replace_all(&cleaned, |caps: &regex::Captures| {
            let tag = caps.get(0).unwrap().as_str();
            if tag.contains("target=") {
                RE_TARGET.replace(tag, r#"target="_blank""#).to_string()
            } else {
                tag.replacen("<a", "<a target=\"_blank\"", 1)
            }
        })
        .to_string()
}

fn strip_html_to_plain(raw_html: &str) -> String {
    if raw_html.trim().is_empty() {
        return String::new();
    }
    let spaced = RE_BLOCK_BOUNDARY.replace_all(raw_html, " ");
    let stripped = Builder::default()
        .tags(HashSet::new())
        .clean(&spaced)
        .to_string();
    let decoded = stripped
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&#X27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_at_char_boundary(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let cutoff = text
        .char_indices()
        .nth(max_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    let slice = &text[..cutoff];
    let trimmed = match slice.rfind(char::is_whitespace) {
        Some(ws_idx) if ws_idx > 0 => slice[..ws_idx].trim_end(),
        _ => slice.trim_end(),
    };
    format!("{trimmed}...")
}

fn extract_opening_excerpt(text: &str, max_sentences: usize, max_chars: usize) -> String {
    let mut sentence_end_byte = None;
    let mut sentence_count = 0;
    let mut iter = text.char_indices().peekable();

    while let Some((idx, ch)) = iter.next() {
        if matches!(ch, '.' | '!' | '?') {
            let next_is_boundary = match iter.peek() {
                None => true,
                Some((_, next_ch)) => next_ch.is_whitespace(),
            };
            if next_is_boundary {
                sentence_count += 1;
                if sentence_count >= max_sentences {
                    sentence_end_byte = Some(idx + ch.len_utf8());
                    break;
                }
            }
        }
    }

    let excerpt = match sentence_end_byte {
        Some(end) if text[..end].chars().count() <= max_chars => return text[..end].to_string(),
        Some(end) => &text[..end],
        None => text,
    };
    truncate_at_char_boundary(excerpt, max_chars)
}

/// Converts HTML summary/content into clean plain text for cards and fallbacks.
/// Prefers `summary_html` (capped at 400 chars); falls back to a brief 2-3 sentence
/// excerpt (capped at 300 chars) from `content_html`.
pub fn extract_plain_summary(summary_html: &str, content_html: &str) -> String {
    let clean_summary = strip_html_to_plain(summary_html);
    if !clean_summary.is_empty() {
        return truncate_at_char_boundary(&clean_summary, 400);
    }
    let clean_content = strip_html_to_plain(content_html);
    if clean_content.is_empty() {
        return String::new();
    }
    extract_opening_excerpt(&clean_content, 3, 300)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizer_removes_dangerous_tags() {
        let dirty = r#"<div><p>Safe content</p><script>alert('xss');</script><iframe src="malicious.com"></iframe><img src="x" onerror="alert(1)"/></div>"#;
        let clean = sanitize_html(dirty);
        assert!(!clean.contains("<script>"), "Must strip <script>");
        assert!(!clean.contains("<iframe"), "Must strip <iframe>");
        assert!(!clean.contains("onerror"), "Must strip onerror attribute");
        assert!(
            clean.contains("<p>Safe content</p>"),
            "Must preserve safe tags"
        );
    }

    #[test]
    fn test_sanitizer_adds_noopener_to_links() {
        let dirty = r#"<p>Check out <a href="https://example.com">Example</a></p>"#;
        let clean = sanitize_html(dirty);
        assert!(clean.contains("href=\"https://example.com\""));
        assert!(clean.contains("rel=\"noopener noreferrer\""));
        assert!(clean.contains("target=\"_blank\""));
    }

    #[test]
    fn test_sanitizer_multiple_links_all_receive_target_blank() {
        let dirty = r#"<p><a target="_blank" href="https://a.com">A</a> and <a href="https://b.com">B</a> and <a target="_self" href="https://c.com">C</a></p>"#;
        let clean = sanitize_html(dirty);
        assert!(clean.contains(
            r#"<a target="_blank" href="https://a.com" rel="noopener noreferrer">A</a>"#
        ));
        assert!(clean.contains(
            r#"<a target="_blank" href="https://b.com" rel="noopener noreferrer">B</a>"#
        ));
        assert!(clean.contains(
            r#"<a target="_blank" href="https://c.com" rel="noopener noreferrer">C</a>"#
        ));
    }

    #[test]
    fn test_sanitizer_preserves_links_and_attributes_with_lazylock() {
        let input = r#"<p><a href="https://example.com" class="link">Visit</a></p>"#;
        let output = sanitize_html(input);
        assert!(output.contains(r#"href="https://example.com""#));
        assert!(output.contains(r#"target="_blank""#));
        assert!(output.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn test_extract_plain_summary_strips_html_and_truncates_content() {
        // 1. Block-tag spacing + script/style removal + entity decoding (&amp; last)
        let summary_html = r#"<p>First sentence.</p><p>Second &amp;lt;tag&amp;gt; &amp; &quot;quoted&quot; &apos;ok&apos;&#x27;s.</p><script>alert('xss')</script><style>.x{color:red}</style>"#;
        let plain = extract_plain_summary(summary_html, "<p>Ignored full content</p>");
        assert_eq!(
            plain,
            "First sentence. Second &lt;tag&gt; & \"quoted\" 'ok''s."
        );

        // 2. Summary > 400 chars truncates on UTF-8 char boundary with "..."
        let long_summary = format!("<p>{}</p>", "ngụy biện ".repeat(60));
        let truncated_summary = extract_plain_summary(&long_summary, "");
        assert!(truncated_summary.ends_with("..."));
        assert!(truncated_summary.chars().count() <= 403);
        assert!(!truncated_summary.contains('<'));

        // 3. Empty summary falls back to brief 2-3 sentence / 300-char excerpt of content_html (UTF-8 safe)
        let long_content = concat!(
            "<p>Câu một với từ ngụy biện và dấu — gạch ngang.</p>",
            "<p>Câu hai phân tích tác động kinh tế.</p>",
            "<p>Câu ba bổ sung bối cảnh thị trường.</p>",
            "<p>Câu bốn không nên xuất hiện vì vượt quá giới hạn trích đoạn ngắn của bài viết gốc rất dài này.</p>"
        );
        let excerpt = extract_plain_summary("   ", long_content);
        assert!(excerpt.contains("Câu một với từ ngụy biện"));
        assert!(excerpt.contains("Câu hai"));
        assert!(excerpt.contains("Câu ba"));
        assert!(
            !excerpt.contains("Câu bốn"),
            "Excerpt must stop after 2-3 sentences: {excerpt}"
        );

        let very_long_sentence_content = format!("<p>{}</p>", "dữ liệu đối chiếu — ".repeat(40));
        let capped_excerpt = extract_plain_summary("", &very_long_sentence_content);
        assert!(capped_excerpt.ends_with("..."));
        assert!(capped_excerpt.chars().count() <= 303);

        // 4. Both empty returns ""
        assert_eq!(extract_plain_summary("", "  "), "");
    }
}
