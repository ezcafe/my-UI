# Design: Counts, Paging, Auto-Mark Read & Fluid Spatial Container

Result: done

## Executive Summary

This design implements the four requirements agreed in ideation and confirmed in grill:
1. **Dynamic counts in Unread and Today filter pills**: Displays live counts `Unread (N)` and `Today (M)` with real-time updates.
2. **Standard Prev/Next pagination at list bottom**: Displays a navigation bar (`Previous`, `Page X of Y`, `Next`) at the bottom of the scrollable list (scrolls with entries) when total items exceed 50, and hides it when items fit on 1 page.
3. **Auto-mark as read on detail view with OOB dot clearing**: Opening an article (`GET /hx/entries/{id}`) marks it read on the backend, preserves the row in place to prevent scroll jumps, and clears the unread dot and updates counters out-of-band (`hx-swap-oob="true"`).
4. **Fluid spatial container**: Uses container query CSS `max-width: min(100cqi, 1440px)` so the UI expands flexibly to container width up to 1440px without hardcoded breakpoints.

---

## Architectural Options

### Option A: Inline GraphQL Total & Scrollable List-Bottom Pagination (Recommended)

- **Explanation**:
  - Expose `Query::entries_result` returning `EntryResultSetGql { total, entries }` and `Query::today_counter` in GraphQL.
  - Compute pagination state (`current_page`, `total_pages`, `prev_offset`, `next_offset`) on the server.
  - Place pagination controls directly at the bottom of the scrollable entries list (`#entries-list`) so they scroll naturally with list content.
  - Perform read mutation inline during `get_entry_detail_htmx`, emitting out-of-band swaps (`hx-swap-oob="true"`) for `#unread-pill`, `#filter-unread-count`, and `#entry-{id}-dot`.
  - Use `max-width: min(100cqi, 1440px)` on `.spatial-window`.

- **Concrete Examples**:
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
  <!-- HTMX Out-of-band swaps returned in detail response -->
  <div id="detail-panel">...detail content...</div>
  <span id="unread-pill" hx-swap-oob="true">12 unread</span>
  <span id="filter-unread-count" hx-swap-oob="true">(12)</span>
  <span id="entry-101-dot" hx-swap-oob="true"></span>
  ```

- **Pros**:
  - **Single roundtrip**: `MinifluxClient::get_entries()` already provides `total` alongside entries; no extra count query required for pagination.
  - **Natural scrolling**: Pagination stays attached to the end of the articles list without obscuring items or needing fixed overlays.
  - **Smooth reading flow**: Marking an entry read clears its unread indicator without removing the row, keeping scroll position steady.
  - **Responsive**: Adapts fluidly with container queries (`cqi`).

- **Cons**:
  - Requires adding `entriesResult` and `todayCounter` to GraphQL schema.

---

### Option B: Client-Side Pagination & Polling Counters

- **Explanation**:
  - Fetch all unpaginated entries to the browser and paginate using client-side JavaScript.
  - Poll `/hx/unread-counter` on a timer interval.
  - Trigger a secondary client-side `POST` request to mark items read when selected.

- **Pros**:
  - No new GraphQL queries needed.

- **Cons**:
  - Loads full dataset into DOM, causing memory bloat and slow renders for large feeds.
  - Periodic polling consumes unnecessary network and battery.
  - Violates HTMX server-driven architecture.

---

### Recommendation

**Option A** is recommended. It aligns directly with the existing Miniflux API responses, respects all user decisions from Grill, and delivers optimal server-driven responsiveness with zero client-side overhead.

---

## Technical Specifications

### 1. GraphQL Schema Updates (`crates/miniflux/src/schema/`)

- Add `EntryResultSetGql` in `crates/miniflux/src/schema/types.rs`:
  ```rust
  #[derive(SimpleObject, Clone, Debug)]
  pub struct EntryResultSetGql {
      pub total: i64,
      pub entries: Vec<EntryGql>,
  }
  ```
- Add to `Query` in `crates/miniflux/src/schema/query.rs`:
  - `entries_result(&self, ctx: &Context<'_>, filter: Option<EntryFilterInput>) -> Result<EntryResultSetGql>`
  - `today_counter(&self, ctx: &Context<'_>) -> Result<i64>`:
    Calculates UTC midnight (`chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0)`) and calls `client.get_entries()` with `published_after` and `limit: 1`.

### 2. Route & Template Updates (`crates/server/` & `crates/ui-components/`)

- **`crates/ui-components/templates/pages/index.html`**:
  - Add Today count badge to `#filter-today`:
    `Today <span id="filter-today-count" class="text-xs text-white/50">({{ today_count }})</span>`
  - Add Unread count badge to `#filter-unread`:
    `Unread <span id="filter-unread-count" class="text-xs text-white/50">({{ unread_count }})</span>`

- **`crates/ui-components/templates/partials/entry_list.html`**:
  - Append pagination navigation bar at the bottom of the entries list when `total_pages > 1`:
    ```html
    {% if total_pages > 1 %}
    <nav class="flex items-center justify-between pt-4 pb-2 border-t border-white/10 text-xs text-white/70" aria-label="Pagination">
      {% if has_prev %}
      <button hx-get="/hx/entries?status={{ status }}&search={{ search }}&offset={{ prev_offset }}&limit={{ limit }}"
              hx-target="#entries-list"
              hx-swap="innerHTML"
              class="px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/20 transition-colors">
        Previous
      </button>
      {% else %}
      <span class="px-3 py-1.5 rounded-lg text-white/30 cursor-not-allowed">Previous</span>
      {% endif %}

      <span>Page {{ current_page }} of {{ total_pages }}</span>

      {% if has_next %}
      <button hx-get="/hx/entries?status={{ status }}&search={{ search }}&offset={{ next_offset }}&limit={{ limit }}"
              hx-target="#entries-list"
              hx-swap="innerHTML"
              class="px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/20 transition-colors">
        Next
      </button>
      {% else %}
      <span class="px-3 py-1.5 rounded-lg text-white/30 cursor-not-allowed">Next</span>
      {% endif %}
    </nav>
    {% endif %}
    ```

- **`crates/ui-components/templates/partials/entry_item.html`**:
  - Wrap the unread indicator in `<span id="entry-{{ entry.id }}-dot">` so it can be cleared via OOB swap while keeping the article row intact.

- **`crates/server/src/routes/entries.rs`**:
  - In `get_entry_detail_htmx`: If entry is unread, execute `client.update_entry_status(vec![id], "read")`.
  - Include OOB swaps in response:
    - `#unread-pill`: updated unread count badge
    - `#filter-unread-count`: updated unread filter pill count
    - `#entry-{id}-dot`: empty span removing the dot

- **`crates/ui-components/templates/base.html`**:
  - Update `.spatial-window` rule to use container queries:
    ```css
    .spatial-shell {
      container-type: inline-size;
    }
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
