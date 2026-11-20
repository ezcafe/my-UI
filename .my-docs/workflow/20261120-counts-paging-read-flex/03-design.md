# Design: Counts, Paging, Auto-Mark Read & Fluid Spatial Container

Result: pending

## Executive Summary

This design implements the four requirements agreed in ideation:
1. **Dynamic counts in Unread and Today filter pills**: Displays live counts `Unread (N)` and `Today (M)` with real-time updates.
2. **Standard Prev/Next pagination**: Displays a clean navigation bar (`Previous`, `Page X of Y`, `Next`) pinned at the bottom when total items exceed 50, and hides it when items fit on 1 page.
3. **Auto-mark as read on detail view**: Opening an article (`GET /hx/entries/{id}`) marks it read on the backend and updates the list item and counter pills out-of-band.
4. **Fluid spatial container**: Uses container query CSS `max-width: min(100cqi, 1440px)` so the UI expands flexibly to container width up to 1440px.

---

## Architectural Options

### Option A: Inline GraphQL Total & Pinned Footer Pagination (Recommended)

- **Description**:
  - Expose `Query::entries_result` returning `EntryResultSetGql { total, entries }` and `Query::today_counter` in GraphQL.
  - Compute pagination state (`current_page`, `total_pages`, `prev_offset`, `next_offset`) on the server.
  - Pin the pagination control in a dedicated footer bar below the scrollable entries list.
  - Perform read mutation inline in `get_entry_detail_htmx`, emitting out-of-band swaps (`hx-swap-oob="true"`) for `#unread-pill`, `#filter-unread-count`, and `#entry-{id}-dot`.
  - Use `max-width: min(100cqi, 1440px)` on `.spatial-window`.

- **Concrete Example**:
  ```graphql
  query {
    entriesResult(filter: { status: UNREAD, limit: 50, offset: 0 }) {
      total
      entries { id title isRead }
    }
    unreadCounter
    todayCounter
  }
  ```
  ```html
  <!-- HTMX Out-of-band swap in detail response -->
  <div id="detail-panel">...detail content...</div>
  <span id="unread-pill" hx-swap-oob="true">12 unread</span>
  <span id="filter-unread-count" hx-swap-oob="true">(12)</span>
  <span id="entry-101-dot" hx-swap-oob="true"></span>
  ```

- **Pros**:
  - Zero extra Miniflux queries for pagination: `MinifluxClient::get_entries()` already provides `total` in every call.
  - Clean separation: list scrolls independently while pagination bar stays pinned and easily reachable.
  - Immediate visual feedback: unread dot disappears instantly without reloading the list.
  - Fully responsive without hardcoded breakpoints.

- **Cons**:
  - Small addition to GraphQL schema for `entriesResult` and `todayCounter`.

---

### Option B: Client-Side Pagination & Polling Counters

- **Description**:
  - Fetch full unpaginated entry arrays to client, paginate in JavaScript.
  - Poll `/hx/unread-counter` on a timer.
  - Trigger a secondary `POST /hx/entries/{id}/read` request from client-side JS when opening detail view.

- **Pros**:
  - Keeps GraphQL query unchanged.

- **Cons**:
  - Wastes bandwidth and memory loading all items into DOM.
  - Delayed feedback and extra network hops for status updates.
  - Violates HTMX-first server-driven pattern of the app.

---

### Recommendation

**Option A** is strongly recommended. It integrates natively with the existing Miniflux API responses, leverages HTMX out-of-band swaps cleanly, and satisfies all performance and user experience criteria.

---

## Technical Specifications

### 1. GraphQL Schema Updates (`crates/miniflux/src/schema/`)

- Add `EntryResultSetGql`:
  ```rust
  #[derive(SimpleObject, Clone, Debug)]
  pub struct EntryResultSetGql {
      pub total: i64,
      pub entries: Vec<EntryGql>,
  }
  ```
- Add to `Query`:
  - `entries_result(&self, ctx: &Context<'_>, filter: Option<EntryFilterInput>) -> Result<EntryResultSetGql>`
  - `today_counter(&self, ctx: &Context<'_>) -> Result<i64>`

### 2. Route & Template Updates (`crates/server/` & `crates/ui-components/`)

- **`crates/ui-components/templates/pages/index.html`**:
  - Add Today count badge to `#filter-today`: `Today <span id="filter-today-count">({{ today_count }})</span>`.
  - Add Unread count badge to `#filter-unread`: `Unread <span id="filter-unread-count">({{ unread_count }})</span>`.
  - Add pagination footer below `#entries-list`.
- **`crates/ui-components/templates/partials/entry_list.html`**:
  - Render pagination navigation bar when `total_pages > 1`:
    - `Previous` button: `hx-get="/hx/entries?status={status}&search={search}&offset={prev_offset}&limit={limit}"`.
    - Page indicator: `Page {current_page} of {total_pages}`.
    - `Next` button: `hx-get="/hx/entries?status={status}&search={search}&offset={next_offset}&limit={limit}"`.
- **`crates/ui-components/templates/partials/entry_item.html`**:
  - Wrap unread indicator in `<span id="entry-{{ entry.id }}-dot">...</span>`.
- **`crates/server/src/routes/entries.rs`**:
  - In `get_entry_detail_htmx`: If entry is unread, execute `client.update_entry_status(vec![id], "read")`. Return OOB elements for `#unread-pill`, `#filter-unread-count`, and `#entry-{id}-dot`.
- **`crates/ui-components/templates/base.html`**:
  - Update `.spatial-window` style:
    ```css
    .spatial-window {
      width: 100%;
      max-width: min(100cqi, 1440px);
      height: min(88vh, 780px);
      display: flex;
      flex-direction: column;
      border-radius: 28px;
      overflow: hidden;
    }
    ```
