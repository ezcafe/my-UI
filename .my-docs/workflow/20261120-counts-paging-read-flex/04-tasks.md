# Tasks: Counts, Paging, Auto-Mark Read & Fluid Spatial Container

Result: planned

## Tasks Breakdown

### Phase 1: GraphQL Schema & Count Query (Miniflux Crate)
- [ ] Task 1.1: Add `EntryResultSetGql` SimpleObject to `crates/miniflux/src/schema/types.rs`.
- [ ] Task 1.2: Implement `Query::entries_result` in `crates/miniflux/src/schema/query.rs` returning `EntryResultSetGql` with `total` and `entries`.
- [ ] Task 1.3: Implement `Query::today_counter` in `crates/miniflux/src/schema/query.rs` calculating UTC midnight and querying `client.get_entries()`.
- [ ] Task 1.4: Unit test `entries_result` and `today_counter` in `crates/miniflux/src/schema/mod.rs`.

### Phase 2: Server Route Handlers & ViewModels (Server & UI Components)
- [ ] Task 2.1: Update `EntryListTemplate` and `IndexTemplate` in `crates/ui-components/src/templates.rs` with `today_count`, `current_page`, `total_pages`, `has_prev`, `has_next`, `prev_offset`, and `next_offset`.
- [ ] Task 2.2: Update `index_page` in `crates/server/src/routes/pages.rs` to fetch `today_counter` and pass initial pagination state.
- [ ] Task 2.3: Update `render_entries_list` in `crates/server/src/routes/entries.rs` to query `entriesResult` and compute pagination attributes.
- [ ] Task 2.4: Update `get_entry_detail_htmx` in `crates/server/src/routes/entries.rs` to auto-mark unread entry as read and emit out-of-band updates for counts and `#entry-{id}-dot`.

### Phase 3: Templates & Modern CSS Layout
- [ ] Task 3.1: Update `crates/ui-components/templates/pages/index.html` filter pills to render `Unread (<count>)` and `Today (<count>)`.
- [ ] Task 3.2: Update `crates/ui-components/templates/partials/entry_list.html` to render pagination bar at the bottom of the scrollable entries list (`Previous`, `Page X of Y`, `Next`) with HTMX paging links when `total_pages > 1`.
- [ ] Task 3.3: Update `crates/ui-components/templates/partials/entry_item.html` to identify the unread indicator dot element as `id="entry-{{ entry.id }}-dot"` so it can be cleared via OOB swap without re-rendering or jumping the row.
- [ ] Task 3.4: Update `crates/ui-components/templates/base.html` `.spatial-window` max-width to `min(100cqi, 1440px)` using container queries.

---

## Planned Tests

1. `test_entries_result_returns_total_and_entries`:
   - Validates that `entriesResult` GraphQL query returns the full total matching entries alongside the page items.
2. `test_today_counter_returns_correct_count`:
   - Validates that `todayCounter` queries Miniflux with `published_after` starting from UTC midnight and returns matching count.
3. `test_entry_list_renders_pagination_when_multiple_pages`:
   - Validates that when total items exceed limit (e.g. 100 items with limit 50), `Previous`, `Page 1 of 2`, and `Next` links are rendered.
4. `test_entry_list_hides_pagination_when_single_page`:
   - Validates that when total items are within limit (e.g. 25 items with limit 50), pagination controls are omitted.
5. `test_get_entry_detail_auto_marks_as_read_and_returns_oob_swaps`:
   - Validates that viewing an unread entry triggers Miniflux status update and includes out-of-band updates (`hx-swap-oob="true"`) for unread pill, filter unread count, and entry dot without deleting the row.
6. `test_spatial_window_fluid_width_container_query`:
   - Validates that `.spatial-window` CSS uses container queries (`min(100cqi, 1440px)`) instead of fixed 720px width.
