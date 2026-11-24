Result: clean

# API Contract Review: Performance, Loading, and Token Optimization

**Date:** 2026-11-24  
**Workflow Slug:** `20261124-perf-loading-token-optimization`  
**Review Stage:** `api-contract-review`  
**Target Design:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03-design.md)  

---

## 1. Executive Summary

This review evaluates the API contracts defined in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03-design.md) for performance, loading, and token optimization. The review focuses specifically on:
1. The new HTMX on-demand rewrite endpoint (`GET /hx/entries/{id}/rewrite`).
2. Canonical page cache key unification (`compute_page_key(status, entry_ids)`).
3. Query parameter handling and filter eligibility in `executive_brief_htmx`.
4. Static CSS delivery and CDN replacement (`GET /static/annuaire-theme.css`).

- **Review Verdict:** `Result: clean`.
- **Summary:** The proposed API contracts are sound, backwards-compatible, and properly isolated. They eliminate redundant LLM executions, avoid layout shifts, and provide instant visual feedback with resilient fallback cascades. Zero Critical or Major issues were identified. Four minor implementation hardening recommendations are noted below.

---

## 2. API Contract Specifications & Evaluation

### 2.1 On-Demand Detail Rewrite Endpoint (`GET /hx/entries/{id}/rewrite`)

- **Contract Definition:**
  - **HTTP Method:** `GET`
  - **Path:** `/hx/entries/{id}/rewrite`
  - **Path Parameter:** `id: i64` (entry identifier)
  - **Authentication:** `_user: AuthenticatedUser` extractor (enforced when `config.auth_enabled` is active)
  - **Request Headers:** Standard HTMX headers (`HX-Request: true`)
  - **Response Headers:**
    - `Content-Type: text/html; charset=utf-8`
    - `Cache-Control: no-cache, no-store, must-revalidate`
  - **Response Payload:** HTML fragment replacing `#ai-rewrite-box` via `hx-swap="outerHTML"`:
    ```html
    <div id="ai-rewrite-box" class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
      {{ rewrite }}
    </div>
    ```

- **Contract Verification:**
  - **Fast-path SQLite check:** The endpoint checks `AiDb` first. If an executive rewrite already exists (such as for the top-1 eager article or a previously opened story), it returns the rendered fragment in `< 5ms` without invoking the LLM.
  - **OuterHTML swap compatibility:** The returned fragment has the matching root ID `id="ai-rewrite-box"`, guaranteeing a clean 1:1 replacement of the skeleton loader without layout deformation.
  - **Graceful degradation:** If the LLM call times out or fails, the handler falls back to rendering the 3-sentence executive brief inside `#ai-rewrite-box`, ensuring the reader is never left with an infinite loading skeleton or blank screen.

---

### 2.2 Canonical Page Cache Key & Deduplication

- **Target Function:** `crates/server/src/ai/mod.rs`
  ```rust
  pub fn compute_page_key(status: &str, entry_ids: &[i64]) -> String
  ```
- **Hashing Logic:**
  ```rust
  let mut hasher = Sha256::new();
  for id in entry_ids {
      hasher.update(id.to_le_bytes());
  }
  let hash = format!("{:x}", hasher.finalize());
  format!("{}:{}", status, &hash[..16])
  ```

- **Contract Verification:**
  - **Divergence resolved:** Previously, `AiPrefetchService` generated keys with prefix `"prefetch"` using only uncached entry IDs, while `executive_brief_htmx` used the filter status (`"unread"`) and all entry IDs on the page. Unifying the signature and arguments guarantees that prefetch and foreground brief compute identical keys for the same page view.
  - **In-flight deduplication:** Both background prefetch and foreground brief acquire the lock on `ai_service.in_flight` using the identical `page_key`. If prefetch is already running for a page, the foreground route awaits the result instead of triggering duplicate LLM work.
  - **Status normalization:** The status parameter defaults canonically to `"unread"` when `query.status` is absent or empty, maintaining key consistency across requests.

---

### 2.3 Query Parameter Handling & Filter Eligibility

