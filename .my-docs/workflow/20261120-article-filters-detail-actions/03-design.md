# Design Specification

Result: done

## Summary
This document specifies the technical design for five user interface and API updates:
1. **Hybrid search** in the article filter bar.
2. **Reordered filter pills** with `Unread` as the default first option.
3. **Mark This Page as Read** button replacing the global mark-all action.
4. **Author metadata removal** from the article detail view.
5. **Mark as Unread icon button** in the article detail header.

---

## Technical Design by Component

### 1. Hybrid Search Box
- **File**: `crates/ui-components/templates/pages/index.html`
- **Behavior**:
  - **Typing (`input` event)**: A lightweight JavaScript function `filterArticlesClientSide(query)` loops over all `.ar-list-item` nodes in `#entries-list`. It compares `query.toLowerCase()` against the article title text. Matching items remain visible (`style.display = ''`); non-matching items are hidden (`style.display = 'none'`). Empty query restores all items.
  - **Submission (`Enter` key or search button)**: An HTML form submits an HTMX request `GET /hx/entries` targeting `#entries-list` with `search=<query>` and `status=all`.
- **Backend support**:
  - In `crates/server/src/routes/entries.rs`, `build_graphql_filter`: When `search` is provided and `status` is not explicitly set, default to no status filter (all entries) instead of forcing `status: UNREAD`.

```html
<!-- Hybrid Search Form in index.html -->
<form 
  id="search-form" 
  hx-get="/hx/entries" 
  hx-target="#entries-list" 
  hx-swap="innerHTML"
  class="relative flex items-center"
  onsubmit="onSearchSubmit(event)"
>
  <input 
    type="search" 
    id="search-input"
    name="search" 
    placeholder="Filter articles..." 
    oninput="filterArticlesClientSide(this.value)"
    class="bg-white/5 border border-white/10 rounded-full pl-3.5 pr-8 py-1 text-xs text-slate-200 placeholder-slate-400 focus:outline-none focus:border-indigo-400/50 w-44"
  />
  <input type="hidden" name="status" value="all" />
  <button type="submit" class="absolute right-2.5 text-slate-400 hover:text-white" title="Search all articles">
    <i data-lucide="search" class="w-3.5 h-3.5"></i>
  </button>
</form>
```

---

### 2. Filter Bar Reordering
- **File**: `crates/ui-components/templates/pages/index.html`
- **New Order**:
  1. `Unread` (`#filter-unread`, active default with `bg-indigo-500/20 text-indigo-300 border-indigo-400/30`)
  2. `Today` (`#filter-today`, inactive default with `bg-white/5 text-slate-400 border-white/10`)
  3. `Starred` (`#filter-starred`, inactive default)
  4. `All titles` (`#filter-all`, inactive default)
- **Active Status input**: Stays `<input type="hidden" id="active-status" name="status" value="unread">`.

---

### 3. Mark This Page as Read
- **Options Evaluated**:

#### Option A: Dynamic JS parameter collection via HTMX `hx-vals` (Recommended)
- **Explanation**: The button uses `hx-vals="js:{entry_ids: getVisibleEntryIds()}"` to read visible entry IDs in `#entries-list` at click time and POSTs to `/hx/mark-page-read`.
- **Concrete Example**:
  ```html
  <button 
    id="mark-page-read-btn"
    type="button" 
    hx-post="/hx/mark-page-read" 
    hx-target="#entries-list" 
    hx-swap="innerHTML" 
    hx-include="#active-status, [name='search']"
    hx-vals="js:{entry_ids: getVisibleEntryIds()}"
    class="p-2 rounded-full bg-white/5 hover:bg-white/15 text-slate-300 transition-colors" 
    title="Mark This Page as Read">
    <i data-lucide="check-check" class="w-4 h-4"></i>
  </button>
  ```
