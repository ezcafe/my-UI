Result: success
Stage: test-full
Test path: full
Updated: 2026-11-22T15:26:30+07:00

# Test & Verification Log: AI Usage Optimization & Executive Presentation

- **Workflow**: `20261122-optimize-ai-usage`
- **Stage**: `test-full`
- **Test path**: `full`
- **Result**: `success`
- **Updated**: `2026-11-22T15:26:30+07:00`

---

### Smoke: smoke-pass

- **Execution Command**: `cargo test --workspace`
- **Result**: `smoke-pass`
- **Test Suites Verified**:
  - `miniflux-client`: 23 passed; 0 failed
  - `server`: 77 passed (73 unit/integration + 4 e2e); 0 failed
  - `ui-components`: 44 passed; 0 failed
  - **Total**: 144 tests passed; 0 failed
- **Key Verifications**:
  1. **Server Logging (`crates/server/src/ai/llm.rs`)**:
     - Verified structured `tracing::info!` for outbound prompt preview (first 200 chars), model, and item count before CLI invocation.
     - Verified structured `tracing::info!` for elapsed duration (ms) and raw response text upon return.
     - Verified structured `tracing::warn!` for JSON parsing issues and fallback activations (regex, extractive).
     - Verified structured `tracing::error!` for CLI timeouts, process failure exits (with stderr preview), or empty outputs.
     - Passed: `ai::llm::tests::test_server_logging_format`.
  2. **Single-Call Batch Prefetch (`crates/server/src/ai/prefetch.rs`)**:
     - Chunking into 12 items removed; all uncached entries for the page are processed in a single LLM call.
     - `unread_count > 20` prefetch gate removed.
     - Persists page brief (`top`) and item records (`t`, `b`) to SQLite in 1 call.
     - Passed: `ai::prefetch::tests::test_page_batch_processing`.
  3. **Filter-Specific Briefs & Removal of AI Filter**:
     - Removed `<button id="filter-ai">` from `crates/ui-components/templates/pages/index.html`.
     - Passed: `ui_components::templates::tests::test_index_template_no_ai_filter`.
     - Filter-specific Top Executive Brief: Only attach `top_executive_brief` when query status is "unread" (or None default) or "today". For "read", "starred", "all", or search without unread/today, set `top_executive_brief = None`.
     - Prefetch trigger: Background prefetch job is only spawned for "unread" and "today" filters.
     - Passed: `routes::entries::tests::test_filter_specific_brief` and `routes::tests::test_filter_specific_brief_route_integration`.

---

## 1. Full Test Suite Verification (`cargo test --workspace`)

Executed full workspace unit, integration, and e2e test suite across all workspace crates:

### Crate: `miniflux-client` (23 passed)
- `client::tests::test_get_me_caches_user_id` ... ok
- `client::tests::test_miniflux_client_get_entries_published_after` ... ok
- `client::tests::test_get_entries_success` ... ok
- `client::tests::test_mark_feed_as_read_endpoint` ... ok
- `client::tests::test_get_feeds_merges_unread_counters` ... ok
- `client::tests::test_get_entry_success` ... ok
- `client::tests::test_mark_all_as_read_uses_cached_user_id` ... ok
- `client::tests::test_update_entry_status_sends_batch_payload` ... ok
- `client::tests::test_toggle_bookmark_mutation` ... ok
- `schema::tests::test_graphql_invalid_filter_arguments` ... ok
- `schema::tests::test_graphql_mutation_update_entries_status` ... ok
- `schema::tests::test_graphql_mutation_mark_feed_as_read` ... ok
- `schema::tests::test_graphql_entry_summary_resolved` ... ok
- `schema::tests::test_entries_result_returns_total_and_entries` ... ok
- `schema::tests::test_graphql_query_entries` ... ok
- `schema::tests::test_graphql_mutation_mark_all_as_read` ... ok
- `summary::tests::test_extractive_summary_returns_first_two_sentences` ... ok
- `summary::tests::test_summary_extractor_strips_html_tags` ... ok
- `schema::tests::test_graphql_query_entries_published_after_and_search` ... ok
- `schema::tests::test_today_counter_returns_correct_count` ... ok
- `schema::tests::test_graphql_query_single_entry` ... ok
- `schema::tests::test_graphql_query_feeds_includes_unread_count` ... ok
- `client::tests::test_miniflux_timeout_handling` ... ok

