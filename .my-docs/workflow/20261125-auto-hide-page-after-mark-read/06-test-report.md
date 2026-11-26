# Test Report

**Result:** success
**Test path last run:** lite
**Smoke source:** Build-verify
**Updated:** 2026-11-26T06:56:55+07:00

## Smoke

- **Build Verify:** `verify-pass`
- **Red Phase (before implementation):** `cargo test -p ui-components` — 77 passed, 5 failed (all 5 new planned tests in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) failed as expected).
- **Green Phase (after implementation):**
  - `cargo test -p ui-components` — 82 passed, 0 failed (`lib` unit tests).
  - `cargo test -p server` — 140 passed, 0 failed (`lib` unit tests) + 8 passed, 0 failed (`tests/e2e_tests.rs`).
- **Files Modified:**
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)

## Suite / e2e

- **Test path:** `lite` (`test-lite` after `code-review-phase` clean)
- **`cargo test --workspace`:**
  - `miniflux-client` (`lib` unit tests): **24 passed, 0 failed**
  - `server` (`lib` unit tests): **140 passed, 0 failed**
  - `server` (`tests/e2e_tests.rs`): **8 passed, 0 failed** (covers `test_app_starts_and_responds_to_health_check`, `test_e2e_authenticated_session_flow`, `test_e2e_guest_mode_flow`, `test_mutation_endpoints_reject_with_403_when_auth_disabled`, `test_executive_brief_ineligible_views_and_empty_guard`, `test_executive_brief_cache_hit_and_oob_swaps`, `test_e2e_page_load_and_detail_drawer_rewrite_flow`, `test_e2e_detail_drawer_analysis_flow`)
  - `ui-components` (`lib` unit tests): **83 passed, 0 failed** (includes all 5 planned tests plus `test_entries_loading_bar_preserved_and_stale_xhr_guarded` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs))
- **`cargo clippy --all-targets`:** 0 warnings (exit 0)
- **`cargo fmt --check`:** clean (exit 0)
