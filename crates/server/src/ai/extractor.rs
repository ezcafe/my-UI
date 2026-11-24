use scraper::{ElementRef, Html, Node, Selector};
use std::sync::LazyLock;
use std::time::Duration;

static ARTICLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("article").expect("valid article selector"));
static BODY_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("body").expect("valid body selector"));

#[derive(Clone)]
pub struct ArticleExtractor {
    client: reqwest::Client,
}

impl ArticleExtractor {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .user_agent("Mozilla/5.0 SpatialRSS/1.0")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub fn parse_html(&self, html_str: &str, _title: &str, teaser: &str) -> String {
        let document = Html::parse_document(html_str);

        // 1. Try finding <article> tag
        let mut extracted_text = String::new();
        if let Some(article_elem) = document.select(&ARTICLE_SELECTOR).next() {
            extracted_text = Self::extract_clean_text(article_elem);
        }

        // 2. If <article> tag is absent or too short (< 50 chars), fallback to <body>
        if extracted_text.len() < 50 {
            if let Some(body_elem) = document.select(&BODY_SELECTOR).next() {
                extracted_text = Self::extract_clean_text(body_elem);
            }
        }

        // 3. If still empty or too short, fallback to teaser
        if extracted_text.len() < 50 {
            extracted_text = miniflux_client::summary::extract_summary(teaser, 3);
            if extracted_text.is_empty() {
                extracted_text = teaser.trim().to_string();
            }
        }

        Self::truncate_words(&extracted_text, 100)
    }

