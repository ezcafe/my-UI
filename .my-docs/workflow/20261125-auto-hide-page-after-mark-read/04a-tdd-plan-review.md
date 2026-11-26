Result: clean

# TDD Plan Review — Auto-Hide Page, Show Loading, and Load Next Page After Mark Page as Read

- **Stage:** `tdd-review`
- **Design & Tasks Reviewed:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/03-design.md) (`Result: done`), [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/04-tasks.md) (5 planned tests)
- **Test Harness:** Rust / Askama template rendering tests in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) (`cargo test -p ui-components` and `cargo test -p server`)

---

## Key Takeaway

The 5 planned tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/04-tasks.md) cover every behavioral requirement, regression guard, and edge case in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/03-design.md) with zero missing test cases. Ready for Gate B before Red-Green-Verify execution.

---

## Planned Test Coverage Audit

| # | Planned Test | Design Requirements Covered | Gaps / Build Notes |
|---|---|---|---|
| 1 | `test_auth_mark_page_read_button_attributes_and_empty_guard` | Auth `#mark-page-read-btn` `hx-disabled-elt="this"` and `hx-include="#active-status, #active-offset, [name='search']"` (Q3=A); `htmx:beforeRequest` empty `getVisibleEntryIds()` guard before `showEntriesLoadingState()` with `disabled`/`.htmx-request` reset; `cancelPendingMarkPageRead()` `typeof cancelPendingGuestMarkRead === 'function'` guard (zero `ReferenceError` in Auth mode). | None. Verify both empty string (`""`) and empty-array edge handling in `htmx:beforeRequest` guard. |
| 2 | `test_entries_scroll_reset_and_filter_cache_invalidation_on_mark_read` | `htmx:afterSwap` resets `entriesListEl.scrollTop = 0` and calls `invalidateFilterCache()` on `#mark-page-read-btn` swaps before `filterCache.set()`; `applyCachedList()` and `renderLocalAllArticles()` reset `scrollTop = 0`; `hideEntriesLoadingState()` removes `.htmx-request` from `#entries-loading-bar`; `htmx:afterRequest` clears `disabled`/`.htmx-request` on `#mark-page-read-btn`. | **Build note:** `renderLocalAllArticles()` is only rendered inside `{% if !auth_enabled %}` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1184)), so assert `renderLocalAllArticles` on a Guest (`auth_enabled: false`) `IndexTemplate` render inside this test. |
| 3 | `test_guest_mark_page_read_htmx_next_page_and_deferred_commit` | Guest `#mark-page-read-btn` `hx-indicator="#entries-loading-bar"` + `hx-disabled-elt="this"`; D-01 helper placement after `syncDomReadStates()` and before `markLocallyRead()`; D-02 `persistLocallyReadIds()` `!readSet.has(id)` deduplication + single `readwrite` IndexedDB tx; `parseVisibleEntryIds()` string/array parsing; `htmx.ajax('GET', nextUrl, { target: '#entries-list', swap: 'innerHTML', source: markBtn })` + `pendingGuestMarkRead === token` `.catch()` guard; `htmx:beforeSwap` commit gates (`requestConfig.elt.id === 'mark-page-read-btn'`, `!isError`, `shouldSwap !== false`, `!cancelled`, `matchesXhr`); M-01 `cancelPendingMarkPageRead()` calls in `handleFilterClick()`, `onSearchSubmit()`, and `htmx:beforeRequest`; existing `closeDetailPanel()` + `showToast('Page marked as read.')` contract before `showToast`. | None. Preserves slice assertions in `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` and `test_paging_loading_overlay_and_read_blur_removed`. |
| 4 | `test_guest_sync_dom_read_states_hides_magazine_on_empty_unread` | `syncDomReadStates()` hides `#entries-magazine-view` via `style.display = 'none'` when `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0` and restores `''` otherwise. | **Build note:** Ensure `magView.style.display = ''` runs in the `else` branch even if `#client-empty-state` (`emptyEl`) was not previously present. |
| 5 | `test_guest_filter_articles_client_side_keeps_read_hidden_on_unread` | `filterArticlesClientSide()` checks `#active-status.value === 'unread'` and keeps `.read` rows `display: none` (excluded from `visibleCount`); `persistLocallyReadIds(..., { syncDom: true })` re-runs `filterArticlesClientSide(searchQuery)` when `!isSearchActive && searchQuery`. | None. Covers M-02 search re-filter safety on `Unread` vs `Today`/`All`. |

---

## Grounding & Verification Sources

- HTMX `htmx.ajax()` (`source`, `target`, `swap`), `hx-disabled-elt`, `hx-include`, and lifecycle events (`htmx:beforeRequest`, `htmx:beforeSwap`, `htmx:afterSwap`, `htmx:afterRequest`, `htmx:abort`): <https://htmx.org/api/#ajax>, <https://htmx.org/events/>, <https://htmx.org/attributes/hx-disabled-elt/>, <https://htmx.org/attributes/hx-include/>
- Codebase test suite & template slices: [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3658-L3881), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L409-L1760), [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L1-L242)
