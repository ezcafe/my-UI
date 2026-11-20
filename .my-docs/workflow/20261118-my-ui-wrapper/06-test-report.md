# Test Report — my-ui wrapper

**Test path:** full
**Result:** success
**Smoke source:** Smoke-Task

## Summary

The entire test suite was executed against the Cargo workspace in macOS Apple Silicon environment following strict Test-Driven Development (TDD) red-green-refactor cycles.

- **Workspace Test Execution:** `cargo test --workspace`
- **Clippy Lint Execution:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Total Tests:** 44 passed; 0 failed; 0 ignored
- **Clippy Warnings:** 0 warnings; 0 errors
- **Exit Status:** 0 (clean)

---

## Detailed Test Breakdown by Crate

### 1. `miniflux-client` (17 tests)

| Test Name | Kind | Purpose | Status |
|---|---|---|---|
| `client::tests::test_get_me_caches_user_id` | WireMock | Verifies `/v1/me` caches `user_id` in client | **Pass** |
| `client::tests::test_mark_all_as_read_uses_cached_user_id` | WireMock | Asserts dispatch to `/v1/users/{id}/mark-all-as-read` | **Pass** |
| `client::tests::test_mark_feed_as_read_endpoint` | WireMock | Validates feed-specific mark-read endpoint | **Pass** |
| `client::tests::test_get_feeds_merges_unread_counters` | WireMock | Verifies concurrent join of feeds and counters | **Pass** |
| `client::tests::test_get_entries_success` | WireMock | Validates deserialization of `/v1/entries` | **Pass** |
| `client::tests::test_get_entry_success` | WireMock | Validates single entry fetch | **Pass** |
| `client::tests::test_update_entry_status_sends_batch_payload` | WireMock | Validates batch update payload | **Pass** |
| `client::tests::test_toggle_bookmark_mutation` | WireMock | Tests bookmark mutation | **Pass** |
| `client::tests::test_miniflux_timeout_handling` | WireMock | Confirms timeout error is returned when upstream hangs | **Pass** |
| `summary::tests::test_extractive_summary_returns_first_two_sentences` | Unit | Tests extractive Unicode sentence segmentation | **Pass** |
| `schema::tests::test_graphql_query_entries` | In-memory GraphQL | Queries schema and checks entries array | **Pass** |
| `schema::tests::test_graphql_query_single_entry` | In-memory GraphQL | Queries single entry via GraphQL | **Pass** |
| `schema::tests::test_graphql_query_feeds_includes_unread_count` | In-memory GraphQL | Verifies `unreadCount` resolved on feed | **Pass** |
| `schema::tests::test_graphql_entry_summary_resolved` | In-memory GraphQL | Verifies 2-sentence summary resolution | **Pass** |
| `schema::tests::test_graphql_mutation_mark_all_as_read` | In-memory GraphQL | Tests root mutation execution | **Pass** |
| `schema::tests::test_graphql_mutation_mark_feed_as_read` | In-memory GraphQL | Tests feed mark read mutation | **Pass** |
| `schema::tests::test_graphql_invalid_filter_arguments` | In-memory GraphQL | Confirms validation errors on negative limits/offsets | **Pass** |

### 2. `ui-components` (5 tests)

| Test Name | Kind | Purpose | Status |
|---|---|---|---|
| `sanitizer::tests::test_sanitizer_removes_dangerous_tags` | Unit | Verifies `<script>`, `<iframe>`, `onerror` stripped | **Pass** |
| `sanitizer::tests::test_sanitizer_adds_noopener_to_links` | Unit | Validates `rel="noopener noreferrer"` and `target="_blank"` | **Pass** |
| `sanitizer::tests::test_sanitizer_multiple_links_all_receive_target_blank` | Unit | Validates multiple links receive target blank and noopener | **Pass** |
| `templates::tests::test_render_entry_item_template` | Unit (Askama) | Checks rendered HTML attributes, buttons, and tokens | **Pass** |
| `templates::tests::test_render_toast_oob_template` | Unit (Askama) | Asserts `hx-swap-oob="beforeend"` presence | **Pass** |