    pub async fn extract_content(&self, url: &str, _title: &str, teaser: &str) -> String {
        // Fast-path: If teaser has >= 50 words, return immediately without network fetch
        let clean_teaser = miniflux_client::summary::extract_summary(teaser, 3);
        let teaser_text = if clean_teaser.is_empty() {
            teaser.trim()
        } else {
            &clean_teaser
        };
        let word_count = teaser_text.split_whitespace().count();
        if word_count >= 50 {
            return Self::truncate_words(teaser_text, 100);
        }

        if url.trim().is_empty() || !url.starts_with("http") {
            return Self::truncate_words(teaser_text, 100);
        }

        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(html) => self.parse_html(&html, _title, teaser),
                Err(_) => Self::truncate_words(teaser_text, 100),
            },
            _ => {
                // HTTP 403, 429, Cloudflare challenge, paywall, timeout, or network error
                Self::truncate_words(teaser_text, 100)
            }
        }
    }

    fn extract_clean_text(element: ElementRef) -> String {
        let mut text_chunks = Vec::new();
        Self::traverse_element(element, &mut text_chunks);
        text_chunks.join(" ")
    }

    fn traverse_element(element: ElementRef, chunks: &mut Vec<String>) {
        const BOILERPLATE_TAGS: &[&str] = &[
            "script", "style", "nav", "header", "footer", "aside", "noscript", "svg", "form",
            "button",
        ];

        for child in element.children() {
            match child.value() {
                Node::Element(elem) => {
                    let tag_name = elem.name();
                    if BOILERPLATE_TAGS.contains(&tag_name) {
                        continue;
                    }
                    if let Some(child_elem) = ElementRef::wrap(child) {
                        Self::traverse_element(child_elem, chunks);
                    }
                }
                Node::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        chunks.push(trimmed.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    fn truncate_words(text: &str, max_words: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > max_words {
            words[..max_words].join(" ")
        } else {
            words.join(" ")
        }
    }
}

impl Default for ArticleExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_extract_article_tag_content_stripping_boilerplate() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Ignored</title></head>
            <body>
                <nav><a href="/">Home</a><a href="/news">News</a></nav>
                <article>
                    <h1>Breaking Discovery</h1>
                    <script>console.log("ad script");</script>
                    <style>.ad { color: red; }</style>
                    <p>Scientists have discovered a new high-temperature superconductor in a laboratory experiment.</p>
                    <aside>Related stories you might like</aside>
                    <p>The material functions at ambient pressure, opening unprecedented avenues for grid-scale energy transport.</p>
                    <footer>Comments and copyright 2026</footer>
                </article>
                <footer>Site Footer</footer>
            </body>
            </html>
        "#;
        let extractor = ArticleExtractor::new();
        let content = extractor.parse_html(html, "Breaking Discovery", "Default teaser");

        assert!(
            content.contains("Scientists have discovered a new high-temperature superconductor")
        );
        assert!(content.contains("The material functions at ambient pressure"));
        assert!(!content.contains("Title:"));
        assert!(!content.contains("ad script"));
        assert!(!content.contains("Site Footer"));
        assert!(!content.contains("Comments and copyright"));
        assert!(!content.contains("Related stories"));
        assert!(!content.contains("Home"));
    }

    #[test]
    fn test_extract_article_fallback_when_article_tag_absent() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <body>
                <header><nav>Navigation</nav></header>
                <div class="main-content">
                    <p>This page has no article tag whatsoever. However, the body contains significant meaningful text that should be extracted when article tags are completely missing from the HTML markup.</p>
                </div>
            </body>
            </html>
        "#;
        let extractor = ArticleExtractor::new();
        let content = extractor.parse_html(html, "No Article Tag", "Default teaser");

        assert!(content.contains("This page has no article tag whatsoever."));
        assert!(!content.contains("Title:"));
        assert!(!content.contains("Navigation"));
    }

    #[tokio::test]
    async fn test_extract_article_fallback_on_http_error_or_block() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/blocked"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let extractor = ArticleExtractor::new();
        let blocked_url = format!("{}/blocked", mock_server.uri());
        let teaser = "Teaser sentence 1. Teaser sentence 2. Teaser sentence 3. Extra sentence 4 that gets truncated.";
        let content = extractor
            .extract_content(&blocked_url, "Blocked News", teaser)
            .await;

        assert!(content.contains("Teaser sentence 1. Teaser sentence 2. Teaser sentence 3."));
        assert!(!content.contains("Title:"));
        assert!(!content.contains("Extra sentence 4"));
    }

    #[test]
    fn test_extractor_parses_content_with_lazylock_selectors() {
        let html = "<article><p>Hello world from lazy selector article extraction!</p></article>";
        let extractor = ArticleExtractor::new();
        let content = extractor.parse_html(html, "Lazy Test", "teaser");
        assert!(!content.contains("Title:"));
        assert!(content.contains("Hello world from lazy selector article extraction!"));
    }

    #[tokio::test]
    async fn test_extractor_teaser_fast_path_skips_http() {
        // Mock server expecting 0 requests
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&mock_server)
            .await;

        // Build a teaser with >= 50 words
        let teaser_words: Vec<String> = (1..=55).map(|i| format!("word{}", i)).collect();
        let teaser = teaser_words.join(" ");

        let extractor = ArticleExtractor::new();
        let fast_path_url = format!("{}/articles/fast", mock_server.uri());
        let content = extractor
            .extract_content(&fast_path_url, "Fast Path Title", &teaser)
            .await;

        // Must return teaser content without triggering HTTP fetch
        assert!(!content.is_empty());
        assert!(content.contains("word1"));
        assert!(content.contains("word55"));
        assert!(!content.contains("Title:"));
        // Mock server verifies that 0 requests were received
    }

    #[test]
    fn test_extractor_word_truncation_100() {
        let words = (0..300)
            .map(|i| format!("word{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        let html = format!(
            "<html><body><article><p>{}</p></article></body></html>",
            words
        );
        let extractor = ArticleExtractor::new();
        let content = extractor.parse_html(&html, "100 Test Title", "teaser");

        let word_count = content.split_whitespace().count();
        assert_eq!(word_count, 100, "Must be truncated to strictly 100 words");
        assert!(!content.starts_with("Title:"));
    }
}
