# Code review log

**Overall Result:** clean

---

## Adversarial

**Result:** clean

### 1. Key Takeaways & Security Boundaries

- **Guest vs Authenticated Scope Isolation**:
  - In [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L61-L89), when `auth_enabled` is false, unread filtering translates to `publishedAfter: <start_of_today_utc>` rather than `status: UNREAD`. This prevents guest users from querying arbitrary user-scoped read/unread statuses upstream, while ensuring unread queries stay bounded to today's articles.
  - In [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs#L26-L94), separate cache keys (`index_page_entries_auth` vs `index_page_entries_guest`) guarantee that guest sessions never serve cached authenticated queries or vice-versa.
  - In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L110-L136), `#filter-starred` remains strictly gated behind `{% if auth_enabled %}`. Guest sessions cannot access or trigger starred article mutations.

- **Client-Side Injection Prevention**:
  - In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L683-L691), `escapeHtml()` escapes HTML entities (`&`, `<`, `>`, `"`, `'`) for titles and URLs prior to injecting them into DOM strings in `renderLocalAllArticles()`.
  - All rendered article links include `target="_blank"` and `rel="noopener noreferrer"`.
  - Article identifiers are strictly parsed and validated using `Number(id)` before any IndexedDB transaction or DOM selector concatenation.

- **Data Integrity & IndexedDB Versioning**:
  - The IndexedDB database `my_news_guest_db` is cleanly versioned from `1` to `2`. In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L694-L703), `onupgradeneeded` checks for `contains('all_articles')` before creation, preventing runtime collision or upgrade failure for existing guest users.
  - Reading from IndexedDB checks `db.objectStoreNames.contains('all_articles')` before initiating transactions, gracefully falling back if the store is not available.

- **Event Propagation & Navigation Safety**:
  - In [entry_item.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html#L18) and [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L26), the "Open in browser" link stops event bubbling (`event.stopPropagation()`), preventing the row click from triggering drawer expansion while simultaneously registering `markArticleRead(id)`.
  - In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L229-L240), `handleFilterClick()` intercepts clicks on `#filter-all` in guest mode with `preventDefault()` and `stopImmediatePropagation()`, completely suppressing HTMX network dispatch and serving instantly from local storage.

---

## Quality

**Result:** clean

### 1. Five-Axis Quality Assessment

- **Correctness**:
  - Implements all user and design requirements from Option B: guest unread displays today's articles only, "All Titles" retrieves all locally cached titles from IndexedDB, and unread counts accurately decrement without negative underflow.
  - All 88 tests pass across `server` (51 unit + 4 e2e) and `ui-components` (33 unit tests) with 0 regressions.
- **Readability**:
  - Clear and self-documenting code. Functions follow concise single-responsibility designs (`cacheLoadedArticlesFromDom`, `renderLocalAllArticles`, `initGuestDb`, `syncDomReadStates`).
  - Scannable code formatting and explanatory comments explaining guest vs authenticated paths.
- **Architecture**:
  - Maintains strict separation of concerns. Server routes handle backend query building and API calls, while guest offline storage is isolated inside client templates under `{% if !auth_enabled %}`.
  - Cache partitioning prevents crosstalk between authentication modes.
- **Security**:
  - All client DOM insertions from IndexedDB run through `escapeHtml()`.
  - Protected filters (`#filter-starred`) and endpoints remain inaccessible in guest mode.
  - External links retain `rel="noopener noreferrer"`.
- **Performance**:
  - In guest mode, clicking "All Titles" produces zero network requests and renders instantly from client-side IndexedDB.
  - Title caching (`cacheLoadedArticlesFromDom`) operates asynchronously on swap without blocking rendering or user interaction.

### 2. Code Health & Simplification

- `cargo clippy -p ui-components --all-targets -- -D warnings` passes with 0 warnings.
- `cargo clippy -p server` on touched route files (`entries.rs` and `pages.rs`) passes with 0 warnings after simplifying redundant type casts and `is_some_and`.

---

## Merged

**Result:** clean
