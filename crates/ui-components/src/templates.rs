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
        assert!(html_unread.contains("title=\"Unread\"") || html_unread.contains("bg-brand"));

        // Omission of dotted leader and date time per user preference
        assert!(!html_unread.contains("leader"), "Must omit dotted leader in entry item");
        assert!(!html_unread.contains("2 hours ago"), "Must omit published_at timestamp in entry item");
        assert!(html_unread.contains("bg-brand") || html_unread.contains("var(--brand)"), "Must include brand red indicator");

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
    }

    #[test]
    fn test_filter_bar_reordering() {
        let template = IndexTemplate {
            unread_count: 3,
            today_count: 0,
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
        };
        let html_disabled = index_disabled.render().expect("Failed to render IndexTemplate disabled");
        assert!(!html_disabled.contains("id=\"filter-ai\""));

        let index_enabled = IndexTemplate {
            unread_count: 5,
            today_count: 0,
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
    fn test_entry_item_wires_open_detail_with_skeleton() {
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
            html.contains("onclick=\"openDetailWithSkeleton()\""),
            "Entry item must trigger openDetailWithSkeleton()"
        );
        assert!(
            !html.contains("document.getElementById('detail-panel').classList.remove('hidden')"),
            "Entry item must not use inline classList toggle"
        );
    }

    #[test]
    fn test_entry_list_wires_open_detail_with_skeleton() {
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
            html.contains("onclick=\"openDetailWithSkeleton()\""),
            "Entry list items must trigger openDetailWithSkeleton()"
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
        };
        let html = template.render().expect("Failed to render IndexTemplate");
        assert!(
            html.contains("function openDetailWithSkeleton()"),
            "Index template must define openDetailWithSkeleton helper function"
        );
        assert!(
            html.contains("function closeDetailPanel()"),
            "Index template must define closeDetailPanel helper function"
        );
    }

    #[test]
    fn test_index_template_skeleton_structure() {
        let template = IndexTemplate {
            unread_count: 0,
            today_count: 0,
            entries_list_html: "".to_string(),
            ai_enabled: false,
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
    fn test_login_page_renders_dual_panel_swiss_brand() {
        let template = LoginTemplate;
        let html = template.render().expect("Failed to render LoginTemplate");

        // Dual-panel Swiss layout
        assert!(html.contains("grid-cols-1 md:grid-cols-2"), "Must use dual-panel grid layout");

        // Left brand panel with rich Annuaire red background and Big Shoulders Display wordmark
        assert!(
            html.contains("bg-[var(--brand-surface)]") || html.contains("var(--brand-surface)"),
            "Left panel must use Annuaire brand surface red"
        );
        assert!(
            html.contains("font-display") && html.contains("MY NEWS"),
            "Wordmark must use font-display and MY NEWS"
        );

        // Right login card with Pocket-ID ink pill
        assert!(html.contains("Sign in with Pocket-ID"), "Must contain Sign in with Pocket-ID");
        assert!(
            html.contains("bg-primary text-primary-foreground") && html.contains("rounded-full"),
            "Login button must be an ink pill"
        );
    }
}
