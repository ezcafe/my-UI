Result: clean

# API Contract Review — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

- **Slug:** `20261126-skeleton-loading-llm-retry`
- **Stage:** `design-verify-phase` (Isolated API surface — Mode `simple`, `Has API = yes`)
- **Surface Reviewed:** Hypermedia endpoints (`/hx/executive-brief`, `/hx/entries/{id}/ai-summary`, `/hx/entries/{id}/analysis`, `/hx/entries/{id}/rewrite`, `/hx/entries/{id}`) and LLM/Prefetch service contracts ([`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs), [`ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs), [`prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs), [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)).

## Key Takeaways

- **Verdict — `clean` (0 Critical, 0 Major):** All 3 Major and 1 Minor API/backend findings from Round 1 have been cleanly resolved in [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) and [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md).
- **404 vs LLM failure separation verified:** `get_entry_ai_summary_htmx` keeps the existing `<div class="ai-summary-card ...">Summary unavailable</div>` response without an LLM demand toast on Miniflux `404`/not-found, preserving [`mod.rs#L1222`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1222), and updates [`mod.rs#L1258-L1259`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1258-L1259) in Task 3.
- **`in_flight` wait timeout & fallback math verified:** Raising `executive_brief_htmx` polling to `250s` (`2500 * 100ms`) safely covers worst-case `ArticleExtractor` (`3s`) + `LlmHttpClient` (`2 * 30s + 1s = 61s`) + `LlmCliRunner` (`2 * 90s + 1s = 181s`) = `245s`, while exiting right away when `!still_in_flight` and DB still lacks a brief.
- **Strict HTTP + CLI retry & legacy guards verified:** `LlmHttpClient` rejects empty HTTP `200` text and unparseable batch/analysis JSON via shared strict parsers without nested `2x2` retries, and detail/analysis/rewrite/prefetch paths skip or ignore `"extractive-fallback"` rows.

---

## Round 1 Findings Verification

| Prior Finding | Resolution Verified in `03-design.md` & `04-tasks.md` | Status |
|---|---|---|
| **Major #1: Ambiguous 404/not-found contract in `get_entry_ai_summary_htmx` & missing `mod.rs:1259` test note** | [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md#L236-L239) keeps `"Summary unavailable"` (no `AiErrorCardTemplate`, no `render_llm_error_toast()`) on Miniflux `404`/not-found or Askama error, and returns `AiErrorCardTemplate` (`#ai-summary-slot-{id}`, `outerHTML`) + `render_llm_error_toast()` exclusively on `Err(_)` from `get_or_create_summary`. [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md#L45) lists `mod.rs` lines `1258, 1259` while leaving `mod.rs:1222` intact. | **Resolved** |
| **Major #2: `executive_brief_htmx` `95s` wait cap shorter than two `90s` CLI attempts** | [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md#L226-L230) raises the cap to `2500` iterations (`250s` at `100ms`), exceeding `3s` extraction + `61s` HTTP + `181s` CLI (`245s`), with immediate early exit once `in_flight` clears and re-checking `get_page_brief_by_key` still returns `None`. | **Resolved** |
| **Major #3: `LlmHttpClient` lacked empty-text and inner parse validation before retry/fallback** | [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md#L189-L210) rejects `text.trim().is_empty()` in `request_chat_completion_once` and wraps `request_chat_completion_once` + `LlmCliRunner::try_parse_batch_response` / `try_parse_impact_and_fallacies_response` inside `retry_once_after_delay` (capped at 1 retry / 2 HTTP attempts before `LlmBackend::Http` falls back to CLI). | **Resolved** |
| **Minor #4: Legacy `"extractive-fallback"` DB rows & `top.is_empty()` save clarity** | [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md#L232-L245) guards `get_entry_detail_htmx`, `get_entry_analysis_htmx`, `get_entry_rewrite_htmx`, and `AiService::analyze_entry_impact_and_fallacies` with `record.model != "extractive-fallback"`, and specifies skipping DB save on full failure while saving non-fallback `items` (`brief_text = None`) on partial real-LLM responses. | **Resolved** |

---

## Endpoint & Service Contract Checklist

| Surface | Contract Behavior | Verified Against Codebase | Status |
|---|---|---|---|
| `GET /hx/executive-brief` | `200 OK`; `in_flight` wait up to `250s` with immediate exit on clear; skips DB write on `"extractive-fallback"` or empty `top && items`; saves valid partial `items` (`brief_text = None`); all brief failures return `AiErrorCardTemplate` (`#page-executive-brief`, `outerHTML`) + `render_llm_error_toast()`. | [`entries.rs#L677-L1030`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L677-L1030), [`extractor.rs#L18-L25`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs#L18-L25) | **Pass** |
| `GET /hx/entries/{id}/ai-summary` | `200 OK`; preserves `AuthenticatedUser` + Miniflux `404` `"Summary unavailable"` (no toast); returns `AiErrorCardTemplate` (`#ai-summary-slot-{id}`, `outerHTML`) + `render_llm_error_toast()` on `get_or_create_summary` `Err(_)`. | [`entries.rs#L1032-L1107`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1032-L1107), [`routes/mod.rs#L1188-L1260`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1188-L1260) | **Pass** |
| `GET /hx/entries/{id}/analysis` | `200 OK`; ignores legacy `"extractive-fallback"` rows; returns `AiErrorCardTemplate` (`#ai-analysis-box`, `outerHTML`) + `render_llm_error_toast()` on `Err(_)`. | [`entries.rs#L1612-L1702`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1612-L1702) | **Pass** |
| `GET /hx/entries/{id}/rewrite` | `200 OK`; ignores legacy `"extractive-fallback"` rows in cache & fallback read; returns `#ai-rewrite-box` (`AiErrorCardTemplate` + fallback prose) + `render_llm_error_toast()` on `Err(_)`. | [`entries.rs#L1704-L1815`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1704-L1815) | **Pass** |
| `GET /hx/entries/{id}` | Ignores legacy `"extractive-fallback"` DB rows (`is_real_ai = record.model != "extractive-fallback"`), falling back cleanly to Miniflux teaser/summary with `has_analysis = false`. | [`entries.rs#L1372-L1405`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1372-L1405) | **Pass** |
| `LlmCliRunner` / `LlmHttpClient` / `AiPrefetchService` | Single `retry_once_after_delay` (`1s`); CLI semaphore permit acquired per attempt (dropped before sleep); HTTP validates non-empty output & strict JSON/batch parse; `AiPrefetchService::run_prefetch` skips persisting `"extractive-fallback"`. | [`llm.rs#L474-L1158`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L474-L1158), [`prefetch.rs#L50-L185`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs#L50-L185) | **Pass** |

### Implementation Note (Non-Blocking)
- In `LlmCliRunner::try_parse_impact_and_fallacies_response`, consider also rejecting an empty JSON object (`{}` where both `raw.impact.trim().is_empty()` and `raw.fallacies.trim().is_empty()` before fallacy normalization) so `{}` triggers the 1-second retry rather than normalizing `fallacies` on an empty object.

---

## Reference Sources

- **Tokio `tokio::time::sleep` & `tokio::sync::Semaphore`:** https://docs.rs/tokio/latest/tokio/time/fn.sleep.html · https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
- **Reqwest `ClientBuilder::timeout`:** https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
- **HTMX swapping & OOB swaps:** https://htmx.org/docs/#swapping · https://htmx.org/attributes/hx-swap-oob/
- **Workflow & Source Files:** [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) · [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) · [`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) · [`ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs) · [`prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs) · [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) · [`routes/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs) · [`error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs)
