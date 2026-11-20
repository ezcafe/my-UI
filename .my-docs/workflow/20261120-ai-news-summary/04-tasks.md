# Tasks: AI News Batch Summary (TDD-Ready)

## Key Takeaways
- Every implementation task begins with a failing unit or integration test before production code is written.
- Tasks cover configuration, connection-pooled SQLite persistence with WAL and upsert, HTML `<article>` tag extraction with fallbacks, LLM CLI execution with concurrency controls, Askama template updates, and Axum HTMX routing with authentication, ownership verification, search query forwarding, and guaranteed 200 replacement swaps.
- Planned tests are fully specified with explicit failing test names and test coverage.

---

## Planned Tests

| Case | Failing test name | Covers |
|------|-------------------|--------|
| 1. Config parsing | `test_config_parses_ai_settings_and_defaults` | Parsing `AI_ENABLED`, `LLM_CLI_CMD`, `LLM_MODEL`, `LLM_EXTRA_ARGS`, and `AI_DB_PATH` with safe defaults |
| 2. SQLite schema versioning | `test_ai_db_initializes_schema_and_user_version` | Creating `article_ai_summaries` table, `created_at` index, and setting `PRAGMA user_version = 1` |
| 3. SQLite connection pool concurrency | `test_ai_db_concurrent_pool_reads_under_wal` | Verifying parallel reads across `r2d2` connection pool with WAL mode and `busy_timeout=5000` |
| 4. SQLite upsert on conflict | `test_ai_db_upsert_on_conflict_updates_summary` | Verifying `ON CONFLICT(entry_id) DO UPDATE` avoids constraint errors and updates existing cache |
| 5. `<article>` tag extraction | `test_extract_article_tag_content_stripping_boilerplate` | Parsing `<article>` tag, stripping scripts/styles/nav, and extracting body text |
| 6. Extraction fallback (no tag) | `test_extract_article_fallback_when_article_tag_absent` | Graceful fallback to `<body>` text or RSS teaser when `<article>` tag is missing |
| 7. Scrape block / 403 fallback | `test_extract_article_fallback_on_http_error_or_block` | Immediate fallback to RSS Title + teaser on 403, Cloudflare, paywall, or timeout |
| 8. Word truncation | `test_extract_article_content_word_truncation` | Limiting extracted content to Title + first 500 words for the LLM prompt |
| 9. LLM prompt & CLI arguments | `test_llm_cli_prompt_formatting_and_args` | CLI argument formatting, prompt delimiters, and zero reasoning budget flag |
| 10. LLM failure fallback | `test_llm_cli_fallback_to_extractive_summary_on_failure` | Instant fallback to extractive sentence segmentation on CLI error or timeout, recording fallback model |
| 11. CLI concurrency throttling | `test_llm_cli_concurrency_throttling` | Semaphore bounding to 2 concurrent CLI subprocesses |
| 12. AI button in filter bar | `test_index_template_renders_ai_button_when_enabled` | Conditional rendering of `[AI]` button before `[Unread]` based on `ai_enabled` |
| 13. Grouped batch template | `test_ai_grouped_list_template_renders_feed_headers_and_canonical_route` | Grouping 15 articles by feed source and rendering `hx-trigger="revealed"` targeting `/hx/entries/{id}/ai-summary` |
| 14. Route `GET /hx/entries?status=ai` | `test_hx_entries_ai_status_forwards_search_query` | Axum handler returning feed-grouped batch of 15 items and forwarding `query.search` to GraphQL |
| 15. Route auth & ownership verification | `test_hx_entries_ai_summary_requires_auth_and_verifies_ownership` | Enforcing `AuthenticatedUser` and verifying entry ownership via Miniflux before accessing cache/LLM |
| 16. Guaranteed 200 outerHTML swap | `test_hx_entries_ai_summary_guarantees_200_swap_on_failure` | Returning HTTP 200 with HTML replacement card on failure/fallback so HTMX skeleton never freezes |

---

## Detailed Task Breakdown

### Task 1: Configuration & Environment Setup
- **Behavior**: Add `AI_ENABLED`, `LLM_CLI_CMD`, `LLM_MODEL`, `LLM_EXTRA_ARGS`, and `AI_DB_PATH` fields to `Config` struct in [crates/server/src/config.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/config.rs). Update `.env.example` with placeholders.
- **Failing test name**: `test_config_parses_ai_settings_and_defaults`
- **Files allowed**:
  - `crates/server/src/config.rs`
  - `.env.example`
- **Done evidence**: `cargo test -p server test_config_parses_ai_settings_and_defaults` passes.

