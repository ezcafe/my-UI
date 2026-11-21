# 02-analysis: Guest Unread Scope, Local All Titles (Option B), and UI Filter Fixes

- **Result:** done

## 1. Context and Problem Statement

The RSS/News Reader web interface currently presents several operational and UX challenges in guest mode (`AUTH_ENABLED=false`):

1. **Overwhelming Unread Scope in Guest Mode**:
   - In guest mode, Miniflux lacks user session isolation. Showing all unread historical articles exposes thousands of unread items that guests cannot meaningfully clear on the server.
   - Requirement: When auth is disabled, "Unread" must show only today's unread news (`publishedAfter: start_of_today`). Client `readSet` filters out read entries locally.

2. **Absence of "All Titles" in Guest Mode (and Option B Selection)**:
   - When auth was initially disabled, `#filter-all` was hidden along with `#filter-starred`.
   - However, users need a way to review historical titles they have already browsed without server-side accounts.
   - Option B was selected: Restore `#filter-all` in guest mode. All articles loaded into the UI during a session are stored in IndexedDB (`all_articles` store inside `my_news_guest_db`). When clicking "All Titles", entries are rendered directly from IndexedDB without hitting the server.

3. **Articles Not Marked as Read When Opened Directly in Browser**:
   - In `entry_item.html` and `entry_list.html`, clicking "Open in browser" (`<a title="Open in browser">`) navigates to external sites without updating read status.
   - Both guest mode (`markLocallyRead`) and auth mode (`/hx/entries/{id}/read`) need eager, optimistic read updates upon link click.

4. **Global Unread Count Wiped Out by Filtered DOM Rows in Guest Mode**:
   - `syncDomReadStates()` counted DOM rows in `#entries-list`, wiping out global unread count (e.g. dropping 48 down to 9 on the "Today" tab).
   - Global unread must be decoupled from visible DOM rows and derived as `Math.max(0, baseServerUnread - readSet.size)`.

5. **Detail Panel Stays Open on Filter / Search Clicks**:
   - Selecting a different filter pill or submitting a search should dismiss any open article detail panel (`closeDetailPanel()`) and return to the list view.

---

## 2. Invariants & Scope

- **Mode and Profile**: Mode `simple`, Review profile `lite`.
- **Card Has Flags**:
  - `Has UI`: `yes` (templates, IndexedDB client storage, filter button behaviors).
  - `Has API`: `yes` (modifying `crates/server/src/routes/entries.rs` and `pages.rs` to filter GraphQL query by `publishedAfter: start_of_today` in guest mode; candidate `tiny-contract-tweak` applies as blast radius is limited to route query construction).
  - `Has DB`: `no` (browser IndexedDB is client-side storage, not server DB).
  - `Has auth/PII/schema`: `no`.
- **Idempotence**: Marking an article as read multiple times must never throw errors, duplicate records, or decrement counters more than once.
- **IndexedDB Upgrades**: `my_news_guest_db` version bumped from 1 to 2; existing `read_entries` keys remain intact while adding `all_articles` object store with `keyPath: 'id'`.
- **Backward Compatibility**: Authenticated mode workflows must continue using Miniflux server state and OOB swaps without regression.

---

## 3. Risks & Mitigations

- **Risk: Duplicate or Missing Articles in `all_articles`**:
  - *Mitigation*: Use `keyPath: 'id'` with `put()` so every insertion is an idempotent upsert. Store structured metadata: `{ id, title, summary, publishedAt, readingTime, url, isBookmarked, status, timestamp }`.
- **Risk: DOM Row Scraping vs Data Attributes for IndexedDB Caching**:
  - *Mitigation*: Each `.annuaire-row` contains full metadata in attributes or child elements (`data-entry-id`, title, summary text, URL, reading time). An extraction helper `cacheLoadedArticles()` reads these rows on `DOMContentLoaded` and `htmx:afterSwap` and batch-puts them into `all_articles`.
- **Risk: Intercepting "All Titles" in HTMX**:
  - *Mitigation*: In guest mode (`!auth_enabled`), `#filter-all` does not issue an HTMX network request. Its click handler intercepts the action (`e.preventDefault()`), activates the filter pill visually, closes the detail panel, and executes `renderLocalAllArticles()`.
- **Risk: Counter Desynchronization**:
  - *Mitigation*: `baseServerUnread` is anchored on `#filter-unread-count` and `#unread-pill`. In guest mode, the baseline reflects today's unread items. `Math.max(0, baseServerUnread - readSet.size)` ensures consistent calculation across all filter switches.
