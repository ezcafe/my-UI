# Codebase Skim: AI Optimization & Presentation

## Core Modules & Touchpoints

1. **AI Subsystem (`crates/server/src/ai/`)**:
   - `mod.rs`: `AiService` coordinates `AiDb`, `ArticleExtractor`, and `LlmCliRunner`. Currently exposes `get_or_create_summary`. Needs methods to fetch cached/batch suggested titles and briefs, or single combined generation.
   - `db.rs`: Manages SQLite pool via `r2d2_sqlite`. Table `article_ai_summaries` currently has `(entry_id, summary, model, created_at)`. Schema version is 1. Needs migration to version 2 adding column `suggested_title TEXT`. Needs query helpers to retrieve suggested title and brief (both individually and batch for page entries).
   - `llm.rs`: `LlmCliRunner` wraps external CLI command with concurrency semaphore (limit 2) and timeout (5s). Prompt currently asks only for a summary (<35 words). Needs combined prompt format requesting `TITLE:` and `BRIEF:`, response parsing, and structured `tracing` debug logs.
   - `extractor.rs`: Scrapes `<article>` or `<body>`, strips boilerplate, truncates words (currently 500 words). Can truncate to 250 words to save tokens.

2. **Routes & Handlers (`crates/server/src/routes/entries.rs`)**:
   - `render_entries_list`: Queries GraphQL for entries, constructs `EntryViewModel`s, and renders `EntryListTemplate`. If AI is enabled, can fetch cached suggested titles and page brief or attach them to the view model.
   - `get_entry_detail_htmx`: Renders `EntryDetailTemplate`. Currently passes Miniflux raw summary into `brief_card.html`. Should query `AiDb` / `AiService` for the Executive Brief when AI is enabled.
   - `get_entry_ai_summary_htmx`: HTMX endpoint for lazy-loading AI summary card.

3. **Templates & Views (`crates/ui-components/`)**:
   - `templates.rs`: `EntryViewModel` has `title`, `summary`, etc. Can include `suggested_title: Option<String>` and/or `original_title: Option<String>`. `EntryListTemplate` can include `page_brief: Option<String>`.
   - `templates/partials/entry_list.html`: Top summary card for Executive Briefs; rows render suggested title with original title on hover / fallback.
   - `templates/partials/entry_item.html`: Single row rendering.
   - `templates/partials/brief_card.html`: Renders Executive Brief in details pane.

## Dependencies & Test Infrastructure

- `crates/server/Cargo.toml`: has `rusqlite`, `r2d2`, `r2d2_sqlite`, `tracing`, `tokio`, `askama`.
- `crates/server/src/ai/db.rs` and `llm.rs`: existing unit tests cover pool initialization, WAL mode, upsert, prompt formatting.

## Result

Result: done