- **Pros**:
  - Accurately respects client-side filtering: only entries currently visible (`display !== 'none'`) are sent.
  - Re-renders `#entries-list` with the current filter state and automatically updates `#unread-pill` out-of-band.
  - Zero disruption to existing templates.
- **Cons**: Requires a 6-line helper function in `index.html`.

#### Option B: Hidden checkboxes inside `#entries-list` with form submit
- **Explanation**: Place hidden `<input type="checkbox" name="entry_ids" checked>` in each list item row, and wrap the list in a form.
- **Pros**: Pure HTML form submission.
- **Cons**: Submits hidden items filtered out by client search unless JS synchronizes checkbox states; clutters list row template.

- **Recommendation**: **Option A** is cleaner, directly matches the user's requirement to mark *visible* entries, and reuses existing HTMX idioms.

- **Backend Route & Mutation**:
  - Route: `POST /hx/mark-page-read` in `crates/server/src/routes/entries.rs` and `crates/server/src/app.rs`.
  - Mutation: Add `update_entries_status(entry_ids: Vec<ID>, status: EntryStatus)` to `Mutation` in `crates/miniflux/src/schema/mutation.rs`.
  - Handler logic:
    1. Parse comma-separated or JSON list of IDs from `entry_ids`.
    2. If IDs are present, invoke `schema.execute` with `updateEntriesStatus(entryIds: [...], status: READ)`.
    3. Re-query entries for current `status` and `search` query.
    4. Render and return `EntryListTemplate { entries, unread_count }`.

---

### 4. Detail Metadata Display Cleanup
- **File**: `crates/ui-components/templates/partials/entry_detail.html`
- **Change**: Remove the author block:
  ```html
  <!-- Before -->
  <div class="flex items-center gap-2 text-xs text-slate-400 mb-2">
    <span class="text-indigo-400 font-medium">{{ entry.feed_title }}</span>
    <span>•</span>
    <span>{{ entry.published_at }}</span>
    {% if let Some(author) = entry.author %}
    <span>•</span>
    <span>By {{ author }}</span>
    {% endif %}
  </div>

  <!-- After -->
  <div class="flex items-center gap-2 text-xs text-slate-400 mb-2">
    <span class="text-indigo-400 font-medium">{{ entry.feed_title }}</span>
    <span>•</span>
    <span>{{ entry.published_at }}</span>
  </div>
  ```

---

### 5. Detail Action: "Mark as Unread" Button
- **File**: `crates/ui-components/templates/partials/entry_detail.html`
- **Location**: In header action group, placed immediately before the Bookmark button.
- **Markup**:
  ```html
  <button 
    hx-post="/hx/entries/{{ entry.id }}/read?status=unread" 
    hx-target="#entry-{{ entry.id }}" 
    hx-swap="outerHTML" 
    class="p-2 rounded-full hover:bg-white/10 text-slate-400 hover:text-indigo-300 transition-colors"
    title="Mark as unread">
    <i data-lucide="circle-dot" class="w-4 h-4"></i>
  </button>
  ```
- **Behavior**:
  - Calls existing `POST /hx/entries/{id}/read?status=unread`.
  - Replaces `#entry-{{ entry.id }}` in the list with the updated entry row containing the unread indicator dot (`<span class="w-2 h-2 rounded-full bg-indigo-400 ...">`).
  - Updates `#unread-pill` via out-of-band swap (`hx-swap-oob="true"`).

---

## Contract & API Summary
| Route | Method | Payload / Params | Response | Target |
|---|---|---|---|---|
| `/hx/entries` | `GET` | `search=<query>&status=all` | HTML (`EntryListTemplate`) | `#entries-list` |
| `/hx/mark-page-read` | `POST` | `entry_ids=<csv>&status=<status>&search=<query>` | HTML (`EntryListTemplate`) | `#entries-list` |
| `/hx/entries/{id}/read` | `POST` | `status=unread` | HTML (`EntryItemTemplate`) | `#entry-{id}` |
