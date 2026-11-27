# Analysis — List UI: Empty State Refresh, Light Theme Filter Active State, Pagination Brief, Mobile Search Toggle

**Result:** done

## Scope & Affected Files

1. **[crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)**
   - **Filter pills & `setActiveFilter(btn)` (lines 84–143, 706–721):**
     - `#filter-unread` has `bg-primary text-primary-foreground font-bold border-primary` without `hover:bg-border`.
     - `#filter-today`, `#filter-starred`, and `#filter-all` have `bg-muted text-muted-foreground hover:bg-border border border-border`.
     - `setActiveFilter(btn)` removes `bg-muted text-muted-foreground border-border font-medium` and adds `bg-primary text-primary-foreground font-bold border-primary`, **but leaves `hover:bg-border` on the button**. On light theme, `hover:bg-border` overrides `bg-primary` on hover/active focus, making `Today` and `All` look different from `Unread`.
   - **Client-side empty unread state in `syncDomReadStates()` (lines 2259–2287):**
     - When `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0`, `#entries-magazine-view` is hidden and `#client-empty-state` is appended, **but `#entries-list nav[aria-label="Pagination"]` remains visible**, and `#client-empty-state` has no Refresh button.
   - **Pagination Executive Brief (lines 698–703, 1368–1420):**
     - Clicking `#pagination-prev-btn` or `#pagination-next-btn` triggers `showEntriesLoadingState()`. If an earlier `/hx/executive-brief` request is in-flight or `#list-executive-brief` / `#page-executive-brief` is still visible before/during swap, the previous page's brief can linger or race.
   - **Mobile Search Bar (lines 84–168):**
     - `#search-form` is currently a sibling to the filter pills `div` inside a single `flex items-center justify-between` bar.

2. **[crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)**
   - **Server-rendered empty state (lines 2–7):**
     - `{% if entries.is_empty() %}` already omits `nav[aria-label="Pagination"]` (which is inside `{% else %}`).
     - Currently lacks a Refresh button below `"No articles match your current filter."`.

3. **[crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)**
   - **Mobile container query `@container (max-width: 768px)` (lines 323–352):**
     - Currently shrinks `#search-input` to `5.5rem` (expanding to `11rem` on `:focus`) inline next to the filter pills.
     - Needs to hide `#search-form` by default on mobile, show `#mobile-search-toggle-btn`, and when toggled open, place `#search-form` full-width **above** the filter buttons row.

4. **[crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)**
   - Existing unit tests assert specific DOM IDs and JS function boundaries (e.g., `D-01 slice boundary between renderLocalAllArticles() and syncDomReadStates()`). Any new JS helpers must not be placed between `renderLocalAllArticles()` and `syncDomReadStates()`.

## Constraints & Risks

- **No backend schema/API changes:** Refresh button reloads `/hx/entries` for the current `status` and `search` without calling `POST /hx/refresh`.
- **Container queries only:** Mobile layout uses the existing `@container (max-width: 768px)` on `.spatial-shell` in `base.html`, with no hardcoded `@media` breakpoints.
- **Test boundary preservation:** Keep `renderLocalAllArticles()` immediately before `syncDomReadStates()` in `index.html`.
