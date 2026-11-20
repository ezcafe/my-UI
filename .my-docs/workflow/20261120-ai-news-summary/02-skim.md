# Skim: Codebase Inspection

**Result:** done

## Key Takeaways
- The existing system is built on **Axum 0.8**, **Askama 0.12**, **async-graphql 7.0**, and **Miniflux Client**.
- The frontend uses **Tailwind-like utility classes**, **Lucide icons**, **HTMX 2.x**, and spatial glass styling (`ar-glass-panel`, `spatial-window`).
- Persistent storage for AI summaries does not yet exist and should be introduced via **`rusqlite` with bundled SQLite**.
- The existing extractive summary utility (`crates/miniflux/src/summary.rs`) provides an ideal zero-latency fallback when scraping or LLM CLI calls fail.

---

## Detailed Findings

### 1. Server Routing & Application State (`crates/server`)
- **Router setup (`crates/server/src/app.rs`)**: Routes are defined on Axum's `Router` with shared `AppState` containing `Arc<Config>` and `AppSchema`.
- **Entries endpoints (`crates/server/src/routes/entries.rs`)**:
  - `list_entries_htmx`: Parses `EntriesFilterQuery`, executes a GraphQL query against Miniflux, and renders templates.
  - Supports status filters: `unread`, `today`, `starred`, `all`. Adding `status=ai` or a dedicated `/hx/ai-summary` endpoint integrates naturally.
- **Configuration (`crates/server/src/config.rs`)**:
  - Validates and parses environment variables.
  - Needs new AI-related configuration variables:
    - `AI_ENABLED: bool` (default `false`)
    - `LLM_CLI_CMD: Option<String>`
    - `LLM_MODEL: Option<String>`
    - `LLM_EXTRA_ARGS: Option<String>`
    - `AI_DB_PATH: Option<String>` (defaults to `data/summaries.db`)

### 2. UI Templates & HTMX Interactions (`crates/ui-components`)
- **Main template (`templates/pages/index.html`)**:
  - The filter bar contains pills for `Unread`, `Today`, `Starred`, and `All Titles`.
  - When `AI_ENABLED` is true, an `AI` pill is rendered before `Unread` with the `sparkles` icon.
  - The entries list container (`#entries-list`) receives swapped HTML via HTMX.
- **Partial templates (`templates/partials/`)**:
  - `entry_list.html`, `entry_item.html`, and `entry_detail.html` render individual articles.
  - A new partial template `ai_summary_group_list.html` will render the 15 items grouped by feed source.
  - A new partial template `ai_summary_card.html` will render the dynamic summary slot with `hx-trigger="revealed"`.

### 3. Content Extraction, Scraper Fallbacks, and Summary Logic
- **Extractive summary (`crates/miniflux/src/summary.rs`)**:
  - `extract_summary(&content, sentences_count)` uses `unicode-segmentation` to get sentence slices.
  - Serves as the reliable, instant fallback when remote fetches fail or LLM is unreachable.
- **Article parsing**:
  - Needs a dedicated module (e.g., `crates/server/src/ai/extractor.rs`) to parse `<article>` tags, extract body text, strip boilerplate/scripts/nav, and fall back to body/teaser.
  - Truncates text to Title + 400–600 words.

### 4. Database & Caching
- **SQLite integration**:
  - Adding `rusqlite = { version = "0.32", features = ["bundled"] }` ensures zero external native dependencies.
  - Dedicated table:
    ```sql
    CREATE TABLE IF NOT EXISTS article_ai_summaries (
      entry_id INTEGER PRIMARY KEY,
      summary TEXT NOT NULL,
      model TEXT NOT NULL,
      created_at TEXT NOT NULL
    );
    ```
  - Thread-safe access via a lightweight connection pool or `Arc<tokio::sync::Mutex<rusqlite::Connection>>` / worker task.
