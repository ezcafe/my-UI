# Test Report — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

**Result:** success
**Test path:** lite
**Smoke source:** Build-verify
**Updated:** 2026-11-26T20:29+07:00

## Smoke

- **Workspace Test Suite (`cargo test --workspace`):** 261 passed, 0 failed, 0 ignored (`miniflux_client`: 24/24, `server` unit: 144/144, `server` e2e: 8/8, `ui_components`: 85/85).
- **Linter (`cargo clippy --workspace --all-targets -- -D warnings`):** Passed with 0 warnings and 0 errors.
- **Note:** `server` tests using `wiremock::MockServer` (`127.0.0.1`) were executed with localhost networking enabled outside the restricted sandbox.

## Build Verification Summary

1. **Task 1 — View-Aware Skeletons, HTMX `outerHTML` Lifecycle, Toast Dedupe & Detail Cache Guard (`crates/ui-components`):**
   - Replaced `#entries-loading-overlay` spinner in `index.html` with `#entries-skeleton-magazine` (3-column `.magazine-3col-grid` + `.magazine-p2-grid`) and `#entries-skeleton-list` (`min-h-[48px]` Annuaire rows), `role="status"`, `aria-live="polite"`, `<span class="sr-only">Loading articles...</span>`, `aria-hidden="true"`, and `animate-pulse motion-reduce:animate-none`.
   - Updated `showEntriesLoadingState()` / `hideEntriesLoadingState()` to toggle skeletons via `currentViewMode === 'ai'` and update `aria-busy` on `#entries-list`.
   - Updated `index.html` `htmx:afterSwap` detail matching to include `#ai-analysis-box`, `#ai-rewrite-box`, and `evt.target?.closest?.('#detail-panel')`, and excluded `.is-error` cards from IndexedDB detail caching.
   - Marked Guest `showToast()` nodes with `data-auto-dismiss="true"` (and removed matching active toasts) to avoid duplicate dismissal timers.
   - Wrapped `#ai-summary-slot-{{ item.id }}` in `<div class="mt-3">` in `ai_grouped_list.html` with reduced-motion-safe pulse bars.
   - Updated `base.html` `htmx:afterSwap` to resolve connected replacement nodes before `lucide.createIcons({ root: liveRoot })`, and `htmx:oobAfterSwap` to deduplicate matching `#toast-container` messages.
   - Verified with `test_view_aware_entries_skeleton_and_accessibility` and `test_toast_deduplication_and_detail_is_error_cache_guard` (85/85 `ui-components` tests passing).

2. **Task 2 — 1-Second Auto-Retry & Strict Parsing (`crates/server/src/ai/llm.rs`):**
   - Added `retry_once_after_delay` (`LLM_RETRY_DELAY = 1s`) with `[AI_RETRY]` warning logs and shared strict parsers `try_parse_batch_response` and `try_parse_impact_and_fallacies_response`.
   - Extracted `LlmCliRunner::run_cli_prompt` acquiring `self.semaphore` per attempt (released before `tokio::time::sleep`) and wired `retry_once_after_delay` into `summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, and `analyze_impact_and_fallacies`.
   - Extracted `LlmHttpClient::request_chat_completion_once` rejecting empty/whitespace bodies and wired single-level `retry_once_after_delay` into `request_chat_completion`, `summarize_batch_structured`, and `analyze_impact_and_fallacies` (avoiding 2×2 retries before CLI fallback).
   - Verified with `test_llm_cli_retries_once_after_one_second_on_failure` and `test_llm_http_client_retries_once_after_one_second` (28/28 `ai::llm::tests` passing).

3. **Task 3 — Prefetch Coordination, Legacy Fallback Guards & Option 1 Route Contracts (`crates/server/src/ai/`, `crates/server/src/routes/`):**
   - Updated `AiService::get_or_create_summary` to return `Result<String, String>` and skip persisting `"extractive-fallback"`; guarded `analyze_entry_impact_and_fallacies` cache reads against `"extractive-fallback"`.
   - Updated `AiPrefetchService::run_prefetch` to skip DB persistence when `model == "extractive-fallback"`.
   - Updated `executive_brief_htmx` to poll `in_flight` up to `250s` with early exit once `in_flight` clears, skip `"extractive-fallback"` persistence, support partial real-LLM item persistence (`brief_text = None`), and append `render_llm_error_toast()` on brief errors.
   - Updated `/hx/entries/{id}/ai-summary` to return `AiErrorCardTemplate` + `render_llm_error_toast()` on LLM `Err(_)` while keeping `"Summary unavailable"` (no toast) on Miniflux `404`.
   - Updated `get_entry_detail_htmx`, `get_entry_analysis_htmx`, and `get_entry_rewrite_htmx` to ignore legacy `"extractive-fallback"` DB rows and append `render_llm_error_toast()` on LLM error responses.
   - Verified with `test_executive_brief_waits_for_inflight_and_surfaces_error_with_toast`, `test_ai_endpoints_return_error_card_and_toast_after_retry_exhausted`, and updated Option 1 assertions in `entries.rs` and `routes/mod.rs` (144/144 `server` unit tests + 8/8 `e2e_tests` passing).

## Test-Lite Execution

- **Command:** `cargo test --workspace` (executed with localhost TCP binding enabled for `wiremock::MockServer`, after verifying source timestamps precede `Last verify-pass at: 2026-11-26T20:23+07:00`).
- **Results (261 passed, 0 failed, 0 ignored):**
  - `miniflux-client` (`src/lib.rs`): 24 passed, 0 failed, 0 ignored (`0.08s`).
  - `server` unit suite (`src/lib.rs`): 144 passed, 0 failed, 0 ignored (`6.63s`), including `test_llm_cli_retries_once_after_one_second_on_failure`, `test_llm_http_client_retries_once_after_one_second`, `test_executive_brief_waits_for_inflight_and_surfaces_error_with_toast`, `test_ai_endpoints_return_error_card_and_toast_after_retry_exhausted`, and Option 1 contract tests in `routes::entries::tests` and `routes::tests`.
  - `server` integration suite (`tests/e2e_tests.rs`): 8 passed, 0 failed, 0 ignored (`1.51s`), covering `/health`, authenticated session flow, guest mode flow, executive brief cache hit/OOB swaps & ineligible view guards, and detail drawer analysis & rewrite flows.
  - `ui-components` (`src/lib.rs`): 85 passed, 0 failed, 0 ignored (`0.02s`), including `test_view_aware_entries_skeleton_and_accessibility` and `test_toast_deduplication_and_detail_is_error_cache_guard`.
  - Doc-tests (`miniflux_client`, `server`, `ui_components`): 0 tests.

## Reference Sources

- **Tokio Testing & Runtime:** https://docs.rs/tokio/latest/tokio/attr.test.html
- **Axum Testing (`tower::ServiceExt::oneshot`):** https://docs.rs/axum/latest/axum/
- **Wiremock-rs (`MockServer`):** https://docs.rs/wiremock/latest/wiremock/struct.MockServer.html
- **Workspace Test Files:** [`e2e_tests.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs) · [`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) · [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) · [`routes/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs) · [`templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
