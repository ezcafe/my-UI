# Design — List UI: Empty State Refresh, Light Theme Filter Active State, Pagination Brief, Mobile Search Toggle

**Result:** done

## Design Options

### Option A: Minimal Targeted DOM + CSS Container-Query Updates (Recommended)
Keep all existing HTMX endpoints, Askama templates, and `.spatial-shell` container queries intact; apply targeted fixes directly in `entry_list.html`, `index.html`, and `base.html`.

- **Explanation:**
  1. **Empty unread state:** Add a Refresh button (`#empty-refresh-btn`) in `entry_list.html` (`entries.is_empty()`) and in `syncDomReadStates()` (`#client-empty-state`), wired to `refreshCurrentEntriesList()` which clears `filterCache` for the active status and calls `htmx.ajax('GET', '/hx/entries?...', '#entries-list')`. In `syncDomReadStates()`, hide `#entries-list nav[aria-label="Pagination"]` when `#client-empty-state` is active and restore it otherwise.
  2. **Light theme active state:** Update `setActiveFilter(btn)` in `index.html` to remove `hover:bg-border` from the active button and add `hover:bg-border` to inactive `.filter-pill` buttons so active `Today`, `Starred`, and `All` share the exact same classes as active `Unread`.
  3. **Pagination Executive Brief:** Track `activeBriefXhr` on `/hx/executive-brief` requests (via `htmx:beforeRequest` / `htmx:afterRequest` / `htmx:afterSwap`), abort any in-flight brief request when pagination starts, and immediately hide `#list-executive-brief` and `#page-executive-brief` when `#pagination-prev-btn` or `#pagination-next-btn` is triggered so the previous page's brief is never visible while loading/showing the new page's brief.
  4. **Mobile search toggle:** Add `#mobile-search-toggle-btn` inside the filter bar in `index.html`. Under `@container (max-width: 768px)` in `base.html`, use `flex-wrap: wrap` on `#filter-search-bar`: hide `#search-form` by default, show `#mobile-search-toggle-btn` on the right of the filter pills, and when `#filter-search-bar[data-mobile-search-open="true"]` is set, display `#search-form` with `order: -1; flex-basis: 100%;` above the filter buttons row and focus `#search-input`.
- **Pros:** Minimal diff, zero backend changes, preserves all existing `templates.rs` invariants and HTMX flows, uses existing `@container` architecture.
- **Cons:** Relies on `htmx:beforeRequest` / `htmx:afterSwap` coordination in `index.html`.

### Option B: Separate Mobile Search Component + Server-Side Empty/Brief State Flags
Split the filter bar into two separate DOM containers (desktop vs mobile) and add server-side query flags to suppress/toggle briefs on pagination.
- **Explanation:** Render duplicate `#search-form` elements for mobile and desktop, and modify `EntriesFilterQuery` in Rust to track pagination brief state.
- **Pros:** Decouples mobile and desktop search markup.
- **Cons:** Duplicates form IDs (`#search-form`, `#search-input`) which breaks `hx-include="[name='search']"` and `document.getElementById('search-input')`, and unnecessarily touches Rust route handlers.

**Recommendation:** **Option A** — it keeps a single `#search-form` / `#search-input` (preserving all HTMX `hx-include` and JS listeners) and solves all four UI issues cleanly in templates and CSS container queries.

## Detailed Component Changes

1. **[crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)**
   - Inside `{% if entries.is_empty() %}`, below `<p class="text-xs text-muted-foreground mt-1 font-sans">No articles match your current filter.</p>`, add:
     ```html
     <button
       id="empty-refresh-btn"
       type="button"
       onclick="refreshCurrentEntriesList()"
       class="mt-4 inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-full border border-border bg-card hover:bg-muted text-foreground text-xs font-bold font-sans transition-colors cursor-pointer">
       <i data-lucide="refresh-cw" class="w-3.5 h-3.5"></i>
       <span>Refresh</span>
     </button>
     ```

2. **[crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)**
   - Give the filter bar `#filter-search-bar` (`id="filter-search-bar"`), and give the filter pills wrapper `.filter-pills-row` (`flex-1 min-w-0`).
   - Add `#mobile-search-toggle-btn` (`type="button"`, `onclick="toggleMobileSearch()"`, `aria-expanded="false"`, `aria-controls="search-form"`, `title="Search articles"`) with `<i data-lucide="search" class="w-4 h-4"></i>`.
   - Add `toggleMobileSearch()` (toggles `data-mobile-search-open="true"` on `#filter-search-bar`, updates `aria-expanded`, and focuses `#search-input` when opened).
   - Add `refreshCurrentEntriesList()` (placed *before* `renderLocalAllArticles()`, never between `renderLocalAllArticles()` and `syncDomReadStates()`): reads `#active-status` and `#search-input`, deletes `filterCache` for that status, calls `showEntriesLoadingState()`, and triggers `htmx.ajax('GET', url, { target: '#entries-list', swap: 'innerHTML' })`.
   - In `setActiveFilter(btn)`:
     - Inactive `.filter-pill` elements: `el.classList.remove('bg-primary', 'text-primary-foreground', 'font-bold', 'border-primary'); el.classList.add('bg-muted', 'text-muted-foreground', 'border-border', 'hover:bg-border', 'font-medium');`
     - Active `btn`: `btn.classList.remove('bg-muted', 'text-muted-foreground', 'border-border', 'hover:bg-border', 'font-medium'); btn.classList.add('bg-primary', 'text-primary-foreground', 'font-bold', 'border-primary');`
   - In `syncDomReadStates()`:
     - When `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0`: hide `#entries-list nav[aria-label="Pagination"]` (`style.display = 'none'`), and include `#empty-refresh-btn` (`onclick="refreshCurrentEntriesList()"`) inside `#client-empty-state`.
     - In the `else` branch: restore `#entries-list nav[aria-label="Pagination"]` (`style.display = ''`).
   - Pagination brief hiding:
     - Track `activeBriefXhr` in `htmx:beforeRequest` when `target.id === 'page-executive-brief'` (and clear in `htmx:afterRequest`).
     - When `trigger.id === 'pagination-prev-btn' || trigger.id === 'pagination-next-btn'`, abort `activeBriefXhr` if present, and hide `#list-executive-brief` and `#page-executive-brief` immediately (`classList.add('hidden')`) before `showEntriesLoadingState()`.

3. **[crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)**
   - Outside `@container (max-width: 768px)`: `#mobile-search-toggle-btn { display: none; }`.
   - Inside `@container (max-width: 768px)`:
     - `#filter-search-bar { flex-wrap: wrap; gap: 0.5rem; }`
     - `#mobile-search-toggle-btn { display: inline-flex; }`
     - `#search-form { display: none; order: -1; flex-basis: 100%; width: 100%; }`
     - `#search-input { width: 100%; }` (remove old `5.5rem` / `11rem` focus width rules)
     - `#filter-search-bar[data-mobile-search-open="true"] #search-form { display: flex; }`
