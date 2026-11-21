Result: clean

# TDD Plan Review: Client-Only Read & Bookmark and Server Mutation Rejection

This document reviews the planned automated test cases defined in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-client-only-read-bookmark/04-tasks.md) against the design specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-client-only-read-bookmark/03-design.md).

---

## 1. Test Coverage Analysis

### Requirement 1: `EntryDetailTemplate` Branching (UI Components)
- **Planned tests**:
  - `test_entry_detail_renders_local_actions_when_auth_disabled` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - `test_entry_detail_renders_htmx_actions_when_auth_enabled` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
- **Target verification**:
  - Validates that when `auth_enabled: false`, `EntryDetailTemplate` suppresses `hx-post` attributes and renders `onclick="toggleLocalReadStatus({{ entry.id }})"` and `onclick="triggerNativeBookmark(...)"`.
  - Validates that when `auth_enabled: true`, `EntryDetailTemplate` renders standard `hx-post` attributes for server-side persistence.
  - Ensures existing tests (`test_render_entry_detail_template`, `test_render_entry_detail_sanitized_content`, and `test_render_entry_detail_with_brief_card`) are updated for the new `auth_enabled` struct field.
- **Verdict**: **Clean** — directly validates both guest mode and authenticated rendering paths.

---

### Requirement 2: Index Template Mark-Page-Read Button Branching (UI Components)
- **Planned tests**:
  - `test_index_template_mark_page_read_button_auth_toggle` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
- **Target verification**:
  - Validates that `#mark-page-read-btn` uses `hx-post="/hx/mark-page-read"` when `auth_enabled: true`.
  - Validates that `#mark-page-read-btn` omits `hx-post` and invokes `onclick="markVisiblePageLocallyRead()"` when `auth_enabled: false`.
  - Validates that guest helper scripts (`toggleLocalReadStatus`, `markVisiblePageLocallyRead`, `triggerNativeBookmark`) are embedded under `{% if !auth_enabled %}`.
- **Verdict**: **Clean** — validates both markup and client script integration.

---

### Requirement 3: Server Mutation Rejection (Server Endpoints)
- **Planned tests**:
  - `test_mark_entry_read_htmx_returns_403_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) or [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)
  - `test_toggle_bookmark_htmx_returns_403_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) or [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)
  - `test_mark_page_read_htmx_returns_403_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) or [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)
  - `test_mark_feed_read_htmx_returns_403_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) or [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)
  - `test_mark_all_read_htmx_returns_403_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) or [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)
- **Target verification**:
  - Validates that all five mutation endpoints immediately return `HTTP 403 Forbidden` when `config.auth_enabled == false`.
  - Ensures existing test in `crates/server/tests/e2e_tests.rs` (which previously checked 200 OK on mark read) is updated to assert `HTTP 403 Forbidden`.
- **Verdict**: **Clean** — comprehensive guardrail verification across all server mutation points.

---

## 2. Review Conclusion

All functional requirements identified during Ideation, Analysis, and Architecture Design are covered by explicit, named automated test cases.
The plan follows the TDD iron law:
- Tests will be written and verified failing (Red) before writing any production code.
- Minimal production code will be implemented to turn tests green (Green).
- Refactoring will proceed under green test coverage.

Result: **clean** — ready for Gate B.
