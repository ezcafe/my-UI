# Test Plan and Verification Report: Client-Only Read & Bookmark (Guest Mode) and Server Mutation Rejection

- **Result:** success
- **Test path:** lite
- **Date:** 2026-11-21
- **Stage:** test-lite

---

## 1. Test Execution Summary

| Crate / Target | Total Tests | Passed | Failed | Status |
|---|---|---|---|---|
| `ui-components` (lib unittests) | 28 | 28 | 0 | pass |
| `server` (lib unittests) | 50 | 50 | 0 | pass |
| `server` (e2e integration `tests/e2e_tests.rs`) | 4 | 4 | 0 | pass |
| `miniflux-client` (lib unittests) | 22 | 22 | 0 | pass |
| **Workspace Total** | **104** | **104** | **0** | **success** |

---

## 2. Test Path: Lite Verification

### Scope & Validation Strategy
1. **Template Branching (`auth_enabled: false`)**:
   - `EntryDetailTemplate` omits `hx-post` mutation endpoints for read/unread and bookmarking, rendering client-side handlers `onclick="toggleLocalReadStatus({{ entry.id }})"` and `onclick="triggerNativeBookmark('...', '...')"`:
     - `cargo test -p ui-components test_entry_detail_renders_local_actions_when_auth_disabled` -> **Passed**
   - `IndexTemplate` `#mark-page-read-btn` omits `hx-post="/hx/mark-page-read"` and renders `onclick="markVisiblePageLocallyRead()"`:
     - `cargo test -p ui-components test_index_template_mark_page_read_button_auth_toggle` -> **Passed**
   - Client-side JavaScript under `{% if !auth_enabled %}` in `index.html` implements `toggleLocalReadStatus(id)`, `markVisiblePageLocallyRead()`, `triggerNativeBookmark(title, url)`, and `showToast(message)`:
     - `cargo test -p ui-components test_index_template_renders_guest_state_script_when_auth_disabled` -> **Passed**

2. **Template Branching (`auth_enabled: true`)**:
   - `EntryDetailTemplate` renders standard server `hx-post` attributes for `/hx/entries/:id/read?status=unread` and `/hx/entries/:id/bookmark`:
     - `cargo test -p ui-components test_entry_detail_renders_htmx_actions_when_auth_enabled` -> **Passed**
   - `IndexTemplate` `#mark-page-read-btn` renders `hx-post="/hx/mark-page-read"`:
     - `cargo test -p ui-components test_index_template_mark_page_read_button_auth_toggle` -> **Passed**

3. **Server Mutation Rejection Guardrails**:
   - All five server write endpoints immediately reject incoming mutation requests with `HTTP 403 Forbidden` when `!config.auth_enabled`:
     - `POST /hx/entries/:id/read`
     - `POST /hx/entries/:id/bookmark`
     - `POST /hx/mark-page-read`
     - `POST /hx/feeds/:id/mark-all-read`
     - `POST /hx/mark-all-read`
   - Verified via integration tests:
     - `cargo test -p server test_mutation_endpoints_reject_with_403_when_auth_disabled` -> **Passed**
     - `cargo test -p server test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled` -> **Passed**
     - `cargo test -p server test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled` -> **Passed**

4. **End-to-End Suite Verification**:
   - Integration tests in `crates/server/tests/e2e_tests.rs`:
     - `test_app_starts_and_responds_to_health_check` -> **Passed**
     - `test_e2e_authenticated_session_flow` -> **Passed**
     - `test_e2e_guest_mode_flow` -> **Passed**
     - `test_mutation_endpoints_reject_with_403_when_auth_disabled` -> **Passed**
   - Full workspace test suite:
     - `cargo test --workspace` -> **Passed (104/104 passed)**

---

## 3. TDD Implementation Verification (`04-tasks.md`)

All planned TDD test cases from `04-tasks.md` are verified and passing:
- `test_entry_detail_renders_local_actions_when_auth_disabled` (`crates/ui-components/src/templates.rs`)
- `test_entry_detail_renders_htmx_actions_when_auth_enabled` (`crates/ui-components/src/templates.rs`)
- `test_index_template_mark_page_read_button_auth_toggle` (`crates/ui-components/src/templates.rs`)
- `test_mutation_endpoints_reject_with_403_when_auth_disabled` (`crates/server/tests/e2e_tests.rs`)
- `test_e2e_guest_mode_flow` (`crates/server/tests/e2e_tests.rs`)
