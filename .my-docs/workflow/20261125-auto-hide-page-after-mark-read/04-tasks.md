# Tasks — Auto-Hide Page, Show Loading, and Load Next Page After Mark Page as Read

## Scope

- **Files modified:**
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) (Auth & Guest `#mark-page-read-btn`, shared cancellation, loading/scroll/cache lifecycle, read-aware `filterArticlesClientSide()`, deferred Guest `htmx.ajax` + `beforeSwap` commit, deduplicated batched IndexedDB persistence, `syncDomReadStates()` Magazine empty-state hiding)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) (template unit tests covering rendered HTML/JS contracts)
- **Out of scope:** Backend route changes (`Has API: no`, `Has DB: no`), clearing `#search-input` on Guest next-page swap, hiding `#list-executive-brief` in List view above `#client-empty-state`.

## Ordered TDD Tasks

1. **Red — Add failing template unit tests in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs):**
   - Assert Auth `#mark-page-read-btn` renders `hx-disabled-elt="this"` and `hx-include="#active-status, #active-offset, [name='search']"`, and `htmx:beforeRequest` prevents empty `getVisibleEntryIds()` before `showEntriesLoadingState()` while resetting `disabled` / `.htmx-request`.
   - Assert `htmx:afterSwap` resets `scrollTop = 0` and invalidates `filterCache` on `#mark-page-read-btn` swaps; `applyCachedList` and `renderLocalAllArticles` reset `scrollTop = 0`; `hideEntriesLoadingState()` clears `.htmx-request` on `#entries-loading-bar`; `htmx:afterRequest` resets `markBtn.disabled = false` and `.htmx-request`.
   - Assert shared `cancelPendingMarkPageRead()` safely guards `typeof cancelPendingGuestMarkRead === 'function'` (zero `ReferenceError` in Auth mode) and is invoked inside `handleFilterClick()`, `onSearchSubmit()`, and `htmx:beforeRequest` (covering `All`, cached `filterCache` tabs, and non-`mark-page-read-btn` `#entries-list` triggers).
   - Assert Guest script defines `persistLocallyReadIds` (deduplicating `!readSet.has(id)` and placed after `syncDomReadStates()`, before `markLocallyRead()`), parses `getVisibleEntryIds()` string/array safely, triggers `htmx.ajax('GET', nextUrl, ...)` with `source: markBtn` when `#pagination-next-btn` exists, defers commit via `htmx:beforeSwap` scoped to `requestConfig.elt.id === 'mark-page-read-btn'` + `!isError` + `shouldSwap !== false` + matching XHR, guards `.catch()` `hideEntriesLoadingState()` with `pendingGuestMarkRead === token`, aborts in-flight mark-read cleanly in `cancelPendingGuestMarkRead()`, hides `#entries-magazine-view` when Unread visible count reaches `0`, and keeps `closeDetailPanel()` + `showToast('Page marked as read.')` inside `markVisiblePageLocallyRead()`.
   - Assert `filterArticlesClientSide()` keeps `.read` rows `display: none` when `#active-status.value === 'unread'`, and `persistLocallyReadIds(..., { syncDom: true })` re-runs `filterArticlesClientSide(searchQuery)` when `!isSearchActive && searchQuery`.
   - Run `cargo test -p ui-components` to confirm new tests fail before implementation.

