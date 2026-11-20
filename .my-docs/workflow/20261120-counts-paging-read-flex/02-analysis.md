# Analysis: Counts, Paging, Auto-Mark Read & Fluid Spatial Container

Result: done

## Context & Architecture

The application is an AR Spatial RSS reader built with:
- **Backend**: Rust using Axum, Askama templates, async-graphql, and a Miniflux HTTP client.
- **Frontend**: Server-rendered HTML with Tailwind CSS, HTMX for reactive out-of-band updates, Lucide icons, and container queries for spatial rendering.
- **Miniflux Integration**: Miniflux REST API (`/v1/entries`, `/v1/feeds`, `/v1/unread-counter`) wrapped by `MinifluxClient` and exposed via `AppSchema` (async-graphql).

The current workflow goals are:
1. **Total item count inside Unread & Today filter pills**:
   - `Unread (<count>)` reflecting total unread entries across feeds.
   - `Today (<count>)` reflecting total entries published today (00:00:00 UTC to now).
   - Real-time updates to unread counters out-of-band when entries are marked read.
2. **Pagination when more than 1 page**:
   - Bottom navigation bar (`Previous`, `Page X of Y`, `Next`) shown when total entries exceed page size (default 50).
   - Hidden when total items fit on 1 page (`total <= limit`).
   - HTMX paging requests swap `#entries-list` while preserving active filter (`status`), search query, and pagination parameters.
3. **Auto-mark as read on detail view**:
   - Viewing an entry via `GET /hx/entries/{id}` marks the entry as read in Miniflux.
   - Returns out-of-band swaps (`hx-swap-oob="true"`) to remove the unread blue dot from `#entry-{id}` and decrement unread counters in header and filter pill.
4. **Flexible layout following container**:
   - Replace fixed `max-width: 720px` in `.spatial-window` with responsive fluid sizing using modern CSS container queries (`min(100cqi, 1440px)`).

---

## Detailed Technical Analysis

### 1. Item Counts (Unread & Today)
- **Unread Count**:
  - `MinifluxClient::get_unread_counter()` queries `/v1/unread-counter`.
  - Exposed in GraphQL via `Query::unread_counter`.
  - Displayed in `#unread-pill` (header) and needs to be displayed inside `#filter-unread` pill.
- **Today Count**:
  - `MinifluxClient::get_entries()` accepts `published_after: <timestamp>`.
  - Passing `start_of_today` (UTC midnight) with `limit: 1` returns `EntryResultSet { total, entries }`. The `total` field provides the exact count of articles published today without loading all records.
  - Adding `Query::today_counter` to GraphQL exposes this cleanly to `index_page` and HTMX endpoints.

### 2. Pagination Bar
- **Data Source**:
  - `MinifluxClient::get_entries()` returns `EntryResultSet { total: i64, entries: Vec<Entry> }`.
  - Currently `Query::entries` discards `total` and returns `Vec<EntryGql>`.
  - Exposing `Query::entries_result` returning `EntryResultSetGql { total: i64, entries: Vec<EntryGql> }` provides total matching entries in the same single Miniflux request.
- **Paging Calculations**:
  - `limit`: defaults to 50.
  - `offset`: defaults to 0.
  - `current_page`: `(offset / limit) + 1`.
  - `total_pages`: `((total + limit - 1) / limit).max(1)`.
  - Navigation controls:
    - `Previous`: disabled or hidden when `offset == 0`; otherwise `offset - limit`.
    - `Page X of Y`: static text indicator.
    - `Next`: disabled or hidden when `offset + limit >= total`; otherwise `offset + limit`.
  - Visibility: When `total_pages <= 1`, pagination bar is omitted / hidden.

### 3. Auto-Mark Read on Detail View
- **Endpoint**: `GET /hx/entries/{id}` (`get_entry_detail_htmx`).
- **Mutation**:
  - If entry is currently unread, invoke Miniflux status update:
    `client.update_entry_status(vec![id], "read")` or GraphQL mutation `updateEntryStatus(id: "{id}", status: READ)`.
- **Out-of-band Swaps**:
  - Primary target `#detail-panel`: receives rendered `EntryDetailTemplate`.
  - OOB Swap 1 (`#unread-pill` and `#filter-unread-count`): updated with new unread count.
  - OOB Swap 2 (`#entry-{id}` or `#entry-{id}-dot`): removes the blue unread ring/dot without unmounting the list row, preserving user reading position.

### 4. Container-Aware Fluid Window
- **Location**: `crates/ui-components/templates/base.html`.
- **CSS Rule**:
  ```css
  .spatial-shell {
    container-type: inline-size;
  }
  .spatial-window {
    width: 100%;
    max-width: min(100cqi, 1440px);
    height: min(88vh, 780px);
  }
  ```
- Complies with user rule forbidding hardcoded breakpoints and leveraging container queries (`cqi`).

---

## Risks & Mitigations

1. **Risk: Unread filter list jump when auto-marking read**
   - *Mitigation*: Swapping out the unread dot rather than deleting the entire row from the DOM keeps the user's scroll position stable while reading. The item naturally clears on subsequent filter navigation.
2. **Risk: Extra network requests for Today count**
   - *Mitigation*: Today count query uses `limit: 1`, making Miniflux API response lightweight and fast.
3. **Risk: Preserving filter & search state across pagination**
   - *Mitigation*: Include `status`, `search`, and `limit` in HTMX pagination query parameters (`/hx/entries?status={status}&search={search}&offset={next_offset}`).
