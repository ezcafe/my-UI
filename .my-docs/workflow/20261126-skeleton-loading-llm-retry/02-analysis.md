# Analysis — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

**Result:** done

## Key Takeaways

- **Loading overlay:** `#entries-loading-overlay` currently shows a centered `<svg class="animate-spin">` spinner on Next, Previous, and Mark Done (`crates/ui-components/templates/pages/index.html`). Existing tests require keeping `#entries-loading-overlay`, `showEntriesLoadingState()`, `hideEntriesLoadingState()`, `#entries-loading-bar`, and stale-XHR request guards intact.
- **LLM retry gaps:** All 4 `LlmCliRunner` methods (`summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, `analyze_impact_and_fallacies`) and `LlmHttpClient::request_chat_completion` run a single attempt (`crates/server/src/ai/llm.rs`). Also `executive_brief_htmx` (`crates/server/src/routes/entries.rs`) only waits 4 seconds (`40 * 100ms`) for an `in_flight` prefetch job without checking whether the job has already finished or failed.
- **Failure visibility gaps:** Under confirmed Option 1, `/hx/entries/{id}/ai-summary` silently hides LLM errors via `extractive-fallback` instead of rendering `AiErrorCardTemplate`, and none of the 4 AI endpoints emit `render_llm_error_toast()`. Several existing tests explicitly assert no toast or `ai-summary-card` on failure and must be updated alongside the implementation.

---

## Context

1. **UI Skeleton & View Modes ([`index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html), [`entry_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), [`ai_grouped_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html)):**
   - `.workspace-layout` holds `#entries-loading-bar` (top 2px indicator for filters/search), `#entries-loading-overlay`, `#entries-list`, and `#detail-panel`.
   - `htmx:beforeRequest` triggers `showEntriesLoadingState()` (after `getVisibleEntryIds()` / empty-page guard) for `#mark-page-read-btn`, `#pagination-prev-btn`, and `#pagination-next-btn`.
   - `currentViewMode` holds `'ai'` (3-column `.magazine-3col-grid` in `#entries-magazine-view`) or `'list'` (stacked `.ar-list-item` rows + `#list-executive-brief` in `#entries-list-view`).
   - `ai_grouped_list.html` also uses a spinning `sparkles` icon (`animate-spin`) inside `.ai-summary-slot` and lacks a unique DOM `id` on each slot for targeted Retry swaps.
   - `base.html` `htmx:oobAfterSwap` handles `#toast-container` auto-dismiss (5000ms) without deduplicating identical concurrent toast messages.
   - `index.html:1211` checks `!panel.querySelector('.stroke-destructive')` before caching detail HTML in IndexedDB, missing `.is-error` cards rendered by `AiErrorCardTemplate` (which uses `text-destructive`).

2. **LLM Execution & Prefetch Wait ([`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs), [`ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs), [`prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs), [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)):**
   - `LlmCliRunner` uses `Semaphore::new(2)` per runner. Holding the permit across a 1-second sleep would block concurrent requests.
   - `summarize_batch_structured` treats empty `top` + empty `items` from `parse_batch_response` on stdout without retrying or falling back; `analyze_impact_and_fallacies` normalizes empty/unparseable JSON to a "no fallacies" string unless JSON parse success is checked before normalization.
   - `LlmBackend::Http { client, fallback }` delegates to `LlmHttpClient` first and falls back to `LlmCliRunner` on `Err`.
   - `executive_brief_htmx` (`entries.rs:841-869`) loops `0..40` sleeping `100ms` (4s total) when `page_key` is in `ai_service.in_flight`, never exiting early when `in_flight` clears on prefetch failure and timing out early when prefetch + 1s retry exceeds 4s.
   - When batch synthesis falls back (`model == "extractive-fallback"`), both `prefetch.rs:163-184` and `entries.rs:957-999` currently write `extractive-fallback` rows with populated `rewrite` into `AiDb`, while `get_entry_detail_htmx` (`entries.rs:1375`) treats any non-empty `record.rewrite` as a real rewrite without checking `record.model != "extractive-fallback"`.

3. **Failure Notification & Error Templates ([`error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs), [`ai_error_card.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_error_card.html), [`toast.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html)):**
   - `render_llm_error_toast()` already renders `ToastTemplate` targeting `#toast-container` with `hx-swap-oob="beforeend"` and `LLM_HIGH_DEMAND_ERROR_MESSAGE`.
   - `AiService::get_or_create_summary` currently returns `String` (saving `"extractive-fallback"` to `AiDb`), hiding failure from `get_entry_ai_summary_htmx`.
   - Existing route tests asserting no toast (`entries.rs:3519, 3600, 3711, 3814`) and `ai-summary-card` on failure (`crates/server/src/routes/mod.rs:1258`) reflect the previous design (`20261124-contextual-ai-error-display`) and must be updated for Option 1.

---

## Risks

- **Test suite duration:** Failing LLM paths that sleep 1 second before retry will add ~1s per sequential failing test (mitigated in part by Tokio parallel test execution and keeping retry helper delay injectable if needed for pure retry unit tests while testing real 1s delay).
- **Semaphore starvation:** Sleeping inside `LlmCliRunner` while holding `self.semaphore` permit would block other LLM requests for 1s; releasing the permit between attempt 1 and attempt 2 eliminates this.
- **Toast stacking:** Multiple concurrent AI failures (e.g., `/rewrite` + `/analysis` or multiple `/ai-summary` cards in `ai_grouped_list`) would stack duplicate toasts unless client-side OOB swap deduplicates identical active error messages.
- **Breaking existing template assertions:** Modifying `#entries-loading-overlay` or `showEntriesLoadingState()` / `hideEntriesLoadingState()` signatures must preserve all assertions in [`crates/ui-components/src/templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3810-L4350).

---

## Open questions

None — all product decisions were confirmed in [`01-idea.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/01-idea.md) and [`00-run.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/00-run.md).