### 3. `server` (22 tests)

| Test Name | Kind | Purpose | Status |
|---|---|---|---|
| `config::tests::test_config_loads_from_valid_env` | Unit | Tests configuration parsing from environment variables | **Pass** |
| `config::tests::test_config_fails_when_secret_too_short` | Unit | Validates SESSION_SECRET < 32 bytes returns error | **Pass** |
| `config::tests::test_config_defaults` | Unit | Checks fallback defaults for host and port | **Pass** |
| `auth::temp_cookie::tests::test_oidc_temp_cookie_encryption_and_roundtrip` | Unit | Tests AES-256-GCM roundtrip of PKCE/CSRF temp cookie | **Pass** |
| `auth::session::tests::test_session_data_size_under_cookie_limit` | Unit | Validates encrypted session cookie is well under 4KB | **Pass** |
| `auth::tests::test_unauthenticated_request_redirects_to_login` | Integration | Confirms 302 redirect to `/login` without session | **Pass** |
| `auth::tests::test_htmx_unauthenticated_request_returns_hx_redirect` | Integration | Confirms `HX-Redirect: /login` for unauthenticated HTMX | **Pass** |
| `auth::tests::test_unauthenticated_graphql_request_rejected_with_401` | Integration | Confirms 401 Unauthorized for GraphQL without session | **Pass** |
| `auth::tests::test_expired_session_rejected` | Integration | Confirms expired session is rejected and cleared | **Pass** |
| `auth::tests::test_secure_cookie_flag_when_https` | Integration | Verifies cookie secure flag matches scheme | **Pass** |
| `auth::tests::test_oidc_callback_state_mismatch_fails` | Integration | Validates CSRF state protection on OIDC callback | **Pass** |
| `auth::tests::test_oidc_callback_clears_temp_cookie` | Integration | Asserts `__oidc_temp` cleared with `Max-Age=0` | **Pass** |
| `routes::tests::test_get_root_page_serves_html_shell` | Integration | Verifies root `/` renders 200 OK spatial glass layout | **Pass** |
| `routes::tests::test_get_entry_detail_htmx_fetches_single_entry` | Integration | Tests fetching single entry modal/drawer detail | **Pass** |
| `routes::tests::test_hx_entries_filter_query_params` | Integration | Tests `GET /hx/entries` filter queries | **Pass** |
| `routes::tests::test_hx_read_returns_oob_unread_pill` | Integration | Verifies out-of-band `#unread-pill` swap on read toggle | **Pass** |
| `routes::tests::test_hx_bookmark_returns_updated_icon` | Integration | Verifies bookmark mutation returns updated button | **Pass** |
| `routes::tests::test_hx_bookmark_toggle_to_unbookmarked` | Integration | Verifies bookmark toggle unbookmark state | **Pass** |
| `routes::tests::test_mark_feed_read_htmx` | Integration | Verifies feed-level mark read HTMX action | **Pass** |
| `routes::tests::test_htmx_error_returns_oob_toast_without_reswapping_target` | Integration | Confirms `HX-Reswap: none` and toast OOB on error | **Pass** |
| `test_app_starts_and_responds_to_health_check` | E2E Integration | Boots full server stack and verifies `GET /health` | **Pass** |
| `test_e2e_authenticated_session_flow` | E2E Integration | Full lifecycle: session -> list -> read toggle -> logout | **Pass** |

---

## Smoke & Verification
- Health endpoint (`GET /health`): Responds 200 OK `{"status": "ok"}`
- Root shell (`GET /`): Authenticated user receives full AR spatial glassmorphism shell with Lucide icon hooks and `#unread-pill`
- HTMX out-of-band synchronization: Verified functioning for read/unread state changes and upstream error retargeting without DOM disruption.
- Clippy workspace zero warnings: Verified `cargo clippy --workspace --all-targets -- -D warnings` completed with exit status 0.
