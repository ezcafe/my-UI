# Code Review Log — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

**Overall Result:** clean
(file-top alone never pays Review clean)

- **Workflow Slug:** `20261126-skeleton-loading-llm-retry`
- **Mode / Profile:** `simple` / `lite`
- **Has UI / API / DB / auth/PII/schema:** `yes | yes | no | no`
- **Lens Plan:** `api`
- **Verification Baseline:** `06-test-report.md` (`verify-pass`, `Smoke source: Build-verify` — 261/261 tests passing, `cargo clippy --workspace --all-targets -- -D warnings` clean; all source timestamps precede `06-test-report.md`).

---

## Adversarial

**Result:** clean

### Scope & Edge Cases Checked

1. **FIRST Gates & Scope Check:**
   - Card has `Has API = yes`, `Has DB = no`, `Lens plan = api` (`missing required api/db` is false; `Mode = simple` / `profile = lite` so E1 Smoke gate does not apply).
2. **LLM Retry Cap, Semaphore Discipline & Subprocess Safety (`crates/server/src/ai/llm.rs`):**
   - `retry_once_after_delay` (`llm.rs#L11-L35`) executes at most 1 retry (`op(1)` -> `[AI_RETRY]` warning -> `tokio::time::sleep(1s)` -> `op(2)`), preventing unbounded retry loops.
   - `LlmCliRunner::run_cli_prompt` (`llm.rs#L517-L571`) acquires `self.semaphore` inside each single attempt, ensuring the permit is released before the 1-second sleep so failing requests do not stall concurrent work. `cmd.kill_on_drop(true)` terminates timed-out child processes cleanly (`90s` batch, `30s` single/rewrite/analysis).
   - `LlmHttpClient::request_chat_completion_once` (`llm.rs#L891-L1001`) checks non-2xx HTTP statuses, validates JSON response envelopes, and rejects whitespace/empty text (`text.trim().is_empty()`). `summarize_batch_structured` (`L1010-L1030`) and `analyze_impact_and_fallacies` (`L1056-L1078`) wrap `request_chat_completion_once` + strict parsers (`try_parse_batch_response`, `try_parse_impact_and_fallacies_response`) directly inside one `retry_once_after_delay`, avoiding nested `2x2` retries before `LlmBackend::Http` falls back to `LlmCliRunner`.
3. **Prefetch Coordination, Cache Poisoning & 404 Separation (`crates/server/src/ai/mod.rs`, `prefetch.rs`, `routes/entries.rs`):**
   - `AiPrefetchService::run_prefetch` (`prefetch.rs#L153-L159`) returns without DB persistence when `model == "extractive-fallback"`, and `AiService::get_or_create_summary` (`ai/mod.rs#L66-L88`) returns `Err("LLM summary unavailable")` without saving `"extractive-fallback"` or blank summaries.
   - `executive_brief_htmx` (`entries.rs#L854-L892`) polls `in_flight` up to `250s` (`2500 * 100ms`, covering `3s` extraction + `61s` HTTP + `181s` CLI = `245s`) and exits immediately when `!still_in_flight` after a final `get_page_brief_by_key` re-check (guarding `InFlightGuard::drop` spawn ordering). Full failures (`model == "extractive-fallback"` or empty `top && items`) skip `save_batch_results`; partial real-LLM batches (`top.is_empty()` with valid `items`) persist `items` with `brief_text = None`, emit OOB titles, and return `AiErrorCardTemplate` + `render_llm_error_toast()`.
   - `get_entry_ai_summary_htmx` (`entries.rs#L1064-L1162`) keeps `AuthenticatedUser` and returns `"Summary unavailable"` without toast on Miniflux `404`/not-found or Askama error (`mod.rs#L1188-L1223`), while returning `AiErrorCardTemplate` (`#ai-summary-slot-{id}`, `outerHTML`) + `render_llm_error_toast()` on `get_or_create_summary` `Err(_)` (`mod.rs#L1226-L1262`).
   - Legacy `"extractive-fallback"` DB rows are filtered out in `get_entry_detail_htmx` (`L1433`), `get_entry_analysis_htmx` (`L1689`), `get_entry_rewrite_htmx` (`L1790`, `L1861`), and `AiService::analyze_entry_impact_and_fallacies` (`ai/mod.rs#L141`).
