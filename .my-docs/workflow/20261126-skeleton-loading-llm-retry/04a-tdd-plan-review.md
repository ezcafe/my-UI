Result: clean

# TDD Plan Review — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

- **Slug:** `20261126-skeleton-loading-llm-retry`
- **Stage:** `tdd-review` (`my-dev-flow-v3-code`, Mode `simple`, Review profile `lite`)
- **Has UI / API / DB / auth/PII/schema:** `yes | yes | no | no`

## Key Conclusions

- **Verdict — `clean` (0 Gaps, 6 Planned Tests Verified):** [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) defines 6 concrete failing-first test cases covering every UI, retry, HTTP validation/fallback, `in_flight` wait, `404` boundary, legacy fallback guard, and Option 1 error-toast contract in [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md).
- **Iron-law TDD order enforced:** Every task starts with named failing tests (`cargo test -p ui-components`, `cargo test -p server ai::llm::tests`, and route integration tests) before production edits.
- **Exact legacy test updates identified:** The 5 existing Option 1 assertion conflicts (`entries.rs` lines `3519, 3600, 3711, 3814`, and `routes/mod.rs` lines `1258, 1259`) are explicitly listed, while keeping `routes/mod.rs:1222` (`404` `"Summary unavailable"`) and `llm.rs:1528` (`LlmCliRunner::summarize` extractive fallback) intact.

---

## Evaluation of Planned Test Cases

| # | Test Name & Target | What It Verifies | Gaps / Notes | Status |
|---|---|---|---|---|
| 1 | `test_view_aware_entries_skeleton_and_accessibility`<br>[`crates/ui-components/src/templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) | `#entries-skeleton-magazine` (3-col `.magazine-3col-grid`) & `#entries-skeleton-list` (`min-h-[48px]`), `showEntriesLoadingState`/`hideEntriesLoadingState` view toggle (`currentViewMode === 'ai'`), `aria-busy`, `role="status"`, `sr-only`, `motion-reduce:animate-none`, and `ai_grouped_list.html` `#ai-summary-slot-{id}` with `<div class="mt-3">` wrapper and no `animate-spin`. | Preserves existing `#entries-loading-bar` & stale-XHR assertions in [`templates.rs#L4244-L4349`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4244-L4349). | **Pass** |
| 2 | `test_toast_deduplication_and_detail_is_error_cache_guard`<br>[`crates/ui-components/src/templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) | `base.html` `#toast-container` OOB deduplication + connected-root `lucide.createIcons({ root: liveRoot })` on detached `outerHTML` swaps, `index.html` Guest `showToast()` `data-auto-dismiss="true"`, `#ai-analysis-box`/`#ai-rewrite-box` match, and `!panel.querySelector('.stroke-destructive, .is-error')`. | Covers all client-side HTMX `outerHTML` lifecycle fixes cleanly. | **Pass** |
| 3 | `test_llm_cli_retries_once_after_one_second_on_failure`<br>[`crates/server/src/ai/llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) | `LlmCliRunner` retries once after `>= 1s` delay and succeeds on 2nd attempt across `summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, and `analyze_impact_and_fallacies`; releases `self.semaphore` permit between attempts; caps persistent failures at 2 attempts. | Can verify semaphore availability during the `1s` sleep window and real `>= 1s` backoff on retry. | **Pass** |
| 4 | `test_llm_http_client_retries_once_after_one_second`<br>[`crates/server/src/ai/llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) | `LlmHttpClient` retries once after `1s` on HTTP 500, empty/whitespace HTTP 200 content, or unparseable batch/analysis JSON; `LlmBackend::Http` falls back to CLI after 2 failed HTTP attempts (no nested `2x2` retry). | Uses `wiremock::MockServer` request expectations (`expect(2)`) to assert exact attempt counts. | **Pass** |
| 5 | `test_executive_brief_waits_for_inflight_and_surfaces_error_with_toast`<br>[`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) | `executive_brief_htmx` waits for `in_flight` past `4s` (`250s` cap), exits immediately once `in_flight` clears without brief, returns `AiErrorCardTemplate` + `render_llm_error_toast()`, skips persisting `"extractive-fallback"`, and persists valid partial items (`brief_text = None`) when `top` is empty. | Recommend `#[tokio::test(start_paused = true)]` so the `>4s` `in_flight` wait runs deterministically in 0ms wall-clock time without slowing `cargo test`. | **Pass** |
| 6 | `test_ai_endpoints_return_error_card_and_toast_after_retry_exhausted`<br>[`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) | `/hx/entries/{id}/ai-summary`, `/analysis`, and `/rewrite` return `AiErrorCardTemplate` + `render_llm_error_toast()` on exhausted retry (`200 OK`); Miniflux `404` keeps `"Summary unavailable"` without toast; `AiPrefetchService::run_prefetch` skips `"extractive-fallback"`; detail/analysis/rewrite ignore legacy `"extractive-fallback"` DB rows. | Existing test updates cleanly scoped to `entries.rs:3519, 3600, 3711, 3814` and `routes/mod.rs:1258, 1259`. | **Pass** |

---

## TDD & Regression Guard Checks

- **Signature ripple check:** Changing `AiService::get_or_create_summary` from `String` to `Result<String, String>` ([`ai/mod.rs#L66`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs#L66)) only affects `get_entry_ai_summary_htmx` in [`entries.rs#L1095`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1095); all other `ai/mod.rs` tests target `generate_rewrite` and `analyze_entry_impact_and_fallacies`.
- **Fallback behavior preservation:** `LlmCliRunner::summarize` still returns `(fallback_text, "extractive-fallback")` after 2 failed attempts so [`test_llm_cli_fallback_to_extractive_summary_on_failure`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L1528) remains valid, while `AiService::get_or_create_summary` turns `"extractive-fallback"` into `Err(...)` without persisting it.
- **Timer testing hygiene:** For `executive_brief_htmx` `in_flight` polling (`100ms` ticks past `4s`), using `#[tokio::test(start_paused = true)]` with async Tokio primitives (`tokio::time::sleep`) keeps simulated multi-second waits instant and deterministic ([Tokio time docs](https://docs.rs/tokio/latest/tokio/time/fn.pause.html)).

---

## Reference Sources

- **Task & Design Plans:** [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) · [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) · [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md)
- **Existing Code & Test Targets:** [`templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4244-L4349) · [`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L1160-L1600) · [`ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs#L66-L171) · [`prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs#L50-L186) · [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L3490-L3840) · [`routes/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1187-L1260)
- **Tokio Paused Time Testing Docs:** https://docs.rs/tokio/latest/tokio/time/fn.pause.html
