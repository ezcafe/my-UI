# Grill Stress-Test & Decision Frontier

Result: frontier-empty

## Decision Tree & Settled Decisions

### 1. Search Box Architecture (Settled)
- **Problem**: Previously, typing in the search box triggered HTMX queries with `#active-status` (defaulting to `unread`). Searching for read articles (like "Microsoft says Bing...") returned 0 results, and typing was laggy.
- **Settled Decision**: User explicitly selected **Option C: Hybrid Search**.
  - On typing (`input` event): Instant client-side DOM filter matching `item.title.toLowerCase().includes(query)`.
  - On Enter / form submit: Server-side search request sent to `/hx/entries?search=...&status=all` to query all articles across Miniflux without being constrained by unread status.

### 2. Filter Bar Ordering (Settled)
- **Settled Decision**: Order changed from `[All Titles, Unread, Today, Starred]` to:
  1. `Unread` (active default)
  2. `Today`
  3. `Starred`
  4. `All titles`

### 3. Mark Page as Read Scope (Settled)
- **Settled Decision**: Replace global "Mark All as Read" with "Mark This Page as Read".
  - Scope: Collects entry IDs of currently visible entries in `#entries-list`.
  - Upstream call: Miniflux `PUT /v1/entries` with payload `{"entry_ids": [...], "status": "read"}`.
  - UI feedback: Refreshes entry list and updates unread count pill out-of-band.

### 4. Detail Metadata Display (Settled)
- **Settled Decision**: Remove the author field (`• By Amanda Caswell`) from `entry_detail.html`. Display only `{{ entry.feed_title }} • {{ entry.published_at }}`.

### 5. Detail Action — Mark as Unread (Settled)
- **Settled Decision**: Add icon button before Bookmark button in `entry_detail.html` header.
  - Action: Issues `POST /hx/entries/{id}/read?status=unread`.
  - UI response: Updates entry row in the list and synchronizes unread pill count out-of-band.

## Frontier Status
- **Open Questions**: None. All requirements and implementation details are fully settled.
- **Frontier**: Empty. Ready for design and implementation tasks.