4. **HTMX `outerHTML` Lifecycle, Toast Timers & XSS Safety (`crates/ui-components/templates/`):**
   - `base.html` `htmx:afterSwap` (`L576-L585`) checks `swapTarget.isConnected` and falls back to `document.getElementById(swapTarget.id)` or `evt.target` / `document.body` so `outerHTML`-replaced `#ai-analysis-box`, `#ai-rewrite-box`, `#page-executive-brief`, and `#ai-summary-slot-{id}` hydrate Lucide icons reliably.
   - `base.html` `htmx:oobAfterSwap` (`L586-L611`) deduplicates active `.pointer-events-auto` toasts in `#toast-container` by trimmed `span.textContent` while only attaching the `5000ms` timer to `:not([data-auto-dismiss])`, keeping Guest `showToast()` (`index.html#L2042-L2064`, `data-auto-dismiss="true"`, `4000ms`) free of double timers.
   - `index.html` `htmx:afterSwap` (`L1317-L1337`) explicitly matches `#ai-analysis-box` and `#ai-rewrite-box` (plus live `evt.target.closest('#detail-panel')`) and skips IndexedDB caching when `.stroke-destructive, .is-error` is present.
   - `ai_grouped_list.html` (`L17-L29`) moves `mt-3` onto the wrapper `<div>` so `AiErrorCardTemplate` retains top spacing and `AiSummaryCardTemplate` collapses vertical margins in block flow.
   - Escaping: Askama templates auto-escape dynamic values; `render_title_oob_span`, `render_analysis_fragment`, and fallback prose use `escape_html(...)`; `build_retry_url()` (`entries.rs#L703-L715`) only interpolates `i64` parameters and `"unread"` (enforced by `is_brief_eligible_filter`).

---

## Quality

**Result:** clean

### Five-Axis Review & Structural Simplification

1. **Correctness:**
   - Every requirement in [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) and [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) is implemented and covered by passing unit/integration assertions (`261/261` tests and `clippy` clean in [`06-test-report.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/06-test-report.md)).
2. **Readability & Simplification:**
   - Extracting `retry_once_after_delay`, `LlmCliRunner::run_cli_prompt`, `extract_fenced_json`, `normalize_fallacies`, `render_brief_error_with_toast`, and `render_cached_brief` removed duplicated CLI spawning, fenced-JSON parsing, and fallacy normalization across 4+ methods with clear, single-purpose helpers.
3. **Architecture:**
   - Layer boundaries hold cleanly: UI skeletons and HTMX DOM hooks live in `ui-components`, retry/parse mechanics live in `ai/llm.rs`, persistence policies live in `ai/mod.rs` and `ai/prefetch.rs`, and HTTP/HTMX fragment assembly lives in `routes/entries.rs` + `error.rs`.
4. **Security:**
   - `Cargo.lock` pins `tokio = "1.54.1"` (patched against CVE-2026-115567 / RUSTSEC-2026-0384 for `Interval`, while retry/polling code uses `tokio::time::sleep` and `timeout`). Parameterized SQL (`sqlx`), Askama escaping, `escape_html`, `AuthenticatedUser` on `/ai-summary`, and strict `clamp`/`max` bounds hold with zero internal error leaks in `AiErrorCardTemplate` or `render_llm_error_toast()`.
5. **Performance:**
   - Semaphore release during retry backoff avoids blocking concurrent LLM jobs; early exit on `!still_in_flight` eliminates artificial `4s`/`250s` stalls when prefetch fails fast; view-aware skeletons use CSS `animate-pulse` with `motion-reduce:animate-none` and `pointer-events-none`.

### Non-Blocking Minor Notes (Informational Only)

- **`llm.rs#L330-L352` (`try_parse_impact_and_fallacies_response`):** An empty JSON object `{}` still parses and normalizes `fallacies` instead of triggering the 1-second retry (consistent with [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md#L41)).
- **`index.html#L185-L187` (Magazine Hero Skeleton):** Uses `sm:flex-row` / `sm:w-1/2` rather than `.magazine-card.priority-1` (`@container (min-width: 36rem)`), which can differ slightly from the loaded card when a narrow desktop split pane narrows the entries column.
- **`entries.rs#L428-L435` (`render_entries_list_html`):** Does not filter `record.model != "extractive-fallback"` when populating cached list card teasers (though `get_unprocessed_entry_ids` and `executive_brief_htmx` treat those rows as uncached and re-process them).

---

## Merged

**Result:** clean (copied from plan-1 `05-lens-api.md`)

---

## Reference Sources

- **Tokio `sleep`, `Semaphore`, `Command::kill_on_drop`, & `1.54.1` Advisory:** https://docs.rs/tokio/latest/tokio/time/fn.sleep.html · https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html · https://docs.rs/tokio/latest/tokio/process/struct.Command.html · https://rustsec.org/advisories/RUSTSEC-2026-0384.html
- **HTMX `outerHTML`, `htmx:afterSwap`, `htmx:oobAfterSwap`:** https://htmx.org/docs/#swapping · https://htmx.org/events/#htmx:afterSwap · https://htmx.org/attributes/hx-swap-oob/
- **Lucide `createIcons({ root })`:** https://lucide.dev/guide/packages/lucide
- **Tailwind CSS `motion-reduce` & MDN `aria-busy`:** https://tailwindcss.com/docs/hover-focus-and-other-states#reduced-motion · https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-busy
- **Repo Artifacts & Code:** [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) · [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) · [`06-test-report.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/06-test-report.md) · [`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) · [`ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs) · [`prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs) · [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) · [`error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs) · [`index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) · [`base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) · [`ai_grouped_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html)
