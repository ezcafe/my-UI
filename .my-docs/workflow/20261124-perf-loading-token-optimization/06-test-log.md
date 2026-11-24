# Test Log: Performance, Loading, and Token Optimization

- **Run slug:** `20261124-perf-loading-token-optimization`
- **Updated:** 2026-11-24T07:52:00+07:00
- **Test path:** full
- **Result:** success

---

## 1. Test Suite Verification Summary (Workspace Full Run)

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 128 | 128 | 0 | PASSED |
| `server` (integration / e2e) | 7 | 7 | 0 | PASSED |
| `ui-components` (unit & templates) | 64 | 64 | 0 | PASSED |
| `miniflux-client` (doc-tests) | 0 | 0 | 0 | PASSED |
| `server` (doc-tests) | 0 | 0 | 0 | PASSED |
| `ui-components` (doc-tests) | 0 | 0 | 0 | PASSED |
| **Total** | **222** | **222** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Full Test Execution Breakdown

### `miniflux-client` (23 tests passed)
- Client authentication caching, user ID memoization (`test_get_me_caches_user_id`, `test_mark_all_as_read_uses_cached_user_id`).
- Feeds and unread counter aggregation (`test_get_feeds_merges_unread_counters`, `test_graphql_query_feeds_includes_unread_count`).
- Entry pagination, search, and published-after filtering (`test_miniflux_client_get_entries_published_after`, `test_graphql_query_entries_published_after_and_search`).
- Bookmark toggling and status mutations (`test_toggle_bookmark_mutation`, `test_update_entry_status_sends_batch_payload`).
- Summary extraction and HTML sanitization (`test_extractive_summary_returns_first_two_sentences`, `test_summary_extractor_strips_html_tags`).

### `server` Unit Tests (128 tests passed)
- **AI Database & Migrations:** Schema v4 migration (`test_migration_v3_to_v4_adds_rewrite_column`), WAL mode configuration (`test_sqlite_wal_mode_active`), concurrent pool reads (`test_ai_db_concurrent_pool_reads_under_wal`), upsert with rewrite preservation (`test_save_and_get_summary_with_rewrite`, `test_save_batch_results_transaction`).
- **Extractor & Fast Path:** 25-word teaser bypass (`test_extractor_teaser_fast_path_25_words`, `test_extractor_teaser_fast_path_skips_http`), 130-word content truncation (`test_extractor_word_truncation_130`), HTTP error and block fallback (`test_extract_article_fallback_on_http_error_or_block`).
- **LLM Prompt & Parser:** Eager rewrite for top-1 story only (`test_format_batch_prompt_eager_rewrite_top_1_only`), strict grounding constraints (`test_format_batch_prompt_strict_grounding_constraint`), JSON parsing with rewrite extraction (`test_parse_batch_response_with_rewrite_json`), multilingual support and extractive fallback.
- **Prefetch & Service:** Canonical page key generation (`test_compute_page_key_consistency`), in-flight request deduplication (`test_ai_service_in_flight_deduplication`), cached briefs handling.
- **Routes & Handlers:** On-demand rewrite endpoint `GET /hx/entries/{id}/rewrite` returning HTML snippet or skeleton, precompiled static CSS endpoint `GET /static/annuaire-theme.css`, mark page read with pagination offset.

### `server` Integration & E2E Tests (7 tests passed)
- `test_app_starts_and_responds_to_health_check`: Server boots cleanly and responds 200 OK on `/health`.
- `test_e2e_page_load_and_detail_drawer_rewrite_flow`: Complete flow from index load, drawer opening, eager top-1 rewrite cache hit, to lazy-loading drawer rewrite fetch.
- `test_graphql_endpoint_resolves_entries_with_filter`: Miniflux backend GraphQL proxying and response delivery.
- `test_authenticated_routes_protection`: Guest and authenticated access guardrails.

### `ui-components` (64 tests passed)
- **Base Template & Assets:** CDN script removal (`test_base_template_excludes_tailwind_cdn_script`), static CSS link inclusion (`test_base_template_includes_annuaire_theme_css`), responsive viewport and container queries.
- **Detail Panel & Rewrite:** Cached rewrite rendering (`test_entry_detail_renders_rewrite_directly_when_cached`), HTMX skeleton placeholder wireup (`test_entry_detail_renders_skeleton_when_rewrite_missing`), close button and keyboard shortcuts.
- **Magazine View & Entry List:** Grid layout CSS, executive brief card, skeleton loader state, pagination controls, offset inputs.
- **Sanitizer:** HTML link attributes (`test_sanitizer_adds_noopener_to_links`), dangerous tag removal (`test_sanitizer_removes_dangerous_tags`).

---

## 4. Smoke Verification Reference

- **Smoke source:** Smoke-Task
- **Result:** smoke-pass
- **Summary:** Verified startup `/health` probe, static CSS route delivery, CDN removal, and end-to-end rewrite flow.