### Crate: `server` (73 unit & integration passed + 4 e2e passed)
- `ai::db::tests::test_ai_db_initializes_schema_and_user_version` ... ok
- `ai::db::tests::test_sqlite_wal_mode_active` ... ok
- `ai::db::tests::test_ai_db_upsert_on_conflict_updates_summary` ... ok
- `ai::db::tests::test_ai_db_concurrent_pool_reads_under_wal` ... ok
- `ai::db::tests::test_migration_v1_to_v2` ... ok
- `ai::db::tests::test_batch_summaries_and_page_brief` ... ok
- `ai::extractor::tests::test_extract_article_tag_content_stripping_boilerplate` ... ok
- `ai::extractor::tests::test_extract_article_fallback_when_article_tag_absent` ... ok
- `ai::extractor::tests::test_extract_article_fallback_on_http_error_or_block` ... ok
- `ai::extractor::tests::test_extractor_parses_content_with_lazylock_selectors` ... ok
- `ai::extractor::tests::test_word_truncation_250` ... ok
- `ai::llm::tests::test_batch_prompt_formatting_and_args` ... ok
- `ai::llm::tests::test_llm_cli_concurrency_throttling` ... ok
- `ai::llm::tests::test_llm_cli_prompt_formatting_and_args` ... ok
- `ai::llm::tests::test_llm_cli_fallback_to_extractive_summary_on_failure` ... ok
- `ai::llm::tests::test_server_logging_format` ... ok
- `ai::llm::tests::test_parse_batch_response` ... ok
- `ai::prefetch::tests::test_page_batch_processing` ... ok
- `auth::session::tests::test_session_data_size_under_cookie_limit` ... ok
- `auth::temp_cookie::tests::test_oidc_temp_cookie_encryption_and_roundtrip` ... ok
- `auth::tests::test_authenticated_user_bypasses_when_auth_disabled` ... ok
- `auth::tests::test_expired_session_rejected` ... ok
- `auth::tests::test_htmx_unauthenticated_request_returns_hx_redirect` ... ok
- `auth::tests::test_oidc_callback_state_mismatch_fails` ... ok
- `auth::tests::test_auth_routes_redirect_to_root_when_auth_disabled` ... ok
- `auth::tests::test_unauthenticated_graphql_request_rejected_with_401` ... ok
- `config::tests::test_config_ai_disabled_when_auth_disabled` ... ok
- `cache::tests::test_miniflux_cache_bounded_capacity` ... ok
- `config::tests::test_config_allows_missing_oidc_when_auth_disabled` ... ok
- `auth::tests::test_unauthenticated_request_redirects_to_login` ... ok
- `config::tests::test_config_ai_matrix_permutations` ... ok
- `config::tests::test_config_defaults` ... ok
- `config::tests::test_config_fails_when_secret_too_short` ... ok
- `config::tests::test_config_parses_ai_settings_and_defaults` ... ok
- `config::tests::test_config_loads_from_valid_env` ... ok
- `config::tests::test_config_parses_auth_enabled_and_defaults` ... ok
- `error::tests::test_handle_htmx_error_returns_clean_toast_without_debug_dump` ... ok
- `error::tests::test_user_facing_error_messages_do_not_leak_internal_details` ... ok
- `rate_limit::tests::test_rate_limiter_parameter_validation_defaults` ... ok
- `rate_limit::tests::test_rate_limiter_allows_burst_and_throttles_excess_requests` ... ok
- `routes::entries::tests::test_entries_filter_query_mapping` ... ok
- `routes::entries::tests::test_build_graphql_filter_search_defaults_to_all` ... ok
- `routes::entries::tests::test_build_graphql_filter_guest_unread_scopes_to_today` ... ok
- `routes::entries::tests::test_filter_specific_brief` ... ok
- `routes::tests::test_filter_specific_brief_route_integration` ... ok
- `routes::tests::test_get_root_page_serves_html_shell` ... ok
- `routes::tests::test_hx_entries_filter_query_params` ... ok
- `routes::tests::test_hx_entries_filter_today_and_search` ... ok
- `routes::tests::test_hx_read_returns_oob_unread_pill` ... ok
- `routes::tests::test_hx_mark_page_read_endpoint` ... ok
- `routes::tests::test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled` ... ok
- `routes::tests::test_list_entries_serves_from_cache_when_cached` ... ok
- `routes::tests::test_list_entries_loads_suggested_titles_and_page_brief_from_db` ... ok
- `routes::tests::test_mark_feed_read_htmx` ... ok
- `routes::tests::test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled` ... ok
- `auth::tests::test_oidc_callback_clears_temp_cookie` ... ok
- `auth::tests::test_secure_cookie_flag_when_https` ... ok
- `cache::tests::test_miniflux_cache_get_and_insert_with_ttl` ... ok
- `routes::tests::test_hx_entries_ai_summary_guarantees_200_swap_on_failure` ... ok
- `routes::tests::test_hx_entries_ai_summary_requires_auth_and_verifies_ownership` ... ok
- `routes::tests::test_hx_bookmark_returns_updated_icon` ... ok
- `routes::tests::test_hx_bookmark_toggle_to_unbookmarked` ... ok
- `routes::tests::test_mark_feed_read_htmx` ... ok
- `routes::tests::test_get_entry_detail_htmx_fetches_single_entry` ... ok
- `routes::tests::test_get_entry_detail_auto_marks_as_read_and_returns_oob_swaps` ... ok
- `routes::tests::test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled` ... ok
- `routes::tests::test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled` ... ok
- `routes::tests::test_htmx_error_returns_oob_toast_without_reswapping_target` ... ok
- `routes::tests::test_hx_entries_ai_summary_disabled_when_ai_flag_off` ... ok
- `routes::tests::test_hx_entries_ai_summary_disabled_in_guest_mode` ... ok
- `routes::tests::test_hx_entries_ai_status_forwards_search_query` ... ok
- `e2e_tests::test_app_starts_and_responds_to_health_check` ... ok
- `e2e_tests::test_mutation_endpoints_reject_with_403_when_auth_disabled` ... ok
- `e2e_tests::test_e2e_guest_mode_flow` ... ok
- `e2e_tests::test_e2e_authenticated_session_flow` ... ok

