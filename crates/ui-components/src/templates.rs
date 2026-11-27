use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryViewModel {
    pub id: i64,
    pub title: String,
    pub suggested_title: Option<String>,
    pub feed_title: String,
    #[serde(default)]
    pub category_title: Option<String>,
    pub published_at: String,
    pub reading_time: i32,
    pub summary: String,
    #[serde(default)]
    pub sanitized_content: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub rewrite: Option<String>,
    #[serde(default)]
    pub body_images: Vec<String>,
    pub url: String,
    pub author: Option<String>,
    pub is_read: bool,
    pub is_bookmarked: bool,
}

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub unread_count: i32,
    pub today_count: i64,
    pub entries_list_html: String,
    pub ai_enabled: bool,
    pub auth_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSummaryGroupViewModel {
    pub feed_title: String,
    pub entries: Vec<EntryViewModel>,
}

#[derive(Template)]
#[template(path = "partials/ai_grouped_list.html")]
pub struct AiGroupedListTemplate {
    pub groups: Vec<FeedSummaryGroupViewModel>,
}

#[derive(Template)]
#[template(path = "partials/ai_summary_card.html")]
pub struct AiSummaryCardTemplate {
    pub summary: String,
}

#[derive(Template)]
#[template(path = "partials/ai_error_card.html")]
pub struct AiErrorCardTemplate<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub retry_url: &'a str,
    pub target: &'a str,
    pub swap: &'a str,
    pub container_id: &'a str,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate;

#[derive(Template)]
#[template(path = "partials/entry_item.html")]
pub struct EntryItemTemplate {
    pub entry: EntryViewModel,
    pub unread_count: Option<i32>,
}

#[derive(Template)]
#[template(path = "partials/entry_list.html")]
pub struct EntryListTemplate {
    pub entries: Vec<EntryViewModel>,
    pub top_executive_brief: Option<String>,
    pub unread_count: Option<i32>,
    pub current_page: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_offset: i64,
    pub next_offset: i64,
    pub status: String,
    pub search: String,
    pub limit: i64,
    pub is_brief_eligible: bool,
    pub offset: i64,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
    pub auth_enabled: bool,
}

#[derive(Template)]
#[template(path = "partials/executive_brief_skeleton.html")]
pub struct ExecutiveBriefSkeletonTemplate {
    pub offset: i64,
    pub limit: i64,
    pub status: String,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
}

#[derive(Template)]
#[template(path = "partials/executive_brief_card.html")]
pub struct ExecutiveBriefCardTemplate<'a> {
    pub brief: &'a str,
}

#[derive(Template)]
#[template(path = "partials/brief_card.html")]
pub struct BriefCardTemplate {
    pub summary: String,
}

#[derive(Template)]
#[template(path = "partials/entry_detail.html")]
pub struct EntryDetailTemplate {
    pub entry: EntryViewModel,
    pub summary: String,
    pub auth_enabled: bool,
    pub rewrite: String,
    pub needs_rewrite: bool,
    pub ai_enabled: bool,
    pub fallback_summary: String,
    pub impact: String,
    pub fallacies: String,
    pub has_analysis: bool,
    pub hero_image: Option<String>,
    pub body_images: Vec<String>,
}

/// Formats an ISO-8601 / RFC 3339 or `YYYY-MM-DD` date string as `DD/MM/YYYY`.
/// Falls back to the trimmed input if parsing fails.
#[cfg(test)]
pub fn format_published_date_ddmmyyyy(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return dt.format("%d/%m/%Y").to_string();
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return date.format("%d/%m/%Y").to_string();
    }
    trimmed.to_string()
}

/// Formats an ISO-8601 / RFC 3339 timestamp relative to `now`, or `YYYY-MM-DD` as `DD/MM/YYYY`.
pub fn format_relative_time_at(raw: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return date.format("%d/%m/%Y").to_string();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        let utc_dt = dt.with_timezone(&chrono::Utc);
        let diff_sec = (now - utc_dt).num_seconds().max(0);
        return match diff_sec {
            0..=59 => "Just now".to_string(),
            60..=3599 => format!("{}m ago", diff_sec / 60),
            3600..=86399 => format!("{}h ago", diff_sec / 3600),
            86400..=604799 => format!("{}d ago", diff_sec / 86400),
            _ => utc_dt.format("%d/%m/%Y").to_string(),
        };
    }
    trimmed.to_string()
}

/// Formats a timestamp relative to the current UTC time.
pub fn format_relative_time(raw: &str) -> String {
    format_relative_time_at(raw, chrono::Utc::now())
}

impl EntryViewModel {
    pub fn relative_published_at(&self) -> String {
        format_relative_time(&self.published_at)
    }
}

impl EntryDetailTemplate {
    pub fn safe_entry_url(&self) -> &str {
        let trimmed = self.entry.url.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("http://")
            || lower.starts_with("https://")
            || (trimmed.starts_with('/') && !trimmed.starts_with("//"))
        {
            trimmed
        } else {
            "#"
        }
    }
}

#[derive(Serialize)]
struct GuestEntryPayload<'a> {
    id: i64,
    title: &'a str,
    feed_title: &'a str,
    published_at: &'a str,
    url: &'a str,
    summary: &'a str,
    rewrite: Option<&'a str>,
    image_url: Option<&'a str>,
    body_images: &'a [String],
}

fn escape_json_for_html_script(json: &str) -> String {
    json.replace('&', r"\u0026")
        .replace('<', r"\u003c")
        .replace('>', r"\u003e")
        .replace('\u{2028}', r"\u2028")
        .replace('\u{2029}', r"\u2029")
}

impl EntryListTemplate {
    pub fn guest_entries_json(&self) -> Option<String> {
        if self.auth_enabled || self.entries.is_empty() {
            return None;
        }
        let payloads: Vec<GuestEntryPayload<'_>> = self
            .entries
            .iter()
            .map(|e| {
                let title = e.suggested_title.as_deref().unwrap_or(&e.title);
                GuestEntryPayload {
                    id: e.id,
                    title,
                    feed_title: &e.feed_title,
                    published_at: &e.published_at,
                    url: &e.url,
                    summary: &e.summary,
                    rewrite: e.rewrite.as_deref(),
                    image_url: e.image_url.as_deref(),
                    body_images: &e.body_images,
                }
            })
            .collect();
        serde_json::to_string(&payloads)
            .ok()
            .map(|json| escape_json_for_html_script(&json))
    }
}

#[derive(Template)]
#[template(path = "partials/analysis_cards.html")]
pub struct AnalysisCardsTemplate<'a> {
    pub impact: &'a str,
    pub fallacies: &'a str,
    pub oob: bool,
    pub entry_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToastUndoPayload {
    pub entry_ids: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i64>,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastUndoAction {
    pub url: String,
    pub vals_json: String,
    pub label: String,
}

impl ToastUndoAction {
    pub fn from_payload(
        url: impl Into<String>,
        label: impl Into<String>,
        payload: &ToastUndoPayload,
    ) -> Self {
        Self {
            url: url.into(),
            vals_json: serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string()),
            label: label.into(),
        }
    }
}

#[derive(Template)]
#[template(path = "partials/toast.html")]
pub struct ToastTemplate {
    pub message: String,
    pub is_error: bool,
    pub undo: Option<ToastUndoAction>,
}

