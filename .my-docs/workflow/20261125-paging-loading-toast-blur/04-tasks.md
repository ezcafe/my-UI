# Tasks — Paging Loading, Mark-Page-Read Toast, and Lighter Read Blur

- **Slug:** `20261125-paging-loading-toast-blur`

## Implementation Tasks

1. **Reduce read-item blur (`crates/ui-components/templates/base.html`)**
   - Remove `blur(0.2px)` from `.ar-list-item.read` and `blur(0.25px)` from `.magazine-card.read`, keeping `opacity` and `grayscale`.
2. **Add loading overlay, `hx-indicator`, and `closeDetailPanel()` wiring (`crates/ui-components/templates/pages/index.html`, `crates/ui-components/templates/partials/entry_list.html`)**
   - Add `id="pagination-prev-btn"`, `hx-indicator="#entries-loading-bar"`, and `onclick="closeDetailPanel()"` to `Previous` and `Next` in `entry_list.html`.
   - Add `hx-indicator="#entries-loading-bar"` and `onclick="closeDetailPanel()"` to `#mark-page-read-btn` (auth) in `index.html`.
   - Add `#entries-loading-overlay` inside `.workspace-layout` and `showEntriesLoadingState()` / `hideEntriesLoadingState()` hooked into `htmx:beforeRequest`, `htmx:afterRequest`, `htmx:afterSwap`, `htmx:responseError`, and `htmx:sendError` for `#entries-list`.
   - Update guest `markVisiblePageLocallyRead()` to call `closeDetailPanel()` and `showToast('Page marked as read.')`.
3. **Always emit success toast in `mark_page_read_htmx` (`crates/server/src/routes/entries.rs`)**
   - Always append `ToastTemplate` (`"Page marked as read."` or `"Page marked as read. You are on the last page."`) on `Ok(entries_html)`.
4. **Tests (`crates/ui-components/src/templates.rs`, `crates/server/src/routes/mod.rs`)**
   - Add/update tests for template wiring, CSS read-state blur removal, guest toast/closeDetailPanel, and server `mark_page_read_htmx` toast on `unread` and non-last-page responses.
   - Run `cargo test -p ui-components` and `cargo test -p server`.
