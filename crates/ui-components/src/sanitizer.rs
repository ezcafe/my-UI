use ammonia::Builder;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static RE_ANCHOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<a(\s+[^>]*)?>"#).expect("Valid anchor regex")
});
static RE_TARGET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"target="[^"]*""#).expect("Valid target regex")
});

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
    RE_ANCHOR.replace_all(&cleaned, |caps: &regex::Captures| {
        let tag = caps.get(0).unwrap().as_str();
        if tag.contains("target=") {
            RE_TARGET.replace(tag, r#"target="_blank""#).to_string()
        } else {
            tag.replacen("<a", "<a target=\"_blank\"", 1)
        }
    }).to_string()
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
        assert!(clean.contains("<p>Safe content</p>"), "Must preserve safe tags");
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
        assert!(clean.contains(r#"<a target="_blank" href="https://a.com" rel="noopener noreferrer">A</a>"#));
        assert!(clean.contains(r#"<a target="_blank" href="https://b.com" rel="noopener noreferrer">B</a>"#));
        assert!(clean.contains(r#"<a target="_blank" href="https://c.com" rel="noopener noreferrer">C</a>"#));
    }

    #[test]
    fn test_sanitizer_preserves_links_and_attributes_with_lazylock() {
        let input = r#"<p><a href="https://example.com" class="link">Visit</a></p>"#;
        let output = sanitize_html(input);
        assert!(output.contains(r#"href="https://example.com""#));
        assert!(output.contains(r#"target="_blank""#));
        assert!(output.contains(r#"rel="noopener noreferrer""#));
    }
}
