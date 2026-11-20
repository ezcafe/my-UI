# TDD Test Plan Review

- **Result:** clean

## 1. Review Summary

The planned tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261119-list-ui-search-filters/04-tasks.md) provide comprehensive, tiered coverage across all components modified for the title-only row, external link button, Today filter, and search/filter fixes.

## 2. Test Coverage Assessment

### UI & Templates Tier (`crates/ui-components`)
- **`test_render_title_only_entry_item`**:
  - Validates title line clamping and styling (`line-clamp-2`, `text-sm font-medium`).
  - Asserts omission of feed summary, reading time, and feed author/title.
  - Verifies presence of external link button with `target="_blank"`, `rel="noopener noreferrer"`, and `onclick="event.stopPropagation()"`.
  - Verifies presence of unread indicator dot when `is_read = false` and absence when `is_read = true`.
- **`test_render_entry_list_with_title_only_items`**:
  - Validates container rendering and row iteration matching the spatial title-only layout.
- **`test_render_index_filter_bar`**:
  - Validates filter button sequence (`All Titles`, `Unread`, `Today`, `Starred`).
  - Validates HTMX endpoints (`/hx/entries?status=today`, etc.) and active state styling hook.

### Miniflux & GraphQL Tier (`crates/miniflux`)
- **`test_miniflux_client_get_entries_published_after`**:
  - Validates `published_after` query parameter serialization in `MinifluxClient::get_entries`.
- **`test_graphql_query_entries_published_after_and_search`**:
  - Validates GraphQL schema `entries` query accepting `publishedAfter` (Int) and `search` (String) in `EntryFilterInput` and propagation to `domain_filter`.

### Server Route Tier (`crates/server`)
- **`test_entries_filter_query_mapping`**:
  - Validates query parameter mapping to GraphQL filter fragments:
    - `status=today` -> `publishedAfter: <start_of_today_utc>` (both read & unread per User Decision 1B)
    - `status=starred` -> `starred: true`
    - `status=all` -> no status restriction
    - `status=unread` (default) -> `status: UNREAD`
    - `status=read` -> `status: READ`
    - `search=<keyword>` -> `search: "<escaped>"` with proper escaping of quotes and backslashes.

## 3. TDD Feasibility & Execution Strategy
- All tests are unit-level, fast-executing, and do not require external network or live Miniflux instances.
- The tests adhere to the strict Red-Green-Refactor iron law: failing test written first, verify-fail confirmed, minimal implementation added, verify-pass confirmed.

## 4. Conclusion
The test plan is complete, well-isolated, and ready for execution.
