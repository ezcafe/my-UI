# Test Log: Automatically Load Next Page After Marking Page as Read

- **Run slug:** `20261123-load-next-page-after-mark-read`
- **Updated:** 2026-11-23T20:41:00+07:00
- **Test path:** lite
- **Result:** success
- **Smoke source:** Build-verify

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 109 | 109 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 59 | 59 | 0 | PASSED |
| **Total** | **197** | **197** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Milestone Test Details

### Milestone 1: UI Template & Client-Side Wiring
- Verified `EntryListTemplate` renders hidden input `<input type="hidden" id="active-offset" name="offset" value="X">` reflecting the active pagination offset.
- Verified `EntryListTemplate` with `has_next: true` renders `id="pagination-next-btn"` on the Next button.
- Verified `IndexTemplate` in authenticated mode renders `#mark-page-read-btn` with `hx-include="#active-status, #active-offset, [name='search']"`, `hx-disabled-elt="this"`, and `aria-label="Mark this page as read"`.
- Verified `IndexTemplate` in guest mode renders `#mark-page-read-btn` with `aria-label="Mark this page as read"`.
- Verified guest mode `markVisiblePageLocallyRead()` guards with `isMarkingPageRead`, programmatically clicks `#pagination-next-btn`, and falls back to `"Page marked as read. You are on the last page."` toast.
- Verified `htmx:afterSwap` scrolls `#entries-list` to `scrollTop = 0` on swap.
- Tests:
  - `templates::tests::test_entry_list_template_renders_active_offset_input`
  - `templates::tests::test_entry_list_template_renders_pagination_next_btn`
  - `templates::tests::test_index_template_mark_page_read_button_attributes`

### Milestone 2: Server-Side Form Parsing & Progression
- Verified `MarkPageReadForm` deserializes `offset` from urlencoded forms.
- Verified `render_entries_list_html` refactoring allows reusable HTML generation and out-of-band toast appending without clippy large-error lint warnings.
- Verified `mark_page_read_htmx` filter-aware progression:
  - For `status == "unread"`: loads `offset: 0` to retrieve the shifted unread batch.
  - For `status != "unread"`: checks total count under filter, advances `offset + 50` when `next_offset < total`, stays on current offset and renders `ToastTemplate` when on final page.
- Tests:
  - `routes::entries::tests::test_mark_page_read_form_offset_deserialization`

### Milestone 3: Route Integration Tests with WireMock
- Verified `POST /hx/mark-page-read` in `status=all` advances from `offset=0` to `offset=50` when `total = 100`.
- Verified `POST /hx/mark-page-read` in `status=all` stays on `offset=0` with out-of-band toast when `total = 50`.
- Verified `POST /hx/mark-page-read` in `status=unread` queries `offset=0` regardless of previous offset.
- Tests:
  - `routes::tests::test_hx_mark_page_read_advances_offset_when_more_pages_exist`
  - `routes::tests::test_hx_mark_page_read_stays_on_last_page_with_toast`
  - `routes::tests::test_hx_mark_page_read_loads_offset_zero_for_unread`

---

## 4. Smoke Verification

- **Smoke source:** Build-verify
- **Result:** verify-pass
- **Summary:** Verified all 197 unit, template, and route integration tests pass cleanly with 0 failures and 0 clippy warnings across the entire workspace.

---

## 5. Test-lite Verification (Suite / Targeted e2e)

- **Test path:** lite
- **Result:** success
- **Executed at:** 2026-11-23T20:41:00+07:00
- **Commands:**
  - `cargo test --workspace`: 197 tests passed, 0 failed.
  - `cargo clippy --workspace --all-targets`: 0 warnings.
- **Coverage breakdown:**
  - `miniflux-client`: 23 passed, 0 failed
  - `server`: 115 passed (109 unit + 6 integration), 0 failed
  - `ui-components`: 59 passed, 0 failed
- **Conclusion:** Full workspace test suite passes cleanly with zero test failures and zero linter warnings.

