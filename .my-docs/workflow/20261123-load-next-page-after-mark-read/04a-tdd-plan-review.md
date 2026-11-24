Result: clean

# TDD Plan Review: Automatically Load Next Page After Marking Page as Read

## Key Takeaways
- **Test coverage is complete**: Planned tests in `04-tasks.md` comprehensively cover UI templates, client-side pagination bindings, form deserialization, and filter-aware server route progression.
- **Fail-first test strategy**: Each test targets specific functionality that fails on the current codebase, ensuring true test-driven development (red-green-refactor).
- **Edge cases and regression safety**: Handled cases include end-of-list bounds, empty/missing query parameters, guest versus authenticated behavior, and scroll reset on page swap.
- **Verdict**: **Clean**. Proceed to Gate B settlement.

---

## Evaluation of Planned Test Cases

### 1. UI Template & Client-Side Wiring

- **`test_entry_list_template_renders_active_offset_input`** (`crates/ui-components/src/templates.rs`)
  - **Scope**: Asserts that `EntryListTemplate` renders `<input type="hidden" id="active-offset" name="offset" value="X">` matching the template's `offset` field value.
  - **Verdict**: **Clean**. Directly verifies that the active offset is present in the DOM for HTMX inclusion during page mark-as-read requests.

- **`test_entry_list_template_renders_pagination_next_btn`** (`crates/ui-components/src/templates.rs`)
  - **Scope**: Asserts that `EntryListTemplate` renders `id="pagination-next-btn"` on the Next button when `has_next` is true, and omits the ID/button when `has_next` is false.
  - **Verdict**: **Clean**. Prevents guest mode JavaScript from clicking an absent or misidentified navigation element.

- **`test_index_template_mark_page_read_button_attributes`** (`crates/ui-components/src/templates.rs`)
  - **Scope**: Asserts that `IndexTemplate` renders `#mark-page-read-btn` with `hx-include` containing `#active-offset`, `hx-disabled-elt="this"`, and `aria-label="Mark this page as read"` when `auth_enabled: true`, and with `aria-label` and `onclick` when `auth_enabled: false`.
  - **Verdict**: **Clean**. Enforces accessibility, duplicate request prevention, and parameter transmission.

### 2. Server Form Parsing & Route Progression

- **`test_mark_page_read_form_offset_deserialization`** (`crates/server/src/routes/entries.rs`)
  - **Scope**: Asserts that `MarkPageReadForm` correctly deserializes `offset` from urlencoded form bodies (e.g., `offset=50`), defaults safely when missing, and handles malformed input.
  - **Verdict**: **Clean**. Guarantees robust input parsing before route progression calculations.

- **`test_hx_mark_page_read_advances_offset_when_more_pages_exist`** (`crates/server/src/routes/mod.rs`)
  - **Scope**: WireMock integration test verifying that for `status=all`, when `next_offset < total`, the route handler advances to `next_offset` (e.g., offset 50) and queries Miniflux for the next batch.
  - **Verdict**: **Clean**. Verifies non-unread pagination advancement when additional pages exist.

- **`test_hx_mark_page_read_stays_on_last_page_with_toast`** (`crates/server/src/routes/mod.rs`)
  - **Scope**: WireMock integration test verifying that for `status=all`, when `next_offset >= total`, the route handler stays on the current offset and renders an out-of-band toast notification (`ToastTemplate` with message `"Page marked as read. You are on the last page."`).
  - **Verdict**: **Clean**. Verifies graceful termination on the final page without empty state degradation or pagination overrun.

- **`test_hx_mark_page_read_loads_offset_zero_for_unread`** (`crates/server/src/routes/mod.rs`)
  - **Scope**: WireMock integration test verifying that for `status=unread` (or default `None`), the route handler always loads `offset: 0` to retrieve the shifted batch of unread articles.
  - **Verdict**: **Clean**. Protects against the critical 50-article skip bug that occurs if offset were unconditionally incremented.

---

## Edge Cases & Regression Safety Review

1. **Default and Missing Filter Parameters**:
   - `MarkPageReadForm` status defaults to `None`, which the route handler correctly treats as `unread` (`is_unread = status == "unread" || status.is_none()`).
   - Covered in route tests with and without explicit `status` field.

2. **Existing Route Integration Test Compatibility**:
   - The existing test `test_hx_mark_page_read_endpoint` in `crates/server/src/routes/mod.rs` tests `status=all` with `total = 0`. Under the updated logic, this lands on the last-page branch (`next_offset >= total`), returning the current page plus toast, retaining the expected unread pill. Regression risk is negligible.

3. **In-Flight Duplicate Clicks**:
   - Authenticated mode uses `hx-disabled-elt="this"`.
   - Guest mode uses a JavaScript boolean lock `isMarkingPageRead` and temporary DOM `btn.disabled = true`.

4. **Naming Consistency**:
   - All test names follow project standard patterns (`test_<component>_<behavior>`) and clearly indicate expected outcomes.