### Task 2: SQLite Persistence Layer (`crates/server/src/ai/db.rs`)
- **Behavior**: Add `rusqlite`, `r2d2`, and `r2d2_sqlite` to `crates/server/Cargo.toml`. Create `AiDb` struct managing `article_ai_summaries` table with an `r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>` (pool size 8). Configure `PRAGMA journal_mode = WAL;`, `PRAGMA busy_timeout = 5000;`, and `PRAGMA synchronous = NORMAL;`. Apply `PRAGMA user_version = 1;` schema initialization with index on `created_at`. Implement async-friendly `get_summary(entry_id)` and `save_summary(entry_id, summary, model)` using `ON CONFLICT(entry_id) DO UPDATE SET summary = excluded.summary, model = excluded.model, created_at = excluded.created_at`.
- **Failing test names**:
  - `test_ai_db_initializes_schema_and_user_version`
  - `test_ai_db_concurrent_pool_reads_under_wal`
  - `test_ai_db_upsert_on_conflict_updates_summary`
- **Files allowed**:
  - `crates/server/Cargo.toml`
  - `crates/server/src/ai/mod.rs`
  - `crates/server/src/ai/db.rs`
- **Done evidence**: `cargo test -p server test_ai_db_` passes.

### Task 3: Article Content Extractor & Fallback Engine (`crates/server/src/ai/extractor.rs`)
- **Behavior**: Implement `ArticleExtractor` using `reqwest` and `scraper`. Parse `<article>` tag content, strip inner tags/scripts/nav, falling back to body/teaser if `<article>` tag is absent. Enforce 3-second HTTP timeout; on 403/Cloudflare/paywall/error, immediately fall back to RSS Title + teaser. Truncate to 500 words.
- **Failing test names**:
  - `test_extract_article_tag_content_stripping_boilerplate`
  - `test_extract_article_fallback_when_article_tag_absent`
  - `test_extract_article_fallback_on_http_error_or_block`
  - `test_extract_article_content_word_truncation`
- **Files allowed**:
  - `crates/server/Cargo.toml`
  - `crates/server/src/ai/extractor.rs`
- **Done evidence**: `cargo test -p server test_extract_article_` passes.

### Task 4: LLM CLI Runner & Concurrency Guard (`crates/server/src/ai/llm.rs`)
- **Behavior**: Implement `LlmCliRunner`. Formats prompt with strict length constraint (<35 words, 1–2 sentences). Spawns CLI subprocess with 5-second timeout and `kill_on_drop(true)`. Uses `Arc<tokio::sync::Semaphore>` (2 permits) for throttling. On any failure, falls back to `miniflux_client::summary::extract_summary` and records model as `"extractive-fallback"`.
- **Failing test names**:
  - `test_llm_cli_prompt_formatting_and_args`
  - `test_llm_cli_fallback_to_extractive_summary_on_failure`
  - `test_llm_cli_concurrency_throttling`
- **Files allowed**:
  - `crates/server/src/ai/llm.rs`
- **Done evidence**: `cargo test -p server test_llm_cli_` passes.

### Task 5: UI Templates & Filter Bar (`crates/ui-components`)
- **Behavior**: Update `IndexTemplate` in `crates/ui-components/src/templates.rs` and `templates/pages/index.html` to conditionally render the `[AI]` button before `[Unread]`. Create `ai_grouped_list.html` with canonical lazy route `hx-get="/hx/entries/{{ item.id }}/ai-summary"`, and create `ai_summary_card.html`.
- **Failing test names**:
  - `test_index_template_renders_ai_button_when_enabled`
  - `test_ai_grouped_list_template_renders_feed_headers_and_canonical_route`
- **Files allowed**:
  - `crates/ui-components/src/templates.rs`
  - `crates/ui-components/templates/pages/index.html`
  - `crates/ui-components/templates/partials/ai_grouped_list.html`
  - `crates/ui-components/templates/partials/ai_summary_card.html`
- **Done evidence**: `cargo test -p ui-components` passes.

### Task 6: Server Routing & Endpoints (`crates/server/src/routes/entries.rs`)
- **Behavior**: Wire `AiService` into `AppState`. Implement `status=ai` query in `routes/entries.rs` to fetch 15 latest articles, group by feed source, support `query.search` forwarding to GraphQL, and render `AiGroupedListTemplate`. Implement canonical route `GET /hx/entries/{id}/ai-summary` with `AuthenticatedUser` requirement, Miniflux entry ownership verification, and guaranteed HTTP 200 outerHTML swap response (even on failure) to prevent UI skeleton freezing.
- **Failing test names**:
  - `test_hx_entries_ai_status_forwards_search_query`
  - `test_hx_entries_ai_summary_requires_auth_and_verifies_ownership`
  - `test_hx_entries_ai_summary_guarantees_200_swap_on_failure`
- **Files allowed**:
  - `crates/server/src/app.rs`
  - `crates/server/src/routes/entries.rs`
  - `crates/server/src/routes/mod.rs`
  - `crates/server/tests/e2e_tests.rs`
- **Done evidence**: `cargo test -p server` and integration tests pass.
