# 01-idea: Unread Count, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

## 1. Summary of Changes

### A. Fix Unread Filter Count Showing 0 on First Load
- **Problem**: When opening the application, the "Unread" filter pill displays `Unread (0)`, even though multiple unread articles are visible in the list.
- **Root Cause**:
  1. In `crates/server/src/routes/pages.rs` (and `entries.rs`), `unread_count` relies solely on `unreadCounter` (in auth mode) or `todayCounter` (in guest mode). If `unreadCounter` is 0 or null from upstream Miniflux, or if `todayCounter` is 0, the server passes `unread_count = 0` to the template, despite `entriesResult` returning `total > 0` unread items.
  2. In `crates/ui-components/templates/pages/index.html`, `syncDomReadStates()` computes `effectiveUnread = Math.max(0, baseServerUnread - readSet.size)`. When `baseServerUnread` is 0, `effectiveUnread` evaluates to 0 even if `visibleUnreadRowCount > 0`.
- **Solution**:
  - In `pages.rs` and `entries.rs`, if `total > 0` for unread items and server unread counter is 0 or null, fall back to `total as i32` so the rendered HTML always reflects the actual items in the list.
  - In `index.html` (`syncDomReadStates()`), when viewing the "Unread" filter, if `effectiveUnread === 0` but `visibleUnreadRowCount > 0`, set `effectiveUnread = visibleUnreadRowCount` to keep the UI badge in sync with the DOM rows.

### B. Fix Infinite "Generating Executive Brief..." when Switching Filters
- **Problem**: On the Unread filter, switching to the Today filter and then switching back to the Unread filter causes the Executive Brief to stay permanently stuck displaying "Generating Executive Brief...".
- **Root Cause**:
  1. `seedInitialFilterCache()` in `index.html` runs at `DOMContentLoaded` and captures `#entries-list.innerHTML` while the brief is still an unrendered skeleton card (`<div id="page-executive-brief" hx-get="/hx/executive-brief..." hx-trigger="load">...</div>`).
  2. When `/hx/executive-brief` returns and replaces `#page-executive-brief` with the completed brief card, `htmx:afterSwap` only checks `if (targetId === 'entries-list')`. It fails to detect swaps targeting `#page-executive-brief`, leaving `filterCache.get('unread')` stale with the loading skeleton.
  3. When switching from "Today" back to "Unread", `handleFilterClick()` / `htmx:beforeRequest` loads the cached HTML into `#entries-list.innerHTML`.
  4. `applyCachedList()` assigns `innerHTML` but never calls `htmx.process(entriesList)`. As a result, HTMX never activates the skeleton's `hx-trigger="load"`, leaving the placeholder frozen forever.
- **Solution**:
  - In `index.html` (`htmx:afterSwap`), listen for swaps on `#page-executive-brief` (or any descendant of `#entries-list`) and update `filterCache` for the active filter with the latest DOM state.
  - In `applyCachedList()`, call `if (window.htmx) htmx.process(entriesList);` to ensure HTMX re-activates any dynamic triggers or pending requests.

### C. Update Suggested Title Restriction from < 10 Words to One Sentence
- **Problem**: The AI prompt in `crates/server/src/ai/llm.rs` currently restricts article titles to `<=10 words`. The requirement is to remove the word limit and restrict the title strictly to one sentence.
- **Root Cause**:
  - In `crates/server/src/ai/llm.rs:78`, `format_batch_prompt_structured` specifies:
    `2. \"items\": For each NEW article id, provide a punchy \"t\" (suggested title, <=10 words) and \"b\" (executive brief, strictly 3 sentences, what changed/why it matters).`
- **Solution**:
  - Change line 78 in `crates/server/src/ai/llm.rs` to:
    `2. \"items\": For each NEW article id, provide a punchy \"t\" (suggested title, strictly in one sentence) and \"b\" (executive brief, strictly 3 sentences, what changed/why it matters).`
  - Update prompt validation tests in `crates/server/src/ai/llm.rs` to assert the new sentence-based instruction.

### D. Throttle Executive Brief Requests on Unread Filter (Only After Reading 10 Items)
- **Problem**: Currently, any click on the Unread filter triggers an AI executive brief request. It should only request executive-brief when the user has read 10 items after the first load or 10 items after the previous click on the Unread filter.
- **Root Cause**:
  - Every click on `filter-unread` requests `/hx/entries?status=unread`.
  - The server always flags `is_brief_eligible = true` for `status=unread` with no search query, causing `entry_list.html` to unconditionally render `<div id="page-executive-brief" hx-get="/hx/executive-brief" hx-trigger="load">`.
- **Solution**:
  - Track `readItemsSinceBrief` count in client-side JavaScript (`index.html`), initialized to 0 on first load.
  - Whenever an article is marked read (`markLocallyRead`, `markArticleRead`, or reading detail panel), increment `readItemsSinceBrief`.
  - When clicking the Unread filter:
    - If `readItemsSinceBrief >= 10`: include `request_brief=true` in the request to `/hx/entries?status=unread`, and reset `readItemsSinceBrief = 0`.
    - If `readItemsSinceBrief < 10`: include `request_brief=false` (or omit). The server renders `is_brief_eligible = false` (or shows the existing `top_executive_brief` without the loading skeleton trigger).
  - In `crates/server/src/routes/entries.rs`:
    - Add `request_brief: Option<bool>` to `EntriesFilterQuery`.
    - `is_eligible` checks: `query.request_brief.unwrap_or(true)` on first load / default, but if `request_brief == Some(false)`, `is_eligible` evaluates to `false`. When `is_eligible` is `false`, load `latest_page_brief` as `top_executive_brief` so the existing brief remains visible without re-generating.

---

## 2. Invariants & Scope

- **Has UI (tentative)**: yes
- **Has API**: no (Axum internal query parameter `request_brief`, no breaking public API change)
- **Has DB**: no
- **Has auth/PII/schema**: no
- **TDD Requirement**:
  1. Unit tests for `pages.rs` and `entries.rs` fallback to `total` when counter is 0.
  2. Unit tests for `entries.rs` respecting `request_brief = false`.
  3. Unit tests for `llm.rs` prompt checking `strictly in one sentence`.
  4. Template and client-side tests for `index.html` and `entry_list.html`.
