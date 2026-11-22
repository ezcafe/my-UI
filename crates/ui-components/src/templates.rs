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
    pub current_page: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_offset: i64,
    pub next_offset: i64,
    pub status: String,
    pub search: String,
    pub limit: i64,
}

#[derive(Template)]
#[template(path = "partials/entry_detail.html")]
pub struct EntryDetailTemplate {
    pub entry: EntryViewModel,
    pub summary: String,
    pub sanitized_content: String,
    pub auth_enabled: bool,
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
        assert!(html_unread.contains("onclick=\"event.stopPropagation(); markArticleRead(42)\""));
        assert!(html_unread.contains("data-lucide=\"external-link\""));
        assert!(html_unread.contains("data-lucide=\"chevron-right\""));

        // Unread indicator dot present when is_read = false
        assert!(html_unread.contains("title=\"Unread\"") || html_unread.contains("bg-brand"));

        // Omission of dotted leader and date time per user preference
        assert!(!html_unread.contains("leader"), "Must omit dotted leader in entry item");
        assert!(!html_unread.contains("2 hours ago"), "Must omit published_at timestamp in entry item");
        assert!(html_unread.contains("bg-brand") || html_unread.contains("var(--brand)"), "Must include brand red indicator");
        assert!(html_unread.contains("last:border-b-0"), "Entry item must include last:border-b-0");

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
            current_page: 1,
            total_pages: 1,
            has_prev: false,
            has_next: false,
            prev_offset: 0,
            next_offset: 0,
            status: "all".to_string(),
            search: "".to_string(),
            limit: 50,
        };
        let html = template.render().expect("Failed to render EntryListTemplate");

        assert!(html.contains("First Title"));
        assert!(html.contains("Second Title"));
        assert!(!html.contains("Summary A"));
        assert!(!html.contains("Summary B"));
        assert!(html.contains("data-lucide=\"external-link\""));
        assert!(html.contains("href=\"https://example.com/1\""));
        assert!(html.contains("href=\"https://example.com/2\""));
        assert!(html.contains("last:border-b-0"), "Entry list item must include last:border-b-0");
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
    fn test_index_template_renders_ai_button_when_enabled() {
        let index_disabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: true,
        };
        let html_disabled = index_disabled.render().expect("Failed to render IndexTemplate disabled");
        assert!(!html_disabled.contains("id=\"filter-ai\""));

        let index_enabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: true,
            auth_enabled: true,
        };
        let html_enabled = index_enabled.render().expect("Failed to render IndexTemplate enabled");
        assert!(html_enabled.contains("id=\"filter-ai\""));
        assert!(html_enabled.contains("hx-get=\"/hx/entries?status=ai\""));

        let pos_ai = html_enabled.find("id=\"filter-ai\"").expect("filter-ai not found");
        let pos_unread = html_enabled.find("id=\"filter-unread\"").expect("filter-unread not found");
        assert!(pos_ai < pos_unread, "AI button must render before Unread button");
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
        let html_enabled = index_auth_enabled.render().expect("Failed to render with auth enabled");
        assert!(html_enabled.contains("/auth/logout"), "Logout button must render when auth_enabled is true");

        let index_auth_disabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
            entries_list_html: "<div>entries</div>".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html_disabled = index_auth_disabled.render().expect("Failed to render with auth disabled");
        assert!(!html_disabled.contains("/auth/logout"), "Logout button must be hidden when auth_enabled is false");
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
        let html = index_auth_disabled.render().expect("Failed to render with auth disabled");
        assert!(html.contains("my_news_guest_db"), "Must initialize guest IndexedDB");
        assert!(html.contains("read_entries"), "Must reference read_entries store");
        assert!(html.contains("readSet"), "Must maintain in-memory readSet");
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
            auth_enabled: true,
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
            auth_enabled: true,
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
    fn test_entry_detail_renders_local_actions_when_auth_disabled() {
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
            auth_enabled: false,
        };

        let html = template.render().expect("Failed to render EntryDetailTemplate");

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
            auth_enabled: true,
        };

        let html = template.render().expect("Failed to render EntryDetailTemplate");

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
        };
        let html = template.render().expect("Failed to render ToastTemplate");

        assert!(html.contains("id=\"toast-container\""));
        assert!(html.contains("hx-swap-oob=\"beforeend\""));
        assert!(html.contains("Upstream service timeout"));
        assert!(html.contains("shadow-float"), "Must use Annuaire .shadow-float elevation");
        assert!(
            html.contains("rounded-full") && html.contains("bg-foreground text-background"),
            "Must use Annuaire ink pill floating toast style"
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
        assert!(
            html.contains("(5)"),
            "Must contain today count badge (5)"
        );
    }

    #[test]
    fn test_entry_list_renders_pagination_when_multiple_pages() {
        let entry = EntryViewModel {
            id: 1,
            title: "Test Entry".to_string(),
            feed_title: "Feed".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 1,
            summary: "".to_string(),
            url: "https://example.com/1".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };
        let template = EntryListTemplate {
            entries: vec![entry],
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
        };
        let html = template.render().expect("Failed to render EntryListTemplate");
        assert!(
            html.contains("aria-label=\"Pagination\""),
            "Must contain pagination nav bar"
        );
        assert!(
            html.contains("Page 1 of 3"),
            "Must display current and total pages"
        );
        assert!(
            html.contains("Previous"),
            "Must have Previous label"
        );
        assert!(
            html.contains("hx-get=\"/hx/entries?status=unread&amp;search=&amp;offset=50&amp;limit=50\"")
                || html.contains("hx-get=\"/hx/entries?status=unread&search=&offset=50&limit=50\""),
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
        };
        let html = template.render().expect("Failed to render EntryListTemplate");
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
            feed_title: "Test Feed".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 1,
            summary: "Summary".to_string(),
            url: "https://example.com/item".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };
        let template = EntryItemTemplate {
            entry,
            unread_count: None,
        };
        let html = template.render().expect("Failed to render EntryItemTemplate");
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
            feed_title: "Test Feed".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 1,
            summary: "Summary".to_string(),
            url: "https://example.com/item".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };
        let template = EntryListTemplate {
            entries: vec![entry],
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
        };
        let html = template.render().expect("Failed to render EntryListTemplate");
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
            feed_title: "Test Feed".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 3,
            summary: "Summary".to_string(),
            url: "https://example.com/detail".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };
        let template = EntryDetailTemplate {
            entry,
            summary: "Summary".to_string(),
            sanitized_content: "<p>Detail body</p>".to_string(),
            auth_enabled: true,
        };
        let html = template.render().expect("Failed to render EntryDetailTemplate");
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
            html.contains("function openDetailWithSkeleton()"),
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
        assert!(html.contains("family=Big+Shoulders+Display"), "Must load Big Shoulders Display font");
        assert!(html.contains("family=Work+Sans"), "Must load Work Sans font");

        // Annuaire OKLCH color tokens in :root and dark theme
        assert!(html.contains("--brand: oklch(0.569 0.213 29)"), "Must define --brand positional red token");
        assert!(html.contains("--primary: oklch(0.21 0.008 274.5)"), "Must define light --primary token");
        assert!(html.contains("[data-theme=\"dark\"]"), "Must include dark theme token overrides");

        // Essential utility classes
        assert!(html.contains(".font-display"), "Must define .font-display utility");
        assert!(html.contains(".font-sans"), "Must define .font-sans utility");
        assert!(html.contains(".tabular"), "Must define .tabular utility");
        assert!(html.contains(".leader"), "Must define .leader utility");
        assert!(html.contains(".shadow-float"), "Must define .shadow-float utility");

        // Theme switcher inline script & toggle function
        assert!(html.contains("function toggleTheme()"), "Must define toggleTheme() function");
        assert!(html.contains("data-theme"), "Must toggle data-theme attribute");
        assert!(html.contains("localStorage"), "Must persist theme in localStorage");
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
        let html_auth = template_auth.render().expect("Failed to render IndexTemplate with auth_enabled: true");
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
        let html_guest = template_guest.render().expect("Failed to render IndexTemplate with auth_enabled: false");
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
        let html_guest = template_guest.render().expect("Failed to render IndexTemplate with auth_enabled: false");
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
        let html_auth = template_auth.render().expect("Failed to render IndexTemplate with auth_enabled: true");
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
            feed_title: "Test Feed".to_string(),
            published_at: "1 hour ago".to_string(),
            reading_time: 3,
            summary: "Summary".to_string(),
            url: "https://example.com/item".to_string(),
            author: None,
            is_read: false,
            is_bookmarked: false,
        };

        // 1. EntryItemTemplate
        let item_template = EntryItemTemplate {
            entry: entry.clone(),
            unread_count: None,
        };
        let item_html = item_template.render().expect("Failed to render EntryItemTemplate");
        assert!(
            item_html.contains("onclick=\"event.stopPropagation(); markArticleRead(42)\""),
            "Entry item external link must call markArticleRead(42)"
        );

        // 2. EntryListTemplate
        let list_template = EntryListTemplate {
            entries: vec![entry.clone()],
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
        };
        let list_html = list_template.render().expect("Failed to render EntryListTemplate");
        assert!(
            list_html.contains("onclick=\"event.stopPropagation(); markArticleRead(42)\""),
            "Entry list external link must call markArticleRead(42)"
        );

        // 3. EntryDetailTemplate
        let detail_template = EntryDetailTemplate {
            entry,
            summary: "Summary".to_string(),
            sanitized_content: "<p>Content</p>".to_string(),
            auth_enabled: false,
        };
        let detail_html = detail_template.render().expect("Failed to render EntryDetailTemplate");
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
                feed_title: "F".into(),
                published_at: "now".into(),
                reading_time: 1,
                summary: "S".into(),
                url: "https://example.com".into(),
                author: None,
                is_read: false,
                is_bookmarked: false,
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

        let pos_handle = html.find("function handleFilterClick").expect("handleFilterClick must exist");
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
        };
        let html = template.render().expect("Failed to render EntryListTemplate");
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
    fn test_mobile_searchbox_compact_and_expand_container_query() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
            auth_enabled: false,
        };
        let html = template.render().expect("Failed to render IndexTemplate");

        // Verify mobile searchbox CSS inside container query
        let cq_index = html
            .find("@container (max-width: 768px)")
            .expect("Missing mobile container query");
        let cq_slice = &html[cq_index..];

        assert!(
            cq_slice.contains("#search-input"),
            "Must define mobile styling for #search-input inside container query"
        );
        assert!(
            cq_slice.contains("width: 5.5rem;") || cq_slice.contains("width: 5rem;"),
            "Mobile #search-input must have compact resting width"
        );
        assert!(
            cq_slice.contains("#search-input:focus"),
            "Must define #search-input:focus expansion"
        );
        assert!(
            cq_slice.contains("width: 11rem;") || cq_slice.contains("width: 10rem;"),
            "Mobile #search-input:focus must expand on focus"
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
}
