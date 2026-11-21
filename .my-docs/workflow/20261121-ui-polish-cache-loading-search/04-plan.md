# Implementation Plan: UI Polish, Client-Side Filter Caching & Loading State

Result: planned

## Phase Breakdown

### Phase 1: Centered Login Card & Template Test Update
- [x] Task 1.1: Refactor `crates/ui-components/templates/pages/login.html` to remove the left brand panel and center the Swiss card with "MY NEWS", subtitle, and Pocket-ID button.
- [x] Task 1.2: Update `test_login_page_renders_dual_panel_swiss_brand` in `crates/ui-components/src/templates.rs` to `test_login_page_renders_centered_card`, testing for the centered layout, title, and button.

### Phase 2: Last Item Border Removal
- [x] Task 2.1: Add `last:border-b-0` to the item container in `crates/ui-components/templates/partials/entry_list.html`.
- [x] Task 2.2: Add `last:border-b-0` to the item container in `crates/ui-components/templates/partials/entry_item.html`.
- [x] Task 2.3: Add unit tests in `crates/ui-components/src/templates.rs` verifying `last:border-b-0` in `EntryListTemplate` and `EntryItemTemplate`.

### Phase 3: HTMX Loading Indicator
- [x] Task 3.1: Add `.htmx-indicator` styles to `<style>` in `crates/ui-components/templates/base.html`.
- [x] Task 3.2: Insert `#entries-loading-bar` above `#entries-list` in `crates/ui-components/templates/pages/index.html`.
- [x] Task 3.3: Add `hx-indicator="#entries-loading-bar"` to filter pills and `#search-form` in `crates/ui-components/templates/pages/index.html`.
- [x] Task 3.4: Add unit test in `crates/ui-components/src/templates.rs` verifying loading indicator markup in `IndexTemplate`.

### Phase 4: Client-Side Filter Caching (60s TTL)
- [x] Task 4.1: In `crates/ui-components/templates/pages/index.html`, declare `filterCache = new Map()`, `FILTER_CACHE_TTL_MS = 60000`, and `invalidateFilterCache()`.
- [x] Task 4.2: Implement `handleFilterClick(btn, event)` in `index.html` to intercept filter clicks, check TTL, swap cached HTML on hit without HTMX request, and update active pill classes.
- [x] Task 4.3: Update filter pills in `index.html` to invoke `handleFilterClick(this, event)`.
- [x] Task 4.4: In `htmx:afterSwap`, store successful `/hx/entries` responses in `filterCache`.
- [x] Task 4.5: Connect `invalidateFilterCache()` to mark-page-read, detail panel mutations, and guest IndexedDB read/bookmark actions.
- [x] Task 4.6: Add unit test in `crates/ui-components/src/templates.rs` verifying filter cache script functions in `IndexTemplate`.

### Phase 5: Verification & Regressions Check
- [x] Task 5.1: Run `cargo test -p ui-components` to ensure all template rendering tests pass.
- [x] Task 5.2: Verify no other template or build regressions exist.
