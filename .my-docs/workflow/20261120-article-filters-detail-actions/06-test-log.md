# Test Execution Log

- **Result:** success
- **Test path:** lite
- **Date:** 2026-11-20

## Summary

Targeted test suite and end-to-end integration tests executed across all workspace crates. All 55 tests passed cleanly with 0 failures or regressions.

| Crate / Target | Tests | Passed | Failed | Result |
|---|---|---|---|---|
| `miniflux-client` (lib) | 20 | 20 | 0 | pass |
| `server` (lib) | 24 | 24 | 0 | pass |
| `server` (e2e integration) | 2 | 2 | 0 | pass |
| `ui-components` (lib) | 9 | 9 | 0 | pass |
| **Total** | **55** | **55** | **0** | **success** |

---

## Detailed Test Breakdown

### 1. `miniflux-client` (20 passed)
- `test_get_entry_success`: Verifies single entry retrieval and parsing.
- `test_get_entries_success`: Verifies entries query with filter criteria.
- `test_get_me_caches_user_id`: Confirms user id cache behavior.
- `test_get_feeds_merges_unread_counters`: Validates unread feed counter aggregation.
- `test_miniflux_client_get_entries_published_after`: Tests published timestamp filters.
- `test_mark_all_as_read_uses_cached_user_id`: Verifies mark all read endpoint.
- `test_mark_feed_as_read_endpoint`: Tests single feed read marking.
- `test_update_entry_status_sends_batch_payload`: Confirms batch PUT payload with multiple entry IDs and status.
- `test_toggle_bookmark_mutation`: Validates bookmark toggling.
- `test_graphql_invalid_filter_arguments`: Tests invalid argument rejection in GraphQL schema.
- `test_graphql_mutation_update_entries_status`: Tests batch GraphQL mutation `updateEntriesStatus`.
- `test_graphql_entry_summary_resolved`: Validates extractive summary resolution.
- `test_graphql_query_entries`: Verifies root entries query.
- `test_graphql_query_entries_published_after_and_search`: Verifies query with both date and search text.
- `test_graphql_mutation_mark_feed_as_read`: Tests GraphQL mutation for feed status.
- `test_extractive_summary_returns_first_two_sentences`: Tests plain text sentence extraction.
- `test_graphql_mutation_mark_all_as_read`: Tests GraphQL mutation for mark all read.
- `test_graphql_query_feeds_includes_unread_count`: Verifies feeds query including unread count field.
- `test_graphql_query_single_entry`: Tests single entry GraphQL resolution.
- `test_miniflux_timeout_handling`: Verifies graceful upstream timeout handling.

### 2. `server` (24 lib + 2 e2e passed)
- `test_oidc_temp_cookie_encryption_and_roundtrip`: Verifies OIDC encryption round-trip.
- `test_session_data_size_under_cookie_limit`: Confirms cookie size within browser limits.
- `test_config_defaults`: Verifies default configuration values.
- `test_config_fails_when_secret_too_short`: Ensures minimum secret length validation.
- `test_config_loads_from_valid_env`: Validates configuration parsing from environment.
- `test_build_graphql_filter_search_defaults_to_all`: Verifies search query defaults to `status=all` when status is unset.
- `test_entries_filter_query_mapping`: Verifies query mapping to internal filter structures.
- `test_unauthenticated_graphql_request_rejected_with_401`: Validates GraphQL auth guard.
- `test_unauthenticated_request_redirects_to_login`: Validates Axum auth middleware redirect.
- `test_oidc_callback_state_mismatch_fails`: Confirms CSRF state check on callback.
- `test_htmx_unauthenticated_request_returns_hx_redirect`: Validates `HX-Redirect` for unauthenticated HTMX requests.
- `test_expired_session_rejected`: Confirms expired cookie rejection.
- `test_htmx_error_returns_oob_toast_without_reswapping_target`: Validates error toast behavior with `HX-Reswap: none`.
- `test_hx_bookmark_toggle_to_unbookmarked`: Verifies bookmark toggle handler.
- `test_hx_entries_filter_query_params`: Validates `/hx/entries` filter parameter handling.
- `test_hx_bookmark_returns_updated_icon`: Tests icon update rendering.
- `test_get_root_page_serves_html_shell`: Validates root page rendering.
- `test_hx_entries_filter_today_and_search`: Tests date filtering combined with search term.
- `test_mark_feed_read_htmx`: Tests HTMX mark feed read endpoint.
- `test_hx_mark_page_read_endpoint`: Tests `/hx/mark-page-read` accepting JSON and comma-delimited ID formats.
- `test_hx_read_returns_oob_unread_pill`: Tests `/hx/entries/{id}/read?status=unread` returning OOB counter updates.
- `test_get_entry_detail_htmx_fetches_single_entry`: Validates entry detail fetch.
- `test_secure_cookie_flag_when_https`: Validates secure cookie flag configuration.
- `test_oidc_callback_clears_temp_cookie`: Validates temporary cookie cleanup post-login.
- `test_app_starts_and_responds_to_health_check`: End-to-end server boot and health probe verification.
- `test_e2e_authenticated_session_flow`: End-to-end full authenticated session flow.

### 3. `ui-components` (9 passed)
- `test_render_title_only_entry_item`: Verifies entry list row rendering.
- `test_detail_template_omits_author`: Verifies removal of author string in article detail view.
- `test_detail_template_has_mark_unread_button`: Verifies existence and attributes of "Mark as unread" action button.
- `test_render_entry_list_with_title_only_items`: Verifies rendering of entry list container.
- `test_render_toast_oob_template`: Validates toast notification template.
- `test_filter_bar_reordering`: Verifies filter pills rendered in order: Unread, Today, Starred, All Titles, with Unread active.
- `test_sanitizer_removes_dangerous_tags`: Verifies XSS sanitizer strips `<script>` and unsafe tags.
- `test_sanitizer_adds_noopener_to_links`: Verifies `rel="noopener noreferrer"` injected into links.
- `test_sanitizer_multiple_links_all_receive_target_blank`: Verifies `target="_blank"` on multiple anchor elements.
