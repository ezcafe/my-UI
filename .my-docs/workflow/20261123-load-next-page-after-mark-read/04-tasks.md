# Tasks: Automatically Load Next Page After Marking Page as Read

## Planned Tests

- [x] `test_entry_list_template_renders_active_offset_input`: Verifies that `EntryListTemplate` renders `<input type="hidden" id="active-offset" name="offset" value="X">` reflecting the provided offset value.
- [x] `test_entry_list_template_renders_pagination_next_btn`: Verifies that `EntryListTemplate` renders `id="pagination-next-btn"` on the Next button when `has_next` is true.
- [x] `test_index_template_mark_page_read_button_attributes`: Verifies that `IndexTemplate` renders `#mark-page-read-btn` with `hx-include` containing `#active-offset`, `hx-disabled-elt="this"`, and `aria-label="Mark this page as read"`.
- [x] `test_mark_page_read_form_offset_deserialization`: Verifies that `MarkPageReadForm` correctly deserializes `offset` from urlencoded form bodies.
- [x] `test_hx_mark_page_read_advances_offset_when_more_pages_exist`: WireMock integration test verifying that for `status=all`, when `next_offset < total`, the route handler advances to `next_offset` (e.g. offset 50) and queries the next batch.
- [x] `test_hx_mark_page_read_stays_on_last_page_with_toast`: WireMock integration test verifying that for `status=all`, when `next_offset >= total`, the route handler remains on the current offset and includes the last-page toast in the response.
- [x] `test_hx_mark_page_read_loads_offset_zero_for_unread`: WireMock integration test verifying that for `status=unread`, the route handler queries `offset: 0` to load the top of the remaining unread pool.

---

## Milestone 1: UI Template & Client-Side Wiring

- [x] **Task 1.1: TDD template unit tests for active offset, pagination next id, and mark-page-read button attributes**
  - Path: `crates/ui-components/src/templates.rs`
  - Action: Add `test_entry_list_template_renders_active_offset_input`, `test_entry_list_template_renders_pagination_next_btn`, and `test_index_template_mark_page_read_button_attributes`.
  - Verification: Run `cargo test --package ui-components templates::tests::test_entry_list_template_renders_pagination_next_btn` (fails prior to template update).
- [x] **Task 1.2: Add active-offset hidden input and id="pagination-next-btn" in entry_list.html**
  - Path: `crates/ui-components/templates/partials/entry_list.html`
  - Action:
    - Add `<input type="hidden" id="active-offset" name="offset" value="{{ offset }}">` at the top of the template.
    - Add `id="pagination-next-btn"` to the Next button (`{% if has_next %}`).
- [x] **Task 1.3: Update mark-page-read-btn attributes in index.html**
  - Path: `crates/ui-components/templates/pages/index.html`
  - Action:
    - In `auth_enabled` branch: Add `hx-disabled-elt="this"`, `aria-label="Mark this page as read"`, and update `hx-include` to include `#active-offset`: `hx-include="#active-status, #active-offset, [name='search']"`.
    - In guest branch: Add `aria-label="Mark this page as read"`.
  - Verification: Re-run `cargo test --package ui-components`.
- [x] **Task 1.4: Update guest mode markVisiblePageLocallyRead() and scroll reset**
  - Path: `crates/ui-components/templates/pages/index.html`
  - Action:
    - In `markVisiblePageLocallyRead()`: Add in-flight guard `isMarkingPageRead` and temporarily disable the button to prevent duplicate triggers; parse IDs properly from string or array; mark read via `markLocallyRead`; look up `document.getElementById('pagination-next-btn')`; if found, call `nextBtn.click()`, else call `showToast('Page marked as read. You are on the last page.')`.
    - In `htmx:afterSwap`: Add `if (evt.detail.target.id === 'entries-list') evt.detail.target.scrollTop = 0;`.

---

## Milestone 2: Server-Side Form Parsing & Progression

- [x] **Task 2.1: TDD tests for MarkPageReadForm offset parsing**
  - Path: `crates/server/src/routes/entries.rs`
  - Action: Add `test_mark_page_read_form_offset_deserialization` testing deserialization with `offset=50`, missing offset, and invalid values.
  - Verification: Run `cargo test --package server routes::entries::tests::test_mark_page_read_form_offset_deserialization` (fails before struct update).
- [x] **Task 2.2: Add offset to MarkPageReadForm**
  - Path: `crates/server/src/routes/entries.rs`
  - Action: Add `pub offset: Option<i64>` to `MarkPageReadForm`.
  - Verification: Re-run `cargo test --package server routes::entries::tests::test_mark_page_read_form_offset_deserialization`.
- [x] **Task 2.3: Refactor render_entries_list_html helper**
  - Path: `crates/server/src/routes/entries.rs`
  - Action: Separate HTML generation into `render_entries_list_html(...) -> Result<String, Box<Response>>`, keeping `render_entries_list` as a wrapper returning `Response`.
- [x] **Task 2.4: Implement filter-aware progression & last-page toast in mark_page_read_htmx**
  - Path: `crates/server/src/routes/entries.rs`
  - Action:
    - If `status == "unread"` or None: target offset = 0.
    - If `status != "unread"`: count total entries with `limit: 1, offset: 0`; if `current_offset + 50 < total`, target offset = `current_offset + 50`; else target offset = `current_offset`, `show_last_page_toast = true`.
    - Render entries via `render_entries_list_html`.
    - If `show_last_page_toast`, append rendered `ToastTemplate` with `"Page marked as read. You are on the last page."`.

---

## Milestone 3: Route Integration Tests with WireMock

- [x] **Task 3.1: Integration tests for mark_page_read_htmx progression**
  - Path: `crates/server/src/routes/mod.rs`
  - Action:
    - Add `test_hx_mark_page_read_advances_offset_when_more_pages_exist`.
    - Add `test_hx_mark_page_read_stays_on_last_page_with_toast`.
    - Add `test_hx_mark_page_read_loads_offset_zero_for_unread`.
  - Verification: Run `cargo test --package server routes::mod::tests::test_hx_mark_page_read`.

---

## Milestone 4: Workspace Verification

- [x] **Task 4.1: Workspace-wide build, tests, and lints**
  - Action: Run `cargo test --workspace` and `cargo clippy --workspace --all-targets`.

---

## Result

Result: done
