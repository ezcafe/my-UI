# Tasks (TDD-ready) — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

Each task: behavior, failing test name first, files allowed, done evidence.

## Planned tests

<!-- ≥1 named case → planned tests present (Prefer tdd-review). Empty / N/A → not present. -->

| Case | Failing test name | Covers |
|------|-------------------|--------|
| View-aware Magazine & List (`min-h-[48px]`) skeletons + `aria-busy` + `motion-reduce` in `index.html` and `ai_grouped_list.html` (`mt-3` wrapper) | `test_view_aware_entries_skeleton_and_accessibility` | UI skeleton markup, `showEntriesLoadingState` / `hideEntriesLoadingState` view toggle, `aria-busy`, `motion-reduce:animate-none`, `#ai-summary-slot-{id}` with `mt-3` wrapper |
| Toast OOB deduplication, Guest `showToast()` `data-auto-dismiss`, connected-root Lucide `outerHTML` hydration, detail `outerHTML` swap match & `.is-error` cache exclusion | `test_toast_deduplication_and_detail_is_error_cache_guard` | `base.html` `#toast-container` deduplication + connected-root `createIcons`, `index.html` `showToast` `data-auto-dismiss="true"` + `#ai-analysis-box`/`#ai-rewrite-box` match + `!panel.querySelector('.stroke-destructive, .is-error')` |
| `LlmCliRunner` 1-second retry succeeds on 2nd attempt across `summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, `analyze_impact_and_fallacies` | `test_llm_cli_retries_once_after_one_second_on_failure` | 1 retry after `>= 1s` delay, semaphore release between attempts, max 2 attempts on persistent failure |
| `LlmHttpClient` retries once after 1 second on HTTP 500, empty HTTP 200 text, or unparseable batch/analysis JSON, and `LlmBackend::Http` falls back to CLI after 2 failed HTTP attempts | `test_llm_http_client_retries_once_after_one_second` | HTTP 500 / empty 200 / invalid batch & analysis JSON -> 1s sleep -> retry -> success or CLI fallback |
| `executive_brief_htmx` waits for `in_flight` prefetch past 4s (250s upper cap) and returns error card + OOB toast immediately when prefetch finishes without brief (skipping `extractive-fallback` persistence) | `test_executive_brief_waits_for_inflight_and_surfaces_error_with_toast` | Coordinated `in_flight` polling + immediate early exit + `AiErrorCardTemplate` + `render_llm_error_toast()` + `top.is_empty()` / `extractive-fallback` persistence rules |
| `/hx/entries/{id}/ai-summary`, `/analysis`, `/rewrite` return `AiErrorCardTemplate` + `render_llm_error_toast()` on exhausted retry (while `404` keeps `"Summary unavailable"` without toast), `prefetch` skips `extractive-fallback`, and detail/rewrite ignore legacy `extractive-fallback` rows | `test_ai_endpoints_return_error_card_and_toast_after_retry_exhausted` | Option 1 parity on `/ai-summary` (vs 404 `"Summary unavailable"`), `/analysis`, `/rewrite` + `AiPrefetchService` no-fallback-cache + legacy `extractive-fallback` DB guards |

---

## Implementation Tasks

1. **Task 1 — View-Aware Skeleton Overlay, `ai_grouped_list` Slot, HTMX `outerHTML` Hooks, Toast Dedupe & Detail Cache Guard (`crates/ui-components`)**
   - **Failing test first:** Write `test_view_aware_entries_skeleton_and_accessibility` and `test_toast_deduplication_and_detail_is_error_cache_guard` in `crates/ui-components/src/templates.rs`, confirm `cargo test -p ui-components` fails on the new assertions.
   - **Behavior:**
     - Replace the spinner in `#entries-loading-overlay` (`index.html`) with `#entries-skeleton-magazine` (3-column `.magazine-3col-grid` skeleton) and `#entries-skeleton-list` (executive brief banner + 10 `min-h-[48px]` Annuaire row skeletons), `role="status"`, `<span class="sr-only">Loading articles...</span>`, `aria-hidden="true"` on shapes, `animate-pulse motion-reduce:animate-none`.
     - Update `showEntriesLoadingState()` / `hideEntriesLoadingState()` to toggle `#entries-skeleton-magazine` vs `#entries-skeleton-list` using `currentViewMode === 'ai'`, and toggle `aria-busy` on `#entries-list`.
     - Update `index.html:1201` detail-panel swap check to explicitly include `targetId === 'ai-analysis-box' || targetId === 'ai-rewrite-box'` and `evt.target?.closest?.('#detail-panel')` (since `outerHTML` detaches `evt.detail.target`), and update `index.html:1211` detail cache guard to `!panel.querySelector('.stroke-destructive, .is-error')`.
     - Set `toast.setAttribute('data-auto-dismiss', 'true')` in Guest `showToast()` (`index.html:1920-1934`) so `base.html` OOB timer logic does not attach a duplicate `5000ms` timer to active Guest toasts.
     - Update `ai_grouped_list.html` to wrap `#ai-summary-slot-{{ item.id }}` in `<div class="mt-3">` (moving `mt-3` from `.ai-summary-slot` to the wrapper so `AiErrorCardTemplate` retains top spacing) and replace `animate-spin` with `animate-pulse motion-reduce:animate-none` skeleton bars.
     - Update `base.html` `htmx:afterSwap` (`576-580`) to resolve connected replacement nodes (`swapTarget.isConnected ? swapTarget : document.getElementById(swapTarget.id) || ...`) before calling `lucide.createIcons({ root: liveRoot })`, and update `htmx:oobAfterSwap` (`581-594`) `#toast-container` handler to deduplicate matching active toast messages.
   - **Files allowed:** `crates/ui-components/templates/pages/index.html`, `crates/ui-components/templates/partials/ai_grouped_list.html`, `crates/ui-components/templates/base.html`, `crates/ui-components/src/templates.rs`.
   - **Done evidence:** `cargo test -p ui-components` passes with all existing loading/cache tests and both new tests green.

