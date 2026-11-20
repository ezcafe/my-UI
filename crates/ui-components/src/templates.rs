use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryViewModel {
    pub id: i64,
    pub title: String,
    pub feed_title: String,
    pub published_at: String,
    pub reading_time: i32,
    pub summary: String,
    pub url: String,
    pub author: Option<String>,
    pub is_read: bool,
    pub is_bookmarked: bool,
}

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub unread_count: i32,
    pub entries_list_html: String,
    pub ai_enabled: bool,
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
    pub unread_count: Option<i32>,
}

#[derive(Template)]
#[template(path = "partials/entry_detail.html")]
pub struct EntryDetailTemplate {
    pub entry: EntryViewModel,
    pub summary: String,
    pub sanitized_content: String,
}

#[derive(Template)]
#[template(path = "partials/toast.html")]
pub struct ToastTemplate {
    pub message: String,
    pub is_error: bool,
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
    fn test_render_title_only_entry_item() {
        let entry_unread = EntryViewModel {
            id: 42,
            title: "Spatial Computing in Rust".to_string(),
            feed_title: "Rust Optics".to_string(),
            published_at: "2 hours ago".to_string(),
            reading_time: 4,
            summary: "Extractive brief of the post.".to_string(),
            url: "https://example.com/spatial".to_string(),
            author: Some("Visionary".to_string()),
            is_read: false,
            is_bookmarked: true,
        };

        let template_unread = EntryItemTemplate {
            entry: entry_unread,
            unread_count: Some(5),
        };
        let html_unread = template_unread.render().expect("Failed to render EntryItemTemplate");

        // Title and styling
        assert!(html_unread.contains("Spatial Computing in Rust"));
        assert!(html_unread.contains("line-clamp-2"));
        assert!(html_unread.contains("text-sm font-medium"));

        // Omission of summary, reading time, and feed title
        assert!(!html_unread.contains("Rust Optics"), "Feed title must be omitted");
        assert!(!html_unread.contains("Extractive brief of the post"), "Summary must be omitted");
        assert!(!html_unread.contains("min read"), "Reading time must be omitted");

        // External link button attributes
        assert!(html_unread.contains("href=\"https://example.com/spatial\""));
        assert!(html_unread.contains("target=\"_blank\""));
        assert!(html_unread.contains("rel=\"noopener noreferrer\""));
        assert!(html_unread.contains("onclick=\"event.stopPropagation()\""));
        assert!(html_unread.contains("data-lucide=\"external-link\""));
        assert!(html_unread.contains("data-lucide=\"chevron-right\""));

        // Unread indicator dot present when is_read = false
        assert!(html_unread.contains("title=\"Unread\"") || html_unread.contains("bg-indigo-400"));

        // When is_read = true, unread indicator dot is omitted
        let entry_read = EntryViewModel {
            id: 43,
            title: "Spatial Computing in Rust - Read".to_string(),
            feed_title: "Rust Optics".to_string(),
            published_at: "2 hours ago".to_string(),
            reading_time: 4,
            summary: "Extractive brief of the post.".to_string(),
            url: "https://example.com/spatial-read".to_string(),
            author: Some("Visionary".to_string()),
            is_read: true,
            is_bookmarked: false,
        };
        let template_read = EntryItemTemplate {
            entry: entry_read,
            unread_count: None,
        };
        let html_read = template_read.render().expect("Failed to render read EntryItemTemplate");
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
                feed_title: "Feed A".to_string(),
                published_at: "1 hour ago".to_string(),
                reading_time: 2,
                summary: "Summary A".to_string(),
                url: "https://example.com/1".to_string(),
                author: None,
                is_read: false,
                is_bookmarked: false,
            },
            EntryViewModel {
                id: 2,
                title: "Second Title".to_string(),
                feed_title: "Feed B".to_string(),
                published_at: "2 hours ago".to_string(),
                reading_time: 5,
                summary: "Summary B".to_string(),
                url: "https://example.com/2".to_string(),
                author: None,
                is_read: true,
                is_bookmarked: true,
            },
        ];

        let template = EntryListTemplate {
            entries,
            unread_count: Some(1),
        };
        let html = template.render().expect("Failed to render EntryListTemplate");

        assert!(html.contains("First Title"));
        assert!(html.contains("Second Title"));
        assert!(!html.contains("Summary A"));
        assert!(!html.contains("Summary B"));
        assert!(html.contains("data-lucide=\"external-link\""));
        assert!(html.contains("href=\"https://example.com/1\""));
        assert!(html.contains("href=\"https://example.com/2\""));
    }

    #[test]
    fn test_filter_bar_reordering() {
        let template = IndexTemplate {
            unread_count: 3,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // Filter button sequence in order: Unread < Today < Starred < All Titles
        let pos_unread = html.find("Unread").expect("Must contain 'Unread'");
        let pos_today = html.find("Today").expect("Must contain 'Today'");
        let pos_starred = html.find("Starred").expect("Must contain 'Starred'");
        let pos_all = html.find("All Titles").expect("Must contain 'All Titles'");

        assert!(pos_unread < pos_today, "Unread must precede Today");
        assert!(pos_today < pos_starred, "Today must precede Starred");
        assert!(pos_starred < pos_all, "Starred must precede All Titles");

        // Unread is active by default
        assert!(
            html.contains("id=\"filter-unread\"")
                && html.contains("bg-indigo-500/20 text-indigo-300")
                && html.contains("border-indigo-400/30"),
            "Unread filter must be active with styling"
        );

        // Today button triggers /hx/entries?status=today
        assert!(html.contains("/hx/entries?status=today"));
        assert!(html.contains("setActiveFilter"));
    }

    #[test]
    fn test_index_template_renders_ai_button_when_enabled() {
        let index_disabled = IndexTemplate {
            unread_count: 5,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
        };
        let html_disabled = index_disabled.render().expect("Failed to render IndexTemplate disabled");
        assert!(!html_disabled.contains("id=\"filter-ai\""));

        let index_enabled = IndexTemplate {
            unread_count: 5,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
        };
        let html_enabled = index_enabled.render().expect("Failed to render IndexTemplate enabled");
        assert!(html_enabled.contains("id=\"filter-ai\""));
        assert!(html_enabled.contains("hx-get=\"/hx/entries?status=ai\""));

        let pos_ai = html_enabled.find("id=\"filter-ai\"").expect("filter-ai not found");
        let pos_unread = html_enabled.find("id=\"filter-unread\"").expect("filter-unread not found");
        assert!(pos_ai < pos_unread, "AI button must render before Unread button");
    }

    #[test]
    fn test_ai_grouped_list_template_renders_feed_headers_and_canonical_route() {
        let entry = EntryViewModel {
            id: 99,
            title: "Autonomous Agent Flow".to_string(),
            feed_title: "AI Daily".to_string(),
            published_at: "10 mins ago".to_string(),
            reading_time: 2,
            summary: "Extractive teaser".to_string(),
            url: "https://example.com/agent".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };
        let group = FeedSummaryGroupViewModel {
            feed_title: "AI Daily".to_string(),
            entries: vec![entry],
        };
        let template = AiGroupedListTemplate {
            groups: vec![group],
        };
        let html = template.render().expect("Failed to render AiGroupedListTemplate");

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
            feed_title: "Audio Weekly".to_string(),
            published_at: "2 hours ago".to_string(),
            reading_time: 3,
            summary: "Spatial summary.".to_string(),
            url: "https://example.com/audio".to_string(),
            author: Some("Visionary Designer".to_string()),
            is_read: false,
            is_bookmarked: false,
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "Extractive summary".to_string(),
            sanitized_content: "<p>Content</p>".to_string(),
        };

        let html = template.render().expect("Failed to render EntryDetailTemplate");

        // Author must be omitted
        assert!(!html.contains("By Visionary Designer"), "Author must be omitted from detail view");
        assert!(!html.contains("By "), "Author prefix must not be rendered");

        // Feed title and publication date must still be present
        assert!(html.contains("Audio Weekly"));
        assert!(html.contains("2 hours ago"));
    }

    #[test]
    fn test_detail_template_has_mark_unread_button() {
        let entry = EntryViewModel {
            id: 102,
            title: "Micro-gestures in WebXR".to_string(),
            feed_title: "XR Trends".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 4,
            summary: "XR summary.".to_string(),
            url: "https://example.com/xr".to_string(),
            author: None,
            is_read: true,
            is_bookmarked: false,
        };

        let template = EntryDetailTemplate {
            entry,
            summary: "XR summary".to_string(),
            sanitized_content: "<p>Content</p>".to_string(),
        };

        let html = template.render().expect("Failed to render EntryDetailTemplate");

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
        let pos_mark_unread = html.find("hx-post=\"/hx/entries/102/read?status=unread\"").unwrap();
        let pos_bookmark = html.find("hx-post=\"/hx/entries/102/bookmark\"").unwrap();
        assert!(
            pos_mark_unread < pos_bookmark,
            "Mark as unread button must precede bookmark button"
        );
    }

    #[test]
    fn test_render_toast_oob_template() {
        let template = ToastTemplate {
            message: "Upstream service timeout".to_string(),
            is_error: true,
        };
        let html = template.render().expect("Failed to render ToastTemplate");

        assert!(html.contains("id=\"toast-container\""));
        assert!(html.contains("hx-swap-oob=\"beforeend\""));
        assert!(html.contains("Upstream service timeout"));
    }
}
