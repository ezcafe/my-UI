Result: clean

# TDD Plan Review

This document evaluates the planned unit and integration tests defined in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-article-filters-detail-actions/04-tasks.md) against the design specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-article-filters-detail-actions/03-design.md).

---

## 1. Test Coverage Analysis

### Requirement 1: Filter bar reordering and default active pill
- **Planned test**: `test_filter_bar_reordering` in [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).
- **Target verification**:
  - Validates sequence of filter pills: `Unread` < `Today` < `Starred` < `All titles`.
  - Verifies `Unread` pill renders with active CSS classes (`bg-indigo-500/20 text-indigo-300`).
  - Verifies other pills render with default inactive styling.
- **Verdict**: **Clean** — directly validates UI ordering and default active state.

### Requirement 2: Detail view author metadata removal
- **Planned test**: `test_detail_template_omits_author` in [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).
- **Target verification**:
  - Instantiates `EntryDetailTemplate` with an entry where `author` is `Some(...)`.
  - Asserts rendered HTML does not contain `By <Author>`.
  - Asserts feed title and publication date remain present.
- **Verdict**: **Clean** — prevents regression on author metadata display while preserving vital feed info.

### Requirement 3: Detail view mark unread action
- **Planned test**: `test_detail_template_has_mark_unread_button` in [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).
- **Target verification**:
  - Verifies rendered detail view includes `hx-post="/hx/entries/{id}/read?status=unread"`.
  - Verifies button DOM position precedes the bookmark toggle button.
- **Verdict**: **Clean** — ensures HTMX attributes, target, and placement match spec.

### Requirement 4: Search filter status default
- **Planned test**: `test_build_graphql_filter_search_defaults_to_all` in [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs).
- **Target verification**:
  - Tests `build_graphql_filter` with `search: Some(...)` and no `status` parameter.
  - Verifies output includes `search: ...` but omits `status: UNREAD`.
- **Verdict**: **Clean** — verifies hybrid search backend contract to search all articles by default.

### Requirement 5: GraphQL batch status mutation
- **Planned test**: `test_graphql_mutation_update_entries_status` in [crates/miniflux/src/schema/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/schema/mod.rs).
- **Target verification**:
  - Verifies GraphQL mutation `updateEntriesStatus` sends HTTP `PUT /v1/entries` to Miniflux client with array of entry IDs and target status.
- **Verdict**: **Clean** — covers GraphQL schema mutation binding and payload marshalling.

### Requirement 6: Mark page read HTMX endpoint
- **Planned test**: `test_hx_mark_page_read_endpoint` in [crates/server/src/routes/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs).
- **Target verification**:
  - End-to-end Wiremock test invoking `POST /hx/mark-page-read` with `entry_ids`.
  - Verifies Miniflux batch update is called and rendered `EntryListTemplate` returns with updated `#unread-pill`.
- **Verdict**: **Clean** — covers complete HTTP route, authentication, and OOB swap markup.

---

## 2. Review Conclusion

All five core functional requirements have corresponding automated unit/integration tests defined. The plan adheres strictly to TDD (red-green-refactor) rules:
- Template tests will fail until templates are edited.
- Server filter test will fail until `build_graphql_filter` logic is adjusted.
- Batch mutation and route tests will fail until schema and route handler are implemented.

Result: **clean** — ready for Gate B.
