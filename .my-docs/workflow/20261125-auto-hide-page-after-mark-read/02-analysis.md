# Analysis

**Result:** done

## Context

When viewing the **Unread** filter (`#active-status.value === 'unread'`) and clicking **Mark This Page as Read** (`#mark-page-read-btn`), Guest mode (`auth_enabled = false`) and Auth mode (`auth_enabled = true`) behave differently:

- **Guest mode (`auth_enabled = false`) — broken next-page loading:**
  - In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L57-L65), `#mark-page-read-btn` calls `markVisiblePageLocallyRead()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1694-L1711)).
  - `getVisibleEntryIds()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L560-L570)) returns a comma-separated string (`ids.join(',')`), while `markVisiblePageLocallyRead()` checks `Array.isArray(ids)`. Because that check is always false, it falls back to `document.querySelectorAll('.annuaire-row')`, including rows hidden by `display: none`.
  - Calling `markLocallyRead(id)` in a loop triggers `syncDomReadStates()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1537-L1642)) on every item, immediately hiding all rows and rendering `#client-empty-state` (`"All caught up!"`) without ever calling `showEntriesLoadingState()` or requesting the next page via `#pagination-next-btn` (`GET /hx/entries?status=unread&offset=...`).
  - On the server side, Guest mode `unread` queries `publishedAfter: start_of_today` without `status: UNREAD` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L124-L136)), so next-page pagination relies on `#pagination-next-btn` ([entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L214-L223)) and client-side `readSet` / IndexedDB filtering (`syncDomReadStates()`).

- **Auth mode (`auth_enabled = true`) — working base flow with polish gaps:**
  - `#mark-page-read-btn` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L42-L55)) sends `POST /hx/mark-page-read` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1109-L1250)), which marks items read in Miniflux, invalidates server cache, and returns the next unread page (`offset = 0`) + OOB unread counts + OOB toast.
  - `htmx:beforeRequest` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L510-L515)) calls `closeDetailPanel()` and `showEntriesLoadingState()`, and `htmx:afterSwap` / `htmx:afterRequest` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L997-L999), [L1131-L1135](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1131-L1135)) calls `hideEntriesLoadingState()`.
  - Gaps in Auth mode:
    1. **Scroll position not reset:** `#entries-list.scrollTop` is never reset to `0` after swap (`htmx:afterSwap` or `applyCachedList`), leaving the new page scrolled down.
    2. **No double-submit protection:** `#mark-page-read-btn` lacks `hx-disabled-elt="this"` (and no empty-page guard when 0 visible entries exist).
    3. **Stale client `filterCache`:** `htmx:afterSwap` overwrites `filterCache.get('unread')` without clearing cached `'today'`, `'all'`, or `'starred'` HTML via `invalidateFilterCache()`.
    4. **Missing `#active-offset` include:** `hx-include` only sends `#active-status, [name='search']`, omitting `#active-offset` ([entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L1)) used on non-`unread` pages.
    5. **Optional backend AI gap:** `mark_page_read_htmx` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1225-L1233)) passes `ai_service = None`, unlike `list_entries_htmx` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L637-L649)).

## Risks

- **HTMX 1.9.12 detached-source cleanup:** If `htmx.ajax()` in Guest mode uses `#pagination-next-btn` (which lives inside `#entries-list`) as `source`, swapping `#entries-list` with `innerHTML` detaches `#pagination-next-btn` before `htmx:afterRequest` fires, risking missed cleanup or stuck `.htmx-request` indicator classes. Using `#mark-page-read-btn` (in `<header>`, outside `#entries-list`) as `source` and clearing `.htmx-request` on `#entries-loading-bar` inside `hideEntriesLoadingState()` prevents stuck loading states.
- **Last page vs. pre-read next page in Guest mode:** When no `#pagination-next-btn` exists (`has_next = false`), triggering a loading state with no HTTP request would hang the spinner; when Page 2 exists but all its items are already in `readSet`, a single fetch reveals `"All caught up!"` unless auto-skipping is desired.
- **Error recovery in Guest mode:** If `readSet` / IndexedDB commits before `GET /hx/entries` finishes and the request fails, restoring `#entries-list` visibility shows `"All caught up!"` behind an error toast even though unread pages remain.
- **Existing unit test contract:** `test_paging_loading_overlay_and_read_blur_removed` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3868-L3880)) asserts that `markVisiblePageLocallyRead()` calls `closeDetailPanel()` and `showToast('Page marked as read.')`.

## Open questions

Resolved into Round 1 frontier in `02b-grill.md`:
1. Guest mode behavior when the next page is already fully read in `readSet` (single fetch vs. auto-skip).
2. Guest mode rollback/deferral when the next-page request fails.
3. Scope of Auth mode polish: UI/HTMX-only (`Has API: no`) vs. also wiring `ai_service` in `mark_page_read_htmx` (`Has API: yes`).
4. Scope across non-`Unread` filters (`Today`, `All`) and active `#search-input` filtering in Guest mode.