2. **Task 2 — 1-Second Auto-Retry & Strict Validation Across `LlmCliRunner` and `LlmHttpClient` (`crates/server/src/ai/llm.rs`)**
   - **Failing test first:** Write `test_llm_cli_retries_once_after_one_second_on_failure` (testing 1st-attempt failure -> ~1s sleep -> 2nd-attempt success, plus 2-attempt cap on persistent failure) and `test_llm_http_client_retries_once_after_one_second` (covering HTTP 500, HTTP 200 whitespace/empty content, unparseable batch/analysis JSON, and `LlmBackend::Http` fallback to CLI) in `crates/server/src/ai/llm.rs`, confirm `cargo test -p server ai::llm::tests` fails first.
   - **Behavior:**
     - Add `retry_once_after_delay` helper using `tokio::time::sleep(Duration::from_secs(1))`.
     - Add shared strict parsers `LlmCliRunner::try_parse_batch_response` (`!top.trim().is_empty() || !items.is_empty()`) and `LlmCliRunner::try_parse_impact_and_fallacies_response` (JSON/fenced-JSON validation before normalization).
     - Extract `LlmCliRunner::run_cli_prompt` acquiring `self.semaphore` per attempt (so sleep never holds a permit) and apply `retry_once_after_delay` across `summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, and `analyze_impact_and_fallacies`.
     - Extract `LlmHttpClient::request_chat_completion_once` rejecting empty/whitespace text (`text.trim().is_empty()`), wire `retry_once_after_delay` into `request_chat_completion` (used by `summarize`), and wrap `request_chat_completion_once` + strict parsing (`try_parse_batch_response` / `try_parse_impact_and_fallacies_response`) in `retry_once_after_delay` inside `LlmHttpClient::summarize_batch_structured` and `LlmHttpClient::analyze_impact_and_fallacies` (avoiding nested 2×2 retries before CLI fallback).
   - **Files allowed:** `crates/server/src/ai/llm.rs`.
   - **Done evidence:** `cargo test -p server ai::llm::tests` passes including timing, validation, fallback, and attempt-count assertions.

3. **Task 3 — Prefetch Coordination, Legacy Fallback Guards, `AiService::get_or_create_summary` Result & Option 1 Route Error Cards + Toast (`crates/server/src/ai/`, `crates/server/src/routes/`)**
   - **Failing test first:** Write `test_executive_brief_waits_for_inflight_and_surfaces_error_with_toast` and `test_ai_endpoints_return_error_card_and_toast_after_retry_exhausted`, and update existing Option 1 assertions in `crates/server/src/routes/entries.rs` (`3519, 3600, 3711, 3814`) and `crates/server/src/routes/mod.rs` (`1258, 1259` in `test_hx_entries_ai_summary_guarantees_200_swap_on_failure`, while keeping `mod.rs:1222` `404` ownership assertion `"Summary unavailable"` unchanged), verifying initial test failure.
   - **Behavior:**
     - Change `AiService::get_or_create_summary` (`ai/mod.rs`) to return `Result<String, String>` and skip saving `"extractive-fallback"` on LLM failure; guard `analyze_entry_impact_and_fallacies` cache lookup with `record.model != "extractive-fallback"`.
     - Update `AiPrefetchService::run_prefetch` (`ai/prefetch.rs`) to skip persisting `"extractive-fallback"` batch items when LLM fails.
     - Update `executive_brief_htmx` (`routes/entries.rs`) to wait on `in_flight` up to `250s` (`2500` × `100ms`, exiting early once `in_flight` clears and DB still lacks a brief), skip persistence on `model == "extractive-fallback"` (and on empty `top && items`), persist valid non-fallback `items` (`brief_text = None`) if `top` is empty while rendering `render_brief_error(&retry_url) + render_llm_error_toast()`, and append `render_llm_error_toast()` on all brief failures.
     - Update `get_entry_ai_summary_htmx` (`routes/entries.rs`) to keep `<div class="ai-summary-card ...">Summary unavailable</div>` (no toast) when Miniflux returns `404`/not-found or template rendering fails, and return `AiErrorCardTemplate` + `crate::error::render_llm_error_toast()` exclusively on `Err(_)` from `get_or_create_summary` (`200 OK`).
     - Update `get_entry_detail_htmx`, `get_entry_analysis_htmx`, and `get_entry_rewrite_htmx` (`routes/entries.rs`) to guard cache reads with `record.model != "extractive-fallback"`, and append `crate::error::render_llm_error_toast()` on `Err(_)` in `get_entry_analysis_htmx` and `get_entry_rewrite_htmx` (`200 OK`).
   - **Files allowed:** `crates/server/src/ai/mod.rs`, `crates/server/src/ai/prefetch.rs`, `crates/server/src/routes/entries.rs`, `crates/server/src/routes/mod.rs`.
   - **Done evidence:** Full `cargo test` passes across all workspace crates (`ui-components`, `miniflux-client`, `server`) with zero skipped or relaxed assertions.
