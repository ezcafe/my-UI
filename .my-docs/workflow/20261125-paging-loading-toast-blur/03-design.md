# Design — Paging Loading, Mark-Page-Read Toast, and Lighter Read Blur

- **Slug:** `20261125-paging-loading-toast-blur`
- **Result:** `done`

## 1. Hide `#entries-list` + `#detail-panel` & Show Loading Indicator on Paging / Mark Page as Read

### Templates (`crates/ui-components/templates/pages/index.html` & `partials/entry_list.html`)

- **`entry_list.html`:**
  - Add `id="pagination-prev-btn"`, `hx-indicator="#entries-loading-bar"`, and `onclick="closeDetailPanel()"` to the `Previous` button.
  - Add `hx-indicator="#entries-loading-bar"` and `onclick="closeDetailPanel()"` to `#pagination-next-btn` (`Next` button).
- **`index.html`:**
  - On `#mark-page-read-btn` (auth branch): add `hx-indicator="#entries-loading-bar"` and `onclick="closeDetailPanel()"`.
  - Inside `.workspace-layout` (next to `#entries-loading-bar` and `#entries-list`): add a centered loading overlay `#entries-loading-overlay` (`hidden absolute inset-0 z-20 flex flex-col items-center justify-center gap-2 bg-card pointer-events-none`) with a CSS/SVG spinner and `"Loading..."` label.
  - Add helper functions `showEntriesLoadingState()` and `hideEntriesLoadingState()` in `index.html`:
    - `showEntriesLoadingState()`: hides `#entries-list` (e.g., `entriesList.classList.add('invisible')` or `opacity-0` so layout/scroll container stays intact without flash) and removes `hidden` from `#entries-loading-overlay`.
    - `hideEntriesLoadingState()`: removes `invisible` / `opacity-0` from `#entries-list` and adds `hidden` to `#entries-loading-overlay`.
  - In `htmx:beforeRequest`: when `!evt.defaultPrevented` and the triggering element is `#mark-page-read-btn`, `#pagination-prev-btn`, or `#pagination-next-btn`, call `closeDetailPanel()` (defensive) and `showEntriesLoadingState()`. Because `htmx:beforeRequest` fires after `hx-vals` is evaluated, `getVisibleEntryIds()` still reads all visible rows before `#entries-list` is hidden.
  - In `htmx:afterSwap`, `htmx:afterRequest`, `htmx:responseError`, and `htmx:sendError`: when `evt.detail.target && evt.detail.target.id === 'entries-list'`, call `hideEntriesLoadingState()`. (Also show an error toast or restore visibility on error so `#entries-list` is never stuck hidden.)
  - In guest mode `markVisiblePageLocallyRead()`: call `closeDetailPanel()` if defined, and call `showToast('Page marked as read.')`.

## 2. Always Show Success Toast on Mark Page as Read (`crates/server/src/routes/entries.rs`)

- In `mark_page_read_htmx`:
  - When `render_entries_list_html` returns `Ok(entries_html)`:
    - Always render `ToastTemplate` with `is_error: false`:
      - `show_last_page_toast == true` -> `"Page marked as read. You are on the last page."`
      - `show_last_page_toast == false` -> `"Page marked as read."`
    - Return `Html(format!("{entries_html}{toast_html}")).into_response()`.

## 3. Reduce Blur on Read Items (`crates/ui-components/templates/base.html`)

- Update `.ar-list-item.read`:
  - Change `filter: grayscale(80%) blur(0.2px);` to `filter: grayscale(80%);` (keeping `opacity: 0.55;` and `color: var(--muted-foreground) !important;` on `.article-title`).
- Update `.magazine-card.read`:
  - Change `filter: grayscale(85%) blur(0.25px);` to `filter: grayscale(85%);` (keeping `opacity: 0.45;` and `color: var(--muted-foreground) !important;` on `.magazine-title`).
- Keep `:hover` reset (`opacity: 0.9; filter: grayscale(0%);`) clean and crisp.

## Verification Plan

- Unit & integration tests:
  - `cargo test -p ui-components` (verify existing template tests pass + add assertions for `pagination-prev-btn`, `hx-indicator`, `closeDetailPanel()`, `entries-loading-overlay`, `showToast` in guest `markVisiblePageLocallyRead`, and no `blur(0.2px)` / `blur(0.25px)` on `.read`).
  - `cargo test -p server` (verify existing `mark_page_read` tests pass + add/update assertion that non-last-page and `unread` responses include `"Page marked as read."` toast).
