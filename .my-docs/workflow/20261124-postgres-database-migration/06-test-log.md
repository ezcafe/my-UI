# Test Log: Migration from SQLite to PostgreSQL 19

- **Run slug:** `20261124-postgres-database-migration`
- **Updated:** 2026-11-24T10:08:30+07:00
- **Stage:** test-full
- **Result:** success
- **Test path:** full

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 117 | 117 | 0 | PASSED |
| `server` (integration / e2e) | 7 | 7 | 0 | PASSED |
| `ui-components` (unit & templates) | 56 | 56 | 0 | PASSED |
| `doc-tests` (all crates) | 0 | 0 | 0 | PASSED |
| **Total** | **203** | **203** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace -- -D warnings`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

- **Command:** `cargo fmt --check`
- **Exit Code:** 0
- **Status:** PASSED (clean formatting)

- **Command:** `docker-compose -f docker-compose.yml config -q && docker-compose -f docker-compose.local.yml config -q`
- **Exit Code:** 0
- **Status:** PASSED (valid compose configurations)

---

## 3. Full Test Suite Coverage Details

### Miniflux Client (`miniflux-client`)
- 23 unit tests verifying:
  - Cache handling for user identity (`test_get_me_caches_user_id`, `test_mark_all_as_read_uses_cached_user_id`).
  - Feed and entry operations (`test_get_entries_success`, `test_get_entry_success`, `test_get_feeds_merges_unread_counters`).
  - Mutations and status toggling (`test_toggle_bookmark_mutation`, `test_update_entry_status_sends_batch_payload`, `test_mark_feed_as_read_endpoint`).
  - GraphQL schema queries and mutations (`test_graphql_query_entries`, `test_graphql_mutation_mark_all_as_read`, `test_graphql_query_feeds_includes_unread_count`, `test_today_counter_returns_correct_count`).
  - Summary extraction utilities (`test_extractive_summary_returns_first_two_sentences`, `test_summary_extractor_strips_html_tags`).
  - Network and timeout handling (`test_miniflux_timeout_handling`).

### Server Core & PostgreSQL Service (`server`)
- 117 unit tests + 7 integration/e2e tests verifying:
  - **PostgreSQL Connection & Schema Bootstrap:** `test_ai_db_connection_and_schema_init` verifies non-blocking connection pool initialization and idempotent DDL setup (`article_ai_summaries`, `page_executive_briefs_v2`, indexes).
  - **Full-Text Search:** `test_ai_db_fts_tsvector_search` verifies `to_tsvector` stored generated column and GIN index queries with `plainto_tsquery` ranking.
  - **Time-Window Filtering:** `test_ai_db_time_window_recent_summaries` verifies chronological range filtering on `created_at DESC`.
  - **Summary Point & Batch Upsert:** `test_ai_db_summary_save_and_retrieve`, `test_ai_db_batch_summaries_and_unprocessed` verify `ON CONFLICT (entry_id) DO UPDATE` and PostgreSQL array binding (`ANY($1::bigint[])`).
  - **Executive Page Briefs & Retention:** `test_ai_db_page_briefs_and_pruning` verifies keyed page brief storage, retrieval, and parameterized date interval pruning.
  - **Adversarial Input & Boundary Checks:** `test_ai_db_edge_cases_empty_query_and_nonpositive_limits` verifies safe short-circuiting for empty queries, non-positive limits, and clamp limits.
  - **Configuration & Credential Masking:** `test_config_database_url_from_env_or_default`, `test_config_mask_database_url_in_fmt` verify `DATABASE_URL` parsing and redaction of credentials in debug logs.
  - **App Wiring & Integration:** `test_app_state_ai_db_wiring`, `test_health_endpoint`, auth flows, LLM dispatchers, and rate limiting.

### UI Components & Templates (`ui-components`)
- 56 tests verifying:
  - SSR template rendering (executive briefs, article cards, magazine grid, reader view).
  - Responsive container queries and theme tokens (`annuaire`).
  - HTML sanitization and link security attributes (`rel="noopener"`).
