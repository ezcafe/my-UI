# 01-idea: Filter Visibility, Guest Unread Scope, Local All Titles, and Read Status Fixes

## 1. Summary of Changes

1. **Guest Mode "Unread" Scoped to Today's Unread Only**:
   - In guest mode (`AUTH_ENABLED=false`), Miniflux has no per-user read tracking. Showing all historical unread items is overwhelming and counter-intuitive.
   - Backend `crates/server/src/routes/entries.rs` (`build_graphql_filter`) and `crates/server/src/routes/pages.rs` (`index_page`) will filter by `publishedAfter: start_of_today` when status is unread or default in guest mode.
   - Client `readSet` in IndexedDB filters out read entries and updates the unread counters accordingly.

2. **Show "All Titles" in Guest Mode Powered by Local Storage (Option B)**:
   - `#filter-all` is restored to the filter toolbar in guest mode (`#filter-starred` remains hidden when `!auth_enabled`).
   - In guest mode, all articles loaded into the UI (via initial page load or pagination/search HTMX swaps) are cached into an IndexedDB object store `all_articles` (in `my_news_guest_db`).
   - Clicking "All Titles" in guest mode renders all entries directly from `all_articles` completely locally, applying read states from `readSet` and active filter styling without calling the server.

3. **Mark as Read on "Open in Browser"**:
   - In `entry_item.html` and `entry_list.html`, clicking "Open in browser" (`<a title="Open in browser">`) marks the article as read before navigating.
   - Guest mode (`!auth_enabled`): calls `markLocallyRead(id)`.
   - Authenticated mode (`auth_enabled`): dispatches an async read request to `/hx/entries/{id}/read` (and updates the DOM row read state immediately).
   - In `entry_detail.html`: opening the detail panel auto-marks the article as read. The "Original" link also calls `markArticleRead(id)` idempotently.

4. **Decouple Global Unread Count from Filtered Rows**:
   - In `index.html`, `syncDomReadStates()` derives the global counter as `Math.max(0, baseServerUnread - readSet.size)`.
   - Visible row counting is strictly reserved for "Unread" tab row visibility and `#client-empty-state`.

5. **Filter Button Clicks and Search Return to List View**:
   - Clicking any filter button (`#filter-unread`, `#filter-today`, `#filter-all`, `#filter-starred`, `#filter-ai`) or submitting a search closes the detail view panel (`closeDetailPanel()`), returning the interface to the main article list.

## 2. Constraints & Invariants

- **Idempotence**: Marking an article as read multiple times must produce no duplicate errors or state corruption.
- **IndexedDB Backward Compatibility**: The guest database `my_news_guest_db` will upgrade schema version from 1 to 2, ensuring existing `read_entries` records are preserved alongside the new `all_articles` store.
- **TDD Requirement**: Unit tests for template rendering, route filters, and script definitions must be written and passing before finalizing code.
- **No breaking changes to Auth mode**: Authenticated flows continue relying on Miniflux server state and OOB unread swaps.
