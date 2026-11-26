# Analysis — Fix Magazine View Flash & Detail Impacts

Result: done

## 1. Root Cause: Magazine View Flash After "Mark Page as Read"

- **Template default (`crates/ui-components/templates/partials/entry_list.html`):**
  - `#entries-list-view` (line 9) renders without `hidden` (visible by default).
  - `#entries-magazine-view` (line 64) renders with `class="hidden magazine-container ..."` (hidden by default).
- **Swap lifecycle (`crates/ui-components/templates/pages/index.html`):**
  - Clicking `#mark-page-read-btn` triggers `POST /hx/mark-page-read` (auth) or `markVisiblePageLocallyRead()` -> `htmx.ajax('GET', nextUrl, { target: '#entries-list', swap: 'innerHTML', source: markBtn })` (guest).
  - HTMX swaps the server's HTML into `#entries-list` with `#entries-list-view` visible and `#entries-magazine-view` hidden.
  - Inside `htmx:afterSwap` (lines 1196–1316), `clearCurrentEntriesLoadingState`, `filterCache.set`, `hydrateGuestEntriesFromDom()`, and `syncDomReadStates()` run before `applyAiViewMode(...)` at line 1315. Because the DOM already contains `#entries-list-view` visible and `#entries-magazine-view` hidden, the browser can paint the list view before `applyAiViewMode` flips the `hidden` classes, causing a visible flash.

## 2. Root Cause: Detail View Missing Photo + Title + Rewrite & LLM Impacts

There are two regressions across Auth and Guest modes:

1. **Auth mode (`openEntryDetail` / `openDetailWithSkeleton` in `crates/ui-components/templates/pages/index.html`):**
   - Lines 981–991: `openEntryDetail` looks up `document.getElementById('entry-' + numId)` (the list-view row, which has no `.magazine-brief` and no `<img>`) instead of `#mag-entry-' + numId` (the magazine card, which has `.magazine-brief` and `img`).
   - `openDetailWithSkeleton(title, brief)` (line 999) does not accept or render an `imageUrl` hero photo.
   - As a result, clicking an item in Magazine view shows a skeleton with no photo and no brief/rewrite while waiting for `/hx/entries/{id}`.

2. **Guest mode (`!AUTH_ENABLED`, `openGuestEntryDetail` / `renderGuestEntryDetail` & `GuestEntryPayload`):**
   - Commit `4cd95ba` added `openEntryDetail` short-circuiting to `openGuestEntryDetail(numId)` when `!AUTH_ENABLED` (`index.html:912-917`).
   - `GuestEntryPayload` (`crates/ui-components/src/templates.rs:155-163`) only serializes `id, title, feed_title, published_at, formatted_date, url, sanitized_content`, omitting `summary` (the list/magazine rewrite/brief) and `image_url` (the list/magazine photo).
   - `cacheLoadedArticlesFromDom()` (`index.html:1625-1671`) also omits `summary` and `image_url` when persisting to IndexedDB `all_articles`.
   - `renderGuestEntryDetail(entry)` (`index.html:1490-1568`) only renders `feed_title`, `date`, `title`, and `sanitized_content`. It omits the hero photo (`image_url`), the rewrite (`summary`), and `<div id="ai-analysis-box" hx-get="/hx/entries/${numId}/analysis" hx-trigger="load" hx-swap="outerHTML">...</div>` + `htmx.process(panel)`.
   - Meanwhile, `GET /hx/entries/{id}/analysis` (`crates/server/src/routes/entries.rs:1672`) is already unauthenticated (no `AuthenticatedUser` extractor) and returns `#ai-analysis-box` with Impact Analysis & Fallacies.
   - In `crates/ui-components/src/templates.rs:3650`, `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` explicitly asserted `!render_fn.contains("/analysis")`, confirming this was removed in `4cd95ba`.

## 3. Scope & `Has API` Determination

- No backend HTTP routes, GraphQL schemas, or API contracts need to change.
- All changes are in `crates/ui-components` (`templates/pages/index.html` and `src/templates.rs`).
- **`Has API`:** `no` (resolved from `unknown`).
