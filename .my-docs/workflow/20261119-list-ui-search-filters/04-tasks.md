# Tasks: Title-Only List, External Link, Today Filter & Search/Filter Fixes

## Planned Tests

### Unit & Template Tests (`crates/ui-components`)
1. **`test_render_title_only_entry_item`**:
   - Verify `EntryItemTemplate` renders title with `line-clamp-2` and `text-sm font-medium`.
   - Verify omission of feed summary, reading time, and feed author/title from the row.
   - Verify presence of external link button with `target="_blank"` and `rel="noopener noreferrer"`.
   - Verify unread indicator presence when `is_read: false` and absence when `is_read: true`.
2. **`test_render_entry_list_with_title_only_items`**:
   - Verify `EntryListTemplate` renders list rows in the title-only format with external link buttons.
3. **`test_render_index_filter_bar`**:
   - Verify filter buttons contain `All Titles`, `Unread`, `Today`, `Starred` in order.
   - Verify `filter-today` triggers `/hx/entries?status=today`.

### Schema & Client Tests (`crates/miniflux`)
4. **`test_miniflux_client_get_entries_published_after`**:
   - Verify `MinifluxClient::get_entries` appends `published_after` query parameter to the URL when specified in `EntriesFilter`.
5. **`test_graphql_query_entries_published_after_and_search`**:
   - Verify GraphQL schema `entries` query accepts `publishedAfter` and `search` in `EntryFilterInput` and correctly propagates them to `domain_filter`.

### Server Route Logic Tests (`crates/server`)
6. **`test_entries_filter_query_mapping`**:
   - Verify mapping of `status=today`, `status=starred`, `status=all`, `status=unread`, and `search=keywords` to GraphQL filter string fragments.

---

## Implementation Tasks

### Task 1: Miniflux Client & GraphQL Schema Extensions
- [x] Add `published_after: Option<i64>` to `EntriesFilter` in `crates/miniflux/src/models.rs`.
- [x] Pass `published_after` query parameter in `crates/miniflux/src/client.rs`.
- [x] Add `published_after: Option<i64>` to `EntryFilterInput` in `crates/miniflux/src/schema/types.rs`.
- [x] Map `f.published_after` to `domain_filter.published_after` in `crates/miniflux/src/schema/query.rs`.

### Task 2: Server Route Search & Filter Resolution
- [x] Update `list_entries_htmx` in `crates/server/src/routes/entries.rs` to map:
  - `status=starred` -> `starred: true`
  - `status=today` -> `publishedAfter: <start_of_today_utc>` (all published today per decision 1B)
  - `status=all` -> no status filter
  - `status=unread` (default) -> `status: UNREAD`
  - `status=read` -> `status: READ`
- [x] Add search term escaping and append `search: "<escaped>"` to GraphQL query string.

### Task 3: Templates & UI Implementation
- [x] Update `crates/ui-components/templates/partials/entry_item.html` to title-only spatial row with external link button.
- [x] Update `crates/ui-components/templates/partials/entry_list.html` to match title-only format with external link button.
- [x] Update `crates/ui-components/templates/pages/index.html`:
  - Add "Today" filter button in sequence (`All Titles`, `Unread`, `Today`, `Starred`).
  - Add active filter pill click handler script.
  - Wire search input with active filter status.
- [x] Update `crates/ui-components/src/templates.rs` unit tests to validate new template markup.
