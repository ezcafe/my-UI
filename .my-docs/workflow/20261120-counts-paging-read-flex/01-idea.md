# Framing & Ideation

Result: confirmed

## Goals
1. **Total item count inside Unread & Today filter pills**:
   - `Unread (<count>)` reflecting total unread entries.
   - `Today (<count>)` reflecting total entries published today.
   - Live updates to unread count when items are marked read.
2. **Pagination when more than 1 page**:
   - Display a clean bottom navigation bar with `Previous`, `Page X of Y`, and `Next` when total items exceed page size (e.g. 50).
   - Hide pagination controls when there is only 1 page.
   - HTMX paging links swap `#entries-list` preserving current filter and search query.
3. **Auto-mark as read on detail view**:
   - When viewing an entry's details (`GET /hx/entries/{id}`), trigger backend status update to mark it as read.
   - Remove the unread indicator dot from the corresponding list item and decrement unread counters out-of-band.
4. **Flexible layout following container**:
   - Replace fixed `max-width: 720px` in `.spatial-window` with responsive fluid width (e.g. up to `max-w-7xl` or `1440px`) adapting naturally to the parent container width.

## User Decision
- **Option A selected**: Fluid spatial container with standard Prev/Next paging.

## Non-Goals
- Arbitrary page jumping / jump-to-page input.
- Database schema changes (uses Miniflux backend API).
