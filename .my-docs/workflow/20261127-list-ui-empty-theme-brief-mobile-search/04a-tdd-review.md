# TDD Plan Review — List UI: Empty State Refresh, Light Theme Filter Active State, Pagination Brief, Mobile Search Toggle

**Result:** clean

## Reviewed Planned Test Cases (`04-plan.md`)

1. **`test_empty_entry_list_hides_pagination_and_renders_refresh_button`** (`crates/ui-components/src/templates.rs`)
   - **Coverage:** Verifies `EntryListTemplate` with empty `entries` omits `aria-label="Pagination"` (even when `total_pages > 1`) and renders `#empty-refresh-btn` wired to `refreshCurrentEntriesList()`.
   - **Verdict:** Clear, isolated Askama render assertion.

2. **`test_set_active_filter_manages_hover_bg_border`** (`crates/ui-components/src/templates.rs`)
   - **Coverage:** Verifies `setActiveFilter(btn)` in `index.html` removes `hover:bg-border` from the active `.filter-pill` and adds `hover:bg-border` to inactive `.filter-pill` elements, matching `Unread`'s active classes in light theme.
   - **Verdict:** Clear assertion on rendered `IndexTemplate` script.

3. **`test_sync_dom_read_states_hides_pagination_and_renders_refresh_button`** (`crates/ui-components/src/templates.rs`)
   - **Coverage:** Verifies `syncDomReadStates()` hides `#entries-list nav[aria-label="Pagination"]` when `#client-empty-state` is shown, restores it when rows are visible, and renders `#empty-refresh-btn` calling `refreshCurrentEntriesList()`. Also confirms `refreshCurrentEntriesList()` is placed before `renderLocalAllArticles()` so existing slice-based tests remain intact.
   - **Verdict:** Clear assertion covering both the client-side empty state and pagination restoration.

4. **`test_pagination_hides_previous_brief_and_aborts_stale_brief_xhr`** (`crates/ui-components/src/templates.rs`)
   - **Coverage:** Verifies `activeBriefXhr` tracking for `page-executive-brief` across `htmx:beforeRequest` / `htmx:afterRequest`, and that `pagination-prev-btn` / `pagination-next-btn` `htmx:beforeRequest` aborts any in-flight `activeBriefXhr` and hides `#list-executive-brief` and `#page-executive-brief`.
   - **Verdict:** Clear assertion covering both brief containers and stale XHR abort.

5. **`test_mobile_search_toggle_markup_and_container_css`** (`crates/ui-components/src/templates.rs`)
   - **Coverage:** Verifies `#mobile-search-toggle-btn`, `toggleMobileSearch()`, desktop `display: none`, and `@container (max-width: 768px)` rules in `base.html` that collapse `#search-form` by default and display it above the filter pills (`order: -1; flex-basis: 100%`) when `#filter-search-bar[data-mobile-search-open="true"]`.
   - **Verdict:** Clear assertion covering both markup/JS and container-query CSS.

## Notes

- All 5 planned tests map directly to the 4 UI fixes in `03-design.md` and follow existing Askama template assertion patterns in `crates/ui-components/src/templates.rs`.
- No additional test cases needed before Gate B.
