Result: clean

# Lens

**Lens:** api  
**Result:** clean

## Key Takeaways

- **Verdict:** **Clean** — zero Critical or Major findings across all exposed HTML/HTMX endpoints, response headers, OOB contracts, error shapes, and embedded `#entries-guest-data` JSON payloads.
- **Contract parity:** Implementation across [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs), and [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) matches every API and OOB contract specified in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md).
- **Additive compatibility & cache safety:** `EntryViewModel` uses `#[serde(default)]` on `rewrite` and `body_images`, while client IndexedDB stores bump (`my_news_article_cache_db` v2, `my_news_guest_db` v3) so stale pre-filter payloads are cleared on upgrade.

## API Checklist

| Check | Status | Evidence |
|---|---|---|
| Typed input and output; create-input vs full entity | Pass | Handlers use typed Axum `Path<i64>`, `Query<EntriesFilterQuery>`, `Query<ReadStatusQuery>`, and `Form<MarkPageReadForm>`; outputs render typed Askama templates (`EntryListTemplate`, `EntryDetailTemplate`, `AnalysisCardsTemplate`, `AiErrorCardTemplate`, `ExecutiveBriefCardTemplate`) and typed JSON structs (`GuestEntryPayload<'a>`, `BriefSyncEntry<'a>`). |
| One stable error shape; correct status codes | Pass | Detail/AI endpoints return HTTP `200 OK` with `AiErrorCardTemplate` / `ErrorCardTemplate` (`.is-error`) + `render_llm_error_toast()` for HTMX 1.9 swapping, while `list_entries_htmx` / mutations use `handle_htmx_error` (`HX-Reswap: none` toast on HTMX; `500` otherwise) and `403 FORBIDDEN` when unauthenticated mutations are disabled; client `htmx:responseError` / `htmx:sendError` handles 429/5xx/network failures on `#ai-rewrite-box` with matching `.is-error` card + fallback + `ai-rewrite-done-{id}`. |
| Validate at system edges only (not re-validate trusted internals) | Pass | URL safety (`safe_entry_url`, `extract_article_images`, `safeHref`), HTML-to-plain sanitization (`extract_plain_summary`, `clean_opt_summary`), and `limit`/`offset` clamping (`executive_brief_htmx`) execute at boundary ingestion/render edges without redundant internal re-validation. |
| Lists paginated; filters via query params; PATCH for partial updates | Pass | `GET /hx/entries` and `GET /hx/executive-brief` paginate via `limit`/`offset` and filter via `status`, `feed_id`, `category_id`, `search`, `request_brief` query params; no new mutating endpoints introduced. |
| Additive fields; no silent breaking changes | Pass | `EntryViewModel` marks `rewrite` and `body_images` `#[serde(default)]`; `EntryListTemplate` (`auth_enabled`), `EntryDetailTemplate` (`needs_rewrite`, `ai_enabled`, `fallback_summary`), and `AnalysisCardsTemplate` (`oob`, `entry_id`) update all callers; `GuestEntryPayload` intentionally removes `sanitized_content`/metadata per approved design and bumps IndexedDB `my_news_guest_db` to v3 (`schema_version: 2`). |
| Naming consistent with the repo | Pass | Route paths (`/hx/entries/{id}`, `/hx/entries/{id}/rewrite`, `/hx/entries/{id}/analysis`, `/hx/executive-brief`), custom event (`ai-rewrite-done-{id}`), attributes (`data-target-entry-id`, `data-fallback-summary`), and OOB container IDs (`#ai-rewrite-box`, `#ai-analysis-box`, `#executive-brief-sync-data`) match repo conventions. |
| Mutating endpoints: idempotent or documented unsafe-to-retry | Pass | All new/modified detail, rewrite, analysis, and executive-brief endpoints are `GET` handlers; existing auth unread-to-read transition in `get_entry_detail_htmx` remains idempotent and gated on `config.auth_enabled`. |
| Contract in `03-design.md` matches what the code exposes (and vice versa) | Pass | Verified across all 5 surfaces below; `AnalysisCardsTemplate` additionally accepts `entry_id: Option<i64>` (`data-target-entry-id`) to enforce cross-article OOB swap guarding cleanly. |
| Do not leak internals in errors | Pass | `get_entry_detail_htmx`, `get_entry_rewrite_htmx`, `get_entry_analysis_htmx`, `executive_brief_htmx`, and `handle_htmx_error` emit fixed user-facing copy (`LLM_HIGH_DEMAND_ERROR_MESSAGE`, `AppError::user_facing_message`) without raw DB errors, stack traces, or upstream URLs. |