- **Query Struct:** `EntriesFilterQuery` in [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs):
  - `status: Option<String>` (`"unread"`, `"read"`, `"starred"`, `"today"`, `"all"`)
  - `feed_id: Option<i64>`
  - `category_id: Option<i64>`
  - `search: Option<String>`
  - `limit: Option<i64>` (default: 15, clamped: 1..100)
  - `offset: Option<i64>` (default: 0, min: 0)
  - `request_brief: Option<bool>`

- **Contract Verification:**
  - **Brief eligibility rule (`is_brief_eligible_filter`):** Executive briefs are only generated when `status` is `"unread"` or `None`, and `search` is empty. For search queries or other statuses (`"read"`, `"starred"`), `executive_brief_htmx` returns an empty 200 OK response immediately, consuming 0 tokens.
  - **Slice isolation:** Page changes via `limit` and `offset` supply different slices of `entry_ids`, which naturally hash to distinct `page_key` values without key collision.
  - **Drawer route isolation:** The on-demand rewrite endpoint (`/hx/entries/{id}/rewrite`) operates strictly on the article ID path parameter and does not depend on page-level filter query parameters, preventing cache pollution from filter states.

---

### 2.4 Static CSS Route Contract (`GET /static/annuaire-theme.css`)

- **Contract Definition:**
  - **HTTP Method:** `GET`
  - **Path:** `/static/annuaire-theme.css`
  - **Response Headers:**
    - `Content-Type: text/css; charset=utf-8`
    - `Cache-Control: public, max-age=86400`
  - **Response Status:** `200 OK` (or `404 Not Found` for non-matching assets)

- **Contract Verification:**
  - **Zero runtime JS evaluation:** Completely eliminates `cdn.tailwindcss.com` and inline `tailwind.config = { ... }` script blocks from `base.html`.
  - **FOUC elimination:** Browser loads precompiled CSS directly via `<link rel="stylesheet" href="/static/annuaire-theme.css">` in `<head>`, removing the 150–300ms layout shift and visual flash on initial load.
  - **Cacheability:** Long-lived `max-age=86400` header enables fast edge and browser caching.

---

## 3. Findings & Implementation Hardening Recommendations

### Critical / Major Issues
- **None (0 findings).**

### Minor Implementation Recommendations (Non-blocking)

1. **Authentication Guard on Rewrite Endpoint:**
   - **Detail:** In `crates/server/src/routes/entries.rs`, ensure `get_entry_rewrite_htmx` includes `_user: AuthenticatedUser` in its signature, matching `get_entry_detail_htmx` and `get_entry_ai_summary_htmx`.
   - **Action:** Enforce authentication when `config.auth_enabled` is true so unauthenticated callers cannot trigger arbitrary LLM rewrites.

2. **AI-Disabled Guard in Rewrite Handler:**
   - **Detail:** If `config.ai_enabled` is `false`, `get_entry_rewrite_htmx` should return the existing summary or teaser formatted inside `#ai-rewrite-box` rather than returning an error card or 404.
   - **Action:** Check `if !config.ai_enabled` at the top of the handler and render the teaser/summary fallback cleanly.

3. **Cache-Control Header Consistency:**
   - **Detail:** Dynamic HTMX fragments should explicitly set `Cache-Control: no-cache, no-store, must-revalidate`.
   - **Action:** Add standard no-cache headers to `get_entry_rewrite_htmx` response to prevent browsers from caching transient failure states.

4. **Preserve Container ID & Classes in All Template States:**
   - **Detail:** Both the skeleton state in `entry_detail.html` and the fulfilled rewrite returned by `get_entry_rewrite_htmx` must have root `<div id="ai-rewrite-box" class="prose dark:prose-invert ...">`.
   - **Action:** Keep container class lists synchronized between the Askama template and the HTMX fragment to avoid visual jumps during `hx-swap="outerHTML"`.

---

## 4. Conclusion & Stage Verdict

The API contract specifications in `03-design.md` satisfy all architectural, performance, and resilience requirements.

- **Stage Verdict:** `Result: clean`
- **Next Steps:** Proceed with the parallel isolated sibling review (`db-design-review`) or parent derivation.
