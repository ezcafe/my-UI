Result: clean

# API Contract Review: AI News Batch Summary

- **Stage:** `api-contract-review` (Mode full / isolated review)
- **Status:** Paid (`clean`)
- **Review Target:** Axum HTTP/HTMX Routes, Askama Partial Templates, and Content Extraction / LLM Integration

---

## 1. Executive Summary

This API contract review evaluates the interface contracts for the AI News Batch Summary feature specified in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03-design.md) following the design update for `<article>` tag content parsing.

Key takeaways:
- **Zero Critical / Major Blockers**: All endpoints, query parameters, HTMX event lifecycles, and error responses comply with REST conventions and project architecture.
- **RESTful Path Standardization**: Canonical route `GET /hx/entries/{id}/ai-summary` aligns cleanly with existing entry endpoints in [crates/server/src/app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L44-L46).
- **Guaranteed UI Unsticking**: Formalizes the HTTP 200 contract for lazy-loaded summary cards so HTMX `revealed` slots always swap out and never leave frozen loading skeletons.
- **DOM Parsing Compatibility**: The update to parse `<article>` tags in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03-design.md#L144-L155) is isolated to backend extraction and does not alter client-facing API parameters or response schemas.

---

## 2. Interface Contract Specifications

### 2.1 `GET /hx/entries?status=ai` (Batch Digest List)

- **Route & Method**: `GET /hx/entries?status=ai`
- **Authentication**: Required (`_user: AuthenticatedUser`). Unauthenticated requests redirect to `/login`.
- **Target Container**: `#entries-list` via `hx-target="#entries-list"` and `hx-swap="innerHTML"`.
- **Query Parameters**:
  - `status` (`String`, required): Must equal `"ai"`.
  - `search` (`Option<String>`, optional): When present and non-empty, filtered entries are constrained to matching titles/content via GraphQL query forwarding.
  - `feed_id` / `category_id` (`Option<i64>`, optional): If present, restricts entries to the selected feed or folder.
  - `limit` (`Option<i64>`, optional): Defaults to `15`, capped at `15`.
- **Response Contract**:
  - **Content-Type**: `text/html; charset=utf-8`.
  - **Success (200 OK)**: Renders `AiGroupedListTemplate` containing up to 15 entries grouped by feed title (`Vec<FeedSummaryGroupViewModel>`).
  - **Empty State (200 OK)**: If no articles exist or match the search query, renders an informative empty state partial.
  - **Feature Disabled (404 Not Found)**: If `config.ai_enabled` is false, returns `handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()))`.
  - **Upstream Error (500 Internal Server Error)**: If Miniflux/GraphQL query fails, returns `handle_htmx_error(&headers, AppError::Upstream(...))` with `HX-Reswap: none`.

### 2.2 `GET /hx/entries/{id}/ai-summary` (Lazy-Loaded Summary Fragment)

- **Route & Method**: `GET /hx/entries/{id}/ai-summary` (canonical REST path matching `/hx/entries/{id}/...` conventions).
- **Authentication**: Required (`_user: AuthenticatedUser`).
- **Path Parameter**: `id` (`i64`, required): Miniflux entry ID.
- **Trigger & Swap**: `hx-trigger="revealed"`, `hx-swap="outerHTML"`.
- **Response Contract**:
  - **Content-Type**: `text/html; charset=utf-8`.
  - **Success (200 OK)**: Renders `AiSummaryCardTemplate` with `summary: String`. Replaces `.ai-summary-slot` with `.ai-summary-card`.
  - **Scraper / LLM Fallback (200 OK)**: If remote scraping fails (403 Forbidden, Cloudflare challenge, network timeout) or LLM CLI fails/times out, returns `AiSummaryCardTemplate` populated with extractive summary from `miniflux_client::summary::extract_summary(teaser, 2)`.
  - **Entry Not Found / Unauthorized (200 OK Fallback)**: If entry `id` does not exist in Miniflux or does not belong to the user, returns a replacement snippet (`<div class="ai-summary-card text-xs text-slate-500 italic mt-3 pt-3 border-t border-white/5">Summary unavailable</div>`) to ensure the skeleton element is replaced in the DOM.

---

## 3. REST & HTMX Conformance Checklist

- [x] **Resource Hierarchy**: Uses canonical REST sub-resource path `/hx/entries/{id}/ai-summary` matching existing endpoints in [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L44-L46).
- [x] **Idempotency & Safe Methods**: Both endpoints use `GET`. Retrieval and cache population do not mutate article state (read/starred flags remain unchanged).
- [x] **Viewport Concurrency Control**: Tokio `Semaphore(2)` in `AppState` serializes concurrent LLM subprocess spawns during rapid scrolling, preventing thread starvation.
- [x] **Cache Fast-Path**: SQLite cache lookups bypass the semaphore and return in under 2ms, avoiding unnecessary lock acquisition.
- [x] **Filter Bar Compatibility**: The `[AI]` button in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) mirrors existing `.filter-pill` buttons with `hx-include="[name='search']"` and `onclick="setActiveFilter(this)"`.
- [x] **DOM Content Parsing**: Clean extraction of `<article>` content conforms to the updated design without requiring any query or header modifications.

---

## 4. Implementation Directives

### Directive API-01: Canonical Route Registration
- **Description**: Standardize the Axum route registration to `/hx/entries/{id}/ai-summary`.
- **Implementation**:
  In [crates/server/src/app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs):
  ```rust
  .route("/hx/entries/{id}/ai-summary", get(get_entry_ai_summary_htmx))
  ```
  In [crates/ui-components/templates/partials/ai_grouped_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html):
  ```html
  <div class="ai-summary-slot mt-3 pt-3 border-t border-white/5"
       hx-get="/hx/entries/{{ item.id }}/ai-summary"
       hx-trigger="revealed"
       hx-swap="outerHTML">
  ```

### Directive API-02: Guaranteed OuterHTML Swap on Reveal
- **Description**: HTMX `revealed` triggers will freeze the loading skeleton permanently if the server returns an error with `HX-Reswap: none`.
- **Implementation**:
  `get_entry_ai_summary_htmx` must always return HTTP 200 with an HTML card fragment replacing `.ai-summary-slot`, even when external web scraping or LLM execution fails.

### Directive API-03: Search Parameter Forwarding in `status=ai`
- **Description**: Users may filter the 15-item digest by typing in the search box.
- **Implementation**:
  When handling `GET /hx/entries?status=ai`, check `query.search`. If non-empty, append `search: "<escaped_query>"` to the GraphQL `entries` query filter.

---

## 5. Contract Sign-Off

- **Critical Findings:** 0
- **Major Findings:** 0
- **Minor Directives:** 3 (addressed in Section 4)
- **Result:** `clean`