## Surfaces Verified

1. **`GET /hx/entries/{id}` (`get_entry_detail_htmx`, `EntryDetailTemplate`, [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)):**
   - Verified at [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1434-L1736) and [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L170).
   - Preserves `raw_summary` for image extraction fallback, ignores `"extractive-fallback"`, requires non-empty `record.rewrite` before marking `needs_rewrite = false`, computes plain-text `fallback_summary` (defaulting to `"No summary available."`), omits `feed_title` / `<time>` / original body HTML, validates `Original` + Guest bookmark URLs via `safe_entry_url()`, scopes `#ai-analysis-box` trigger to `ai-rewrite-done-{id} from:body` when `needs_rewrite` (`load` otherwise, omitted when `!ai_enabled`), and tags both boxes with `data-target-entry-id`.

2. **`GET /hx/entries/{id}/rewrite` (`get_entry_rewrite_htmx`):**
   - Verified at [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1738-L1793) and [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1912-L2051).
   - Emits `HX-Trigger-After-Settle: ai-rewrite-done-{id}` across all 4 execution paths (cache hit, `!config.ai_enabled`, LLM success, LLM failure).
   - Returns rewrite-only `#ai-rewrite-box` (`data-target-entry-id="{id}"`) plus sibling `AnalysisCardsTemplate { oob: true, entry_id: Some(id) }` when `impact` or `fallacies` is non-empty.
   - Falls back to sanitized plain text (or `"No summary available."`) on both `!config.ai_enabled` (inline warning) and LLM error (`AiErrorCardTemplate` + `data-fallback-summary` + `render_llm_error_toast()`).

3. **`GET /hx/entries/{id}/analysis` (`get_entry_analysis_htmx`, `AnalysisCardsTemplate`, [analysis_cards.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards.html)):**
   - Verified at [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1795-L1910) and [analysis_cards_inner.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html).
   - Reuses `AnalysisCardsTemplate { oob: false, entry_id: Some(id) }`; shares card markup via `analysis_cards_inner.html`; returns `AiErrorCardTemplate` + OOB toast on LLM failure.

4. **`GET /hx/executive-brief` (`executive_brief_htmx`):**
   - Verified at [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L704-L735), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L889-L1189), and [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L240).
   - Filters `extractive-fallback`, prefers `!item.brief.trim().is_empty()` with fallback to `item.rewrite`, sanitizes synced summaries via `clean_opt_summary`, and emits `#entry-title-{id}` OOB spans plus `<div id="executive-brief-sync-data" hidden hx-swap-oob="outerHTML" data-entries="..."></div>` across cache hit, in-flight completion, and fresh synthesis (including Case C partial responses).

5. **Embedded `#entries-guest-data` JSON (`EntryViewModel`, `EntryListTemplate`, `GuestEntryPayload`):**
   - Verified at [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4-L27), [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L172-L216), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L209-L498), and [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs#L163-L204).
   - Gates `#entries-guest-data` on `!self.auth_enabled && !self.entries.is_empty()`, emits `{ id, title, url, summary, rewrite, image_url, body_images }` escaped via `escape_json_for_html_script`, omits all original content/author/date/feed metadata, and keeps Auth/Guest `http://`/`https://` image extraction aligned.

## Findings

None.

## Reference Sources

- Axum `0.8.x` routing & extractors: <https://docs.rs/axum/latest/axum/>
- HTMX `1.9` `hx-swap-oob`, `htmx:beforeSwap`, `htmx:oobBeforeSwap`, `HX-Trigger-After-Settle`: <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/headers/hx-trigger/> · <https://htmx.org/events/>
- Askama `0.12` templates: <https://docs.rs/askama/0.12.1/askama/>
- `ammonia` `4.x` `Builder::tags`: <https://docs.rs/ammonia/latest/ammonia/struct.Builder.html#method.tags>
- Design & task contracts: [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md) · [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03a-api-contract-review.md)
