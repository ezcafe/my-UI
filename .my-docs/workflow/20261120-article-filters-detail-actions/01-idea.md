# Idea / framing

## Intent
Fix article list filtering and search behavior, reorder filter pills, replace global mark-all-read with page-scoped mark-read, clean up metadata display in article detail, and add a "Mark as Unread" icon button in article detail.

## Goals
1. **Search Box Fix ("Filter articles...")**:
   - Hybrid search (Option C confirmed by user):
     - Typing instantly filters visible titles in `#entries-list` client-side (`item.title.toLowerCase().includes(query)`).
     - Pressing `Enter` / clicking search icon triggers server-side query across all articles (`status=all` by default so read articles like "Microsoft says Bing ..." are not excluded by unread filter).
2. **Filter Buttons Reordering**:
   - Update filter bar order to: `Unread`, `Today`, `Starred`, `All titles`.
   - Default active state matches `Unread`.
3. **Mark This Page as Read**:
   - Replace global `Mark All as Read` (`/v1/users/{id}/mark-all-as-read`) with `Mark This Page as Read`.
   - Collects entry IDs currently loaded on the page and sends batch update (`PUT /v1/entries` with `{ entry_ids, status: "read" }`).
4. **Detail Header Meta Cleanup**:
   - Remove author display (`• By <Author>`), showing only feed title and published timestamp (`Feed • 2026-11-19T07:45:11+07:00`).
5. **Detail "Mark as Unread" Action**:
   - Add icon button before Bookmark button in detail view header to mark active entry as unread (`POST /hx/entries/{id}/read?status=unread`).

## Non-goals
- Modifying backend database schema or altering OIDC authentication.

## Constraints
- Spatial glassmorphism styling (`ar-list.html` / `ar-detail.html`).
- Never read or parse `.env`.
- Follow TDD: test cases written and verified before implementation.

## Shared understanding
- [x] User confirmed Option C (Hybrid search) + goals: `user: "The \"Filter articals\" search box is not working... Option C: Hybrid"`
