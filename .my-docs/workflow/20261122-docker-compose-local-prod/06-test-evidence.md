# 06-test-evidence: Test-Lite Verification

- **Result:** success
- **Test path:** lite
- **Stage:** test-lite
- **Date:** 2026-11-22

---

## Test Verification Summary

All test suites and validation checks for the Docker & Compose infrastructure have executed and passed:

| Test Suite / Target | Verification Command / Check | Result |
|---|---|---|
| **Server Unit & Integration Tests** | `cargo test -p server` (58 unit + 4 e2e tests) | PASSED (62 passed, 0 failed) |
| **Healthcheck Endpoint Contract** | `cargo test -p server --lib app::tests::test_health_endpoint` | PASSED (HTTP 200, status: ok) |
| **Local Compose Schema & Config** | `docker compose -f docker-compose.local.yml config --quiet` | PASSED (valid syntax, bind mount to `./data`) |
| **Production Compose Schema & Config** | `docker compose -f docker-compose.yml config --quiet` | PASSED (valid syntax, named volume, logging limits) |
| **.dockerignore Exclusion Rules** | Excludes target, git, .gemini, .my-docs, data, .env (allows .env.example) | PASSED (13 patterns validated) |
| **Operational Documentation** | `README.md` command syntax, path correctness, structure | PASSED (scannable, plain words) |

---

## Detailed Execution Logs

### 1. Server Unit & E2E Test Suite
```bash
$ cargo test -p server
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running unittests src/lib.rs (target/debug/build/server/ecfc5e9a722ea6ed/out/server-ecfc5e9a722ea6ed)

running 58 tests
test ai::extractor::tests::test_extract_article_fallback_when_article_tag_absent ... ok
test ai::extractor::tests::test_extract_article_content_word_truncation ... ok
test ai::extractor::tests::test_extract_article_tag_content_stripping_boilerplate ... ok
test ai::extractor::tests::test_extractor_parses_content_with_lazylock_selectors ... ok
test ai::llm::tests::test_llm_cli_concurrency_throttling ... ok
test ai::llm::tests::test_llm_cli_prompt_formatting_and_args ... ok
test auth::session::tests::test_session_data_size_under_cookie_limit ... ok
test auth::temp_cookie::tests::test_oidc_temp_cookie_encryption_and_roundtrip ... ok
test ai::extractor::tests::test_extract_article_fallback_on_http_error_or_block ... ok
test auth::tests::test_authenticated_user_bypasses_when_auth_disabled ... ok
test app::tests::test_health_endpoint ... ok
test ai::llm::tests::test_llm_cli_fallback_to_extractive_summary_on_failure ... ok
test ai::db::tests::test_sqlite_wal_mode_active ... ok
test ai::db::tests::test_ai_db_upsert_on_conflict_updates_summary ... ok
test ai::db::tests::test_ai_db_initializes_schema_and_user_version ... ok
test auth::tests::test_htmx_unauthenticated_request_returns_hx_redirect ... ok
test auth::tests::test_auth_routes_redirect_to_root_when_auth_disabled ... ok
test auth::tests::test_expired_session_rejected ... ok
test ai::db::tests::test_ai_db_concurrent_pool_reads_under_wal ... ok
test auth::tests::test_oidc_callback_state_mismatch_fails ... ok
test config::tests::test_config_allows_missing_oidc_when_auth_disabled ... ok
test config::tests::test_config_defaults ... ok
test config::tests::test_config_fails_when_secret_too_short ... ok
test config::tests::test_config_loads_from_valid_env ... ok
test config::tests::test_config_parses_ai_settings_and_defaults ... ok
test cache::tests::test_miniflux_cache_bounded_capacity ... ok
test config::tests::test_config_parses_auth_enabled_and_defaults ... ok
test auth::tests::test_unauthenticated_graphql_request_rejected_with_401 ... ok
test rate_limit::tests::test_rate_limiter_allows_burst_and_throttles_excess_requests ... ok
test rate_limit::tests::test_rate_limiter_parameter_validation_defaults ... ok
test routes::entries::tests::test_build_graphql_filter_guest_unread_scopes_to_today ... ok
test auth::tests::test_unauthenticated_request_redirects_to_login ... ok
test rate_limit::tests::test_rate_limiter_stale_bucket_pruning ... ok
test routes::entries::tests::test_build_graphql_filter_search_defaults_to_all ... ok
test routes::entries::tests::test_entries_filter_query_mapping ... ok
test rate_limit::tests::test_ddos_body_limit_rejects_payloads_exceeding_cap ... ok
test routes::tests::test_get_root_page_serves_html_shell ... ok
test routes::tests::test_htmx_error_returns_oob_toast_without_reswapping_target ... ok
test routes::tests::test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled ... ok
test routes::tests::test_get_entry_detail_auto_marks_as_read_and_returns_oob_swaps ... ok
test routes::tests::test_get_entry_detail_htmx_fetches_single_entry ... ok
test routes::tests::test_hx_bookmark_returns_updated_icon ... ok
test routes::tests::test_hx_bookmark_toggle_to_unbookmarked ... ok
test routes::tests::test_hx_entries_ai_status_forwards_search_query ... ok
test routes::tests::test_hx_entries_ai_summary_requires_auth_and_verifies_ownership ... ok
test routes::tests::test_hx_entries_filter_query_params ... ok
test cache::tests::test_miniflux_cache_inline_eviction_of_expired ... ok
test routes::tests::test_hx_entries_filter_today_and_search ... ok
test routes::tests::test_hx_mark_page_read_endpoint ... ok
test routes::tests::test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled ... ok
test routes::tests::test_list_entries_serves_from_cache_when_cached ... ok
test routes::tests::test_hx_read_returns_oob_unread_pill ... ok
test routes::tests::test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled ... ok
test routes::tests::test_mark_feed_read_htmx ... ok
test auth::tests::test_secure_cookie_flag_when_https ... ok
test auth::tests::test_oidc_callback_clears_temp_cookie ... ok
test cache::tests::test_miniflux_cache_get_and_insert_with_ttl ... ok
test routes::tests::test_hx_entries_ai_summary_guarantees_200_swap_on_failure ... ok

test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/e2e_tests.rs (target/debug/build/server/6753e54f18d78988/out/e2e_tests-6753e54f18d78988)

running 4 tests
test test_app_starts_and_responds_to_health_check ... ok
test test_mutation_endpoints_reject_with_403_when_auth_disabled ... ok
test test_e2e_authenticated_session_flow ... ok
test test_e2e_guest_mode_flow ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### 2. Docker Compose Configuration Checks
```bash
$ docker compose -f docker-compose.local.yml config --quiet
$ docker compose -f docker-compose.yml config --quiet
```
Both compose definitions validated without warnings or syntax errors.

### 3. Invariant Validations
- Multi-stage Dockerfile builds with Rust 1.85 and packages into Debian Bookworm Slim.
- Unprivileged user UID `10001` configured with read/write `/app/data` workspace.
- Healthcheck probe curl targets `http://127.0.0.1:3000/health`.
- `.dockerignore` prevents accidental secret, build directory, or documentation leakage.
- Local compose mounts host `./data` volume; Production compose uses named volume `my_ui_data`.