#[derive(Template)]
#[template(path = "partials/error_card.html")]
pub struct ErrorCardTemplate {
    pub message: String,
    pub retry_url: String,
    pub target: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_models() {
        let entry = EntryViewModel {
            id: 10,
            title: "Original Title".to_string(),
            suggested_title: Some("AI Punchy Title".to_string()),
            feed_title: "Tech Feed".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 2,
            summary: "Extractive snippet".to_string(),
            url: "https://example.com/10".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        assert_eq!(entry.suggested_title.as_deref(), Some("AI Punchy Title"));

        let list_tmpl = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: Some(
                "Today's key themes: AI optimization and Rust speed.".to_string(),
            ),
            unread_count: Some(1),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 20,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 20,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        assert_eq!(
            list_tmpl.top_executive_brief.as_deref(),
            Some("Today's key themes: AI optimization and Rust speed.")
        );
    }

    #[test]
    fn test_entry_list_and_item_render_suggested_title_and_brief() {
        let entry_with_ai = EntryViewModel {
            id: 101,
            title: "Original Long Headline Goes Here".to_string(),
            suggested_title: Some("AI Suggested Headline".to_string()),
            feed_title: "Feed".to_string(),
            published_at: "10m ago".to_string(),
            reading_time: 1,
            summary: "Summary".to_string(),
            url: "https://example.com/101".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let item_tmpl = EntryItemTemplate {
            entry: entry_with_ai.clone(),
            unread_count: None,
        };
        let item_html = item_tmpl.render().expect("render item");
        assert!(item_html.contains("AI Suggested Headline"));
        assert!(item_html.contains(r#"title="Original Long Headline Goes Here""#));

        let list_tmpl = EntryListTemplate {
            entries: vec![entry_with_ai],
            top_executive_brief: Some("Big shifts in AI chip market announced today.".to_string()),
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 20,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 20,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let list_html = list_tmpl.render().expect("render list");
        assert!(list_html.contains("Executive Brief"));
        assert!(list_html.contains("Big shifts in AI chip market announced today."));
        assert!(list_html.contains("AI Suggested Headline"));
        assert!(list_html.contains(r#"title="Original Long Headline Goes Here""#));

        let brief_tmpl = BriefCardTemplate {
            summary: "Nvidia unveiled Blackwell Ultra, boosting memory bandwidth by 40%."
                .to_string(),
        };
        let brief_html = brief_tmpl.render().expect("render brief card");
        assert!(brief_html.contains("Executive Brief"));
        assert!(brief_html.contains("Nvidia unveiled Blackwell Ultra"));
    }

    #[test]
    fn test_render_title_only_entry_item() {
        let entry_unread = EntryViewModel {
            id: 42,
            title: "Spatial Computing in Rust".to_string(),
            suggested_title: None,
            feed_title: "Rust Optics".to_string(),
            published_at: "2 hours ago".to_string(),
            reading_time: 4,
            summary: "Extractive brief of the post.".to_string(),
            url: "https://example.com/spatial".to_string(),
            author: Some("Visionary".to_string()),
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: true,
            rewrite: None,
            body_images: vec![],
        };

        let template_unread = EntryItemTemplate {
            entry: entry_unread,
            unread_count: Some(5),
        };
        let html_unread = template_unread
            .render()
            .expect("Failed to render EntryItemTemplate");

        // Title, metadata, and styling
        assert!(html_unread.contains("Spatial Computing in Rust"));
        assert!(html_unread.contains("line-clamp-2"));
        assert!(html_unread.contains("text-sm font-medium"));

        // Feed title and relative time render cleanly while summary and reading time remain omitted
        assert!(
            html_unread.contains("Rust Optics"),
            "Feed title must render in entry item metadata"
        );
        assert!(
            html_unread.contains("2 hours ago") && html_unread.contains(" · "),
            "Published time and separator must render in entry item metadata"
        );
        assert!(
            !html_unread.contains("Extractive brief of the post"),
            "Summary must be omitted"
        );
        assert!(
            !html_unread.contains("min read"),
            "Reading time must be omitted"
        );

        // External link button attributes
        assert!(html_unread.contains("href=\"https://example.com/spatial\""));
        assert!(html_unread.contains("target=\"_blank\""));
        assert!(html_unread.contains("rel=\"noopener noreferrer\""));
        assert!(html_unread.contains("onclick=\"event.stopPropagation(); markArticleRead(42)\""));
        assert!(html_unread.contains("data-lucide=\"external-link\""));
        assert!(html_unread.contains("data-lucide=\"chevron-right\""));

        // Unread indicator dot present when is_read = false
        assert!(html_unread.contains("title=\"Unread\"") || html_unread.contains("bg-brand"));

        // Omission of dotted leader
        assert!(
            !html_unread.contains("leader"),
            "Must omit dotted leader in entry item"
        );
        assert!(
            html_unread.contains("bg-brand") || html_unread.contains("var(--brand)"),
            "Must include brand red indicator"
        );
        assert!(
            html_unread.contains("last:border-b-0"),
            "Entry item must include last:border-b-0"
        );

        // When is_read = true, unread indicator dot is omitted
        let entry_read = EntryViewModel {
            id: 43,
            title: "Spatial Computing in Rust - Read".to_string(),
            suggested_title: None,
            feed_title: "Rust Optics".to_string(),
            published_at: "2 hours ago".to_string(),
            reading_time: 4,
            summary: "Extractive brief of the post.".to_string(),
            url: "https://example.com/spatial-read".to_string(),
            author: Some("Visionary".to_string()),
            is_read: true,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template_read = EntryItemTemplate {
            entry: entry_read,
            unread_count: None,
        };
        let html_read = template_read
            .render()
            .expect("Failed to render read EntryItemTemplate");
        assert!(!html_read.contains("title=\"Unread\""));

        // OOB unread counter
        assert!(html_unread.contains("id=\"unread-pill\""));
        assert!(html_unread.contains("hx-swap-oob=\"true\""));
    }

    #[test]
    fn test_render_entry_list_with_title_only_items() {
        let entries = vec![
            EntryViewModel {
                id: 1,
                title: "First Title".to_string(),
                suggested_title: None,
                feed_title: "Feed A".to_string(),
                published_at: "1 hour ago".to_string(),
                reading_time: 2,
                summary: "Summary A".to_string(),
                url: "https://example.com/1".to_string(),
                author: None,
                is_read: false,
                category_title: None,
                image_url: None,
                sanitized_content: None,
                is_bookmarked: false,
                rewrite: None,
                body_images: vec![],
            },
            EntryViewModel {
                id: 2,
                title: "Second Title".to_string(),
                suggested_title: None,
                feed_title: "Feed B".to_string(),
                published_at: "2 hours ago".to_string(),
                reading_time: 5,
                summary: "Summary B".to_string(),
                url: "https://example.com/2".to_string(),
                author: None,
                is_read: true,
                category_title: None,
                image_url: None,
                sanitized_content: None,
                is_bookmarked: true,
                rewrite: None,
                body_images: vec![],
            },
        ];

        let template = EntryListTemplate {
            entries,
            top_executive_brief: None,
            unread_count: Some(1),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "all".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = template
            .render()
            .expect("Failed to render EntryListTemplate");

        let list_view_html = html.split(r#"id="entries-magazine-view""#).next().unwrap();
        assert!(list_view_html.contains("First Title"));
        assert!(list_view_html.contains("Second Title"));
        assert!(
            !list_view_html.contains("Summary A"),
            "List view rows must be title-only without summary"
        );
        assert!(
            !list_view_html.contains("Summary B"),
            "List view rows must be title-only without summary"
        );
        assert!(html.contains("data-lucide=\"external-link\""));
        assert!(html.contains("href=\"https://example.com/1\""));
        assert!(html.contains("href=\"https://example.com/2\""));
        assert!(
            html.contains("last:border-b-0"),
            "Entry list item must include last:border-b-0"
        );
    }

    #[test]
    fn test_filter_bar_reordering() {
        let template = IndexTemplate {
            unread_count: 3,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // Filter button sequence in order: Unread < Today < Starred < All
        let pos_unread = html.find("Unread").expect("Must contain 'Unread'");
        let pos_today = html.find("Today").expect("Must contain 'Today'");
        let pos_starred = html.find("Starred").expect("Must contain 'Starred'");
        let pos_all = html.find("All").expect("Must contain 'All'");

        assert!(pos_unread < pos_today, "Unread must precede Today");
        assert!(pos_today < pos_starred, "Today must precede Starred");
        assert!(pos_starred < pos_all, "Starred must precede All");

        // Unread is active by default with Annuaire ink pill styling
        assert!(
            html.contains("id=\"filter-unread\"")
                && html.contains("bg-primary text-primary-foreground"),
            "Unread filter must be active with Annuaire ink pill styling"
        );

        // Today button triggers /hx/entries?status=today
        assert!(html.contains("/hx/entries?status=today"));
        assert!(html.contains("setActiveFilter"));
    }

    #[test]
    fn test_index_template_no_ai_filter() {
        let index_enabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        };
        let html_enabled = index_enabled
            .render()
            .expect("Failed to render IndexTemplate enabled");
        assert!(
            !html_enabled.contains("id=\"filter-ai\""),
            "filter-ai button must be completely removed from index.html"
        );
        assert!(
            !html_enabled.contains("status=ai"),
            "status=ai must not be present in index.html"
        );
    }

    #[test]
    fn test_index_template_hides_logout_button_when_auth_disabled() {
        let index_auth_enabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html_enabled = index_auth_enabled
            .render()
            .expect("Failed to render with auth enabled");
        assert!(
            html_enabled.contains("/auth/logout"),
            "Logout button must render when auth_enabled is true"
        );

        let index_auth_disabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html_disabled = index_auth_disabled
            .render()
            .expect("Failed to render with auth disabled");
        assert!(
            !html_disabled.contains("/auth/logout"),
            "Logout button must be hidden when auth_enabled is false"
        );
    }

    #[test]
    fn test_index_template_renders_guest_state_script_when_auth_disabled() {
        let index_auth_disabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = index_auth_disabled
            .render()
            .expect("Failed to render with auth disabled");
        assert!(
            html.contains("my_news_guest_db"),
            "Must initialize guest IndexedDB"
        );
        assert!(
            html.contains("read_entries"),
            "Must reference read_entries store"
        );
        assert!(html.contains("readSet"), "Must maintain in-memory readSet");
    }

    #[test]
    fn test_ai_grouped_list_template_renders_feed_headers_and_canonical_route() {
        let entry = EntryViewModel {
            id: 99,
            title: "Autonomous Agent Flow".to_string(),
            suggested_title: None,
            feed_title: "AI Daily".to_string(),
            published_at: "10 mins ago".to_string(),
            reading_time: 2,
            summary: "Extractive teaser".to_string(),
            url: "https://example.com/agent".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let group = FeedSummaryGroupViewModel {
            feed_title: "AI Daily".to_string(),
            entries: vec![entry],
        };
        let template = AiGroupedListTemplate {
            groups: vec![group],
        };
        let html = template
            .render()
            .expect("Failed to render AiGroupedListTemplate");

        assert!(html.contains("AI Daily"));
        assert!(html.contains("Autonomous Agent Flow"));
        assert!(html.contains("hx-get=\"/hx/entries/99/ai-summary\""));
        assert!(html.contains("hx-trigger=\"revealed\""));
        assert!(html.contains("hx-swap=\"outerHTML\""));
    }

    #[test]
    fn test_detail_template_omits_author() {
        let entry = EntryViewModel {
            id: 101,
            title: "Exploring Spatial Audio".to_string(),
            suggested_title: None,
            feed_title: "Audio Weekly".to_string(),
            published_at: "2 hours ago".to_string(),
            reading_time: 3,
            summary: "Spatial summary.".to_string(),
            url: "https://example.com/audio".to_string(),
            author: Some("Visionary Designer".to_string()),
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Extractive summary".to_string(),
            auth_enabled: true,
            rewrite: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate");

        // Author and reading_time must be omitted, while feed_title and <time> render cleanly
        assert!(
            !html.contains("By Visionary Designer"),
            "Author must be omitted from detail view"
        );
        assert!(!html.contains("By "), "Author prefix must not be rendered");
        assert!(
            html.contains("Audio Weekly")
                && html.contains("2 hours ago")
                && html.contains(" · ")
                && html.contains(r#"data-relative-time="true""#),
            "Feed title and relative time must render in detail view: {html}"
        );
    }

    #[test]
    fn test_detail_template_omits_metadata_and_orders_sections() {
        let make_entry = |url: &str| EntryViewModel {
            id: 250,
            title: "Filtered Article".to_string(),
            suggested_title: Some("AI Headline".to_string()),
            feed_title: "Hidden Feed".to_string(),
            published_at: "2026-11-26T10:00:00Z".to_string(),
            reading_time: 5,
            summary: "Summary".to_string(),
            url: url.to_string(),
            author: Some("Hidden Author".to_string()),
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        // 1. Option C pending rewrite with unsafe URL & body images
        let tmpl = EntryDetailTemplate {
            entry: make_entry("javascript:alert(1)"),
            summary: String::new(),
            auth_enabled: false,
            rewrite: String::new(),
            needs_rewrite: true,
            ai_enabled: true,
            fallback_summary: "Fallback plain text.".to_string(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: Some("https://example.com/hero.jpg".to_string()),
            body_images: vec!["https://example.com/photo1.jpg".to_string()],
        };
        assert_eq!(tmpl.safe_entry_url(), "#");
        let html = tmpl.render().expect("render Option C detail template");
        assert!(html.contains("Hidden Feed"));
        assert!(html.contains(r#"datetime="2026-11-26T10:00:00Z""#));
        assert!(html.contains("<time"));
        assert!(html.contains(r#"data-relative-time="true""#));
        assert!(!html.contains("Hidden Author"));
        assert!(!html.contains("5 min read"));
        assert!(!html.contains("data-date-format"));
        assert!(!html.contains("javascript:alert(1)"));
        assert!(html.contains(r##"href="#""##));
        assert!(html.contains(r##"data-url="#""##));
        assert!(html.contains(r#"hx-get="/hx/entries/250/rewrite""#));
        assert!(html.contains(r#"hx-trigger="load""#));
        assert!(html.contains(r#"data-fallback-summary="Fallback plain text.""#));
        assert!(html.contains("AI rewrite is not ready yet — generating..."));
        assert!(html.contains(r#"hx-trigger="ai-rewrite-done-250 from:body""#));

        let pos_hero = html.find("https://example.com/hero.jpg").unwrap();
        let pos_rewrite = html.find("id=\"ai-rewrite-box\"").unwrap();
        let pos_body_img = html.find("https://example.com/photo1.jpg").unwrap();
        let pos_analysis = html.find("id=\"ai-analysis-box\"").unwrap();
        assert!(pos_hero < pos_rewrite);
        assert!(pos_rewrite < pos_body_img);
        assert!(pos_body_img < pos_analysis);

        // 2. Protocol-relative URL blocked, valid URL (including uppercase scheme) allowed
        let tmpl_rel = EntryDetailTemplate {
            entry: make_entry("//evil.example.com/x"),
            summary: String::new(),
            auth_enabled: true,
            rewrite: String::new(),
            needs_rewrite: false,
            ai_enabled: false,
            fallback_summary: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
        };
        assert_eq!(tmpl_rel.safe_entry_url(), "#");
        let tmpl_upper = EntryDetailTemplate {
            entry: make_entry("HTTPS://example.com/article"),
            ..tmpl_rel
        };
        assert_eq!(tmpl_upper.safe_entry_url(), "HTTPS://example.com/article");
        let html_disabled = tmpl_upper.render().expect("render !ai_enabled detail");
        assert!(html_disabled.contains("AI rewrite is unavailable."));
        assert!(html_disabled.contains("No summary available."));
        assert!(!html_disabled.contains("id=\"ai-analysis-box\""));
    }

    #[test]
    fn test_analysis_cards_template_oob_attribute() {
        let oob_html = AnalysisCardsTemplate {
            impact: "Impact text",
            fallacies: "Fallacies text",
            oob: true,
            entry_id: Some(505),
        }
        .render()
        .expect("render oob analysis cards");
        assert!(oob_html.contains(r#"id="ai-analysis-box" hx-swap-oob="outerHTML""#));
        assert!(oob_html.contains(r#"data-target-entry-id="505""#));
        assert!(oob_html.contains("Impact text"));
        assert!(oob_html.contains("Fallacies text"));

        let normal_html = AnalysisCardsTemplate {
            impact: "Impact text",
            fallacies: "",
            oob: false,
            entry_id: None,
        }
        .render()
        .expect("render normal analysis cards");
        assert!(normal_html.contains(r#"id="ai-analysis-box""#));
        assert!(!normal_html.contains("hx-swap-oob"));
        assert!(!normal_html.contains("data-target-entry-id"));
    }

    #[test]
    fn test_entry_detail_template_renders_analysis_cards_without_sanitized_content() {
        let entry = EntryViewModel {
            id: 105,
            title: "Analysis and Rewrite Article".to_string(),
            suggested_title: None,
            feed_title: "Tech Wire".to_string(),
            published_at: "10 mins ago".to_string(),
            reading_time: 2,
            summary: "Brief text.".to_string(),
            url: "https://example.com/article".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Executive brief text.".to_string(),
            auth_enabled: true,
            rewrite: "Inverted pyramid rewrite text.".to_string(),
            impact: "Substantial economic impact across software sector.".to_string(),
            fallacies: "Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"
                .to_string(),
            has_analysis: true,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template.render().expect("render detail template");

        assert!(!html.contains("Executive Brief"));
        assert!(!html.contains("Executive brief text."));
        assert!(html.contains("Impact Analysis"));
        assert!(html.contains("Substantial economic impact across software sector."));
        assert!(html.contains("Logical Fallacy & Objectivity Check"));
        assert!(html.contains("Không phát hiện lỗi ngụy biện"));
        assert!(html.contains("Inverted pyramid rewrite text."));
        assert!(html.contains("id=\"ai-rewrite-box\""));
        assert!(html.contains("id=\"ai-analysis-box\""));

        let pos_rewrite = html.find("Inverted pyramid rewrite text.").unwrap();
        let pos_impact = html.find("Impact Analysis").unwrap();
        let pos_fallacy = html.find("Logical Fallacy & Objectivity Check").unwrap();
        assert!(
            pos_rewrite < pos_impact,
            "Rewrite must appear before Impact Analysis"
        );
        assert!(
            pos_impact < pos_fallacy,
            "Impact Analysis must appear before Fallacy Check"
        );
    }

    #[test]
    fn test_entry_detail_template_shows_rewrite_and_analysis_trigger_when_not_cached() {
        let entry = EntryViewModel {
            id: 106,
            title: "Article Awaiting Analysis".to_string(),
            suggested_title: None,
            feed_title: "Tech Wire".to_string(),
            published_at: "10 mins ago".to_string(),
            reading_time: 2,
            summary: "Brief text.".to_string(),
            url: "https://example.com/article".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Teaser text.".to_string(),
            auth_enabled: true,
            rewrite: "Immediate rewrite text loaded instantly.".to_string(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template.render().expect("render detail template");

        assert!(!html.contains("Executive Brief"));
        assert!(html.contains("Immediate rewrite text loaded instantly."));
        assert!(html.contains("id=\"ai-rewrite-box\""));
        assert!(html.contains("id=\"ai-analysis-box\""));
        assert!(html.contains("hx-get=\"/hx/entries/106/analysis\""));
        assert!(html.contains("hx-trigger=\"load\""));
        assert!(html.contains("hx-swap=\"outerHTML\""));
    }

    #[test]
    fn test_detail_template_has_mark_unread_button() {
        let entry = EntryViewModel {
            id: 102,
            title: "Micro-gestures in WebXR".to_string(),
            suggested_title: None,
            feed_title: "XR Trends".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 4,
            summary: "XR summary.".to_string(),
            url: "https://example.com/xr".to_string(),
            author: None,
            is_read: true,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "XR summary".to_string(),
            auth_enabled: true,
            rewrite: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate");

        // Mark as unread button present
        assert!(
            html.contains("hx-post=\"/hx/entries/102/read?status=unread\""),
            "Detail view must contain mark unread button"
        );
        assert!(
            html.contains("hx-target=\"#entry-102\""),
            "Mark unread button must target #entry-102"
        );
        assert!(
            html.contains("title=\"Mark as unread\""),
            "Mark unread button must have tooltip title"
        );

        // Precedes bookmark button
        let pos_mark_unread = html
            .find("hx-post=\"/hx/entries/102/read?status=unread\"")
            .unwrap();
        let pos_bookmark = html.find("hx-post=\"/hx/entries/102/bookmark\"").unwrap();
        assert!(
            pos_mark_unread < pos_bookmark,
            "Mark as unread button must precede bookmark button"
        );
    }

    #[test]
    fn test_entry_detail_renders_local_actions_when_auth_disabled() {
        let entry = EntryViewModel {
            id: 102,
            title: "Micro-gestures in WebXR".to_string(),
            suggested_title: None,
            feed_title: "XR Trends".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 4,
            summary: "XR summary.".to_string(),
            url: "https://example.com/xr".to_string(),
            author: None,
            is_read: true,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "XR summary".to_string(),
            auth_enabled: false,
            rewrite: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate");

        assert!(
            !html.contains("hx-post=\"/hx/entries/102/read"),
            "Guest mode must not contain hx-post for mark read"
        );
        assert!(
            !html.contains("hx-post=\"/hx/entries/102/bookmark\""),
            "Guest mode must not contain hx-post for bookmark"
        );
        assert!(
            html.contains("onclick=\"toggleLocalReadStatus(102)\""),
            "Guest mode must render onclick=\"toggleLocalReadStatus(102)\""
        );
        assert!(
            html.contains("onclick=\"triggerNativeBookmark("),
            "Guest mode must render onclick=\"triggerNativeBookmark(...)\""
        );
    }

    #[test]
    fn test_entry_detail_renders_htmx_actions_when_auth_enabled() {
        let entry = EntryViewModel {
            id: 102,
            title: "Micro-gestures in WebXR".to_string(),
            suggested_title: None,
            feed_title: "XR Trends".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 4,
            summary: "XR summary.".to_string(),
            url: "https://example.com/xr".to_string(),
            author: None,
            is_read: true,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "XR summary".to_string(),
            auth_enabled: true,
            rewrite: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate");

        assert!(
            html.contains("hx-post=\"/hx/entries/102/read?status=unread\""),
            "Auth mode must contain hx-post for mark read"
        );
        assert!(
            html.contains("hx-post=\"/hx/entries/102/bookmark\""),
            "Auth mode must contain hx-post for bookmark"
        );
        assert!(
            !html.contains("toggleLocalReadStatus"),
            "Auth mode must not contain toggleLocalReadStatus"
        );
        assert!(
            !html.contains("triggerNativeBookmark"),
            "Auth mode must not contain triggerNativeBookmark"
        );
    }

    #[test]
    fn test_render_toast_oob_template() {
        let template = ToastTemplate {
            message: "Upstream service timeout".to_string(),
            is_error: true,
            undo: None,
        };
        let html = template.render().expect("Failed to render ToastTemplate");

        assert!(html.contains("id=\"toast-container\""));
        assert!(html.contains("hx-swap-oob=\"beforeend\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-atomic=\"true\""));
        assert!(html.contains("Upstream service timeout"));
        assert!(
            html.contains("shadow-float"),
            "Must use Annuaire .shadow-float elevation"
        );
        assert!(
            html.contains("rounded-full") && html.contains("bg-foreground text-background"),
            "Must use Annuaire ink pill floating toast style"
        );
        assert!(
            html.contains("onclick=\"dismissToast(this)\""),
            "Toast must have a dismiss/close button"
        );
        assert!(
            html.contains("data-lucide=\"x\""),
            "Toast close button must have x icon"
        );
    }

    #[test]
    fn test_index_template_renders_today_and_unread_counts() {
        let template = IndexTemplate {
            unread_count: 12,
            today_count: 5,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(
            html.contains("id=\"filter-unread-count\""),
            "Must contain filter-unread-count"
        );
        assert!(
            html.contains("(12)"),
            "Must contain unread count badge (12)"
        );
        assert!(
            html.contains("id=\"filter-today-count\""),
            "Must contain filter-today-count"
        );
        assert!(html.contains("(5)"), "Must contain today count badge (5)");
    }

    #[test]
    fn test_entry_list_renders_pagination_when_multiple_pages() {
        let entry = EntryViewModel {
            id: 1,
            title: "Test Entry".to_string(),
            suggested_title: None,
            feed_title: "Feed".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 1,
            summary: "".to_string(),
            url: "https://example.com/1".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 3,
            has_prev: false,
            has_next: true,
            prev_offset: 0,
            next_offset: 50,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(
            html.contains("aria-label=\"Pagination\""),
            "Must contain pagination nav bar"
        );
        assert!(
            html.contains("Page 1 of 3"),
            "Must display current and total pages"
        );
        assert!(html.contains("Previous"), "Must have Previous label");
        assert!(
            html.contains(
                "hx-get=\"/hx/entries?status=unread&amp;search=&amp;offset=50&amp;limit=50\""
            ) || html.contains("hx-get=\"/hx/entries?status=unread&search=&offset=50&limit=50\""),
            "Must have Next button with HTMX pagination query"
        );
        assert!(
            html.contains("cursor-not-allowed"),
            "Previous button must be disabled/not-allowed on first page"
        );
    }

    #[test]
    fn test_entry_list_hides_pagination_when_single_page() {
        let template = EntryListTemplate {
            entries: vec![],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(
            !html.contains("aria-label=\"Pagination\""),
            "Must NOT contain pagination nav bar when total_pages <= 1"
        );
        assert!(
            !html.contains("Page 1 of 1"),
            "Must NOT display pagination page numbers when single page"
        );
    }

    #[test]
    fn test_spatial_window_fluid_width_container_query() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(
            html.contains("container-type: inline-size;"),
            "Must define container-type: inline-size on spatial-shell"
        );
        assert!(
            html.contains("max-width: min(100cqi, 1440px);"),
            "Must define max-width: min(100cqi, 1440px) on spatial-window"
        );
        assert!(
            !html.contains("max-width: 720px;"),
            "Must not have fixed 720px max-width"
        );
    }

    #[test]
    fn test_entry_item_wires_open_entry_detail() {
        let entry = EntryViewModel {
            id: 42,
            title: "Test Entry".to_string(),
            suggested_title: None,
            feed_title: "Test Feed".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 1,
            summary: "Summary".to_string(),
            url: "https://example.com/item".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template = EntryItemTemplate {
            entry,
            unread_count: None,
        };
        let html = template
            .render()
            .expect("Failed to render EntryItemTemplate");
        assert!(
            html.contains("onclick=\"openEntryDetail(42)\""),
            "Entry item must trigger openEntryDetail(id)"
        );
        assert!(
            !html.contains("document.getElementById('detail-panel').classList.remove('hidden')"),
            "Entry item must not use inline classList toggle"
        );
    }

    #[test]
    fn test_entry_list_wires_open_entry_detail() {
        let entry = EntryViewModel {
            id: 42,
            title: "Test Entry in List".to_string(),
            suggested_title: None,
            feed_title: "Test Feed".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 1,
            summary: "Summary".to_string(),
            url: "https://example.com/item".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(
            html.contains("onclick=\"openEntryDetail(42)\""),
            "Entry list items must trigger openEntryDetail(id)"
        );
        assert!(
            !html.contains("document.getElementById('detail-panel').classList.remove('hidden')"),
            "Entry list must not use inline classList toggle"
        );
    }

    #[test]
    fn test_entry_detail_wires_close_detail_panel() {
        let entry = EntryViewModel {
            id: 105,
            title: "Test Entry Detail".to_string(),
            suggested_title: None,
            feed_title: "Test Feed".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 3,
            summary: "Summary".to_string(),
            url: "https://example.com/detail".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template = EntryDetailTemplate {
            entry,
            summary: "Summary".to_string(),
            auth_enabled: true,
            rewrite: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };
        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate");
        assert!(
            html.contains("onclick=\"closeDetailPanel()\""),
            "Detail back button must trigger closeDetailPanel()"
        );
        assert!(
            !html.contains("document.getElementById('detail-panel').classList.add('hidden')"),
            "Detail back button must not use inline classList toggle"
        );
    }

    #[test]
    fn test_index_template_defines_detail_panel_helpers() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(
            html.contains("function openEntryDetail("),
            "Index template must define openEntryDetail helper function"
        );
        assert!(
            html.contains("function openDetailWithSkeleton(")
                || html.contains("function openDetailWithSkeleton"),
            "Index template must define openDetailWithSkeleton helper function"
        );
        assert!(
            html.contains("function closeDetailPanel()"),
            "Index template must define closeDetailPanel helper function"
        );
        assert!(
            html.contains("cached_articles"),
            "Index template must reference cached_articles store for 30-day detail cache"
        );
        assert!(
            html.contains("ARTICLE_CACHE_TTL_MS"),
            "Index template must define 30-day ARTICLE_CACHE_TTL_MS"
        );
    }

    #[test]
    fn test_index_template_skeleton_structure() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(
            html.contains("animate-pulse"),
            "Skeleton placeholder must include animate-pulse"
        );
        assert!(
            html.contains("onclick=\"closeDetailPanel()\""),
            "Skeleton header must include interactive Back button wired to closeDetailPanel()"
        );
        assert!(
            html.contains("bg-muted") || html.contains("bg-primary"),
            "Skeleton should reference Annuaire token styling"
        );
    }

    #[test]
    fn test_magazine_grid_css_in_base_template() {
        let template = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = template.render().expect("render index template");
        assert!(
            html.contains(".magazine-3col-grid"),
            "base.html must include .magazine-3col-grid in style block"
        );
        assert!(
            html.contains("magazine-container"),
            "base.html must include magazine-container"
        );
        assert!(
            html.contains("minmax(220px, 1fr) minmax(380px, 2.5fr) minmax(220px, 1.1fr)"),
            "base.html must define 3-column grid template"
        );
        assert!(
            html.contains(".magazine-p2-grid"),
            "base.html must include .magazine-p2-grid"
        );
        assert!(
            html.contains("repeat(3, minmax(0, 1fr))"),
            "base.html must define 3-column grid for priority 2: {}",
            html
        );
    }

    #[test]
    fn test_annuaire_theme_tokens_and_toggle_present() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // Google Fonts for Big Shoulders Display and Work Sans
        assert!(
            html.contains("family=Big+Shoulders+Display"),
            "Must load Big Shoulders Display font"
        );
        assert!(
            html.contains("family=Work+Sans"),
            "Must load Work Sans font"
        );

        // Annuaire OKLCH color tokens in :root and dark theme
        assert!(
            html.contains("--brand: oklch(0.569 0.213 29)"),
            "Must define --brand positional red token"
        );
        assert!(
            html.contains("--primary: oklch(0.21 0.008 274.5)"),
            "Must define light --primary token"
        );
        assert!(
            html.contains("[data-theme=\"dark\"]"),
            "Must include dark theme token overrides"
        );

        // Essential utility classes
        assert!(
            html.contains(".font-display"),
            "Must define .font-display utility"
        );
        assert!(
            html.contains(".font-sans"),
            "Must define .font-sans utility"
        );
        assert!(html.contains(".tabular"), "Must define .tabular utility");
        assert!(html.contains(".leader"), "Must define .leader utility");
        assert!(
            html.contains(".shadow-float"),
            "Must define .shadow-float utility"
        );

        // Theme switcher inline script & toggle function
        assert!(
            html.contains("function toggleTheme()"),
            "Must define toggleTheme() function"
        );
        assert!(
            html.contains("data-theme"),
            "Must toggle data-theme attribute"
        );
        assert!(
            html.contains("localStorage"),
            "Must persist theme in localStorage"
        );
    }

    #[test]
    fn test_login_page_renders_centered_card() {
        let template = LoginTemplate;
        let html = template.render().expect("Failed to render LoginTemplate");

        // Centered Swiss card layout
        assert!(
            html.contains("min-h-screen flex items-center justify-center"),
            "Must use centered layout"
        );
        assert!(
            html.contains("max-w-sm rounded-lg border border-border bg-card"),
            "Must use Swiss card container"
        );

        // Wordmark and badge
        assert!(
            html.contains("Editorial Directory"),
            "Must contain Editorial Directory badge"
        );
        assert!(
            html.contains("font-display") && html.contains("MY NEWS"),
            "Wordmark must use font-display and MY NEWS"
        );

        // Pocket-ID ink pill button
        assert!(
            html.contains("Sign in with Pocket-ID"),
            "Must contain Sign in with Pocket-ID"
        );
        assert!(
            html.contains("bg-primary text-primary-foreground") && html.contains("rounded-full"),
            "Login button must be an ink pill"
        );
    }

    #[test]
    fn test_index_template_mark_page_read_button_auth_toggle() {
        let template_auth = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html_auth = template_auth
            .render()
            .expect("Failed to render IndexTemplate with auth_enabled: true");
        assert!(
            html_auth.contains("hx-post=\"/hx/mark-page-read\""),
            "Authenticated mode must include hx-post on mark-page-read button"
        );
        assert!(
            !html_auth.contains("markVisiblePageLocallyRead()"),
            "Authenticated mode must not use client-side markVisiblePageLocallyRead"
        );

        let template_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html_guest = template_guest
            .render()
            .expect("Failed to render IndexTemplate with auth_enabled: false");
        assert!(
            !html_guest.contains("hx-post=\"/hx/mark-page-read\""),
            "Guest mode must not include hx-post on mark-page-read button"
        );
        assert!(
            html_guest.contains("onclick=\"markVisiblePageLocallyRead()\""),
            "Guest mode must use onclick=\"markVisiblePageLocallyRead()\" on mark-page-read button"
        );
        assert!(
            html_guest.contains("function markVisiblePageLocallyRead()"),
            "Guest mode must define markVisiblePageLocallyRead() function"
        );
        assert!(
            html_guest.contains("function toggleLocalReadStatus("),
            "Guest mode must define toggleLocalReadStatus() function"
        );
        assert!(
            html_guest.contains("function triggerNativeBookmark("),
            "Guest mode must define triggerNativeBookmark() function"
        );
    }

    #[test]
    fn test_index_template_loading_indicator_and_filter_cache() {
        let template = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // HTMX loading indicator
        assert!(
            html.contains("id=\"entries-loading-bar\""),
            "Must render entries loading bar"
        );
        assert!(
            html.contains("class=\"htmx-indicator h-0.5 bg-brand animate-pulse"),
            "Loading bar must have htmx-indicator and pulse classes"
        );
        assert!(
            html.contains(".htmx-indicator"),
            "Base styles must include .htmx-indicator"
        );
        assert!(
            html.contains("hx-indicator=\"#entries-loading-bar\""),
            "Filter pills and search must declare hx-indicator"
        );

        // Client-side filter cache
        assert!(
            html.contains("const FILTER_CACHE_TTL_MS = 60000;"),
            "Must declare FILTER_CACHE_TTL_MS constant"
        );
        assert!(
            html.contains("const filterCache = new Map();"),
            "Must declare filterCache Map"
        );
        assert!(
            html.contains("function handleFilterClick(btn, event)"),
            "Must declare handleFilterClick function"
        );
        assert!(
            html.contains("function invalidateFilterCache()"),
            "Must declare invalidateFilterCache function"
        );
        assert!(
            html.contains("onclick=\"handleFilterClick(this, event)\"")
                || html.contains("handleFilterClick(this, event)"),
            "Filter pills must wire handleFilterClick"
        );
    }

    #[test]
    fn test_index_template_hides_starred_but_shows_all_titles_when_auth_disabled() {
        let template_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html_guest = template_guest
            .render()
            .expect("Failed to render IndexTemplate with auth_enabled: false");
        assert!(
            !html_guest.contains("id=\"filter-starred\""),
            "Guest mode must not render filter-starred"
        );
        assert!(
            html_guest.contains("id=\"filter-all\""),
            "Guest mode must render filter-all (All)"
        );
        assert!(
            html_guest.contains("id=\"filter-unread\""),
            "Guest mode must render filter-unread"
        );
        assert!(
            html_guest.contains("id=\"filter-today\""),
            "Guest mode must render filter-today"
        );

        let template_auth = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html_auth = template_auth
            .render()
            .expect("Failed to render IndexTemplate with auth_enabled: true");
        assert!(
            html_auth.contains("id=\"filter-starred\""),
            "Authenticated mode must render filter-starred"
        );
        assert!(
            html_auth.contains("id=\"filter-all\""),
            "Authenticated mode must render filter-all"
        );
    }

    #[test]
    fn test_open_in_browser_link_wires_mark_article_read() {
        let entry = EntryViewModel {
            id: 42,
            title: "Test Article".to_string(),
            suggested_title: None,
            feed_title: "Test Feed".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 3,
            summary: "Summary".to_string(),
            url: "https://example.com/item".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        // 1. EntryItemTemplate
        let item_template = EntryItemTemplate {
            entry: entry.clone(),
            unread_count: None,
        };
        let item_html = item_template
            .render()
            .expect("Failed to render EntryItemTemplate");
        assert!(
            item_html.contains("onclick=\"event.stopPropagation(); markArticleRead(42)\""),
            "Entry item external link must call markArticleRead(42)"
        );

        // 2. EntryListTemplate
        let list_template = EntryListTemplate {
            entries: vec![entry.clone()],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let list_html = list_template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(
            list_html.contains("onclick=\"event.stopPropagation(); markArticleRead(42)\""),
            "Entry list external link must call markArticleRead(42)"
        );

        // 3. EntryDetailTemplate
        let detail_template = EntryDetailTemplate {
            entry,
            summary: "Summary".to_string(),
            auth_enabled: false,
            rewrite: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };
        let detail_html = detail_template
            .render()
            .expect("Failed to render EntryDetailTemplate");
        assert!(
            detail_html.contains("markArticleRead(42)"),
            "Entry detail 'Original' link must call markArticleRead(42)"
        );
    }

    #[test]
    fn test_index_template_defines_all_articles_store_and_helpers() {
        let template = IndexTemplate {
            unread_count: 7,
            today_count: 3,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        assert!(
            html.contains("all_articles"),
            "Index template must define all_articles IndexedDB object store"
        );
        assert!(
            html.contains("function renderLocalAllArticles"),
            "Index template must define renderLocalAllArticles helper"
        );
        assert!(
            html.contains("function cacheLoadedArticlesFromDom"),
            "Index template must define cacheLoadedArticlesFromDom helper"
        );
        assert!(
            html.contains("id=\"filter-unread-count\"") && html.contains("data-base-unread=\"7\""),
            "filter-unread-count must include data-base-unread attribute"
        );
    }

    #[test]
    fn test_index_template_defines_mark_article_read_and_data_base_unread() {
        let template = IndexTemplate {
            unread_count: 7,
            today_count: 3,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        assert!(
            html.contains("function markArticleRead("),
            "Index template must define markArticleRead function in script block"
        );
        assert!(
            html.contains("id=\"filter-unread-count\"") && html.contains("data-base-unread=\"7\""),
            "filter-unread-count must include data-base-unread attribute"
        );
        assert!(
            html.contains("baseServerUnread"),
            "syncDomReadStates must reference baseServerUnread"
        );
        assert!(
            html.contains("readSet.size"),
            "syncDomReadStates must reference readSet.size"
        );

        let item_template = EntryItemTemplate {
            entry: EntryViewModel {
                id: 1,
                title: "T".into(),
                suggested_title: None,
                feed_title: "F".into(),
                published_at: "now".into(),
                reading_time: 1,
                summary: "S".into(),
                url: "https://example.com".into(),
                author: None,
                is_read: false,
                category_title: None,
                image_url: None,
                sanitized_content: None,
                is_bookmarked: false,
                rewrite: None,
                body_images: vec![],
            },
            unread_count: Some(5),
        };
        let item_html = item_template.render().expect("render item");
        assert!(
            item_html.contains("data-base-unread=\"5\""),
            "EntryItem unread pill must include data-base-unread"
        );

        let list_template = EntryListTemplate {
            entries: vec![],
            top_executive_brief: None,
            unread_count: Some(5),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let list_html = list_template.render().expect("render list");
        assert!(
            list_html.contains("data-base-unread=\"5\""),
            "EntryList unread count must include data-base-unread"
        );
    }

    #[test]
    fn test_filter_click_closes_detail_panel() {
        let template = IndexTemplate {
            unread_count: 10,
            today_count: 3,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        let pos_handle = html
            .find("function handleFilterClick")
            .expect("handleFilterClick must exist");
        let handle_fn_slice = &html[pos_handle..pos_handle + 300];
        assert!(
            handle_fn_slice.contains("closeDetailPanel()"),
            "handleFilterClick must call closeDetailPanel() to return to list view"
        );
    }

    #[test]
    fn test_entry_list_hides_pagination_when_no_news() {
        let template = EntryListTemplate {
            entries: vec![],
            top_executive_brief: None,
            unread_count: Some(0),
            current_page: 1,
            total_pages: 5,
            has_prev: false,
            has_next: true,
            prev_offset: 0,
            next_offset: 50,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(!html.contains("aria-label=\"Pagination\""));
        assert!(!html.contains("Page 1 of 5"));
        assert!(html.contains("All caught up!"));
    }

    #[test]
    fn test_index_template_omits_title_unread_counting() {
        let template = IndexTemplate {
            unread_count: 7,
            today_count: 3,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(html.contains("My News"));
        assert!(!html.contains("7 unread"));
    }

    #[test]
    fn test_htmx_indicator_hidden_by_default() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(html.contains(".htmx-indicator {\n      display: none;"));
        assert!(html.contains(".htmx-request.htmx-indicator,\n    .htmx-request .htmx-indicator {\n      display: block;"));
    }

    #[test]
    fn test_spatial_shell_and_window_mobile_container_queries() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // Body must define container-type: inline-size
        assert!(
            html.contains("body {\n      container-type: inline-size;")
                || html.contains("body { container-type: inline-size; }"),
            "body must establish container-type: inline-size"
        );

        // Container query block for mobile
        assert!(
            html.contains("@container (max-width: 768px)"),
            "Must define @container (max-width: 768px) query"
        );

        // Mobile overrides within container query
        let cq_index = html
            .find("@container (max-width: 768px)")
            .expect("Missing mobile container query");
        let cq_slice = &html[cq_index..];

        // spatial-shell mobile overrides
        assert!(
            cq_slice.contains("padding: 0;"),
            "Mobile spatial-shell must have padding: 0"
        );
        assert!(
            cq_slice.contains("min-height: 100vh;") && cq_slice.contains("min-height: 100dvh;"),
            "Mobile spatial-shell must declare min-height: 100vh with 100dvh fallback"
        );

        // spatial-window mobile overrides
        assert!(
            cq_slice.contains("width: 100%;"),
            "Mobile spatial-window must have width: 100%"
        );
        assert!(
            cq_slice.contains("max-width: 100%;"),
            "Mobile spatial-window must have max-width: 100%"
        );
        assert!(
            cq_slice.contains("height: 100vh;") && cq_slice.contains("height: 100dvh;"),
            "Mobile spatial-window must declare height: 100vh with 100dvh fallback"
        );
        assert!(
            cq_slice.contains("border-radius: 0;"),
            "Mobile spatial-window must have border-radius: 0"
        );
        assert!(
            cq_slice.contains("border: none;"),
            "Mobile spatial-window must have border: none"
        );
    }

    #[test]
    fn test_spatial_window_desktop_card_preserved() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        assert!(
            html.contains("padding: clamp(0.75rem, 3vw, 2.5rem);"),
            "Desktop spatial-shell must preserve clamp padding"
        );
        assert!(
            html.contains("max-width: min(100cqi, 1440px);"),
            "Desktop spatial-window must preserve fluid max-width"
        );
        assert!(
            html.contains("height: min(90vh, 840px);"),
            "Desktop spatial-window must preserve bounded height"
        );
        assert!(
            html.contains("border-radius: var(--radius);"),
            "Desktop spatial-window must preserve rounded borders"
        );
        assert!(
            html.contains("border: 1px solid var(--border);"),
            "Desktop spatial-window must preserve border styling"
        );
    }

    #[test]
    fn test_mobile_search_toggle_markup_and_container_css() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // Verify mobile search toggle button and JS helper
        assert!(
            html.contains("id=\"mobile-search-toggle-btn\"")
                && html.contains("onclick=\"toggleMobileSearch()\""),
            "IndexTemplate must render #mobile-search-toggle-btn calling toggleMobileSearch()"
        );
        assert!(
            html.contains("function toggleMobileSearch()"),
            "IndexTemplate must define toggleMobileSearch()"
        );

        // Verify mobile search toggle CSS inside container query
        let cq_index = html
            .find("@container (max-width: 768px)")
            .expect("Missing mobile container query");
        let cq_slice = &html[cq_index..];

        assert!(
            cq_slice.contains("#filter-search-bar") && cq_slice.contains("position: relative"),
            "Mobile container query must position #filter-search-bar relatively for overlay search"
        );
        assert!(
            cq_slice.contains("#mobile-search-toggle-btn")
                && cq_slice.contains("display: inline-flex")
                && cq_slice.contains("z-index: 20"),
            "Mobile container query must display #mobile-search-toggle-btn above overlay"
        );
        assert!(
            cq_slice.contains("#search-form") && cq_slice.contains("display: none"),
            "Mobile container query must hide #search-form by default"
        );
        assert!(
            cq_slice.contains("#filter-search-bar[data-mobile-search-open=\"true\"] .filter-pills-row")
                && cq_slice.contains("visibility: hidden"),
            "Mobile container query must keep filter pills in-flow (visibility: hidden) so they are not pushed down"
        );
        assert!(
            cq_slice.contains("#filter-search-bar[data-mobile-search-open=\"true\"] #search-form")
                && cq_slice.contains("display: flex")
                && cq_slice.contains("position: absolute"),
            "Mobile container query must overlay #search-form on top of filter pills when data-mobile-search-open=\"true\""
        );
        assert!(
            cq_slice.contains("#search-input") && cq_slice.contains("width: 100%"),
            "Mobile #search-input must span 100% width when open"
        );

        // Verify HTML filter bar layout prevents cut off
        assert!(
            html.contains("overflow-x-auto") && html.contains("shrink"),
            "Filter pills container must be scrollable and shrinkable on mobile"
        );
        assert!(
            html.contains("id=\"search-form\"") && html.contains("shrink-0"),
            "Search form must have shrink-0 to prevent being squished"
        );
    }

    #[test]
    fn test_mobile_detail_body_removes_padding_right_container_query() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        let cq_index = html
            .find("@container (max-width: 768px)")
            .expect("Missing mobile container query");
        let cq_slice = &html[cq_index..];

        assert!(
            cq_slice.contains("#detail-panel .detail-scrollbar")
                || cq_slice.contains(".detail-body"),
            "Must define mobile override removing padding-right for detail body inside container query"
        );
        assert!(
            cq_slice.contains("padding-right: 0;"),
            "Must set padding-right: 0 on mobile detail body"
        );
    }

    #[test]
    fn test_mobile_entries_list_and_items_reduce_horizontal_padding_container_query() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        let cq_index = html
            .find("@container (max-width: 768px)")
            .expect("Missing mobile container query");
        let cq_slice = &html[cq_index..];

        assert!(
            cq_slice.contains("#entries-list"),
            "Must define mobile padding override for #entries-list inside container query"
        );
        assert!(
            cq_slice.contains(".annuaire-row") || cq_slice.contains(".ar-list-item"),
            "Must define mobile padding override for list items inside container query"
        );
        assert!(
            cq_slice.contains("#detail-panel"),
            "Must define mobile padding override for #detail-panel inside container query"
        );
        assert!(
            cq_slice.contains("padding-left: 0.5rem;")
                && cq_slice.contains("padding-right: 0.5rem;"),
            "Must reduce left/right padding on mobile entries-list to maximize content area"
        );
    }

    #[test]
    fn test_entry_item_and_list_tight_spacing_between_bar_and_title() {
        let entry = EntryViewModel {
            id: 99,
            title: "Compact Spacing Article".to_string(),
            suggested_title: None,
            feed_title: "Tech".to_string(),
            published_at: "5m ago".to_string(),
            reading_time: 1,
            summary: "Summary".to_string(),
            url: "https://example.com/99".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let item_template = EntryItemTemplate {
            entry: entry.clone(),
            unread_count: None,
        };
        let item_html = item_template
            .render()
            .expect("Failed to render EntryItemTemplate");

        // The outer row and inner title wrapper both use compact gap-1.5 (6px)
        assert!(
            item_html.contains("annuaire-row") && item_html.contains("gap-1.5"),
            "Entry item row must use gap-1.5 to reduce space between red bar and title"
        );
        assert!(
            item_html.contains("entry-99-dot") && item_html.contains("empty:hidden"),
            "Dot container must have empty:hidden so read articles collapse phantom space"
        );

        let list_template = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 20,
            status: "all".to_string(),
            search: "".to_string(),
            limit: 20,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let list_html = list_template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(
            list_html.contains("gap-1.5"),
            "Entry list rows must use gap-1.5 to tighten spacing between red bar and title"
        );
    }

    #[test]
    fn test_executive_brief_skeleton_and_card_templates() {
        let skeleton = ExecutiveBriefSkeletonTemplate {
            offset: 0,
            limit: 15,
            status: "unread".to_string(),
            feed_id: Some(12),
            category_id: Some(3),
        };
        let sk_html = skeleton.render().expect("render skeleton");
        assert!(sk_html.contains("id=\"page-executive-brief\""));
        assert!(sk_html.contains("animate-pulse"));
        assert!(sk_html.contains("hx-get=\"/hx/executive-brief?offset=0&amp;limit=15&amp;status=unread&amp;feed_id=12&amp;category_id=3\"")
            || sk_html.contains("hx-get=\"/hx/executive-brief?offset=0&limit=15&status=unread&feed_id=12&category_id=3\""));
        assert!(sk_html.contains("hx-trigger=\"load\""));
        assert!(sk_html.contains("hx-swap=\"outerHTML\""));
        assert!(sk_html.contains("Generating Executive Brief..."));

        let card = ExecutiveBriefCardTemplate {
            brief: "Synthesis of page content.",
        };
        let card_html = card.render().expect("render card");
        assert!(card_html.contains("id=\"page-executive-brief\""));
        assert!(card_html.contains("Synthesis of page content."));
        assert!(card_html.contains("Executive Brief"));
    }

    #[test]
    fn test_entry_list_renders_skeleton_when_eligible_and_not_empty() {
        let entry = EntryViewModel {
            id: 1,
            title: "Test".into(),
            suggested_title: None,
            feed_title: "Feed".into(),
            published_at: "now".into(),
            reading_time: 1,
            summary: "S".into(),
            url: "https://example.com".into(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 15,
            is_brief_eligible: true,
            offset: 0,
            feed_id: Some(42),
            category_id: None,
            auth_enabled: false,
        };
        let html = template.render().expect("render");
        assert!(html.contains("id=\"page-executive-brief\""));
        assert!(html.contains("animate-pulse"));
        assert!(html.contains("Generating Executive Brief..."));
        assert!(
            html.contains("id=\"entry-title-1\""),
            "Must include entry-title span"
        );
    }

    #[test]
    fn test_entry_list_guards_skeleton_when_empty() {
        let template = EntryListTemplate {
            entries: vec![],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 15,
            is_brief_eligible: true,
            offset: 0,
            feed_id: Some(42),
            category_id: None,
            auth_enabled: false,
        };
        let html = template.render().expect("render");
        assert!(
            !html.contains("id=\"page-executive-brief\""),
            "Empty entries list must not render skeleton"
        );
        assert!(!html.contains("animate-pulse"));
        assert!(html.contains("All caught up!"));
    }

    #[test]
    fn test_annuaire_row_click_target() {
        let entry = EntryViewModel {
            id: 42,
            title: "Row Click Test".into(),
            suggested_title: None,
            feed_title: "Tech".into(),
            published_at: "now".into(),
            reading_time: 2,
            summary: "Summary text".into(),
            url: "https://example.com/42".into(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        // 1. Check EntryItemTemplate
        let item_tpl = EntryItemTemplate {
            entry: entry.clone(),
            unread_count: None,
        };
        let item_html = item_tpl.render().expect("render entry item");
        assert!(
            item_html.contains(r#"class="annuaire-row"#)
                && item_html.contains(r#"onclick="openEntryDetail(42)""#),
            "Outer annuaire-row must have onclick handler: {}",
            item_html
        );
        // Ensure inner title container does not also have onclick
        assert!(
            !item_html.contains(r#"<div class="flex items-center gap-1.5 flex-1 min-w-0" onclick="openEntryDetail(42)">"#),
            "Inner title container must not have onclick handler"
        );
        // Ensure external link has stopPropagation
        assert!(
            item_html.contains("event.stopPropagation()"),
            "External link button must retain stopPropagation"
        );

        // 2. Check EntryListTemplate
        let list_tpl = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: None,
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 15,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let list_html = list_tpl.render().expect("render entry list");
        assert!(
            list_html.contains(r#"<div class="annuaire-row"#)
                && list_html.contains(r#"onclick="openEntryDetail(42)""#),
            "Outer annuaire-row in list must have onclick handler: {}",
            list_html
        );
        assert!(
            !list_html.contains(r#"<div class="flex items-center gap-1.5 flex-1 min-w-0" onclick="openEntryDetail(42)">"#),
            "Inner title container in list must not have onclick handler"
        );
    }

    #[test]
    fn test_pagination_nav_no_top_border() {
        let entry = EntryViewModel {
            id: 1,
            title: "Test".into(),
            suggested_title: None,
            feed_title: "Feed".into(),
            published_at: "now".into(),
            reading_time: 1,
            summary: "S".into(),
            url: "https://example.com".into(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let list_tpl = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: None,
            current_page: 2,
            total_pages: 5,
            has_prev: true,
            has_next: true,
            prev_offset: 0,
            next_offset: 30,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 15,
            is_brief_eligible: false,
            offset: 15,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = list_tpl
            .render()
            .expect("render entry list with pagination");
        assert!(
            html.contains(r#"aria-label="Pagination""#),
            "Must contain pagination nav"
        );
        assert!(
            !html.contains("border-t border-border"),
            "Pagination nav must not have border-t border-border: {}",
            html
        );
    }

    #[test]
    fn test_render_entry_list_with_static_top_executive_brief() {
        let entry = EntryViewModel {
            id: 1,
            title: "Test".into(),
            suggested_title: None,
            feed_title: "Feed".into(),
            published_at: "now".into(),
            reading_time: 1,
            summary: "S".into(),
            url: "https://example.com".into(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let list_tpl = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: Some("Pre-existing Executive Brief Summary".to_string()),
            unread_count: Some(5),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 15,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let html = list_tpl
            .render()
            .expect("render entry list with static brief");
        assert!(
            html.contains("Pre-existing Executive Brief Summary"),
            "Must contain static executive brief: {}",
            html
        );
        assert!(
            html.contains("Executive Brief"),
            "Must contain Executive Brief header: {}",
            html
        );
        assert!(
            html.contains("id=\"list-executive-brief\""),
            "Must contain list-executive-brief for list view: {}",
            html
        );
        assert!(
            html.contains("id=\"page-executive-brief\""),
            "Must contain page-executive-brief for magazine view: {}",
            html
        );
        assert!(
            !html.contains("hx-trigger=\"load\""),
            "Must not contain hx-trigger=load: {}",
            html
        );
        assert!(
            !html.contains("Generating Executive Brief..."),
            "Must not contain loading skeleton: {}",
            html
        );
    }

    #[test]
    fn test_index_template_brief_caching_and_throttling_scripts() {
        let template = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        assert!(
            html.contains("readSinceLastBriefSet = new Set()"),
            "Must define readSinceLastBriefSet"
        );
        assert!(
            html.contains("htmx.process(entriesList)"),
            "Must call htmx.process in applyCachedList"
        );
        assert!(
            html.contains("request_brief"),
            "Must configure request_brief parameter"
        );
        assert!(
            html.contains("effectiveUnread = visibleUnreadRowCount"),
            "Must fallback effectiveUnread in syncDomReadStates"
        );
        assert!(
            html.contains("isBriefSwap = targetId === 'page-executive-brief'"),
            "Must handle brief swap in htmx:afterSwap"
        );
        assert!(
            html.contains("isErrorCard"),
            "Must check for isErrorCard in brief afterSwap handler"
        );
        assert!(
            html.contains("Executive Brief Unavailable"),
            "Must render Executive Brief Unavailable into list-executive-brief on error"
        );
        assert!(
            html.contains("htmx.process(listBrief)"),
            "Must process HTMX on listBrief on error sync"
        );
    }

    #[test]
    fn test_entry_view_model_category_and_image_url() {
        let entry = EntryViewModel {
            id: 42,
            title: "Tech Article".into(),
            suggested_title: Some("Punchy Title".into()),
            feed_title: "TechCrunch".into(),
            category_title: Some("Technology".into()),
            published_at: "10 mins ago".into(),
            reading_time: 3,
            summary: "Executive brief content".into(),
            image_url: Some("https://example.com/photo.jpg".into()),
            url: "https://example.com/article".into(),
            author: None,
            is_read: false,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        assert_eq!(entry.category_title.as_deref(), Some("Technology"));
        assert_eq!(
            entry.image_url.as_deref(),
            Some("https://example.com/photo.jpg")
        );

        // Test serde default roundtrip when fields are missing
        let json_data = r#"{"id":42,"title":"T","feed_title":"F","published_at":"now","reading_time":1,"summary":"S","url":"U","is_read":false,"is_bookmarked":false}"#;
        let deserialized: EntryViewModel =
            serde_json::from_str(json_data).expect("deserialize with default category and image");
        assert_eq!(deserialized.category_title, None);
        assert_eq!(deserialized.image_url, None);
        assert_eq!(deserialized.rewrite, None);
        assert!(deserialized.body_images.is_empty());
    }

    #[test]
    fn test_entry_list_template_renders_magazine_view_and_cards() {
        let mut entries = Vec::new();
        // Entry 0: Priority 1 (Hero)
        entries.push(EntryViewModel {
            id: 101,
            title: "Hero Article Title".into(),
            suggested_title: Some("Hero Suggested Headline".into()),
            feed_title: "TechCrunch".into(),
            category_title: Some("Technology".into()),
            published_at: "10 mins ago".into(),
            reading_time: 3,
            summary: "AI summary brief for hero article".into(),
            image_url: Some("https://example.com/photo1.jpg".into()),
            url: "https://example.com/101".into(),
            author: None,
            is_read: false,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        });
        // Entries 1..=6: Priority 2 (6 featured cards in center column)
        for i in 1..=6 {
            entries.push(EntryViewModel {
                id: 101 + i,
                title: format!("Standard Article {}", i),
                suggested_title: None,
                feed_title: "BBC News".into(),
                category_title: Some("World".into()),
                published_at: "1 hour ago".into(),
                reading_time: 2,
                summary: format!("Brief for standard article {}", i),
                image_url: if i % 2 == 0 {
                    Some(format!("https://example.com/photo{}.jpg", i))
                } else {
                    None
                },
                url: format!("https://example.com/{}", 101 + i),
                author: None,
                is_read: i == 1, // entry 102 is read
                sanitized_content: None,
                is_bookmarked: false,
                rewrite: None,
                body_images: vec![],
            });
        }
        // Entry 7: Priority 3 (Card in right column)
        entries.push(EntryViewModel {
            id: 108,
            title: "Compact Bottom Article".into(),
            suggested_title: None,
            feed_title: "Hacker News".into(),
            category_title: Some("Tech".into()),
            published_at: "2 hours ago".into(),
            reading_time: 1,
            summary: "Summary for Priority 3".into(),
            image_url: None,
            url: "https://example.com/108".into(),
            author: None,
            is_read: false,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        });

        let list_tpl = EntryListTemplate {
            entries,
            top_executive_brief: Some("Executive brief for the page".to_string()),
            unread_count: Some(1),
            current_page: 1,
            total_pages: 3,
            has_prev: false,
            has_next: true,
            prev_offset: 0,
            next_offset: 50,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };

        let html = list_tpl.render().expect("render entry list template");

        // Executive brief as 1 full-width row
        assert!(
            html.contains("id=\"page-executive-brief\""),
            "Must contain executive brief row: {}",
            html
        );
        assert!(
            html.contains("Executive brief for the page"),
            "Must render executive brief text: {}",
            html
        );

        // Dual views containers
        assert!(
            html.contains("id=\"entries-list-view\""),
            "Must contain list view container: {}",
            html
        );
        assert!(
            html.contains("id=\"entries-magazine-view\""),
            "Must contain magazine view container: {}",
            html
        );

        let mag_view = html
            .split("id=\"entries-magazine-view\"")
            .nth(1)
            .unwrap()
            .split("aria-label=\"Pagination\"")
            .next()
            .unwrap();

        // Assert 3-column editorial magazine layout
        assert!(
            mag_view.contains("id=\"mag-col-brief\""),
            "Must contain left column #mag-col-brief: {}",
            mag_view
        );
        assert!(
            mag_view.contains("id=\"mag-col-center\""),
            "Must contain center column #mag-col-center: {}",
            mag_view
        );
        assert!(
            mag_view.contains("id=\"mag-col-right\""),
            "Must contain right column #mag-col-right: {}",
            mag_view
        );

        // Left column houses Executive Brief
        let brief_col = mag_view
            .split("id=\"mag-col-brief\"")
            .nth(1)
            .unwrap()
            .split("id=\"mag-col-center\"")
            .next()
            .unwrap();
        assert!(
            brief_col.contains("id=\"page-executive-brief\""),
            "Left column must contain #page-executive-brief: {}",
            brief_col
        );
        assert!(
            brief_col.contains("Executive brief for the page"),
            "Left column must render executive brief text: {}",
            brief_col
        );

        // Center column houses Priority 1 and Priority 2
        let center_col = mag_view
            .split("id=\"mag-col-center\"")
            .nth(1)
            .unwrap()
            .split("id=\"mag-col-right\"")
            .next()
            .unwrap();
        assert!(
            center_col.contains("priority-1"),
            "Center column must contain priority-1: {}",
            center_col
        );
        assert!(
            center_col.contains("priority-2"),
            "Center column must contain priority-2: {}",
            center_col
        );
        assert!(
            !center_col.contains("priority-3"),
            "Center column must NOT contain priority-3: {}",
            center_col
        );

        // Right column houses Priority 3
        let right_col = mag_view.split("id=\"mag-col-right\"").nth(1).unwrap();
        assert!(
            right_col.contains("priority-3"),
            "Right column must contain priority-3: {}",
            right_col
        );
        assert!(
            !right_col.contains("priority-1"),
            "Right column must NOT contain priority-1: {}",
            right_col
        );
        assert!(
            !right_col.contains("priority-2"),
            "Right column must NOT contain priority-2: {}",
            right_col
        );

        // Rule: Remove news badge!
        assert!(
            !mag_view.contains("magazine-topic-badge"),
            "Magazine view must NOT contain news badge: {}",
            mag_view
        );
        assert!(
            !mag_view.contains("uppercase tracking-wider text-brand mb-1"),
            "Must NOT contain category badge in P2"
        );
        assert!(
            !mag_view.contains("uppercase tracking-wider text-muted-foreground mb-1"),
            "Must NOT contain category badge in P3"
        );

        // Priority 1 checks: Hero photo on left (50% width, 100% height self-stretch), title and brief on right
        assert!(
            mag_view.contains("src=\"https://example.com/photo1.jpg\""),
            "Hero must render photo: {}",
            mag_view
        );
        assert!(
            mag_view.contains("self-stretch"),
            "Hero photo container must be self-stretch: {}",
            mag_view
        );
        assert!(
            mag_view.contains("Hero Suggested Headline"),
            "Hero must render suggested title: {}",
            mag_view
        );
        assert!(
            mag_view.contains("AI summary brief for hero article"),
            "Hero must render brief: {}",
            mag_view
        );

        // Priority 2 checks: 6 featured cards with small photo + brief
        assert!(
            mag_view.contains("Brief for standard article 1"),
            "Priority 2 must render brief: {}",
            mag_view
        );
        assert!(
            mag_view.contains("Brief for standard article 6"),
            "Priority 2 must render all 6 items: {}",
            mag_view
        );

        // Priority 3 checks: Renders title in right column with normal font weight and tooltip brief
        assert!(
            mag_view.contains("Compact Bottom Article"),
            "Priority 3 must render title: {}",
            mag_view
        );
        assert!(
            mag_view.contains(r#"title="Summary for Priority 3""#),
            "Priority 3 title must have tooltip brief: {}",
            mag_view
        );
        assert!(
            !right_col.contains("magazine-brief"),
            "Priority 3 must NOT render visible brief paragraph: {}",
            right_col
        );
        assert!(
            mag_view.contains("font-normal"),
            "Priority 3 title must use normal text font weight: {}",
            mag_view
        );

        // Rule: Do NOT show source and date time for all 3 in magazine view!
        assert!(
            !mag_view.contains("10 mins ago"),
            "Magazine view must NOT show date time: {}",
            mag_view
        );
        assert!(
            !mag_view.contains("1 hour ago"),
            "Magazine view must NOT show date time: {}",
            mag_view
        );
        assert!(
            !mag_view.contains("2 hours ago"),
            "Magazine view must NOT show date time: {}",
            mag_view
        );
        // Note: data-feed-title attribute exists for hot topic tracking, but visible source text is omitted
        assert!(
            !mag_view.contains("<span class=\"truncate font-medium\">TechCrunch</span>"),
            "Must NOT show source text: {}",
            mag_view
        );

        // Titles and stretched-link click handler
        assert!(
            mag_view.contains(r#"id="mag-title-101""#)
                && mag_view.contains(r#"href="/entries/101""#)
                && mag_view.contains(r#"class="magazine-card-link""#)
                && mag_view.contains("openEntryFromMagazine(101, this.closest('.magazine-card')?.dataset.feedTitle || '')"),
            "Must wire stretched-link openEntryFromMagazine using dataset.feedTitle: {}",
            mag_view
        );

        // Read styling in magazine view: entry 102 is read (opacity-50 grayscale), entry 101 is unread (foreground)
        assert!(
            mag_view.contains("id=\"mag-title-101\""),
            "Must have mag-title-101 id"
        );
        assert!(
            mag_view.contains("id=\"mag-title-102\""),
            "Must have mag-title-102 id"
        );
        assert!(
            mag_view.contains("read opacity-50 grayscale"),
            "Read card must have opacity-50 grayscale: {}",
            mag_view
        );

        // Shared bottom pagination
        assert!(
            html.contains("aria-label=\"Pagination\""),
            "Must render pagination nav: {}",
            html
        );
        assert!(
            html.contains("Page 1 of 3"),
            "Must display current page: {}",
            html
        );
    }

    #[test]
    fn test_index_template_renders_ai_toggle_button_and_scripts() {
        let template = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = template
            .render()
            .expect("render index template with ai toggle");

        // Assert button presence and ordering after theme-toggle-btn
        let theme_pos = html
            .find("id=\"theme-toggle-btn\"")
            .expect("must have theme toggle button");
        let ai_btn_pos = html
            .find("id=\"ai-toggle-btn\"")
            .expect("must have ai toggle button");
        assert!(
            ai_btn_pos > theme_pos,
            "ai-toggle-btn must be positioned after theme-toggle-btn"
        );

        assert!(
            html.contains("onclick=\"toggleAiView()\""),
            "AI toggle button must call toggleAiView()"
        );
        assert!(
            html.contains("border-border bg-card hover:bg-muted text-foreground"),
            "Button must use neutral styling without active background: {}",
            html
        );
        assert!(
            !html.contains("border-primary bg-primary"),
            "Button must NOT use primary active background: {}",
            html
        );
        assert!(
            html.contains("id=\"view-icon-magazine\""),
            "Must contain magazine icon: {}",
            html
        );
        assert!(
            html.contains("id=\"view-icon-list\""),
            "Must contain list icon: {}",
            html
        );
        assert!(
            html.contains("toggleAiView"),
            "Must define toggleAiView in script"
        );
        assert!(
            html.contains("applyAiViewMode"),
            "Must define applyAiViewMode in script"
        );
        assert!(
            html.contains("recordSourceClick"),
            "Must define recordSourceClick in script"
        );
        assert!(
            html.contains("updateHotTopicHighlighting"),
            "Must define updateHotTopicHighlighting in script"
        );
        assert!(
            html.contains("openEntryFromMagazine"),
            "Must define openEntryFromMagazine in script"
        );
        assert!(
            html.contains("'my_news_view_mode'"),
            "Must persist view mode in localStorage"
        );
        assert!(
            html.contains("localStorage.setItem('my_news_view_mode', 'ai')"),
            "Must force 'ai' (Magazine) view on initial load"
        );
    }

    #[test]
    fn test_executive_brief_card_has_single_container() {
        let card = ExecutiveBriefCardTemplate {
            brief: "Unified executive summary text.",
        };
        let html = card.render().expect("render executive brief card");
        assert!(
            html.contains("id=\"page-executive-brief\""),
            "Must contain page-executive-brief"
        );
        assert!(
            !html.contains("id=\"list-executive-brief\""),
            "Must NOT contain duplicate list-executive-brief"
        );
        assert!(
            !html.contains("hx-swap-oob"),
            "Must NOT contain hx-swap-oob attribute"
        );
    }

    #[test]
    fn test_index_template_instant_detail_title_and_cache_validation() {
        let template = IndexTemplate {
            unread_count: 3,
            today_count: 1,
            entries_list_html: "<div>test entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = template.render().expect("render index template");

        // Verify cache validation does not require obsolete 'Executive Brief' string
        assert!(
            !html.contains("cached.html.includes('Executive Brief')"),
            "Must not require obsolete Executive Brief check in cache"
        );
        assert!(html.contains("isCacheValid"), "Must check isCacheValid");

        // Verify openEntryDetail extracts title and passes to openDetailWithSkeleton
        assert!(html.contains("entry-title-"), "Must look up entry title");
        assert!(
            html.contains("openDetailWithSkeleton"),
            "Must call openDetailWithSkeleton"
        );

        // Verify afterSwap synchronizes brief to list-executive-brief
        assert!(
            html.contains("list-executive-brief"),
            "Must reference list-executive-brief in sync script"
        );
    }

    #[test]
    fn test_base_template_toast_auto_dismiss_and_icon_init() {
        let index_tmpl = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = index_tmpl.render().expect("render index template");
        assert!(html.contains("htmx:oobAfterSwap"));
        assert!(html.contains("toast-container"));
        assert!(html.contains("5000"));
    }

    #[test]
    fn test_workspace_layout_and_empty_placeholder_rendered() {
        let index_tmpl = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div id=\"test-entry\">Item</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = index_tmpl.render().expect("render index template");

        // Workspace must use container query layout
        assert!(
            html.contains("workspace-layout"),
            "index.html must render .workspace-layout container"
        );
        assert!(
            html.contains("id=\"detail-empty-placeholder\""),
            "detail panel must contain #detail-empty-placeholder for desktop empty state"
        );
        assert!(
            html.contains("Select an article to read"),
            "placeholder must contain 'Select an article to read' text"
        );
    }

    #[test]
    fn test_workspace_layout_container_queries_and_ar_tokens_in_css() {
        let index_tmpl = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = index_tmpl.render().expect("render index template");

        // Container query workspace context
        assert!(
            html.contains("container-name: workspace")
                || html.contains("container: workspace / inline-size"),
            "base.html must define workspace container query context"
        );

        // Optical waveguide and gaze target tokens
        assert!(
            html.contains("rgba(18, 19, 22, 0.92)"),
            "base.html dark mode must provide waveguide-safe backdrop"
        );
        assert!(
            html.contains("min-height: 48px") || html.contains("min-h-[48px]"),
            "Must define 48px minimum gaze target height for ar-list-item"
        );
    }

    #[test]
    fn test_detail_panel_slide_in_animation_and_no_split_on_load() {
        let index_tmpl = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        };
        let html = index_tmpl.render().expect("render index template");

        // 1. Must define .animate-slide-in-right and keyframes
        assert!(
            html.contains("slideInFromRight") && html.contains("animate-slide-in-right"),
            "base.html must define slideInFromRight keyframes and .animate-slide-in-right"
        );

        // 2. Must not force desktop split on load
        assert!(
            !html.contains("minmax(20rem, 24rem) 1fr"),
            "base.html must not force two-column desktop split on initial load"
        );
        assert!(
            !html.contains(
                ".workspace-layout .detail-pane #detail-back-btn { display: none !important; }"
            ) && !html.contains("#detail-back-btn {\n        display: none !important;"),
            "detail back button must remain accessible on all screen sizes"
        );

        // 3. Wide desktop detail panel width equals Column 2 + Column 3
        assert!(
            html.contains("alignDetailPanelWidth") && html.contains("mag-col-center"),
            "index.html must dynamically align detail panel width with mag-col-center (col 2 + col 3)"
        );
        assert!(
            html.contains("(3.6 / 4.6)"),
            "base.html must define CSS fallback for detail panel matching col 2 + col 3 proportion"
        );
    }

    #[test]
    fn test_entry_list_no_hardcoded_viewport_media_queries() {
        let raw_entry_list = include_str!("../templates/partials/entry_list.html");
        assert!(
            !raw_entry_list.contains("lg:grid-cols"),
            "entry_list.html must not use lg:grid-cols viewport breakpoint"
        );
        assert!(
            !raw_entry_list.contains("sm:flex-row"),
            "entry_list.html must not use sm:flex-row viewport breakpoint"
        );
        assert!(
            !raw_entry_list.contains("sm:w-1/2"),
            "entry_list.html must not use sm:w-1/2 viewport breakpoint"
        );
        assert!(
            !raw_entry_list.contains("md:p-6"),
            "entry_list.html must not use md:p-6 viewport breakpoint"
        );
        assert!(
            !raw_entry_list.contains("md:text-xl"),
            "entry_list.html must not use md:text-xl viewport breakpoint"
        );
    }

    #[test]
    fn test_magazine_view_brief_line_clamping() {
        let raw_entry_list = include_str!("../templates/partials/entry_list.html");
        assert!(
            raw_entry_list.contains("line-clamp-6"),
            "entry_list.html Priority 1 card brief must use line-clamp-6"
        );
        assert!(
            raw_entry_list.contains("line-clamp-4 leading-relaxed font-sans flex-1"),
            "entry_list.html Priority 2 card brief must use line-clamp-4"
        );
    }

    #[test]
    fn test_entry_detail_template_renders_hero_image_with_overlaid_title() {
        let entry = EntryViewModel {
            id: 201,
            title: "Magazine Cover Story".to_string(),
            suggested_title: Some("Overlaid Punchy Headline".to_string()),
            feed_title: "Vogue Tech".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 3,
            summary: "Summary text.".to_string(),
            url: "https://example.com/201".to_string(),
            author: None,
            is_read: false,
            category_title: Some("Design".to_string()),
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Brief".to_string(),
            auth_enabled: false,
            rewrite: "Inverted pyramid rewrite content here.".to_string(),
            impact: "Impact text.".to_string(),
            fallacies: "Fallacies text.".to_string(),
            has_analysis: true,
            hero_image: Some("https://example.com/hero.jpg".to_string()),
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate with hero image");

        assert!(
            html.contains("bg-gradient-to-t"),
            "Hero image container must have bg-gradient-to-t scrim"
        );
        assert!(
            html.contains("relative z-10"),
            "Title and metadata overlay must have relative z-10"
        );
        assert!(
            html.contains("https://example.com/hero.jpg"),
            "Hero image src must be rendered"
        );
        assert!(
            html.contains("Overlaid Punchy Headline"),
            "Hero image overlay must render suggested title"
        );
        assert!(
            html.contains("text-white"),
            "Overlay text must use text-white for high contrast"
        );
    }

    #[test]
    fn test_entry_detail_template_renders_without_hero_image() {
        let entry = EntryViewModel {
            id: 202,
            title: "Clean Typography Story".to_string(),
            suggested_title: None,
            feed_title: "Ars Technica".to_string(),
            published_at: "30 mins ago".to_string(),
            reading_time: 2,
            summary: "Summary text.".to_string(),
            url: "https://example.com/202".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Brief".to_string(),
            auth_enabled: false,
            rewrite: "Inverted pyramid rewrite content here.".to_string(),
            impact: "Impact text.".to_string(),
            fallacies: "Fallacies text.".to_string(),
            has_analysis: true,
            hero_image: None,
            body_images: vec![],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate without hero image");

        assert!(
            !html.contains("bg-gradient-to-t"),
            "Detail view without hero must not render hero gradient scrim"
        );
        assert!(
            html.contains("text-foreground"),
            "Clean typography title must use text-foreground"
        );
        assert!(
            html.contains("Clean Typography Story"),
            "Clean typography title must render entry.title"
        );
    }

    #[test]
    fn test_entry_detail_template_renders_body_images() {
        let entry = EntryViewModel {
            id: 203,
            title: "Multi Photo Story".to_string(),
            suggested_title: None,
            feed_title: "National Geographic".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 5,
            summary: "Summary text.".to_string(),
            url: "https://example.com/203".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Brief".to_string(),
            auth_enabled: false,
            rewrite: "Rewrite content.".to_string(),
            impact: "Impact text.".to_string(),
            fallacies: "Fallacies text.".to_string(),
            has_analysis: true,
            hero_image: Some("https://example.com/hero.jpg".to_string()),
            body_images: vec![
                "https://example.com/photo1.jpg".to_string(),
                "https://example.com/photo2.jpg".to_string(),
            ],
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
        };

        let html = template
            .render()
            .expect("Failed to render EntryDetailTemplate with body images");

        assert!(
            html.contains("https://example.com/photo1.jpg"),
            "Body image 1 must be rendered in body gallery"
        );
        assert!(
            html.contains("https://example.com/photo2.jpg"),
            "Body image 2 must be rendered in body gallery"
        );
        assert!(
            html.contains("loading=\"lazy\""),
            "Body images must have lazy loading attribute"
        );
    }

    #[test]
    fn test_ai_error_card_template_rendering() {
        let card_with_id = AiErrorCardTemplate {
            title: "Executive Brief Unavailable",
            message: "This model is currently experiencing high demand.",
            retry_url: "/hx/executive-brief?status=unread",
            target: "#page-executive-brief",
            swap: "outerHTML",
            container_id: "page-executive-brief",
        };

        let html = card_with_id
            .render()
            .expect("Failed to render AiErrorCardTemplate with container_id");

        assert!(html.contains("id=\"page-executive-brief\""));
        assert!(html.contains("is-error"));
        assert!(html.contains("Executive Brief Unavailable"));
        assert!(html.contains("This model is currently experiencing high demand."));
        assert!(html.contains("hx-get=\"/hx/executive-brief?status=unread\""));
        assert!(html.contains("hx-target=\"#page-executive-brief\""));
        assert!(html.contains("hx-swap=\"outerHTML\""));
        assert!(html.contains("hx-disabled-elt=\"this\""));
        assert!(html.contains("data-lucide=\"alert-circle\""));
        assert!(html.contains("data-lucide=\"refresh-cw\""));
        assert!(html.contains("Retry"));

        let card_without_id = AiErrorCardTemplate {
            title: "AI Rewrite Unavailable",
            message: "Temporary error",
            retry_url: "/hx/entries/123/rewrite",
            target: "#ai-rewrite-box",
            swap: "outerHTML",
            container_id: "",
        };

        let html_no_id = card_without_id
            .render()
            .expect("Failed to render AiErrorCardTemplate without container_id");

        assert!(!html_no_id.contains("id=\"\""));
        assert!(!html_no_id.contains("id="));
        assert!(html_no_id.contains("is-error"));
        assert!(html_no_id.contains("AI Rewrite Unavailable"));
        assert!(html_no_id.contains("hx-get=\"/hx/entries/123/rewrite\""));
    }

    #[test]
    fn test_format_published_date_ddmmyyyy_parses_rfc3339_and_falls_back() {
        assert_eq!(
            format_published_date_ddmmyyyy("2026-11-23T12:00:00Z"),
            "23/11/2026"
        );
        assert_eq!(
            format_published_date_ddmmyyyy("2026-01-05T23:30:00-07:00"),
            "05/01/2026"
        );
        assert_eq!(
            format_published_date_ddmmyyyy("2026-03-09T04:15:30.123456Z"),
            "09/03/2026"
        );
        assert_eq!(format_published_date_ddmmyyyy("2026-07-04"), "04/07/2026");
        assert_eq!(format_published_date_ddmmyyyy(""), "");
        assert_eq!(format_published_date_ddmmyyyy("   "), "");
        assert_eq!(format_published_date_ddmmyyyy("2 hours ago"), "2 hours ago");
    }

    #[test]
    fn test_entry_detail_omits_ddmmyyyy_time_element_in_both_headers() {
        let make_entry = |published_at: &str| EntryViewModel {
            id: 301,
            title: "Date Format Test".to_string(),
            suggested_title: None,
            feed_title: "Daily Wire".to_string(),
            published_at: published_at.to_string(),
            reading_time: 3,
            summary: "Summary".to_string(),
            url: "https://example.com/301".to_string(),
            author: None,
            is_read: false,
            category_title: None,
            image_url: None,
            sanitized_content: None,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };

        // Hero header renders feed_title + <time data-relative-time="true"> with " · " while omitting data-date-format, author, and reading_time
        let hero_html = EntryDetailTemplate {
            entry: make_entry("2026-11-23T12:00:00Z"),
            summary: String::new(),
            auth_enabled: false,
            rewrite: "Rewrite.".to_string(),
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: Some("https://example.com/hero.jpg".to_string()),
            body_images: vec![],
        }
        .render()
        .expect("render hero detail template");
        assert!(hero_html.contains("<time"));
        assert!(hero_html.contains(r#"data-relative-time="true""#));
        assert!(hero_html.contains("Daily Wire"));
        assert!(hero_html.contains(" · "));
        assert!(hero_html.contains("text-white/85"));
        assert!(!hero_html.contains("data-date-format"));
        assert!(!hero_html.contains("min read"));

        // Standard header renders feed_title + <time data-relative-time="true"> with " · " while omitting data-date-format, author, and reading_time
        let standard_html = EntryDetailTemplate {
            entry: make_entry("2026-11-23T12:00:00Z"),
            summary: String::new(),
            auth_enabled: false,
            rewrite: "Rewrite.".to_string(),
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
        }
        .render()
        .expect("render standard detail template");
        assert!(standard_html.contains("<time"));
        assert!(standard_html.contains(r#"data-relative-time="true""#));
        assert!(standard_html.contains("Daily Wire"));
        assert!(standard_html.contains(" · "));
        assert!(!standard_html.contains("data-date-format"));
        assert!(!standard_html.contains("min read"));
    }

    #[test]
    fn test_index_template_omits_detail_date_formatter_and_guards_auth_cache() {
        let html = IndexTemplate {
            unread_count: 1,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("render IndexTemplate");

        assert!(html.contains("const AI_ENABLED = true;"));
        assert!(
            !html.contains("formatDetailDates"),
            "formatDetailDates must be completely removed from index.html"
        );

        // openDetailWithSkeleton must omit metadata skeleton bars and Executive Brief heading
        let skel_pos = html
            .find("function openDetailWithSkeleton(")
            .expect("openDetailWithSkeleton");
        let align_pos = html[skel_pos..]
            .find("function alignDetailPanelWidth(")
            .expect("alignDetailPanelWidth")
            + skel_pos;
        let skel_fn = &html[skel_pos..align_pos];
        assert!(!skel_fn.contains("Feed info & date"));
        assert!(!skel_fn.contains("Executive Brief"));

        // syncExecutiveBriefUpdates runs in isBriefSwap BEFORE filterCache.set
        assert!(html.contains("function syncExecutiveBriefUpdates("));
        let after_swap_pos = html
            .find("document.addEventListener('htmx:afterSwap'")
            .expect("htmx:afterSwap");
        let after_swap_slice = &html[after_swap_pos..];
        let sync_call_pos = after_swap_slice
            .find("syncExecutiveBriefUpdates(")
            .expect("syncExecutiveBriefUpdates call in htmx:afterSwap");
        let filter_cache_pos = after_swap_slice
            .find("filterCache.set(status")
            .expect("filterCache.set in htmx:afterSwap");
        assert!(
            sync_call_pos < filter_cache_pos,
            "syncExecutiveBriefUpdates must run before filterCache.set"
        );
        assert!(
            after_swap_slice.contains(
                "if (isEntriesSwap && !AUTH_ENABLED && typeof hydrateGuestEntriesFromDom"
            ),
            "hydrateGuestEntriesFromDom must run only on isEntriesSwap"
        );

        // Detail swap branch: Guest rewrite .is-error guard & AUTH_ENABLED saveCachedArticle guard
        let detail_swap_pos = after_swap_slice
            .find("targetId === 'detail-panel'")
            .expect("detail-panel swap branch");
        let detail_swap_end = after_swap_slice[detail_swap_pos..]
            .find("targetId.startsWith('entry-')")
            .expect("entry- branch")
            + detail_swap_pos;
        let detail_swap_branch = &after_swap_slice[detail_swap_pos..detail_swap_end];
        assert!(detail_swap_branch.contains("!rewriteBox.classList.contains('is-error')"));
        assert!(detail_swap_branch
            .contains("!rewriteBox.querySelector('.is-error, .stroke-destructive')"));
        assert!(detail_swap_branch.contains("AUTH_ENABLED && entryId && !panel.querySelector('[hx-get][hx-trigger], .stroke-destructive, .is-error')"));

        // htmx:responseError / htmx:sendError handle target.id === 'ai-rewrite-box'
        assert!(html.contains("target.id === 'ai-rewrite-box'"));
        assert!(html.contains("target.getAttribute('hx-get') || evt.detail.elt?.getAttribute('hx-get') || evt.detail.requestConfig?.path"));
        assert!(html.contains("data-fallback-summary"));
        assert!(html.contains("target.outerHTML"));
        assert!(html.contains("'ai-rewrite-done-' + numId"));

        // IndexedDB version bumps: article cache v3, guest db v3
        assert!(html.contains("indexedDB.open(ARTICLE_DB_NAME, 3)"));
        assert!(html.contains("indexedDB.open(DB_NAME, 3)"));

        // Cross-article swap guards & case-insensitive safeHref
        assert!(html.contains("function abortActiveDetailRequests("));
        assert!(html.contains("document.addEventListener('htmx:beforeSwap'"));
        assert!(html.contains("document.addEventListener('htmx:oobBeforeSwap'"));
        assert!(html.contains("data-target-entry-id"));
        assert!(html.contains("trimmed.toLowerCase()"));
    }

    #[test]
    fn test_entry_list_guest_json_excludes_original_content_and_includes_images_and_rewrite() {
        let entry_with_content = EntryViewModel {
            id: 401,
            title: "Original </script><script>alert(1)</script> & <Tag>".to_string(),
            suggested_title: Some("Suggested </script> Title & <More>".to_string()),
            feed_title: "Feed & Co".to_string(),
            category_title: None,
            published_at: "2026-11-23T12:00:00Z".to_string(),
            reading_time: 5,
            summary: "Summary".to_string(),
            sanitized_content: Some("<p>Original body that must not be exposed</p>".to_string()),
            image_url: Some("https://example.com/hero.jpg".to_string()),
            url: "https://example.com/401".to_string(),
            author: Some("Alice Author".to_string()),
            is_read: false,
            is_bookmarked: false,
            rewrite: Some("Cached </script> rewrite & <More>".to_string()),
            body_images: vec![
                "https://example.com/body1.jpg".to_string(),
                "https://example.com/body2.jpg".to_string(),
            ],
        };

        let template_guest = EntryListTemplate {
            entries: vec![entry_with_content],
            top_executive_brief: None,
            unread_count: Some(1),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: String::new(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };

        let json = template_guest
            .guest_entries_json()
            .expect("guest_entries_json must return Some when !auth_enabled and entries non-empty");
        assert!(
            !json.contains("</script>") && !json.contains('<') && !json.contains('>'),
            "guest_entries_json must WHATWG-escape <, >, and </script>: {}",
            json
        );
        assert!(json.contains(r"\u003c/script\u003e"));
        assert!(json.contains(r"\u0026"));

        let html = template_guest.render().expect("render guest list template");
        assert!(
            html.contains(r#"<div id="executive-brief-sync-data" hidden></div>"#),
            "EntryListTemplate must include #executive-brief-sync-data with boolean hidden"
        );
        assert!(
            html.contains(r#"<script id="entries-guest-data" type="application/json">"#),
            "EntryListTemplate must render #entries-guest-data script block: {}",
            html
        );
        let start = html
            .find(r#"<script id="entries-guest-data" type="application/json">"#)
            .unwrap()
            + r#"<script id="entries-guest-data" type="application/json">"#.len();
        let end = html[start..].find("</script>").unwrap() + start;
        let raw_script_body = &html[start..end];
        assert!(!raw_script_body.contains("</script>"));

        let parsed: serde_json::Value =
            serde_json::from_str(raw_script_body).expect("parse guest JSON");
        let item = &parsed.as_array().unwrap()[0];
        assert_eq!(item["id"], 401);
        assert_eq!(item["title"], "Suggested </script> Title & <More>");
        assert_eq!(item["summary"], "Summary");
        assert_eq!(item["rewrite"], "Cached </script> rewrite & <More>");
        assert_eq!(item["image_url"], "https://example.com/hero.jpg");
        assert_eq!(
            item["body_images"],
            serde_json::json!([
                "https://example.com/body1.jpg",
                "https://example.com/body2.jpg"
            ])
        );
        assert_eq!(item["url"], "https://example.com/401");
        assert!(item.get("sanitized_content").is_none());
        assert_eq!(item["feed_title"], "Feed & Co");
        assert_eq!(item["published_at"], "2026-11-23T12:00:00Z");
        assert!(item.get("formatted_date").is_none());
        assert!(item.get("author").is_none());
        assert!(item.get("reading_time").is_none());
        assert!(item.get("readingTime").is_none());

        // When auth_enabled is true, no script block is emitted
        let entry_auth = EntryViewModel {
            id: 402,
            title: "Auth Only".to_string(),
            suggested_title: None,
            feed_title: "Feed".to_string(),
            category_title: None,
            published_at: "2026-11-23T12:00:00Z".to_string(),
            reading_time: 4,
            summary: "Summary".to_string(),
            sanitized_content: None,
            image_url: None,
            url: "https://example.com/402".to_string(),
            author: Some("Bob".to_string()),
            is_read: false,
            is_bookmarked: false,
            rewrite: None,
            body_images: vec![],
        };
        let template_auth = EntryListTemplate {
            entries: vec![entry_auth],
            top_executive_brief: None,
            unread_count: Some(1),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "unread".to_string(),
            search: String::new(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: true,
        };
        assert!(template_auth.guest_entries_json().is_none());
        let auth_html = template_auth.render().expect("render auth list template");
        assert!(!auth_html.contains("entries-guest-data"));
    }

    #[test]
    fn test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time() {
        let html = IndexTemplate {
            unread_count: 2,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("render guest IndexTemplate");

        assert!(html.contains("const AUTH_ENABLED = false;"));
        assert!(html.contains("const AI_ENABLED = true;"));
        assert!(html.contains("function safeHref(rawUrl)"));
        assert!(html.contains("function readGuestEntriesJson()"));
        assert!(html.contains("function hydrateGuestEntriesFromDom()"));
        assert!(html.contains("async function getGuestArticle(numId)"));
        assert!(html.contains("async function openGuestEntryDetail(numId)"));
        assert!(html.contains("function renderGuestEntryDetail(entry)"));
        assert!(html.contains("function openGuestDb()"));

        // openEntryDetail must short-circuit when !AUTH_ENABLED before getCachedArticle or htmx.ajax
        let open_pos = html
            .find("async function openEntryDetail(id)")
            .expect("openEntryDetail");
        let cache_pos = html[open_pos..]
            .find("getCachedArticle(numId)")
            .expect("getCachedArticle inside openEntryDetail");
        let open_head = &html[open_pos..open_pos + cache_pos];
        assert!(
            open_head.contains("if (!AUTH_ENABLED)")
                && open_head.contains("openGuestEntryDetail(numId)"),
            "openEntryDetail must short-circuit to openGuestEntryDetail before cache/HTMX"
        );

        // renderGuestEntryDetail must omit metadata & guest-detail-content, while rendering hero, Option C rewrite, body_images, and #ai-analysis-box
        let render_pos = html
            .find("function renderGuestEntryDetail(entry)")
            .expect("renderGuestEntryDetail");
        let render_end = html[render_pos..]
            .find("async function openGuestEntryDetail(numId)")
            .expect("openGuestEntryDetail after renderGuestEntryDetail")
            + render_pos;
        let render_fn = &html[render_pos..render_end];
        assert!(!render_fn.contains("guest-detail-content"));
        assert!(!render_fn.contains("data-date-format"));
        assert!(render_fn.contains("feed_title"));
        assert!(!render_fn.contains("formatted_date"));
        assert!(render_fn.contains("published_at") && render_fn.contains("data-relative-time=\"true\""));
        assert!(!render_fn.contains("sanitized_content"));
        assert!(render_fn.contains("safeHref("));
        assert!(render_fn.contains("entry.image_url"));
        assert!(render_fn.contains("entry.body_images"));
        assert!(render_fn.contains("safeImg && safeImg !== '#'"));
        assert!(render_fn.contains("entry.rewrite"));
        assert!(render_fn.contains("entry.summary"));
        assert!(render_fn.contains("id=\"ai-rewrite-box\""));
        assert!(render_fn.contains("data-fallback-summary="));
        assert!(render_fn.contains("/hx/entries/${numId}/rewrite"));
        assert!(render_fn.contains("id=\"ai-analysis-box\""));
        assert!(render_fn.contains("ai-rewrite-done-${numId} from:body"));
        assert!(render_fn.contains("/analysis"));
        assert!(render_fn.contains("htmx.process(panel)"));
        assert!(!render_fn.contains("author"));
        assert!(!render_fn.contains("readingTime"));
        assert!(!render_fn.contains("reading_time"));

        // Cold deep-link miss fallback
        assert!(html.contains("Article Not Available Offline"));
        assert!(html.contains("Back to List"));

        // renderLocalAllArticles must wrap rows in #entries-list-view and use an isolated empty-state ID
        let local_all_pos = html
            .find("function renderLocalAllArticles()")
            .expect("renderLocalAllArticles");
        let local_all_end = html[local_all_pos..]
            .find("function syncDomReadStates()")
            .expect("syncDomReadStates");
        let local_all_fn = &html[local_all_pos..local_all_pos + local_all_end];
        assert!(
            local_all_fn.contains("<div id=\"entries-list-view\""),
            "renderLocalAllArticles must wrap rows in #entries-list-view"
        );
        assert!(
            local_all_fn.contains("id=\"local-all-empty-state\"")
                && !local_all_fn.contains("id=\"client-empty-state\""),
            "renderLocalAllArticles must not reuse #client-empty-state deleted by syncDomReadStates"
        );

        // initGuestDb must merge keys into readSet rather than overwriting concurrent deep-link reads
        let init_db_pos = html.find("function initGuestDb()").expect("initGuestDb");
        let init_db_end = html[init_db_pos..]
            .find("function cacheLoadedArticlesFromDom()")
            .expect("cacheLoadedArticlesFromDom")
            + init_db_pos;
        let init_db_fn = &html[init_db_pos..init_db_end];
        assert!(
            init_db_fn.contains("readSet.add(id)") && !init_db_fn.contains("readSet = new Set("),
            "initGuestDb must merge into readSet rather than overwriting it"
        );
    }

    #[test]
    fn test_index_template_tab_view_rules_and_toggle_visibility() {
        let html = IndexTemplate {
            unread_count: 2,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("render IndexTemplate");

        assert!(html.contains("let currentViewMode = 'ai';"));
        assert!(html.contains("let isSearchActive = false;"));
        assert!(html.contains("function shouldHideAiToggle(status, searchActive)"));
        assert!(html.contains("function getDefaultViewForState(status, searchActive)"));
        assert!(html.contains("function updateAiToggleVisibility(status, searchActive)"));

        // shouldHideAiToggle hides on today and all when not searching
        let hide_pos = html
            .find("function shouldHideAiToggle(status, searchActive)")
            .unwrap();
        let hide_end = html[hide_pos..].find('}').unwrap() + hide_pos;
        let hide_fn = &html[hide_pos..=hide_end];
        assert!(hide_fn.contains("status === 'today'") && hide_fn.contains("status === 'all'"));
        assert!(hide_fn.contains("!searchActive"));

        // getDefaultViewForState: search/today/all/starred -> 'list', unread -> 'ai'
        let def_pos = html
            .find("function getDefaultViewForState(status, searchActive)")
            .unwrap();
        let def_end = html[def_pos..].find('}').unwrap() + def_pos;
        let def_fn = &html[def_pos..=def_end];
        assert!(def_fn.contains("status === 'today'"));
        assert!(def_fn.contains("status === 'all'"));
        assert!(def_fn.contains("status === 'starred'"));
        assert!(def_fn.contains("return 'list'"));
        assert!(def_fn.contains("return 'ai'"));

        // toggleAiView flips currentViewMode rather than stale localStorage
        let toggle_pos = html.find("function toggleAiView()").unwrap();
        let toggle_end = html[toggle_pos..].find('}').unwrap() + toggle_pos;
        let toggle_fn = &html[toggle_pos..=toggle_end];
        assert!(
            toggle_fn.contains("currentViewMode === 'ai' ? 'list' : 'ai'"),
            "toggleAiView must flip currentViewMode"
        );

        // htmx:afterSwap must inspect requestConfig.elt for trigger element
        assert!(
            html.contains("requestConfig") && html.contains("requestConfig.elt"),
            "htmx:afterSwap must inspect evt.detail.requestConfig.elt"
        );
    }

    #[test]
    fn test_paging_loading_overlay_and_read_blur_removed() {
        // 1. Read-item CSS blur removed from base.html
        let raw_base = include_str!("../templates/base.html");
        assert!(
            !raw_base.contains("blur(0.2px)"),
            "base.html must not apply blur(0.2px) to .ar-list-item.read"
        );
        assert!(
            !raw_base.contains("blur(0.25px)"),
            "base.html must not apply blur(0.25px) to .magazine-card.read"
        );
        assert!(
            raw_base.contains("filter: grayscale(80%);"),
            "base.html must keep grayscale(80%) on .ar-list-item.read"
        );
        assert!(
            raw_base.contains("filter: grayscale(85%);"),
            "base.html must keep grayscale(85%) on .magazine-card.read"
        );

        // 2. Pagination prev/next buttons have ids, hx-indicator, and closeDetailPanel()
        let entry = EntryViewModel {
            id: 1,
            title: "Sample".to_string(),
            suggested_title: None,
            summary: "Summary".to_string(),
            sanitized_content: None,
            url: "https://example.com/1".to_string(),
            author: None,
            feed_title: "Feed".to_string(),
            category_title: None,
            is_read: false,
            is_bookmarked: false,
            published_at: "2026-11-25T00:00:00Z".to_string(),
            reading_time: 1,
            image_url: None,
            rewrite: None,
            body_images: vec![],
        };
        let list_template = EntryListTemplate {
            entries: vec![entry],
            top_executive_brief: None,
            unread_count: Some(1),
            current_page: 2,
            total_pages: 3,
            has_prev: true,
            has_next: true,
            prev_offset: 0,
            next_offset: 100,
            status: "all".to_string(),
            search: String::new(),
            limit: 50,
            is_brief_eligible: false,
            offset: 50,
            feed_id: None,
            category_id: None,
            auth_enabled: false,
        };
        let list_html = list_template
            .render()
            .expect("Failed to render EntryListTemplate");
        assert!(
            list_html.contains("id=\"pagination-prev-btn\""),
            "entry_list.html must include id=\"pagination-prev-btn\" on Previous button"
        );
        assert!(
            list_html.contains("id=\"pagination-next-btn\""),
            "entry_list.html must include id=\"pagination-next-btn\" on Next button"
        );
        assert!(
            list_html
                .matches("hx-indicator=\"#entries-loading-bar\"")
                .count()
                >= 2,
            "Both Previous and Next buttons must declare hx-indicator=\"#entries-loading-bar\""
        );
        assert!(
            list_html.matches("onclick=\"closeDetailPanel()\"").count() >= 2,
            "Both Previous and Next buttons must call onclick=\"closeDetailPanel()\""
        );

        // 3. IndexTemplate: mark-page-read-btn wiring, #entries-loading-overlay, and JS loading helpers
        let index_auth = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render auth IndexTemplate");

        assert!(
            index_auth.contains("id=\"entries-loading-overlay\""),
            "index.html must include #entries-loading-overlay"
        );
        assert!(
            index_auth.contains("function showEntriesLoadingState()"),
            "index.html must define showEntriesLoadingState()"
        );
        assert!(
            index_auth.contains("function hideEntriesLoadingState()"),
            "index.html must define hideEntriesLoadingState()"
        );
        assert!(
            index_auth.contains("htmx:afterRequest"),
            "index.html must hook hideEntriesLoadingState() into htmx:afterRequest"
        );

        let mark_btn_pos = index_auth.find("id=\"mark-page-read-btn\"").unwrap();
        let mark_btn_end = index_auth[mark_btn_pos..].find('>').unwrap() + mark_btn_pos;
        let mark_btn_tag = &index_auth[mark_btn_pos..=mark_btn_end];
        assert!(
            mark_btn_tag.contains("hx-indicator=\"#entries-loading-bar\""),
            "Auth #mark-page-read-btn must declare hx-indicator=\"#entries-loading-bar\""
        );
        assert!(
            mark_btn_tag.contains("onclick=\"closeDetailPanel()\""),
            "Auth #mark-page-read-btn must declare onclick=\"closeDetailPanel()\""
        );

        // 4. Guest mode markVisiblePageLocallyRead() closes detail panel and shows toast
        let index_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render guest IndexTemplate");

        let guest_fn_pos = index_guest
            .find("function markVisiblePageLocallyRead()")
            .unwrap();
        let guest_fn_end = index_guest[guest_fn_pos..]
            .find("function showToast")
            .unwrap()
            + guest_fn_pos;
        let guest_fn = &index_guest[guest_fn_pos..guest_fn_end];
        assert!(
            guest_fn.contains("closeDetailPanel()"),
            "Guest markVisiblePageLocallyRead() must call closeDetailPanel()"
        );
        assert!(
            guest_fn.contains("showToast('Page marked as read.')"),
            "Guest markVisiblePageLocallyRead() must show 'Page marked as read.' toast"
        );
    }

    #[test]
    fn test_auth_mark_page_read_button_attributes_and_empty_guard() {
        let html_auth = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render auth IndexTemplate");

        let mark_btn_pos = html_auth.find("id=\"mark-page-read-btn\"").unwrap();
        let mark_btn_end = html_auth[mark_btn_pos..].find('>').unwrap() + mark_btn_pos;
        let mark_btn_tag = &html_auth[mark_btn_pos..=mark_btn_end];
        assert!(
            mark_btn_tag.contains("hx-disabled-elt=\"this\""),
            "Auth #mark-page-read-btn must declare hx-disabled-elt=\"this\""
        );
        assert!(
            mark_btn_tag.contains("hx-include=\"#active-status, #active-offset, [name='search']\""),
            "Auth #mark-page-read-btn must include #active-status, #active-offset, [name='search']"
        );

        let before_req_pos = html_auth
            .find("document.addEventListener('htmx:beforeRequest'")
            .expect("htmx:beforeRequest listener");
        let before_req_end = html_auth[before_req_pos..]
            .find("function setActiveFilter(")
            .expect("setActiveFilter")
            + before_req_pos;
        let before_req_fn = &html_auth[before_req_pos..before_req_end];
        let guard_pos = before_req_fn
            .find("getVisibleEntryIds")
            .expect("htmx:beforeRequest must check getVisibleEntryIds()");
        let loading_pos = before_req_fn
            .find("showEntriesLoadingState()")
            .expect("htmx:beforeRequest must call showEntriesLoadingState()");
        assert!(
            guard_pos < loading_pos,
            "Empty-page guard must run before showEntriesLoadingState()"
        );
        assert!(
            before_req_fn.contains("evt.preventDefault()")
                && before_req_fn.contains("trigger.disabled = false")
                && before_req_fn.contains("trigger.classList.remove('htmx-request')"),
            "Empty-page guard must prevent request and reset disabled/.htmx-request"
        );

        let cancel_pos = html_auth
            .find("function cancelPendingMarkPageRead()")
            .expect("cancelPendingMarkPageRead helper");
        let cancel_end = html_auth[cancel_pos..].find("\n  function ").unwrap() + cancel_pos;
        let cancel_fn = &html_auth[cancel_pos..cancel_end];
        assert!(
            cancel_fn.contains("typeof cancelPendingGuestMarkRead === 'function'"),
            "cancelPendingMarkPageRead must guard guest helper with typeof check in Auth mode"
        );
        assert!(
            cancel_fn.contains("htmx:abort")
                && cancel_fn.contains("markBtn.disabled = false")
                && cancel_fn.contains("markBtn.classList.remove('htmx-request')")
                && cancel_fn.contains("hideEntriesLoadingState()"),
            "cancelPendingMarkPageRead must abort request, reset button state, and hide loading state"
        );
    }

    #[test]
    fn test_entries_scroll_reset_and_filter_cache_invalidation_on_mark_read() {
        let html_auth = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render auth IndexTemplate");

        let html_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render guest IndexTemplate");

        let apply_pos = html_auth.find("function applyCachedList(").unwrap();
        let apply_end = html_auth[apply_pos..]
            .find("function handleFilterClick(")
            .unwrap()
            + apply_pos;
        let apply_fn = &html_auth[apply_pos..apply_end];
        assert!(
            apply_fn.contains("entriesList.scrollTop = 0"),
            "applyCachedList must reset entriesList.scrollTop = 0"
        );

        let local_all_pos = html_guest
            .find("function renderLocalAllArticles()")
            .unwrap();
        let local_all_end = html_guest[local_all_pos..]
            .find("function syncDomReadStates()")
            .unwrap()
            + local_all_pos;
        let local_all_fn = &html_guest[local_all_pos..local_all_end];
        assert!(
            local_all_fn.contains("list.scrollTop = 0"),
            "renderLocalAllArticles must reset list.scrollTop = 0"
        );

        let hide_pos = html_auth
            .find("function hideEntriesLoadingState()")
            .unwrap();
        let hide_end = html_auth[hide_pos..]
            .find("document.addEventListener('htmx:beforeRequest'")
            .unwrap()
            + hide_pos;
        let hide_fn = &html_auth[hide_pos..hide_end];
        assert!(
            hide_fn.contains("entries-loading-bar")
                && hide_fn.contains("classList.remove('htmx-request')"),
            "hideEntriesLoadingState must clear .htmx-request on #entries-loading-bar"
        );

        let after_swap_pos = html_auth
            .find("document.addEventListener('htmx:afterSwap'")
            .unwrap();
        let after_swap_end = html_auth[after_swap_pos..]
            .find("document.addEventListener('htmx:afterRequest'")
            .unwrap()
            + after_swap_pos;
        let after_swap_fn = &html_auth[after_swap_pos..after_swap_end];
        assert!(
            after_swap_fn.contains("evt.detail.target.scrollTop = 0")
                || after_swap_fn.contains("entriesListEl.scrollTop = 0"),
            "htmx:afterSwap must reset scrollTop = 0 on #entries-list swap"
        );
        let inv_pos = after_swap_fn
            .find("invalidateFilterCache()")
            .expect("htmx:afterSwap must call invalidateFilterCache() on mark-page-read swap");
        let set_pos = after_swap_fn
            .find("filterCache.set(")
            .expect("htmx:afterSwap must cache active list");
        assert!(
            after_swap_fn.contains("triggerElt.id === 'mark-page-read-btn'") && inv_pos < set_pos,
            "htmx:afterSwap must invalidateFilterCache() on mark-page-read-btn swap before filterCache.set()"
        );

        let after_req_pos = html_auth
            .find("document.addEventListener('htmx:afterRequest'")
            .unwrap();
        let after_req_end = html_auth[after_req_pos..]
            .find("document.addEventListener('htmx:responseError'")
            .unwrap()
            + after_req_pos;
        let after_req_fn = &html_auth[after_req_pos..after_req_end];
        assert!(
            after_req_fn.contains("mark-page-read-btn")
                && after_req_fn.contains("disabled = false")
                && after_req_fn.contains("classList.remove('htmx-request')"),
            "htmx:afterRequest must reset disabled and .htmx-request on #mark-page-read-btn"
        );
    }

    #[test]
    fn test_guest_mark_page_read_htmx_next_page_and_deferred_commit() {
        let html_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render guest IndexTemplate");

        let mark_btn_pos = html_guest.find("id=\"mark-page-read-btn\"").unwrap();
        let mark_btn_end = html_guest[mark_btn_pos..].find('>').unwrap() + mark_btn_pos;
        let mark_btn_tag = &html_guest[mark_btn_pos..=mark_btn_end];
        assert!(
            mark_btn_tag.contains("hx-indicator=\"#entries-loading-bar\"")
                && mark_btn_tag.contains("hx-disabled-elt=\"this\""),
            "Guest #mark-page-read-btn must declare hx-indicator and hx-disabled-elt"
        );

        // Preserve D-01 slice boundary between renderLocalAllArticles() and syncDomReadStates()
        let local_all_pos = html_guest
            .find("function renderLocalAllArticles()")
            .unwrap();
        let sync_pos = html_guest.find("function syncDomReadStates()").unwrap();
        let local_all_slice = &html_guest[local_all_pos..sync_pos];
        assert!(
            !local_all_slice.contains("persistLocallyReadIds")
                && !local_all_slice.contains("pendingGuestMarkRead"),
            "Guest mark-read helpers must not be inserted between renderLocalAllArticles() and syncDomReadStates()"
        );

        let mark_local_pos = html_guest.find("function markLocallyRead(").unwrap();
        let between_sync_and_mark = &html_guest[sync_pos..mark_local_pos];
        assert!(
            between_sync_and_mark.contains("let pendingGuestMarkRead")
                && between_sync_and_mark.contains("function parseVisibleEntryIds()")
                && between_sync_and_mark.contains("function persistLocallyReadIds(")
                && between_sync_and_mark.contains("function cancelPendingGuestMarkRead()"),
            "Guest mark-read helpers must live after syncDomReadStates() and before markLocallyRead()"
        );

        let persist_pos = html_guest.find("function persistLocallyReadIds(").unwrap();
        let persist_end = html_guest[persist_pos..]
            .find("function cancelPendingGuestMarkRead()")
            .unwrap()
            + persist_pos;
        let persist_fn = &html_guest[persist_pos..persist_end];
        assert!(
            persist_fn.contains("!readSet.has(id)")
                && persist_fn.contains("db.transaction(STORE_NAME, 'readwrite')"),
            "persistLocallyReadIds must deduplicate !readSet.has(id) in one readwrite IndexedDB tx"
        );

        let guest_fn_pos = html_guest
            .find("function markVisiblePageLocallyRead()")
            .unwrap();
        let guest_fn_end = html_guest[guest_fn_pos..]
            .find("function showToast")
            .unwrap()
            + guest_fn_pos;
        let guest_fn = &html_guest[guest_fn_pos..guest_fn_end];
        assert!(
            guest_fn.contains("closeDetailPanel()")
                && guest_fn.contains("parseVisibleEntryIds()")
                && guest_fn.contains("pagination-next-btn")
                && guest_fn.contains("htmx.ajax('GET', nextUrl")
                && guest_fn.contains("source: markBtn")
                && guest_fn.contains("pendingGuestMarkRead === token")
                && guest_fn.contains("showToast('Page marked as read.')"),
            "markVisiblePageLocallyRead must fetch next page via htmx.ajax with source: markBtn and token guard"
        );

        assert!(
            between_sync_and_mark.contains("htmx:beforeSwap")
                && between_sync_and_mark.contains("requestConfig.elt")
                && between_sync_and_mark.contains("mark-page-read-btn")
                && between_sync_and_mark.contains("shouldSwap !== false")
                && between_sync_and_mark.contains("!evt.detail.isError")
                && between_sync_and_mark.contains("persistLocallyReadIds(token.ids, { syncDom: false })"),
            "Guest htmx:beforeSwap must defer commit strictly for non-error mark-page-read-btn swaps"
        );

        let filter_pos = html_guest.find("function handleFilterClick(").unwrap();
        let filter_end = html_guest[filter_pos..]
            .find("function showEntriesLoadingState()")
            .unwrap()
            + filter_pos;
        assert!(
            html_guest[filter_pos..filter_end].contains("cancelPendingMarkPageRead()"),
            "handleFilterClick must cancel pending mark-page-read requests"
        );

        let search_pos = html_guest.find("function onSearchSubmit(").unwrap();
        let search_end = html_guest[search_pos..]
            .find("const ARTICLE_DB_NAME")
            .unwrap()
            + search_pos;
        assert!(
            html_guest[search_pos..search_end].contains("cancelPendingMarkPageRead()"),
            "onSearchSubmit must cancel pending mark-page-read requests"
        );

        let before_req_pos = html_guest
            .find("document.addEventListener('htmx:beforeRequest'")
            .unwrap();
        let before_req_end = html_guest[before_req_pos..]
            .find("function setActiveFilter(")
            .unwrap()
            + before_req_pos;
        assert!(
            html_guest[before_req_pos..before_req_end].contains("cancelPendingMarkPageRead()"),
            "htmx:beforeRequest must cancel pending mark-page-read requests on tab/search/pagination transitions"
        );
    }

    #[test]
    fn test_guest_sync_dom_read_states_hides_magazine_on_empty_unread() {
        let html_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render guest IndexTemplate");

        let sync_pos = html_guest.find("function syncDomReadStates()").unwrap();
        let sync_end = html_guest[sync_pos..]
            .find("let pendingGuestMarkRead")
            .or_else(|| html_guest[sync_pos..].find("function markLocallyRead("))
            .unwrap()
            + sync_pos;
        let sync_fn = &html_guest[sync_pos..sync_end];
        assert!(
            sync_fn.contains("entries-magazine-view")
                && sync_fn.contains("magView.style.display = 'none'")
                && sync_fn.contains("magView.style.display = ''"),
            "syncDomReadStates must hide #entries-magazine-view when Unread reaches 0 visible items and restore otherwise"
        );
    }

    #[test]
    fn test_guest_filter_articles_client_side_keeps_read_hidden_on_unread() {
        let html_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render guest IndexTemplate");

        let filter_pos = html_guest
            .find("function filterArticlesClientSide(")
            .unwrap();
        let filter_end = html_guest[filter_pos..]
            .find("function getVisibleEntryIds()")
            .unwrap()
            + filter_pos;
        let filter_fn = &html_guest[filter_pos..filter_end];
        assert!(
            filter_fn.contains("active-status")
                && filter_fn.contains("'unread'")
                && filter_fn.contains("classList.contains('read')")
                && filter_fn.contains("item.style.display = 'none'"),
            "filterArticlesClientSide must keep .read rows hidden when #active-status is 'unread'"
        );

        let persist_pos = html_guest.find("function persistLocallyReadIds(").unwrap();
        let persist_end = html_guest[persist_pos..]
            .find("function cancelPendingGuestMarkRead()")
            .unwrap()
            + persist_pos;
        let persist_fn = &html_guest[persist_pos..persist_end];
        assert!(
            persist_fn.contains("!isSearchActive")
                && persist_fn.contains("filterArticlesClientSide(searchQuery)"),
            "persistLocallyReadIds with syncDom must re-apply filterArticlesClientSide(searchQuery) when !isSearchActive && searchQuery"
        );
    }

    #[test]
    fn test_entries_loading_bar_preserved_and_stale_xhr_guarded() {
        let html_auth = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render auth IndexTemplate");

        let html_guest = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render guest IndexTemplate");

        let hide_pos = html_auth
            .find("function hideEntriesLoadingState()")
            .unwrap();
        let hide_end = html_auth[hide_pos..]
            .find("document.addEventListener('htmx:beforeRequest'")
            .unwrap()
            + hide_pos;
        let hide_fn = &html_auth[hide_pos..hide_end];
        assert!(
            hide_fn.contains("!activeEntriesRequestXhr"),
            "hideEntriesLoadingState must preserve #entries-loading-bar .htmx-request while activeEntriesRequestXhr is active"
        );

        let before_req_pos = html_auth
            .find("document.addEventListener('htmx:beforeRequest'")
            .unwrap();
        let before_req_end = html_auth[before_req_pos..]
            .find("function setActiveFilter(")
            .unwrap()
            + before_req_pos;
        let before_req_fn = &html_auth[before_req_pos..before_req_end];
        let set_xhr_pos = before_req_fn
            .rfind("activeEntriesRequestXhr = evt.detail.xhr || null;")
            .expect("htmx:beforeRequest must set activeEntriesRequestXhr before cancelling pending mark-read");
        let cancel_pos = before_req_fn
            .rfind("cancelPendingMarkPageRead()")
            .expect("htmx:beforeRequest must call cancelPendingMarkPageRead()");
        assert!(
            set_xhr_pos < cancel_pos,
            "htmx:beforeRequest must assign activeEntriesRequestXhr before cancelPendingMarkPageRead() so incoming request loading bar stays intact"
        );
        assert!(
            before_req_fn.contains("pendingAuthMarkReadXhr"),
            "htmx:beforeRequest must track pendingAuthMarkReadXhr for Auth mark-page-read cancellation"
        );

        let cancel_fn_pos = html_auth
            .find("function cancelPendingMarkPageRead()")
            .unwrap();
        let cancel_fn_end =
            html_auth[cancel_fn_pos..].find("\n  function ").unwrap() + cancel_fn_pos;
        let cancel_fn = &html_auth[cancel_fn_pos..cancel_fn_end];
        assert!(
            cancel_fn.contains("if (!pendingAuthMarkReadXhr) return;"),
            "cancelPendingMarkPageRead must no-op when no Auth mark-page-read request is pending"
        );

        let guest_cancel_pos = html_guest
            .find("function cancelPendingGuestMarkRead()")
            .unwrap();
        let guest_cancel_end = html_guest[guest_cancel_pos..]
            .find("document.addEventListener('htmx:beforeRequest'")
            .unwrap()
            + guest_cancel_pos;
        let guest_cancel_fn = &html_guest[guest_cancel_pos..guest_cancel_end];
        assert!(
            guest_cancel_fn.contains("if (!pendingGuestMarkRead) return false;"),
            "cancelPendingGuestMarkRead must no-op when no Guest mark-page-read request is pending"
        );

        let after_req_pos = html_auth
            .find("document.addEventListener('htmx:afterRequest'")
            .unwrap();
        let after_req_end = html_auth[after_req_pos..]
            .find("document.addEventListener('htmx:responseError'")
            .unwrap()
            + after_req_pos;
        assert!(
            html_auth[after_req_pos..after_req_end]
                .contains("clearCurrentEntriesLoadingState(xhr)"),
            "htmx:afterRequest must guard loading cleanup via clearCurrentEntriesLoadingState(xhr)"
        );

        let resp_err_pos = html_auth
            .find("document.addEventListener('htmx:responseError'")
            .unwrap();
        let resp_err_end = html_auth[resp_err_pos..]
            .find("document.addEventListener('htmx:sendError'")
            .unwrap()
            + resp_err_pos;
        assert!(
            html_auth[resp_err_pos..resp_err_end].contains("clearCurrentEntriesLoadingState(evt.detail.xhr)"),
            "htmx:responseError must guard loading cleanup via clearCurrentEntriesLoadingState(evt.detail.xhr)"
        );
    }

    #[test]
    fn test_view_aware_entries_skeleton_and_accessibility() {
        let html = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render IndexTemplate");

        let overlay_start = html
            .find("id=\"entries-loading-overlay\"")
            .expect("Missing #entries-loading-overlay");
        let overlay_end = html[overlay_start..]
            .find("id=\"entries-list\"")
            .expect("Missing #entries-list after overlay")
            + overlay_start;
        let overlay_html = &html[overlay_start..overlay_end];

        assert!(
            overlay_html.contains("role=\"status\""),
            "#entries-loading-overlay must have role=\"status\""
        );
        assert!(
            overlay_html.contains("aria-live=\"polite\""),
            "#entries-loading-overlay must have aria-live=\"polite\""
        );
        assert!(
            overlay_html.contains("<span class=\"sr-only\">Loading articles...</span>"),
            "#entries-loading-overlay must include sr-only loading status"
        );
        assert!(
            !overlay_html.contains("animate-spin"),
            "#entries-loading-overlay must replace spinner with skeleton"
        );
        assert!(
            overlay_html.contains("id=\"entries-skeleton-magazine\""),
            "Missing #entries-skeleton-magazine"
        );
        assert!(
            overlay_html.contains("magazine-3col-grid")
                && overlay_html.contains("magazine-p2-grid"),
            "Magazine skeleton must match 3-col and p2 grid classes"
        );
        assert!(
            overlay_html.contains("id=\"entries-skeleton-list\""),
            "Missing #entries-skeleton-list"
        );
        assert!(
            overlay_html.contains("min-h-[48px]"),
            "List skeleton rows must match .ar-list-item min-h-[48px]"
        );
        assert!(
            overlay_html.contains("animate-pulse")
                && overlay_html.contains("motion-reduce:animate-none")
                && overlay_html.contains("aria-hidden=\"true\""),
            "Skeleton containers must include animate-pulse, motion-reduce:animate-none, and aria-hidden"
        );

        let show_pos = html.find("function showEntriesLoadingState()").unwrap();
        let hide_pos = html.find("function hideEntriesLoadingState()").unwrap();
        let after_hide = html[hide_pos..]
            .find("document.addEventListener('htmx:beforeRequest'")
            .unwrap()
            + hide_pos;
        let show_fn = &html[show_pos..hide_pos];
        let hide_fn = &html[hide_pos..after_hide];

        assert!(
            show_fn.contains("currentViewMode === 'ai'"),
            "showEntriesLoadingState must switch skeleton based on currentViewMode === 'ai'"
        );
        assert!(
            show_fn.contains("entries-skeleton-magazine")
                && show_fn.contains("entries-skeleton-list"),
            "showEntriesLoadingState must toggle magazine and list skeleton elements"
        );
        assert!(
            show_fn.contains("aria-busy") && show_fn.contains("'true'"),
            "showEntriesLoadingState must set aria-busy=\"true\" on #entries-list"
        );
        assert!(
            hide_fn.contains("aria-busy") && hide_fn.contains("'false'"),
            "hideEntriesLoadingState must set aria-busy=\"false\" on #entries-list"
        );
        assert!(
            hide_fn.contains("entries-skeleton-magazine")
                && hide_fn.contains("entries-skeleton-list"),
            "hideEntriesLoadingState must hide both skeleton variants"
        );

        let grouped_html = AiGroupedListTemplate {
            groups: vec![FeedSummaryGroupViewModel {
                feed_title: "AI Daily".to_string(),
                entries: vec![EntryViewModel {
                    id: 99,
                    title: "Autonomous Agent Flow".to_string(),
                    suggested_title: None,
                    feed_title: "AI Daily".to_string(),
                    published_at: "10 mins ago".to_string(),
                    reading_time: 2,
                    summary: "Extractive teaser".to_string(),
                    url: "https://example.com/agent".to_string(),
                    author: None,
                    is_read: false,
                    category_title: None,
                    image_url: None,
                    sanitized_content: None,
                    is_bookmarked: false,
                    rewrite: None,
                    body_images: vec![],
                }],
            }],
        }
        .render()
        .expect("Failed to render AiGroupedListTemplate");

        assert!(
            grouped_html.contains("<div class=\"mt-3\">"),
            "ai_grouped_list.html must wrap #ai-summary-slot-{{id}} in <div class=\"mt-3\">"
        );
        assert!(
            grouped_html.contains("id=\"ai-summary-slot-99\""),
            "ai_grouped_list.html must set id=\"ai-summary-slot-99\""
        );
        assert!(
            !grouped_html.contains("class=\"ai-summary-slot mt-3"),
            "mt-3 should live on the wrapper div rather than .ai-summary-slot"
        );
        assert!(
            grouped_html.contains("animate-pulse")
                && grouped_html.contains("motion-reduce:animate-none")
                && grouped_html.contains("Generating summary..."),
            "ai_grouped_list.html slot must use reduced-motion-safe pulse skeleton"
        );
        assert!(
            !grouped_html.contains("animate-spin"),
            "ai_grouped_list.html must not use animate-spin"
        );
    }

    #[test]
    fn test_toast_deduplication_and_detail_is_error_cache_guard() {
        let html = IndexTemplate {
            unread_count: 5,
            today_count: 2,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render IndexTemplate");

        let show_toast_pos = html.find("function showToast(message)").unwrap();
        let show_toast_end = html[show_toast_pos..]
            .find("function triggerNativeBookmark")
            .unwrap()
            + show_toast_pos;
        let show_toast_fn = &html[show_toast_pos..show_toast_end];
        assert!(
            show_toast_fn.contains("showClientToast(message, false)"),
            "Guest showToast must delegate to shared showClientToast(message, false)"
        );
        assert!(
            html.contains(r#"toast.setAttribute('data-auto-dismiss', 'true')"#)
                && html.contains(r#"toast.setAttribute('role', 'status')"#),
            "base.html showClientToast must set data-auto-dismiss=\"true\" and role=\"status\""
        );

        let swap_pos = html
            .find("targetId === 'detail-panel'")
            .expect("Missing detail-panel swap branch");
        let swap_end = html[swap_pos..]
            .find("targetId.startsWith('entry-')")
            .unwrap()
            + swap_pos;
        let swap_branch = &html[swap_pos..swap_end];
        assert!(
            swap_branch.contains("targetId === 'ai-analysis-box'")
                && swap_branch.contains("targetId === 'ai-rewrite-box'"),
            "Detail swap branch must explicitly match ai-analysis-box and ai-rewrite-box outerHTML targets"
        );
        assert!(
            swap_branch.contains("evt.target && evt.target.closest"),
            "Detail swap branch must check live evt.target.closest('#detail-panel')"
        );
        assert!(
            swap_branch.contains(".stroke-destructive, .is-error"),
            "Detail IndexedDB cache guard must exclude .is-error cards"
        );

        let base_after_swap_pos = html
            .find("document.body.addEventListener(\"htmx:afterSwap\"")
            .unwrap();
        let base_oob_pos = html
            .find("document.body.addEventListener(\"htmx:oobAfterSwap\"")
            .unwrap();
        let base_end = html[base_oob_pos..].find("</script>").unwrap() + base_oob_pos;
        let base_after_swap = &html[base_after_swap_pos..base_oob_pos];
        let base_oob_swap = &html[base_oob_pos..base_end];

        assert!(
            base_after_swap.contains("swapTarget.isConnected")
                && base_after_swap.contains("document.getElementById(swapTarget.id)")
                && base_after_swap.contains("lucide.createIcons({ root: liveRoot })"),
            "base.html htmx:afterSwap must resolve connected root for outerHTML swaps before lucide.createIcons"
        );
        assert!(
            base_oob_swap.contains("textContent") && base_oob_swap.contains("existing.remove()"),
            "base.html htmx:oobAfterSwap must deduplicate matching toasts in #toast-container"
        );
    }

    #[test]
    fn test_index_template_data_view_mode_css_and_before_swap_sync() {
        let html = IndexTemplate {
            unread_count: 2,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("render IndexTemplate");

        assert!(
            html.contains(
                r#"#entries-list[data-view-mode="ai"] #entries-list-view { display: none; }"#
            ) && html.contains(
                r#"#entries-list[data-view-mode="ai"] #entries-magazine-view { display: block; }"#
            ) && html.contains(
                r#"#entries-list[data-view-mode="list"] #entries-list-view { display: block; }"#
            ) && html.contains(
                r#"#entries-list[data-view-mode="list"] #entries-magazine-view { display: none; }"#
            ),
            "index.html must define CSS data-view-mode rules for #entries-list"
        );

        let apply_pos = html
            .find("function applyAiViewMode(")
            .expect("applyAiViewMode");
        let apply_end = html[apply_pos..]
            .find("function recordSourceClick(")
            .expect("recordSourceClick")
            + apply_pos;
        let apply_fn = &html[apply_pos..apply_end];
        assert!(
            apply_fn.contains("setAttribute('data-view-mode', effectiveMode)"),
            "applyAiViewMode must sync data-view-mode on #entries-list"
        );

        assert!(
            html.contains("htmx:beforeSwap") && html.contains("setAttribute('data-view-mode'"),
            "index.html must sync data-view-mode in htmx:beforeSwap before #entries-list paints"
        );
    }

    #[test]
    fn test_index_template_auth_skeleton_extracts_mag_entry_brief_and_hero_image() {
        let html = IndexTemplate {
            unread_count: 2,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("render IndexTemplate");

        let open_pos = html
            .find("async function openEntryDetail(id)")
            .expect("openEntryDetail");
        let skel_pos = html
            .find("function openDetailWithSkeleton(")
            .expect("openDetailWithSkeleton");
        let open_fn = &html[open_pos..skel_pos];
        assert!(
            open_fn.contains("mag-entry-")
                && open_fn.contains(".magazine-brief")
                && open_fn.contains("openDetailWithSkeleton(title, brief, imageUrl)"),
            "openEntryDetail must extract brief and imageUrl from #mag-entry-{{id}} and pass them to openDetailWithSkeleton"
        );

        let skel_end = html[skel_pos..]
            .find("function alignDetailPanelWidth(")
            .expect("alignDetailPanelWidth")
            + skel_pos;
        let skel_fn = &html[skel_pos..skel_end];
        assert!(
            skel_fn.contains("function openDetailWithSkeleton(title, brief, imageUrl)")
                && skel_fn.contains("safeHref(")
                && skel_fn.contains("<img src="),
            "openDetailWithSkeleton must render hero image when imageUrl is present"
        );
    }

    #[test]
    fn test_empty_entry_list_hides_pagination_and_renders_refresh_button() {
        let template = EntryListTemplate {
            entries: vec![],
            top_executive_brief: None,
            unread_count: Some(0),
            current_page: 1,
            total_pages: 3,
            has_prev: false,
            has_next: true,
            prev_offset: 0,
            next_offset: 50,
            status: "unread".to_string(),
            search: "".to_string(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: true,
        };
        let html = template
            .render()
            .expect("Failed to render empty EntryListTemplate");

        assert!(
            !html.contains(r#"aria-label="Pagination""#),
            "Empty EntryListTemplate must not render pagination nav"
        );
        assert!(
            html.contains(r#"id="empty-refresh-btn""#)
                && html.contains(r#"onclick="refreshCurrentEntriesList()""#),
            "Empty EntryListTemplate must render #empty-refresh-btn calling refreshCurrentEntriesList()"
        );
    }

    #[test]
    fn test_set_active_filter_manages_hover_bg_border() {
        let html = IndexTemplate {
            unread_count: 1,
            today_count: 0,
            entries_list_html: String::new(),
            ai_enabled: false,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render IndexTemplate");

        let fn_start = html
            .find("function setActiveFilter(btn)")
            .expect("setActiveFilter");
        let fn_end = html[fn_start..]
            .find("function isCurrentEntriesRequest(")
            .expect("isCurrentEntriesRequest")
            + fn_start;
        let fn_body = &html[fn_start..fn_end];

        assert!(
            fn_body.contains("el.classList.add('bg-muted', 'text-muted-foreground', 'border-border', 'hover:bg-border', 'font-medium')"),
            "setActiveFilter must add hover:bg-border to inactive .filter-pill buttons"
        );
        assert!(
            fn_body.contains("btn.classList.remove('bg-muted', 'text-muted-foreground', 'border-border', 'hover:bg-border', 'font-medium')"),
            "setActiveFilter must remove hover:bg-border from the active .filter-pill button"
        );
    }

    #[test]
    fn test_sync_dom_read_states_hides_pagination_and_renders_refresh_button() {
        let html = IndexTemplate {
            unread_count: 1,
            today_count: 0,
            entries_list_html: String::new(),
            ai_enabled: false,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render IndexTemplate");

        assert!(
            html.contains("function refreshCurrentEntriesList()"),
            "IndexTemplate must define refreshCurrentEntriesList()"
        );
        let render_local_pos = html
            .find("function renderLocalAllArticles()")
            .expect("renderLocalAllArticles");
        let refresh_pos = html
            .find("function refreshCurrentEntriesList()")
            .expect("refreshCurrentEntriesList");
        assert!(
            refresh_pos < render_local_pos,
            "refreshCurrentEntriesList() must be placed before renderLocalAllArticles()"
        );

        let sync_start = html
            .find("function syncDomReadStates()")
            .expect("syncDomReadStates");
        let sync_end = html[sync_start..]
            .find("let pendingGuestMarkRead = null;")
            .expect("pendingGuestMarkRead")
            + sync_start;
        let sync_fn = &html[sync_start..sync_end];

        assert!(
            sync_fn.contains(r##"#entries-list nav[aria-label="Pagination"]"##)
                && sync_fn.contains("paginationNav.style.display = 'none'")
                && sync_fn.contains("paginationNav.style.display = ''"),
            "syncDomReadStates must hide pagination when #client-empty-state is shown and restore otherwise"
        );
        assert!(
            sync_fn.contains(r#"id="empty-refresh-btn""#)
                && sync_fn.contains(r#"onclick="refreshCurrentEntriesList()""#),
            "syncDomReadStates #client-empty-state must include #empty-refresh-btn"
        );
        assert!(
            sync_fn.contains("pagination-next-btn")
                && sync_fn.contains("autoAdvancingUnreadUrl")
                && sync_fn.contains("htmx.ajax('GET', nextUrl"),
            "syncDomReadStates must auto-advance to next unread page when #pagination-next-btn exists instead of showing All caught up"
        );
    }

    #[test]
    fn test_pagination_hides_previous_brief_and_aborts_stale_brief_xhr() {
        let html = IndexTemplate {
            unread_count: 1,
            today_count: 0,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("Failed to render IndexTemplate");

        let before_start = html
            .find("document.addEventListener('htmx:beforeRequest'")
            .expect("htmx:beforeRequest");
        let before_end = html[before_start..]
            .find("function setActiveFilter(")
            .expect("setActiveFilter")
            + before_start;
        let before_fn = &html[before_start..before_end];

        assert!(
            before_fn.contains("target && target.id === 'page-executive-brief'")
                && before_fn.contains("activeBriefXhr = evt.detail.xhr || null"),
            "htmx:beforeRequest must track activeBriefXhr for #page-executive-brief"
        );
        assert!(
            before_fn.contains("pagination-prev-btn")
                && before_fn.contains("pagination-next-btn")
                && before_fn.contains("activeBriefXhr.abort()")
                && before_fn.contains("list-executive-brief")
                && before_fn.contains("page-executive-brief"),
            "Pagination in htmx:beforeRequest must abort activeBriefXhr and hide previous briefs"
        );
    }

    #[test]
    fn test_first_load_dedup_and_guest_brief_gating() {
        let html = IndexTemplate {
            unread_count: 50,
            today_count: 50,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("Failed to render IndexTemplate");

        assert!(
            html.contains("let guestDbReady = AUTH_ENABLED;"),
            "Must initialize guestDbReady from AUTH_ENABLED"
        );
        assert!(
            html.contains("if (!AUTH_ENABLED && !guestDbReady)")
                && html.contains("pendingGuestBriefEl = trigger || target;")
                && html.contains("flushPendingGuestBrief()"),
            "Must defer #page-executive-brief request in guest mode until IndexedDB readSet loads"
        );
        assert!(
            html.contains("activeBriefXhr && activeBriefUrl && requestPath && requestPath === activeBriefUrl"),
            "Must deduplicate concurrent #page-executive-brief requests for the same URL"
        );
        assert!(
            html.contains("activeEntriesRequestXhr && activeEntriesRequestUrl && requestPath && requestPath === activeEntriesRequestUrl"),
            "Must deduplicate concurrent #entries-list requests for the same URL"
        );
        assert!(
            html.contains("abortActiveBriefRequest();"),
            "Must abort stale brief request when auto-advancing or swapping #entries-list"
        );
        assert!(
            html.contains("hasPendingBrief = Boolean(entriesList && entriesList.querySelector('#page-executive-brief[hx-get]'))"),
            "seedInitialFilterCache must not cache unresolved #page-executive-brief skeleton"
        );
    }

    #[test]
    fn test_magazine_cards_use_full_card_stretched_link_click() {
        let make_entry = |id: i64, feed: &str, is_read: bool| EntryViewModel {
            id,
            title: format!("Article {id}"),
            suggested_title: None,
            feed_title: feed.to_string(),
            category_title: Some("Tech".to_string()),
            published_at: "2026-11-27T09:00:00Z".to_string(),
            reading_time: 2,
            summary: format!("Summary {id}"),
            sanitized_content: None,
            image_url: if id <= 2 {
                Some(format!("https://example.com/{id}.jpg"))
            } else {
                None
            },
            rewrite: None,
            body_images: vec![],
            url: format!("https://example.com/{id}"),
            author: None,
            is_read,
            is_bookmarked: false,
        };

        let entries: Vec<EntryViewModel> = (1..=9)
            .map(|i| make_entry(i, "O'Reilly", i == 2))
            .collect();
        let list_html = EntryListTemplate {
            entries,
            top_executive_brief: None,
            unread_count: Some(8),
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 50,
            status: "unread".to_string(),
            search: String::new(),
            limit: 50,
            is_brief_eligible: false,
            offset: 0,
            feed_id: None,
            category_id: None,
            auth_enabled: true,
        }
        .render()
        .expect("render EntryListTemplate");

        let mag_start = list_html.find(r#"id="entries-magazine-view""#).unwrap();
        let mag_html = &list_html[mag_start..];

        // P1 (id=1), P2 (id=2), P3 (id=8) all use <a id="mag-title-..." class="magazine-card-link">
        for id in [1, 2, 8] {
            assert!(
                mag_html.contains(&format!(r#"<a id="mag-title-{id}""#))
                    && mag_html.contains(&format!(r#"href="/entries/{id}""#))
                    && mag_html.contains(r#"class="magazine-card-link""#),
                "Magazine card {id} must use <a id=\"mag-title-{id}\" class=\"magazine-card-link\">: {mag_html}"
            );
            assert!(
                !mag_html.contains(&format!("openEntryFromMagazine({id}, 'O'Reilly')")),
                "Must never interpolate feed_title directly into onclick JS string"
            );
        }
        assert!(
            mag_html.contains("if (!event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey)")
                && mag_html.contains("this.closest('.magazine-card')?.dataset.feedTitle || ''"),
            "Magazine stretched link must guard modifier keys and read dataset.feedTitle"
        );
        assert!(
            !mag_html.contains(r#"class="hero-content-box p-5 flex flex-col justify-center flex-1 w-full relative"#),
            "hero-content-box must remain position: static (no relative class)"
        );
        let p3_anchor_pos = mag_html.find(r#"<a id="mag-title-8""#).unwrap();
        let p3_anchor_end = mag_html[p3_anchor_pos..].find('>').unwrap() + p3_anchor_pos;
        let p3_anchor = &mag_html[p3_anchor_pos..=p3_anchor_end];
        assert!(
            p3_anchor.contains(r#"title="Summary 8""#),
            "P3 anchor must carry title tooltip: {p3_anchor}"
        );

        let index_html = IndexTemplate {
            unread_count: 8,
            today_count: 8,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("render IndexTemplate");

        assert!(
            index_html.contains(".magazine-card { position: relative; cursor: pointer; }")
                && index_html.contains(".magazine-card-link { color: inherit; text-decoration: none; }")
                && index_html.contains(".magazine-card-link::after { content: \"\"; position: absolute; inset: 0; z-index: 1; }")
                && index_html.contains(".magazine-card:has(.magazine-card-link:focus-visible)")
                && index_html.contains(".magazine-card.read .magazine-card-link"),
            "base.html must define stretched-link, focus-visible ring, and read link CSS"
        );
        assert!(
            index_html.contains("(magTitleEl.querySelector('.magazine-card-link') || magTitleEl).textContent = suggestedTitle"),
            "syncExecutiveBriefUpdates must preserve .magazine-card-link"
        );
    }

    #[test]
    fn test_detail_prev_next_buttons_and_navigation_helpers() {
        let entry = EntryViewModel {
            id: 42,
            title: "Navigable Article".to_string(),
            suggested_title: None,
            feed_title: "Tech Feed".to_string(),
            category_title: None,
            published_at: "2026-11-27T09:00:00Z".to_string(),
            reading_time: 2,
            summary: "Summary".to_string(),
            sanitized_content: None,
            image_url: None,
            rewrite: Some("Rewrite body.".to_string()),
            body_images: vec![],
            url: "https://example.com/42".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };
        let detail_html = EntryDetailTemplate {
            entry,
            summary: String::new(),
            auth_enabled: true,
            rewrite: "Rewrite body.".to_string(),
            needs_rewrite: false,
            ai_enabled: true,
            fallback_summary: String::new(),
            impact: String::new(),
            fallacies: String::new(),
            has_analysis: false,
            hero_image: None,
            body_images: vec![],
        }
        .render()
        .expect("render EntryDetailTemplate");

        assert!(
            detail_html.contains(r#"id="detail-prev-btn""#)
                && detail_html.contains(r#"id="detail-next-btn""#)
                && detail_html.contains(r#"onclick="navigateDetailArticle(-1)""#)
                && detail_html.contains(r#"onclick="navigateDetailArticle(1)""#)
                && detail_html.contains(r#"type="button""#)
                && detail_html.contains(r#"class="detail-original-link"#)
                && detail_html.contains(r#"<span class="detail-original-label">Original</span>"#)
                && detail_html.contains("flex items-center gap-1.5"),
            "entry_detail.html must render type=button Prev/Next buttons, compactable Original link, and gap-1.5"
        );

        let index_guest = IndexTemplate {
            unread_count: 2,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("render guest IndexTemplate");

        assert!(
            index_guest.contains("@container workspace (max-width: 360px)")
                && index_guest.contains(".detail-original-label")
                && index_guest.contains(".detail-original-link")
                && index_guest.contains("function finishGuestDbInit()"),
            "base.html and index.html must include <=360px container query compact header and finishGuestDbInit"
        );
        assert!(
            index_guest.contains("indexedDB.open(ARTICLE_DB_NAME, 3)")
                && index_guest.contains("e.oldVersion < 3")
                && index_guest.contains(r#"cached.html.includes('id="detail-prev-btn"')"#),
            "Auth detail IndexedDB cache must bump to v3 and check id=\"detail-prev-btn\""
        );
        assert!(
            index_guest.contains("let detailNavIds = [];")
                && index_guest.contains("function syncDetailNavigationIds()")
                && index_guest.contains("function updateDetailNavButtons(entryId)")
                && index_guest.contains("function navigateDetailArticle(delta)")
                && index_guest.contains("detailNavIds.indexOf(Number(entryId))"),
            "index.html must define detailNavIds and navigation helpers"
        );

        // openDetailWithSkeleton must preserve 3-arg signature and include Prev/Next buttons
        let skel_pos = index_guest.find("function openDetailWithSkeleton(title, brief, imageUrl)").unwrap();
        let align_pos = index_guest[skel_pos..].find("function alignDetailPanelWidth(").unwrap() + skel_pos;
        let skel_fn = &index_guest[skel_pos..align_pos];
        assert!(
            skel_fn.contains(r#"id="detail-prev-btn""#)
                && skel_fn.contains(r#"id="detail-next-btn""#)
                && skel_fn.contains(r#"disabled:opacity-40 disabled:pointer-events-none"#)
                && skel_fn.contains(r#"onclick="navigateDetailArticle(-1)""#)
                && skel_fn.contains("gap-1.5"),
            "openDetailWithSkeleton must render interactive disabled-aware Prev/Next controls with gap-1.5"
        );

        // renderGuestEntryDetail must include Prev/Next controls and call updateDetailNavButtons
        let render_pos = index_guest.find("function renderGuestEntryDetail(entry)").unwrap();
        let open_guest_pos = index_guest[render_pos..].find("async function openGuestEntryDetail(numId)").unwrap() + render_pos;
        let render_fn = &index_guest[render_pos..open_guest_pos];
        assert!(
            render_fn.contains(r#"id="detail-prev-btn""#)
                && render_fn.contains(r#"id="detail-next-btn""#)
                && render_fn.contains("updateDetailNavButtons(numId)"),
            "renderGuestEntryDetail must render Prev/Next controls and call updateDetailNavButtons(numId)"
        );

        // markLocallyRead / unmarkLocallyRead / toggleLocalReadStatus must NOT rebuild detailNavIds
        let mark_local_pos = index_guest.find("function markLocallyRead(").unwrap();
        let mark_vis_pos = index_guest[mark_local_pos..].find("function markVisiblePageLocallyRead()").unwrap() + mark_local_pos;
        let session_read_slice = &index_guest[mark_local_pos..mark_vis_pos];
        assert!(
            !session_read_slice.contains("syncDetailNavigationIds"),
            "Session read toggles must not rebuild detailNavIds"
        );
    }

    #[test]
    fn test_feed_and_relative_time_metadata_in_list_detail_and_guest() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-11-27T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert_eq!(format_relative_time_at("", now), "");
        assert_eq!(format_relative_time_at("   ", now), "");
        assert_eq!(format_relative_time_at("2026-07-04", now), "04/07/2026");
        assert_eq!(format_relative_time_at("2026-11-27T11:59:30Z", now), "Just now");
        assert_eq!(format_relative_time_at("2026-11-27T11:45:00Z", now), "15m ago");
        assert_eq!(format_relative_time_at("2026-11-27T09:00:00Z", now), "3h ago");
        assert_eq!(format_relative_time_at("2026-11-25T12:00:00Z", now), "2d ago");
        assert_eq!(format_relative_time_at("2026-11-19T12:00:00Z", now), "19/11/2026");
        assert_eq!(format_relative_time_at("2 hours ago", now), "2 hours ago");

        let make_entry = |id: i64, feed: &str, published_at: &str| EntryViewModel {
            id,
            title: format!("Title {id}"),
            suggested_title: None,
            feed_title: feed.to_string(),
            category_title: None,
            published_at: published_at.to_string(),
            reading_time: 5,
            summary: "Summary".to_string(),
            sanitized_content: None,
            image_url: None,
            rewrite: Some("Rewrite".to_string()),
            body_images: vec![],
            url: format!("https://example.com/{id}"),
            author: Some("Hidden Author".to_string()),
            is_read: false,
            is_bookmarked: false,
        };

        // Guarded separator in EntryItemTemplate
        let item_only_feed = EntryItemTemplate {
            entry: make_entry(1, "Feed Only", ""),
            unread_count: None,
        }
        .render()
        .unwrap();
        assert!(item_only_feed.contains("Feed Only") && !item_only_feed.contains(" · ") && !item_only_feed.contains("<time"));

        let item_empty_both = EntryItemTemplate {
            entry: make_entry(2, "", ""),
            unread_count: None,
        }
        .render()
        .unwrap();
        assert!(!item_empty_both.contains("entry-meta"));

        let item_both = EntryItemTemplate {
            entry: make_entry(3, "Tech Feed", "2026-11-20T12:00:00Z"),
            unread_count: None,
        }
        .render()
        .unwrap();
        assert!(
            item_both.contains("Tech Feed")
                && item_both.contains(" · ")
                && item_both.contains(r#"data-relative-time="true""#)
                && item_both.contains(r#"data-feed-title="Tech Feed""#)
                && item_both.contains(r#"data-published-at="2026-11-20T12:00:00Z""#),
            "Entry item must render Feed · <time> and data attributes: {item_both}"
        );
        // Dot remains outside .flex-col title+meta wrapper
        let dot_pos = item_both.find(r#"id="entry-3-dot""#).unwrap();
        let col_pos = item_both.find("flex flex-col min-w-0 flex-1 gap-0.5").unwrap();
        assert!(dot_pos < col_pos, "Unread dot must precede vertical flex-col wrapper");

        // Auth IndexTemplate must include shared formatRelativeTimeLabel & refreshRelativeTimes
        let index_auth = IndexTemplate {
            unread_count: 1,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: false,
            auth_enabled: true,
        }
        .render()
        .unwrap();
        assert!(
            index_auth.contains("function formatRelativeTimeLabel(raw)")
                && index_auth.contains("function refreshRelativeTimes(root)"),
            "Auth mode must include shared relative time helpers above guest block"
        );
    }

    #[test]
    fn test_toast_role_status_hover_focus_pause_and_guest_undo() {
        let payload = ToastUndoPayload {
            entry_ids: "10,11".to_string(),
            status: Some("unread".to_string()),
            search: Some("rust \"async\"".to_string()),
            feed_id: None,
            category_id: None,
            offset: 0,
        };
        let undo = ToastUndoAction::from_payload("/hx/mark-page-unread", "Undo", &payload);
        let toast_html = ToastTemplate {
            message: "Page marked as read.".to_string(),
            is_error: false,
            undo: Some(undo),
        }
        .render()
        .expect("render ToastTemplate with undo");

        assert!(
            toast_html.contains(r#"role="status""#)
                && toast_html.contains(r#"aria-atomic="true""#)
                && toast_html.contains(r#"class="toast-undo-btn"#)
                && toast_html.contains(r#"hx-post="/hx/mark-page-unread""#)
                && toast_html.contains(r##"hx-target="#entries-list""##)
                && toast_html.contains(r##"hx-indicator="#entries-loading-bar""##)
                && toast_html.contains(r#"hx-disabled-elt="this""#)
                && !toast_html.contains(r#"id="toast-undo-btn""#),
            "ToastTemplate with undo must render accessible role=status and .toast-undo-btn: {toast_html}"
        );

        let index_guest = IndexTemplate {
            unread_count: 3,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: false,
        }
        .render()
        .expect("render guest IndexTemplate");

        // Auth #mark-page-read-btn passes unread_entry_ids
        let index_auth = IndexTemplate {
            unread_count: 3,
            today_count: 1,
            entries_list_html: String::new(),
            ai_enabled: true,
            auth_enabled: true,
        }
        .render()
        .expect("render auth IndexTemplate");
        assert!(
            index_auth.contains("unread_entry_ids: getVisibleUnreadEntryIds()"),
            "Auth #mark-page-read-btn must pass unread_entry_ids"
        );

        // base.html toast helpers, HX-Reswap !== 'none' guard, hover/focus pause
        assert!(
            index_guest.contains("function escapeToastHtml(str)")
                && index_guest.contains("function attachToastAutoDismiss(toast, durationMs)")
                && index_guest.contains("function showClientToast(message, isError = false)")
                && index_guest.contains("toast.contains(document.activeElement)")
                && index_guest.contains("toast.matches(':hover')")
                && index_guest.contains("reswap.toLowerCase() !== 'none'")
                && index_guest.contains("toast.dataset.undoInFlight = 'true'")
                && index_guest.contains("attachToastAutoDismiss(toast, hasUndo ? 8000 : 5000)"),
            "base.html must implement shared showClientToast, hover/focus/in-flight pause, and HX-Reswap check"
        );

        // htmx:responseError / htmx:sendError route .toast-undo-btn first
        let resp_err_pos = index_guest.find("document.addEventListener('htmx:responseError'").unwrap();
        let send_err_pos = index_guest.find("document.addEventListener('htmx:sendError'").unwrap();
        let resp_err_slice = &index_guest[resp_err_pos..send_err_pos];
        assert!(
            resp_err_slice.find(".toast-undo-btn").unwrap()
                < resp_err_slice.find("evt.detail.target.id === 'entries-list'").unwrap(),
            "htmx:responseError must check .toast-undo-btn before #entries-list"
        );

        // Guest Undo wiring
        assert!(
            index_guest.contains("function showUndoToast(message, onUndo)")
                && index_guest.contains("function undoGuestMarkPageRead(snapshot)")
                && index_guest.contains("newlyReadIds")
                && index_guest.contains("snapshot.ids.forEach(id => readSet.delete(Number(id)))"),
            "Guest mode must wire showUndoToast and undoGuestMarkPageRead synchronously deleting readSet"
        );
    }
}
