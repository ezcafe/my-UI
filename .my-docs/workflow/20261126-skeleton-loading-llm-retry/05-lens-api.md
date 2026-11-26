Result: clean

# Lens: API Contract Review

- **Workflow Slug:** `20261126-skeleton-loading-llm-retry`
- **Lens:** `api`
- **Result:** clean

## Summary

All modified HTMX endpoint contracts (`/hx/executive-brief`, `/hx/entries/{id}/ai-summary`, `/hx/entries/{id}`, `/hx/entries/{id}/analysis`, `/hx/entries/{id}/rewrite`) and backend LLM/prefetch service interfaces (`LlmCliRunner`, `LlmHttpClient`, `LlmBackend`, `AiService`, `AiPrefetchService`) match [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) and API design best practices.

---

## API Checklist

| Requirement | Status | Verification Notes |
|---|---|---|
| **Typed input and output; create-input vs full entity** | Pass | Strong Rust request/domain types (`EntriesFilterQuery`, `Path<i64>`, `BatchArticleInput`, `BatchAiResponse`, `DetailAnalysisResult`, `AiSummaryRecord`) and Askama view templates (`AiErrorCardTemplate`, `ToastTemplate`, `AiSummaryCardTemplate`, `ExecutiveBriefCardTemplate`). |
| **One stable error shape; correct status codes** | Pass | LLM exhaustion across all 4 AI endpoints returns `200 OK` (preserving HTMX `outerHTML` swap semantics) with `AiErrorCardTemplate` (`class="... is-error"`) + OOB `ToastTemplate` (`render_llm_error_toast()`), while Miniflux `404`/not-found on `/hx/entries/{id}/ai-summary` retains `"Summary unavailable"` without toast. |
| **Validate at system edges only** | Pass | Axum validates `Path<i64>` and `Query<EntriesFilterQuery>` (`limit.clamp(1, 100)`, `offset.max(0)`, `is_brief_eligible_filter`) at router edges; `LlmHttpClient::request_chat_completion_once` and `LlmCliRunner::try_parse_*` validate external LLM payloads once at the AI boundary without redundant internal re-validation. |
| **Lists paginated; filters via query params** | Pass | `/hx/executive-brief` uses `offset`, `limit`, `status`, `feed_id`, `category_id` query parameters and `build_retry_url()` preserves active query parameters on Retry buttons. |
| **Additive fields; no silent breaking changes** | Pass | HTTP contracts remain backward-compatible `200 OK` fragment swaps targeting `#page-executive-brief`, `#ai-summary-slot-{id}`, `#ai-analysis-box`, and `#ai-rewrite-box` plus `#toast-container` OOB. Internal `AiService::get_or_create_summary` return type change to `Result<String, String>` is updated at its call site in `entries.rs`. |
| **Naming consistent with the repo** | Pass | Follows existing `*_htmx` handler names, `AiErrorCardTemplate`, `LLM_HIGH_DEMAND_ERROR_MESSAGE`, `render_llm_error_toast()`, and `extractive-fallback` model sentinel naming across `crates/server` and `crates/ui-components`. |
| **Mutating endpoints idempotent or documented** | Pass | All modified AI routes are `GET` hypermedia fragments with read-through DB caching, `in_flight` deduplication, idempotent SQL upserts (`ON CONFLICT DO UPDATE`), and `hx-disabled-elt="this"` on Retry buttons; failed runs skip persisting `"extractive-fallback"` rows. |
| **Contract in `03-design.md` matches code** | Pass | Verified parity across `executive_brief_htmx` (`250s` `in_flight` cap with immediate exit, partial vs full failure persistence), `get_entry_ai_summary_htmx` (`404` vs `Err(_)` separation), `get_entry_detail_htmx`, `get_entry_analysis_htmx`, `get_entry_rewrite_htmx`, `LlmCliRunner`, `LlmHttpClient`, `AiService`, and `AiPrefetchService`. |
| **Do not leak internals in errors** | Pass | User-facing error cards and OOB toasts render static `LLM_HIGH_DEMAND_ERROR_MESSAGE` with Askama HTML escaping; internal `CliAttemptError` variants, HTTP status codes, and raw stderr stay in server-side `tracing` logs. |

---

## Findings

- **0 Critical, 0 Major issues found.**
- **Verified Endpoints & Contracts:**
  - `GET /hx/executive-brief` ([`entries.rs#L677-L1062`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L677-L1062)): waits up to `250s` on `in_flight` with early exit on clear; skips DB save on `"extractive-fallback"` or empty `top && items`; persists non-fallback `items` (`brief_text = None`) when `top` is empty; emits `AiErrorCardTemplate` (`#page-executive-brief`, `outerHTML`) + `render_llm_error_toast()` on brief errors.
  - `GET /hx/entries/{id}/ai-summary` ([`entries.rs#L1064-L1162`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1064-L1162), [`ai_grouped_list.html#L17-L29`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html#L17-L29)): requires `AuthenticatedUser`; preserves `"Summary unavailable"` (no error card, no toast) on Miniflux `404`/not-found; returns `AiErrorCardTemplate` (`#ai-summary-slot-{id}`, `outerHTML`) + `render_llm_error_toast()` on `get_or_create_summary` `Err(_)`.
  - `GET /hx/entries/{id}` ([`entries.rs#L1427-L1466`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1427-L1466)): ignores legacy `"extractive-fallback"` rows (`model != "extractive-fallback"`).
  - `GET /hx/entries/{id}/analysis` ([`entries.rs#L1672-L1771`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1672-L1771)): ignores legacy `"extractive-fallback"` rows; returns `AiErrorCardTemplate` (`#ai-analysis-box`, `outerHTML`) + `render_llm_error_toast()` on `Err(_)`.
  - `GET /hx/entries/{id}/rewrite` ([`entries.rs#L1773-L1895`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1773-L1895)): ignores legacy `"extractive-fallback"` rows in cache & graceful fallback; returns `#ai-rewrite-box` (`AiErrorCardTemplate` + fallback prose) + `render_llm_error_toast()` on `Err(_)`.
  - `LlmCliRunner` / `LlmHttpClient` / `AiPrefetchService` ([`llm.rs#L11-L1193`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L11-L1193), [`prefetch.rs#L150-L159`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs#L150-L159)): single 1s retry per backend without `2x2` multiplication, semaphore permit release before sleep, strict HTTP empty/parse checks, and no persistence of `"extractive-fallback"`.
- **Non-Blocking Implementation Notes:**
  - `try_parse_impact_and_fallacies_response` (`llm.rs#L330-L352`) accepts `{}` and normalizes `fallacies` instead of retrying (already tracked as non-blocking in [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md#L41)).
  - `render_entries_list_html` (`entries.rs#L428-L435`) does not filter `record.model != "extractive-fallback"` on initial list teaser rendering (though `get_unprocessed_entry_ids` and `executive_brief_htmx` treat those entries as uncached for re-generation).

---

## Reference Sources

- **Axum 0.8 Routing & Extractors:** https://docs.rs/axum/latest/axum/
- **Reqwest 0.12 Client & Timeouts:** https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
- **Tokio `sleep` & `Semaphore`:** https://docs.rs/tokio/latest/tokio/time/fn.sleep.html · https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
- **HTMX Swapping & OOB:** https://htmx.org/docs/#swapping · https://htmx.org/attributes/hx-swap-oob/
- **Repo Artifacts & Source:** [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) · [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md) · [`entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) · [`routes/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs) · [`llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) · [`ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs) · [`prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs) · [`error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs)
