# Plan — List UI: Empty State Refresh, Light Theme Filter Active State, Pagination Brief, Mobile Search Toggle

**Result:** done

## Implementation Steps

1. Update [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html) to add `#empty-refresh-btn` inside `{% if entries.is_empty() %}`.
2. Update [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) to style `#mobile-search-toggle-btn` and collapsible `#search-form` above the filter pills under `@container (max-width: 768px)`.
3. Update [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html):
   - Add `#filter-search-bar` and `#mobile-search-toggle-btn` + `toggleMobileSearch()`.
   - Fix `setActiveFilter(btn)` to remove/add `hover:bg-border` consistently.
   - Add `refreshCurrentEntriesList()` (before `renderLocalAllArticles()`) and update `syncDomReadStates()` to hide `nav[aria-label="Pagination"]` and include `#empty-refresh-btn` in `#client-empty-state`.
   - Hide `#list-executive-brief` / `#page-executive-brief` and abort `activeBriefXhr` on pagination requests.
4. Add/update unit tests in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) covering all 4 behaviors.

## Planned Test Cases (TDD)

1. `test_empty_entry_list_hides_pagination_and_renders_refresh_button`: Verify `EntryListTemplate` with empty `entries` omits `aria-label="Pagination"` and renders `#empty-refresh-btn` calling `refreshCurrentEntriesList()`.
2. `test_set_active_filter_manages_hover_bg_border`: Verify `setActiveFilter` in `index.html` removes `hover:bg-border` from the active pill and adds it to inactive pills.
3. `test_sync_dom_read_states_hides_pagination_and_renders_refresh_button`: Verify `syncDomReadStates` hides `nav[aria-label="Pagination"]` when `#client-empty-state` is shown and renders `#empty-refresh-btn`.
4. `test_pagination_hides_previous_brief_and_aborts_stale_brief_xhr`: Verify pagination `htmx:beforeRequest` hides `#list-executive-brief` / `#page-executive-brief` and aborts `activeBriefXhr`.
5. `test_mobile_search_toggle_markup_and_container_css`: Verify `#mobile-search-toggle-btn`, `toggleMobileSearch()`, and `@container (max-width: 768px)` rules in `base.html` place `#search-form` above the filter pills when `data-mobile-search-open="true"`.
