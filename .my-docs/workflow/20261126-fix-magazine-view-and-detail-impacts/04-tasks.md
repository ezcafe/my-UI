# Tasks — Fix Magazine View Flash & Detail Impacts

## Implementation Tasks

1. **CSS & `data-view-mode` flash fix (`crates/ui-components/templates/pages/index.html`):**
   - Add CSS rules for `#entries-list[data-view-mode="ai"]` and `#entries-list[data-view-mode="list"]` to control `#entries-list-view` and `#entries-magazine-view` visibility immediately on DOM swap.
   - Update `applyAiViewMode` to set `entriesList.setAttribute('data-view-mode', effectiveMode)`.
   - In `htmx:beforeSwap` for `#entries-list`, pre-set `data-view-mode` on `#entries-list` according to the target state (e.g., `currentViewMode` for `mark-page-read-btn` / pagination, or `getDefaultViewForState` for tab/search swaps) so the swapped HTML never paints in the wrong view mode.

2. **Auth detail skeleton photo + brief (`crates/ui-components/templates/pages/index.html`):**
   - Update `openEntryDetail` to extract `brief` from `#mag-entry-{id} .magazine-brief` (fallback `#entry-{id} .magazine-brief` or `#mag-entry-{id}` `title`) and `imageUrl` from `#mag-entry-{id} img`.
   - Update `openDetailWithSkeleton(title, brief, imageUrl)` to render the hero photo when `imageUrl` is present alongside `title` and `brief`.

3. **Guest detail photo + rewrite + LLM impacts (`crates/ui-components/src/templates.rs`, `crates/ui-components/templates/pages/index.html`):**
   - Add `summary` and `image_url` to `GuestEntryPayload` in `templates.rs`.
   - Preserve `summary` and `image_url` in `cacheLoadedArticlesFromDom()` in `index.html`.
   - Update `renderGuestEntryDetail(entry)` in `index.html` to render the hero photo (`image_url`), rewrite/brief (`summary`), and `#ai-analysis-box` (`hx-get="/hx/entries/${numId}/analysis" hx-trigger="load" hx-swap="outerHTML"`), and call `htmx.process(panel)`.

## Planned Tests (`crates/ui-components/src/templates.rs`)

- Update `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` to verify `renderGuestEntryDetail` includes `#ai-analysis-box`, `/analysis`, `htmx.process(panel)`, `image_url`, and `summary`, while still omitting `author`, `readingTime`, and `/rewrite`.
- Add unit tests verifying:
  - `#entries-list[data-view-mode="ai"]` CSS rules and `applyAiViewMode` / `htmx:beforeSwap` `data-view-mode` synchronization.
  - `openEntryDetail` extracts from `#mag-entry-` and passes `imageUrl` to `openDetailWithSkeleton`.
  - `guest_entries_json` serializes `summary` and `image_url`.