### Crate: `ui-components` (44 passed)
- `templates::tests::test_ai_grouped_list_template_renders_feed_headers_and_canonical_route` ... ok
- `templates::tests::test_detail_template_omits_author` ... ok
- `templates::tests::test_entry_detail_renders_local_actions_when_auth_disabled` ... ok
- `templates::tests::test_entry_detail_renders_htmx_actions_when_auth_enabled` ... ok
- `templates::tests::test_detail_template_has_mark_unread_button` ... ok
- `templates::tests::test_annuaire_theme_tokens_and_toggle_present` ... ok
- `templates::tests::test_entry_detail_wires_close_detail_panel` ... ok
- `templates::tests::test_entry_item_wires_open_entry_detail` ... ok
- `templates::tests::test_entry_list_hides_pagination_when_no_news` ... ok
- `templates::tests::test_entry_list_and_item_render_suggested_title_and_brief` ... ok
- `templates::tests::test_entry_list_hides_pagination_when_single_page` ... ok
- `templates::tests::test_entry_list_wires_open_entry_detail` ... ok
- `templates::tests::test_entry_list_renders_pagination_when_multiple_pages` ... ok
- `templates::tests::test_filter_click_closes_detail_panel` ... ok
- `templates::tests::test_htmx_indicator_hidden_by_default` ... ok
- `templates::tests::test_filter_bar_reordering` ... ok
- `templates::tests::test_index_template_defines_all_articles_store_and_helpers` ... ok
- `templates::tests::test_index_template_hides_logout_button_when_auth_disabled` ... ok
- `templates::tests::test_index_template_defines_mark_article_read_and_data_base_unread` ... ok
- `templates::tests::test_index_template_hides_starred_but_shows_all_titles_when_auth_disabled` ... ok
- `templates::tests::test_index_template_loading_indicator_and_filter_cache` ... ok
- `templates::tests::test_index_template_defines_detail_panel_helpers` ... ok
- `templates::tests::test_index_template_mark_page_read_button_auth_toggle` ... ok
- `templates::tests::test_index_template_omits_title_unread_counting` ... ok
- `templates::tests::test_index_template_renders_today_and_unread_counts` ... ok
- `templates::tests::test_index_template_skeleton_structure` ... ok
- `templates::tests::test_index_template_no_ai_filter` ... ok
- `sanitizer::tests::test_sanitizer_removes_dangerous_tags` ... ok
- `templates::tests::test_mobile_detail_body_removes_padding_right_container_query` ... ok
- `templates::tests::test_mobile_entries_list_and_items_reduce_horizontal_padding_container_query` ... ok
- `templates::tests::test_login_page_renders_centered_card` ... ok
- `sanitizer::tests::test_sanitizer_preserves_links_and_attributes_with_lazylock` ... ok
- `templates::tests::test_open_in_browser_link_wires_mark_article_read` ... ok
- `templates::tests::test_index_template_renders_guest_state_script_when_auth_disabled` ... ok
- `templates::tests::test_mobile_searchbox_compact_and_expand_container_query` ... ok
- `templates::tests::test_render_entry_list_with_title_only_items` ... ok
- `templates::tests::test_render_toast_oob_template` ... ok
- `templates::tests::test_render_title_only_entry_item` ... ok
- `templates::tests::test_spatial_shell_and_window_mobile_container_queries` ... ok
- `templates::tests::test_view_models` ... ok
- `templates::tests::test_spatial_window_desktop_card_preserved` ... ok
- `sanitizer::tests::test_sanitizer_adds_noopener_to_links` ... ok
- `templates::tests::test_spatial_window_fluid_width_container_query` ... ok
- `sanitizer::tests::test_sanitizer_multiple_links_all_receive_target_blank` ... ok

