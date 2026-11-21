Result: clean

# TDD Plan Review

This document evaluates the planned unit and integration tests defined in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-counts-paging-read-flex/04-tasks.md) against the design specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-counts-paging-read-flex/03-design.md).

---

## 1. Test Coverage Analysis

### Requirement 1: Dynamic counts in Unread and Today filter pills
- **Planned tests**:
  - `test_today_counter_returns_correct_count` in `crates/miniflux/src/schema/mod.rs`
  - `test_index_template_renders_today_and_unread_counts` in `crates/ui-components/src/templates.rs`
- **Target verification**:
  - Validates GraphQL `todayCounter` correctly queries Miniflux with `published_after` set to UTC midnight.
  - Validates `IndexTemplate` renders `Today (<count>)` badge inside `#filter-today-count` and `Unread (<count>)` badge inside `#filter-unread-count`.
- **Verdict**: **Clean** — covers both the GraphQL query implementation and template badge rendering.

---

### Requirement 2: Standard Prev/Next pagination at list bottom
- **Planned tests**:
  - `test_entries_result_returns_total_and_entries` in `crates/miniflux/src/schema/mod.rs`
  - `test_entry_list_renders_pagination_when_multiple_pages` in `crates/ui-components/src/templates.rs`
  - `test_entry_list_hides_pagination_when_single_page` in `crates/ui-components/src/templates.rs`
- **Target verification**:
  - Validates `entriesResult` query returns `EntryResultSetGql` with total matching count and entry items.
  - Validates that when `total_pages > 1`, `EntryListTemplate` renders pagination controls at the bottom of the list with `Previous`, `Page X of Y`, and `Next` buttons containing appropriate HTMX attributes (`hx-get`, `hx-target="#entries-list"`, `hx-swap="innerHTML"`).
  - Validates pagination controls are omitted when `total_pages <= 1`.
  - Validates `has_prev` / `has_next` button states (disabled styling vs clickable button).
- **Verdict**: **Clean** — verifies GraphQL total extraction, template pagination logic, and HTMX attributes.

---

### Requirement 3: Auto-mark as read on detail view with OOB dot clearing
- **Planned test**:
  - `test_get_entry_detail_auto_marks_as_read_and_returns_oob_swaps` in `crates/server/src/routes/entries.rs`
- **Target verification**:
  - Validates that `get_entry_detail_htmx` inspects the entry's read state and sends a status update (`"read"`) when unread.
  - Verifies the response includes out-of-band updates (`hx-swap-oob="true"`) for `#unread-pill`, `#filter-unread-count`, and `#entry-{id}-dot`.
  - Verifies the article row is retained in place (no DOM row removal) to preserve the user's scroll position.
- **Verdict**: **Clean** — covers backend state mutation, out-of-band updates, and UI stability.

---

### Requirement 4: Fluid spatial container
- **Planned test**:
  - `test_spatial_window_fluid_width_container_query` in `crates/ui-components/src/templates.rs`
- **Target verification**:
  - Asserts that `base.html` defines `.spatial-shell` container context and `.spatial-window` using modern CSS container query rules `max-width: min(100cqi, 1440px)` rather than fixed 720px width.
- **Verdict**: **Clean** — verifies responsive layout rules without hardcoded breakpoints.

---

## 2. Review Conclusion

All four core functional requirements from ideation and design are thoroughly covered by planned automated tests.
The test plan strictly enforces the TDD iron law:
- Tests will be written first and fail on the existing codebase (Red).
- Implementation changes will be applied to satisfy the assertions (Green).
- Refactoring and cleanup will follow under green tests.

Result: **clean** — ready for Gate B.
