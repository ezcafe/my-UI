# Analysis: Title-Only List, External Link, Today Filter & Search/Filter Fixes

- **Result:** done

## 1. Executive Summary

This feature updates the article list UI to match the spatial glassmorphism style of `ar-list.html` with title-only rows and an external link button, adds a "Today" filter button, and fixes the broken search and filter functionality across the frontend templates, Axum server routes, and GraphQL/Miniflux layer.

## 2. Current Architecture & State

### Frontend & UI Layer (`crates/ui-components`)
- **Templates**: Askama HTML templates located in `crates/ui-components/templates/`.
  - `pages/index.html`: Contains the top navigation header, filter pill toolbar, search input, and `#entries-list` container.
  - `partials/entry_item.html` & `partials/entry_list.html`: Currently render multi-line cards containing feed title, relative published date, reading time, multi-line title, text summary, bookmark button, and read toggle button.
- **Styling**: Tailwind CSS with custom spatial glassmorphism classes (`ar-glass-panel`, `ar-list-item`, `spatial-window`, etc.) defined in `templates/base.html`.
- **Icons**: Lucide icons initialized on page load and swapped via HTMX.

### Server Route Layer (`crates/server/src/routes/entries.rs`)
- `list_entries_htmx`:
  - Parses `EntriesFilterQuery`: `status`, `feed_id`, `category_id`, `search`, `limit`, `offset`.
  - **Bug 1 (Search omitted)**: `query.search` is parsed into the query struct but completely omitted when building the GraphQL query string for `entries(filter: { ... })`.
  - **Bug 2 (Filter mapping broken)**: `status_filter` matches only `Some("read") => "status: READ"`, `Some("all") => ""`, and falls back to `_ => "status: UNREAD"`. Requests for `status=starred` or `status=today` fall back to `UNREAD`.
  - **Bug 3 (Active pill styling static)**: `pages/index.html` hardcodes the active styling (`bg-indigo-500/20 text-indigo-300 border-indigo-400/30`) on the "Unread" button. Switching filters does not update button styles.

### Miniflux & GraphQL Schema Layer (`crates/miniflux`)
- `EntryFilterInput` in `crates/miniflux/src/schema/types.rs`: Supports `status`, `feed_id`, `category_id`, `starred`, `search`, `limit`, `offset`, `order`, `direction`.
- `EntriesFilter` in `crates/miniflux/src/models.rs`: Domain filter representation matching `EntryFilterInput`.
- `MinifluxClient::get_entries`: Appends `search`, `status`, `starred`, etc. as query parameters to `/v1/entries`.
- **Gap for "Today"**: Miniflux REST API supports `published_after` (Unix timestamp integer in seconds). `EntryFilterInput` and `EntriesFilter` currently lack `published_after`.

## 3. Detailed Gap Analysis

| Area | Current Behavior | Target Behavior |
|------|------------------|-----------------|
| **List Row UI** | Shows feed, timestamp, reading time, title, snippet summary, bookmark & read buttons | Shows title only (`line-clamp-2`, matching `ar-list.html`), unread status indicator, external link button, chevron reader trigger |
| **External Link** | None on row; only internal detail navigation | Icon button on the right side with `target="_blank" rel="noopener noreferrer"` and `event.stopPropagation()` |
| **Today Filter** | Missing from filter toolbar | Filter pill positioned immediately after "Unread" (`All Titles`, `Unread`, `Today`, `Starred`) |
| **Search Function** | Extracted by Axum query but omitted from GraphQL | Passed to GraphQL `entries(filter: { search: "..." })` and Miniflux client query |
| **Filter Function** | Drops `starred` and `today`; falls back to `UNREAD` | Maps `starred` to `starred: true`, `today` to `published_after: <timestamp>`, `all` to no status, `unread` to `status: UNREAD` |
| **Active Pill State** | Hardcoded static styles on "Unread" | Dynamically reflects the currently active filter pill upon selection |

## 4. Risk & Constraint Assessment

- **HTMX Event Bubbling**: Clicking the external link button inside an item row must not bubble up and trigger the HTMX reader modal (`event.stopPropagation()`).
- **Input Sanitization**: Search terms in GraphQL query string construction must be properly escaped against quotes or backslashes to avoid query syntax errors.
- **Timezone for Today**: Server UTC timestamp vs local day boundaries must be handled deterministically.
