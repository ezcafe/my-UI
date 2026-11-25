# Ideation — Paging Loading, Mark-Page-Read Toast, and Lighter Read Blur

- **Slug:** `20261125-paging-loading-toast-blur`
- **Has UI (tentative):** `yes`
- **Size:** Small UI/HTMX change

## Shared Understanding (Playback)

Based on the current codebase (`crates/ui-components/templates/base.html`, `crates/ui-components/templates/pages/index.html`, `crates/ui-components/templates/partials/entry_list.html`, and `crates/server/src/routes/entries.rs`):

1. **Hide current page & show loading indicator on paging / mark-page-read:**
   - Currently, `#entries-loading-bar` is a 2px top bar (`h-0.5`), `#mark-page-read-btn` and the `Previous`/`Next` pagination buttons in `entry_list.html` lack `hx-indicator`, and `#entries-list` stays visible while the next/previous page loads.
   - **Goal:** When clicking `Previous`, `Next`, or `Mark This Page as Read`, immediately hide the current page content in `#entries-list`, display a visible loading indicator, and swap in the new page data once loaded.
2. **Success message on Mark Page as Read:**
   - Currently, `mark_page_read_htmx` in `crates/server/src/routes/entries.rs` only returns an OOB `ToastTemplate` when `show_last_page_toast` is true (`"Page marked as read. You are on the last page."`), and returns no toast in `unread` mode or when more pages remain.
   - **Goal:** Always show a success toast when marking the page as read succeeds (e.g. `"Page marked as read."`, or `"Page marked as read. You are on the last page."` on the last page), including in guest/local mode if applicable.
3. **Less blur on read items:**
   - Currently in `crates/ui-components/templates/base.html`, `.ar-list-item.read` uses `opacity: 0.55; filter: grayscale(80%) blur(0.2px);` and `.magazine-card.read` uses `opacity: 0.45; filter: grayscale(85%) blur(0.25px);`.
   - **Goal:** Reduce the blur effect on read items so titles and summaries remain easier to scan while still clearly looking "read".
4. **Hide details page on Next / Previous / Mark Page as Read:**
   - Currently, `#detail-panel` (the slide-over details pane in `crates/ui-components/templates/pages/index.html`) is closed via `closeDetailPanel()` when clicking filter pills (`handleFilterClick` / `setActiveFilter`) or submitting search (`onSearchSubmit`), which hides `#detail-panel`, resets it to `#detail-empty-placeholder`, and clears `?entry=` / `/entries/<id>` from the URL.
   - However, clicking `Previous` or `Next` in `crates/ui-components/templates/partials/entry_list.html` or `#mark-page-read-btn` in `crates/ui-components/templates/pages/index.html` (both HTMX `hx-post="/hx/mark-page-read"` and guest `markVisiblePageLocallyRead()`) leaves `#detail-panel` open if an article is currently displayed.
   - **Goal:** Also call `closeDetailPanel()` whenever the user clicks `Previous`, `Next`, or `Mark This Page as Read` so the details page is hidden and cleared.

## Non-Goals / Constraints

- No changes to pagination math, Miniflux GraphQL queries, or database schema.
- Preserve existing keyboard navigation, AI/List view toggle, and filter-pill caching behavior.
- Use existing HTMX (`hx-indicator`, `htmx:beforeRequest` / `htmx:afterSwap` / `htmx:responseError`), `closeDetailPanel()`, and `ToastTemplate` patterns.

## Open Question for User

*(Reply to confirm or say **"just start"** to skip ideation and proceed with defaults.)*

Does this updated summary (including hiding `#detail-panel` via `closeDetailPanel()` on `Previous`, `Next`, and `Mark This Page as Read`) match what you want, and for the hidden-page loading state and read-item blur, what feels right to you (for example, a centered spinner vs. skeleton rows while loading, and a subtle `0.1px` blur vs. `0px` blur with grayscale/opacity)?
