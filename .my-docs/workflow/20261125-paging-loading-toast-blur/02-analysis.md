# Analysis — Paging Loading, Mark-Page-Read Toast, and Lighter Read Blur

- **Slug:** `20261125-paging-loading-toast-blur`
- **Status:** `Result: done`

## Current State & Gaps

1. **Paging & Mark-Page-Read Loading State (`#entries-list` & `#entries-loading-bar`):**
   - `crates/ui-components/templates/pages/index.html` (lines 160–165) renders `.workspace-layout` containing `#entries-loading-bar` (`class="htmx-indicator h-0.5 bg-brand animate-pulse ..."`) and `#entries-list`.
   - Filter pills (`#filter-unread`, `#filter-today`, `#filter-starred`, `#filter-all`) and `#search-form` declare `hx-indicator="#entries-loading-bar"`.
   - `#mark-page-read-btn` (`index.html:43-53`), `Previous` (`entry_list.html:197-202`), and `Next` (`entry_list.html:212-217`) do **not** declare `hx-indicator`, and `#entries-list` content remains visible during the HTMX request.
   - Note: `#mark-page-read-btn` uses `hx-vals="js:{entry_ids: getVisibleEntryIds()}"`, which reads `#entries-list .annuaire-row, #entries-list .ar-list-item` where `item.style.display !== 'none'`. Any hiding of `#entries-list` during loading must happen **after** `hx-vals` is evaluated (e.g., inside `htmx:beforeRequest` when `!evt.defaultPrevented`, or via a CSS class on `#entries-list` that does not mutate `item.style.display` prior to parameter collection).

2. **Detail Panel (`#detail-panel`) Stays Open on Paging / Mark Page as Read:**
   - `closeDetailPanel()` (`index.html:881-909`) hides `#detail-panel`, resets it to `#detail-empty-placeholder`, and clears `?entry=` / `/entries/<id>` from the URL.
   - `handleFilterClick`, `setActiveFilter`, and `onSearchSubmit` already call `closeDetailPanel()`.
   - `Previous`, `Next`, and `#mark-page-read-btn` (both auth HTMX and guest `markVisiblePageLocallyRead()`) do not call `closeDetailPanel()`.

3. **Mark Page as Read Success Toast:**
   - `mark_page_read_htmx` (`crates/server/src/routes/entries.rs:1109-1249`) only appends `ToastTemplate` when `show_last_page_toast` is `true` (`"Page marked as read. You are on the last page."`). When `is_unread` is `true` or `next_offset < total`, it returns `Html(entries_html)` with no toast.
   - Guest mode `markVisiblePageLocallyRead()` (`index.html:1644-1657`) marks items locally read and invalidates `filterCache` without calling `showToast(...)`.

4. **Read Item Blur (`crates/ui-components/templates/base.html`):**
   - `.ar-list-item.read` (`base.html:461-465`) uses `opacity: 0.55; filter: grayscale(80%) blur(0.2px);`.
   - `.magazine-card.read` (`base.html:474-478`) uses `opacity: 0.45; filter: grayscale(85%) blur(0.25px);`.

## Existing Test Constraints

- `crates/ui-components/src/templates.rs` (`test_index_template_loading_indicator_and_filter_cache`, lines 1520-1560) asserts:
  - `id="entries-loading-bar"`
  - `class="htmx-indicator h-0.5 bg-brand animate-pulse`
  - `.htmx-indicator` in base styles
  - `hx-indicator="#entries-loading-bar"`
- `crates/server/src/routes/mod.rs` (`test_hx_mark_page_read_stays_on_last_page_with_toast`, line 986) asserts `html.contains("Page marked as read. You are on the last page.")`.
