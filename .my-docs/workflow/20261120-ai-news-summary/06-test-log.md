# Test Execution Log

- **Result:** success
- **Test path:** full
- **Date:** 2026-11-20
- **Execution:** `cargo test --workspace` (71 passed, 0 failed, 0 ignored)

## Test-Full Execution

- **Stage:** `test-full`
- **Status:** pass
- **Command:** `cargo test --workspace`
- **Result Output:**
  - `miniflux-client`: 20 passed; 0 failed
  - `server` (lib): 38 passed; 0 failed
  - `server` (e2e integration): 2 passed; 0 failed
  - `ui-components`: 11 passed; 0 failed
  - Total: 71 passed; 0 failed; 0 ignored

## Smoke

- **Smoke source:** Smoke-Task
- **Status:** pass
- **Command:** `cargo test -p server test_hx_entries_ai && cargo test -p ui-components && cargo test --workspace`
- **Output:** 71 passed, 0 failed, 0 ignored (independent smoke verification)
- **Verified Areas:**
  - AI status query forwarding and search filter routing (`test_hx_entries_ai_status_forwards_search_query`)
  - HTMX AI summary endpoint auth enforcement and ownership verification (`test_hx_entries_ai_summary_requires_auth_and_verifies_ownership`)
  - 200 outerHTML swap guarantee on LLM extraction failure (`test_hx_entries_ai_summary_guarantees_200_swap_on_failure`)
  - Dynamic AI filter button rendering in index navigation (`test_index_template_renders_ai_button_when_enabled`)
  - AI grouped feed list template rendering with canonical reveal routes (`test_ai_grouped_list_template_renders_feed_headers_and_canonical_route`)

---

## Summary

Targeted TDD test suite and integration tests executed across all workspace crates. All 71 tests passed cleanly with 0 failures or regressions.

| Crate / Target | Tests | Passed | Failed | Result |
|---|---|---|---|---|
| `miniflux-client` (lib) | 20 | 20 | 0 | pass |
| `server` (lib) | 38 | 38 | 0 | pass |
| `server` (e2e integration) | 2 | 2 | 0 | pass |
| `ui-components` (lib) | 11 | 11 | 0 | pass |
| **Total** | **71** | **71** | **0** | **success** |

---

## Detailed Test Breakdown

### 1. `server` AI Tests (15 new passed)
- `test_config_parses_ai_settings_and_defaults`: Verifies parsing of `AI_ENABLED`, `LLM_CLI_CMD`, `LLM_MODEL`, `LLM_EXTRA_ARGS`, and `AI_DB_PATH` with fallback defaults.
- `test_ai_db_initializes_schema_and_user_version`: Verifies SQLite schema creation, index creation, and `PRAGMA user_version = 1`.
- `test_ai_db_concurrent_pool_reads_under_wal`: Validates parallel reads across `r2d2` connection pool with WAL mode and `busy_timeout=5000`.
- `test_ai_db_upsert_on_conflict_updates_summary`: Confirms `ON CONFLICT(entry_id) DO UPDATE` updates summary and model without constraint errors.
- `test_extract_article_tag_content_stripping_boilerplate`: Validates DOM parsing of `<article>` tags while stripping `<script>`, `<style>`, `<nav>`, `<header>`, `<footer>`, `<aside>`.
- `test_extract_article_fallback_when_article_tag_absent`: Confirms fallback to `<body>` text when `<article>` tag is missing.
- `test_extract_article_fallback_on_http_error_or_block`: Validates immediate fallback to title + teaser excerpt on 403 Forbidden or network errors.
- `test_extract_article_content_word_truncation`: Verifies word truncation capped at 500 words for the LLM prompt.
- `test_llm_cli_prompt_formatting_and_args`: Validates CLI argument construction (`-m <model> <extra_args>`) and prompt structure under 35 words.
- `test_llm_cli_fallback_to_extractive_summary_on_failure`: Confirms graceful fallback to extractive 2-sentence summary and `"extractive-fallback"` model tag when subprocess fails or times out.
- `test_llm_cli_concurrency_throttling`: Tests bounded concurrency semaphore limiting active executions to 2 permits.
- `test_hx_entries_ai_status_forwards_search_query`: Validates `GET /hx/entries?status=ai` forwarding `search` queries to GraphQL and rendering feed-grouped list with lazy-load routes.
- `test_hx_entries_ai_summary_requires_auth_and_verifies_ownership`: Validates authentication enforcement and entry ownership check before cache or LLM access.
- `test_hx_entries_ai_summary_guarantees_200_swap_on_failure`: Verifies guaranteed HTTP 200 outerHTML swap response even on failure, ensuring HTMX skeleton never freezes.

### 2. `ui-components` AI Tests (2 new passed)
- `test_index_template_renders_ai_button_when_enabled`: Verifies conditional rendering of `[AI]` button before `[Unread]` based on `ai_enabled`.
- `test_ai_grouped_list_template_renders_feed_headers_and_canonical_route`: Verifies feed grouping and canonical lazy route `hx-get="/hx/entries/{id}/ai-summary"` with `hx-trigger="revealed"` and `hx-swap="outerHTML"`.

### 3. Upstream & Regression Tests (54 passed)
- All 20 tests in `miniflux-client` passed.
- All 23 pre-existing tests in `server` and 2 `e2e_tests` passed.
- All 9 pre-existing tests in `ui-components` passed.
