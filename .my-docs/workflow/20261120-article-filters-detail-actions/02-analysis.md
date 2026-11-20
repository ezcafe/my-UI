# System Analysis

Result: done

## Executive Summary
This change refines the article browsing and reading experience in the Spatial RSS reader across five specific areas:
1. **Search Box Fix**: Implement hybrid search (Option C confirmed by user) so typing filters visible articles client-side immediately, while pressing Enter queries the Miniflux backend across all articles (`status=all`) without getting blocked by unread filters.
2. **Filter Bar Reorder**: Reorganize filter pills to: `Unread` (active default), `Today`, `Starred`, `All titles`.
3. **Mark Page as Read**: Replace global mark-all-read with a page-scoped action that sends visible entry IDs to Miniflux (`PUT /v1/entries` with `{ entry_ids, status: "read" }`) and refreshes unread counts.
4. **Detail Header Meta Cleanup**: Remove the author line (`• By <Author>`) from the article detail view, keeping only feed title and publication date.
5. **Detail "Mark as Unread" Action**: Add an icon button in the article detail header that sends `POST /hx/entries/{id}/read?status=unread` to restore unread status and update counters.

## Architecture Touchpoints
- **Frontend Templates (`crates/ui-components/templates/`)**:
  - `pages/index.html`: Reorder filter pills, update search input to hybrid mode (client filter on typing, form submit on Enter), and replace mark-all-read button with mark-this-page-as-read.
  - `partials/entry_list.html`: Ensure `data-entry-id` attribute is present on item rows for client-side collection.
  - `partials/entry_item.html`: Ensure consistent markup with list items.
  - `partials/entry_detail.html`: Remove author display; add "Mark as Unread" button before bookmark button.
- **Backend Handlers & Routing (`crates/server/src/routes/entries.rs`, `crates/server/src/app.rs`)**:
  - `list_entries_htmx`: Default to `status=all` when search query is provided without explicit status.
  - New route `POST /hx/mark-page-read`: Parse entry IDs, invoke batch update mutation, and re-render entry list with updated unread counter.
  - `mark_entry_read_htmx`: Already supports `status=unread` query param; ensure it smoothly swaps the updated item and out-of-band unread pill.
- **GraphQL Schema & Client (`crates/miniflux/`)**:
  - `MinifluxClient::update_entry_status`: Already supports batch `entry_ids: Vec<i64>` and `PUT /v1/entries`.
  - `Mutation::update_entries_status`: Expose batch update in GraphQL mutation schema.

## Risk Analysis & Mitigation
- **Risk 1: Client-side search hiding entries during Mark This Page as Read**:
  - *Symptom*: If a user types a filter that hides 8 out of 10 items, does "Mark This Page as Read" mark all 10 or only the 2 visible items?
  - *Mitigation*: The requirement explicitly states "Reads visible entry IDs in list". The JS helper `getVisibleEntryIds()` filters by `display !== 'none'`, ensuring only visible entries are included.
- **Risk 2: Miniflux unread counter synchronization**:
  - *Symptom*: Updating entries on Miniflux can leave UI counters stale.
  - *Mitigation*: Both `mark_page_read_htmx` and `mark_entry_read_htmx` fetch fresh `unreadCounter` from Miniflux and return it via `<span id="unread-pill" hx-swap-oob="true">`.
- **Risk 3: Search input event storms**:
  - *Symptom*: Typing in the search box previously sent debounced HTTP requests to `/hx/entries`.
  - *Mitigation*: Typing will now execute purely in client-side JS (`oninput="filterArticlesClientSide(this.value)"`), eliminating network latency and server load. Network request only fires on Enter / form submit.