2. **Green — Update [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html):**
   - Update Auth `#mark-page-read-btn` (`hx-disabled-elt="this"`, `hx-include="#active-status, #active-offset, [name='search']"`).
   - Update Guest `#mark-page-read-btn` (`hx-indicator="#entries-loading-bar"`, `hx-disabled-elt="this"`).
   - Update `hideEntriesLoadingState()` to also clear `.htmx-request` on `#entries-loading-bar`.
   - Update `applyCachedList()`, `renderLocalAllArticles()`, and `htmx:afterSwap` (`isEntriesSwap`) to reset `scrollTop = 0`.
   - Add shared `cancelPendingMarkPageRead()` (using `typeof cancelPendingGuestMarkRead === 'function'` guard for Guest and `htmx:abort` + `disabled = false` + `.htmx-request` removal + `hideEntriesLoadingState()` for Auth) and invoke it in `handleFilterClick()`, `onSearchSubmit()`, and `htmx:beforeRequest` (for `.filter-pill` intercepts and non-`mark-page-read-btn` `#entries-list` requests).
   - Update `htmx:beforeRequest` to prevent `#mark-page-read-btn` when `getVisibleEntryIds()` is empty (before `showEntriesLoadingState()`, resetting `disabled` / `.htmx-request`), and update `htmx:afterRequest` to explicitly clear `disabled` / `.htmx-request` on `#mark-page-read-btn`.
   - Update `htmx:afterSwap` to call `invalidateFilterCache()` on `#mark-page-read-btn` swaps before re-caching the active status HTML.
   - Update `filterArticlesClientSide()` so `.read` items stay `style.display = 'none'` (and excluded from `visibleCount`) when `#active-status.value === 'unread'`.
   - In the Guest `<script>` block **after `syncDomReadStates()` and before `markLocallyRead()`** (preserving the `renderLocalAllArticles() .. syncDomReadStates()` slice boundary): add `pendingGuestMarkRead`, `parseVisibleEntryIds()`, `persistLocallyReadIds()` (deduplicating `!readSet.has(id)` and re-running `filterArticlesClientSide(searchQuery)` when `syncDom && !isSearchActive && searchQuery`), `cancelPendingGuestMarkRead()`, and Guest `htmx:beforeRequest` / `htmx:beforeSwap` (`requestConfig?.elt?.id === 'mark-page-read-btn'`, `!isError`, `shouldSwap !== false`, `!cancelled`, `matchesXhr`) / `htmx:afterRequest` listeners; delegate `markLocallyRead()` to `persistLocallyReadIds()`; rewrite `markVisiblePageLocallyRead()` (kept right before `showToast()`) with `pendingGuestMarkRead === token` `.then()`/`.catch()` guards.
   - Update `syncDomReadStates()` to hide `#entries-magazine-view` via `style.display = 'none'` when `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0` (and restore `''` otherwise).

3. **Verify — Run test suite:**
   - Run `cargo test -p ui-components` and `cargo test -p server` to confirm all existing and new tests pass cleanly.

---

## Planned tests

1. **`test_auth_mark_page_read_button_attributes_and_empty_guard`** — Verifies Auth `IndexTemplate` renders `#mark-page-read-btn` with `hx-disabled-elt="this"` and `hx-include="#active-status, #active-offset, [name='search']"`, `htmx:beforeRequest` cancels empty `getVisibleEntryIds()` before `showEntriesLoadingState()`, and `cancelPendingMarkPageRead()` safely checks `typeof cancelPendingGuestMarkRead === 'function'` without referencing undeclared Guest symbols in Auth mode.
2. **`test_entries_scroll_reset_and_filter_cache_invalidation_on_mark_read`** — Verifies `htmx:afterSwap` resets `entriesListEl.scrollTop = 0` and invalidates `filterCache` on `#mark-page-read-btn` swaps, `applyCachedList` and `renderLocalAllArticles` reset `scrollTop = 0`, `hideEntriesLoadingState()` clears `#entries-loading-bar` `.htmx-request`, and `htmx:afterRequest` resets `disabled` / `.htmx-request` on `#mark-page-read-btn`.
3. **`test_guest_mark_page_read_htmx_next_page_and_deferred_commit`** — Verifies Guest `IndexTemplate` renders `#mark-page-read-btn` with `hx-indicator="#entries-loading-bar"` and `hx-disabled-elt="this"`, helper placement stays outside `renderLocalAllArticles() .. syncDomReadStates()`, `persistLocallyReadIds()` filters `!readSet.has(id)` in one `readwrite` IndexedDB tx, `markVisiblePageLocallyRead()` parses string/array IDs, uses `htmx.ajax('GET', nextUrl, ...)` with `source: markBtn` and `pendingGuestMarkRead === token` `.catch()` guard, `htmx:beforeSwap` commits only on `requestConfig.elt.id === 'mark-page-read-btn'` + `!isError` + `shouldSwap !== false`, `handleFilterClick()` / `onSearchSubmit()` / `htmx:beforeRequest` call `cancelPendingMarkPageRead()`, and `markVisiblePageLocallyRead()` preserves `closeDetailPanel()` + `showToast('Page marked as read.')`.
4. **`test_guest_sync_dom_read_states_hides_magazine_on_empty_unread`** — Verifies Guest `syncDomReadStates()` hides `#entries-magazine-view` when `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0` so Magazine view displays `"All caught up!"` cleanly on the last Unread page.
5. **`test_guest_filter_articles_client_side_keeps_read_hidden_on_unread`** — Verifies `filterArticlesClientSide()` checks `#active-status` (`'unread'`) and keeps `.read` rows `display: none`, and `persistLocallyReadIds(..., { syncDom: true })` re-applies `filterArticlesClientSide(searchQuery)` when `!isSearchActive && searchQuery`.
