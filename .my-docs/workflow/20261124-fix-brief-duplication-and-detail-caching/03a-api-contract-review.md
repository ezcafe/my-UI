Result: clean

# API Contract Review: Executive Brief and Detail Caching

- **Workflow Slug:** `20261124-fix-brief-duplication-and-detail-caching`
- **Stage Id:** `api-contract-review`
- **Review Date:** 2026-11-24
- **Verdict:** `clean` (Zero Critical/Major issues)

---

## 1. Executive Summary

This review assesses the API contract modifications introduced in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03-design.md):
1. **Executive Brief Wire Contract (`GET /hx/executive-brief`)**: Removal of duplicate Out-of-Band (OOB) markup `#list-executive-brief` from the wire payload in favor of client event synchronization.
2. **Analysis Endpoint Contract (`GET /hx/entries/{id}/analysis`)**: Database reuse for stored analysis, returning identical HTML fragments without upstream Miniflux or LLM calls.
3. **Detail View Cache Contract**: IndexedDB client cache validation fixes and DOM-driven instant title display.
4. **LLM Generation Contract**: Inverted pyramid rules and strict word count limits within existing JSON schemas.

All contracts maintain backward compatibility, preserve HTTP semantics, and reduce wire payload bloat.

---

## 2. Detailed Contract Evaluations

### 2.1 Executive Brief Endpoint (`GET /hx/executive-brief`)

- **Method**: `GET`
- **Route**: `/hx/executive-brief`
- **Query Parameters**:
  - `offset` (`u64`, default `0`): Pagination offset.
  - `limit` (`u64`, default `20`): Page size limit.
  - `status` (`String`): Entry status filter (`unread`, `all`, `starred`, `today`).
  - `feed_id` (optional `i64`): Feed filter.
  - `category_id` (optional `i64`): Category filter.
- **Wire Response Changes**:
  - **Previous Payload**: Returned both `#page-executive-brief` and `#list-executive-brief` (with `hx-swap-oob="true"`). This caused duplicated markup rendering in magazine and dual-column layouts.
  - **New Payload**: Returns only `#page-executive-brief` (plus optional OOB title spans `entry-title-{id}`).
  - **Fallback Contract**: Returns `<div id="page-executive-brief" class="hidden"></div>` when brief generation is disabled, empty, or fails.
- **Client Synchronization**:
  - The client listens for `htmx:afterSwap` on `#page-executive-brief`.
  - When `#page-executive-brief` swaps with content, the client updates `#list-executive-brief` and clears its pulse animation.
  - **Recommendation**: If `#page-executive-brief` is hidden, the client listener should also hide `#list-executive-brief` so neither view shows an orphaned pulse skeleton.

### 2.2 Dedicated Detail Analysis Endpoint (`GET /hx/entries/{id}/analysis`)

- **Method**: `GET`
- **Route**: `/hx/entries/{id}/analysis`
- **Path Parameter**: `id` (`i64`): Entry ID.
- **Response Shape**:
  - **Success (with analysis)**:
    ```html
    <div id="ai-analysis-box" class="space-y-6 mt-6">
      <div class="rounded-lg p-4 bg-muted/40 border border-border">...Impact...</div>
      <div class="rounded-lg p-4 bg-muted/40 border border-border">...Fallacies...</div>
    </div>
    ```
  - **Disabled or Failure**:
    ```html
    <div id="ai-analysis-box" class="hidden"></div>
    ```
- **Contract Stability**:
  - Zero wire contract breaking changes.
  - The server checks `ai_service.db.get_summary_and_title(id).await` first.
  - If `impact` or `fallacies` is present, it formats and returns `render_analysis_cards_fragment` directly.
  - GraphQL Miniflux calls and LLM calls are completely bypassed on cache hits.

### 2.3 Client Detail Cache & Instant Rendering Contract

- **Storage**: Client IndexedDB store `article-details`.
- **Validation**:
  - Fixes obsolete check `cached.html.includes('Executive Brief')` to `Boolean(cached && cached.html && cached.html.trim())`.
  - Enables valid 30-day client cache hits for instant detail navigation.
- **Instant Title Contract**:
  - `openEntryDetail(id)` extracts text content from `#entry-title-{id}` in the DOM.
  - Renders title into `#entry-detail-title` immediately before the network fetch begins.

### 2.4 LLM Structured Prompt Contracts (Internal)

- **JSON Schemas**:
  - Single detail: `{"r": "...", "i": "...", "f": "..."}` deserialized into `DetailAnalysisResult`.
  - Batch page: `{"top": "...", "items": [{"i": 123, "t": "...", "r": "..."}]}` deserialized into `BatchLlmResponse`.
- **Prompt Constraints**:
  - Rewrite length: Under 110 words total (target 30–60 words).
  - Inverted pyramid structure: Lead with outcome and key impacts in first 2 sentences.
  - Supporting context: 2–3 concise bullet points.
  - Clean tone: Strictly remove conversational opening/closing boilerplate.

---

## 3. Findings and Recommendations

- **No Critical or Major findings**: All API routes and contracts adhere to HTTP standards and maintain clean separation of concerns.
- **Minor Implementation Note**: In `crates/server/src/routes/entries.rs`, verify that both `impact` and `fallacies` can be rendered independently if only one of them is present in the database record. `render_analysis_cards_fragment` already supports rendering either or both, matching the updated check `!imp.is_empty() || !fal.is_empty()`.

---

## 4. References

- HTMX Out-of-Band Swaps: <https://htmx.org/attributes/hx-swap-oob/>
- TanStack Query Deduplication and Caching: <https://tanstack.com/query/latest/docs/framework/react/guides/caching>
- Implementation routes: [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
- Card template: [executive_brief_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/executive_brief_card.html)
- LLM prompt templates: [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs)
