# System Analysis: Unread Count, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

## 1. Context & Objectives

This change addresses four critical UX bugs and functional improvements across the reader interface:

1. **Unread Filter Count Showing 0 on First Load**:
   - The "Unread" filter pill displays `Unread (0)` upon loading the app, even though valid unread articles are populated in the article list.
2. **Infinite "Generating Executive Brief..." Placeholder**:
   - Switching from the Unread filter to Today and then back to Unread leaves the executive brief permanently frozen in the loading skeleton state.
3. **AI Suggested Title Sentence Constraint**:
   - The AI prompt restricts article titles to `<=10 words`. This constraint must be updated to strictly one sentence (removing the word count restriction).
4. **Executive Brief Request Throttling on Unread Filter**:
   - Switching back to the Unread filter currently triggers an LLM executive brief generation every time. It should only request a new brief after the user has read at least 10 articles (either from first load or since the previous brief request).

---

## 2. Technical & Architectural Breakdown

### A. Unread Count Fallback (`crates/server/src/routes/pages.rs`, `entries.rs`, `index.html`)

- **Root Cause**:
  1. In `pages.rs` (lines 87–95), `unread_count` is retrieved from `data.get("unreadCounter")` (auth mode) or `data.get("todayCounter")` (guest mode). If `todayCounter` is 0 (no articles published today) or upstream Miniflux returns 0/null for `unreadCounter`, `unread_count` evaluates to 0, despite `entriesResult.total` being greater than 0.
  2. In `index.html` (`syncDomReadStates()`), `baseServerUnread` is parsed from `data-base-unread`. If it is 0, `effectiveUnread = Math.max(0, 0 - readSet.size)` remains 0 even if `visibleUnreadRowCount > 0`.
- **Target Architecture**:
  1. Server fallback: In `pages.rs` and `entries.rs`, if `unread_count` evaluates to 0 but `total > 0`, fall back to `total as i32` so initial server-rendered HTML reflects the actual unread items in the list.
  2. Client safeguard: In `syncDomReadStates()`, when viewing the Unread filter, if `effectiveUnread === 0` but `visibleUnreadRowCount > 0`, fall back to `visibleUnreadRowCount`.

### B. Brief Infinite Loading on Filter Switch (`crates/ui-components/templates/pages/index.html`)

- **Root Cause**:
  1. `seedInitialFilterCache()` captures `#entries-list.innerHTML` at `DOMContentLoaded`, before `/hx/executive-brief` finishes loading. The cached HTML contains `<div id="page-executive-brief" hx-trigger="load">`.
  2. When `/hx/executive-brief` returns and replaces `#page-executive-brief`, `htmx:afterSwap` only inspects `targetId === 'entries-list'`, ignoring child swaps targeting `#page-executive-brief`. The cache for the Unread filter remains stale with the skeleton.
  3. When switching from Today back to Unread, `applyCachedList()` injects the cached skeleton via `entriesList.innerHTML = cachedHtml` without calling `htmx.process(entriesList)`. HTMX never fires the trigger, leaving the skeleton stuck.
- **Target Architecture**:
  1. In `htmx:afterSwap`, recognize swaps targeting `#page-executive-brief` (or any child swap within `#entries-list`), and immediately update `filterCache` for the active filter with the latest rendered DOM.
  2. In `applyCachedList()`, invoke `if (window.htmx && typeof htmx.process === 'function') htmx.process(entriesList);` to ensure HTMX binds triggers to any dynamic elements.

### C. Suggested Title Sentence Restriction (`crates/server/src/ai/llm.rs`)

- **Root Cause**:
  - In `crates/server/src/ai/llm.rs:78`, `format_batch_prompt_structured` specifies:
    `2. "items": For each NEW article id, provide a punchy "t" (suggested title, <=10 words) and "b" (executive brief, strictly 3 sentences, what changed/why it matters).`
- **Target Architecture**:
  - Update line 78 in `crates/server/src/ai/llm.rs` to:
    `2. "items": For each NEW article id, provide a punchy "t" (suggested title, strictly in one sentence) and "b" (executive brief, strictly 3 sentences, what changed/why it matters).`
  - Update unit tests in `crates/server/src/ai/llm.rs` to verify the new phrasing.

### D. Executive Brief Throttling (`crates/server/src/routes/entries.rs`, `index.html`)

- **Root Cause**:
  - Clicks on `filter-unread` issue `GET /hx/entries?status=unread`.
  - The server always flags `is_brief_eligible = true`, rendering the loading skeleton `<div id="page-executive-brief" hx-get="/hx/executive-brief" hx-trigger="load">`.
- **Target Architecture**:
  1. Client: Maintain a session counter `readItemsSinceBrief = 0`. Whenever an item is marked read (locally or via server call), increment `readItemsSinceBrief`.
  2. Request parameter: In `htmx:configRequest` or filter click handling, if clicking `filter-unread`:
     - If `readItemsSinceBrief >= 10`: pass `request_brief=true` and reset `readItemsSinceBrief = 0`.
     - If `readItemsSinceBrief < 10`: pass `request_brief=false`.
  3. Server: Add `request_brief: Option<bool>` to `EntriesFilterQuery`. When `request_brief == Some(false)`:
     - `is_eligible` becomes `false` (no loading skeleton rendered).
     - Load `ai.db.get_latest_page_brief().await` into `top_executive_brief` so the existing brief remains visible.

---

## 3. Risks & Mitigations

- **Risk 1: Stale executive brief when user reads < 10 items**:
  - *Mitigation*: The latest generated brief is retained in `top_executive_brief` and displayed statically, ensuring the user always sees context without waiting for new LLM generation.
- **Risk 2: Duplicate article reads inflating counter**:
  - *Mitigation*: Track read articles via a Set (`readItemsSinceBriefSet`) rather than an unchecked integer so re-clicking an already read article does not increment the throttle counter.
- **Risk 3: HTMX swap recursion or double processing**:
  - *Mitigation*: `htmx.process(entriesList)` is scoped specifically to `#entries-list` only on cache restoration, avoiding extra page-level events.

## Result

Result: done