---

## 2. Behavioral Verification Summary

- **Scenario 1: Single-Page Background Batch Processing**
  - Chunking removed; single-call batch processes all uncached entries on the rendered list page.
  - Verified with `test_page_batch_processing`.

- **Scenario 2: Structured Server Logging**
  - Outbound prompt preview, model, and item count logged via `info!`.
  - Elapsed duration and raw response output logged via `info!`.
  - JSON parse fallback and extractive fallback logged via `warn!`.
  - CLI timeouts, process failure exits, and empty output logged via `error!`.
  - Verified with `test_server_logging_format`.

- **Scenario 3: Filter-Specific Top Brief & AI Filter Removal**
  - `<button id="filter-ai">` completely removed from `index.html`.
  - Top Executive Brief card only loaded for "unread" and "today" filters.
  - Background batch prefetch only triggered for "unread" and "today" filters.
  - Verified with `test_index_template_no_ai_filter`, `test_filter_specific_brief`, and `test_filter_specific_brief_route_integration`.

---

## 3. Acceptance Criteria Checklist

- [x] Schema migration v1 -> v2 (`suggested_title`, `page_executive_briefs`) implemented and tested.
- [x] ArticleExtractor 250-word truncation limit implemented and tested.
- [x] LlmCliRunner micro-batch prompt formatting & resilient JSON parsing implemented and tested.
- [x] Structured server logging (`info!`, `warn!`, `error!`) implemented and tested.
- [x] AiPrefetchService single-call page batch prefetch (no chunking, no unread threshold) implemented and tested.
- [x] `<button id="filter-ai">` completely removed from `index.html`.
- [x] Filter-specific Top Executive Brief and prefetch trigger implemented and tested.
- [x] All 144 workspace unit, integration, and e2e tests pass (0 failures).
