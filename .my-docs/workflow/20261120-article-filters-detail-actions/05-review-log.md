# Review Log

- **Result:** clean
- **Profile:** lite (bundled Adversarial + Quality)
- **Date:** 2026-11-20

## Summary

Completed code review for the article search filter, filter pill reordering, "Mark this page as read" button, author removal in detail view, and "Mark as unread" action across `crates/ui-components`, `crates/miniflux`, and `crates/server`.

All changes adhere to design specifications and settled decisions, and all 55 workspace automated tests pass cleanly.

---

## Adversarial

- **Result:** clean

### Edge Cases & Security Checks
1. **Batch Mark Page Read Parsing & Empty States**:
   - In `crates/server/src/routes/entries.rs`, `mark_page_read_htmx` supports both JSON array formats (`["101", "102"]`) and comma-delimited strings (`101,102`).
   - Safely handles empty, missing, or whitespace-only inputs without calling upstream mutations or returning errors.
   - Upstream IDs are parsed strictly into `i64` in GraphQL mutation `updateEntriesStatus`, rejecting malformed IDs before executing network requests.
2. **Selective Marking of Visible Articles Only**:
   - The frontend helper `getVisibleEntryIds()` filters out articles with `style.display === 'none'`.
   - When client-side search filtering hides articles, only the visible subset is sent to `/hx/mark-page-read`, preventing accidental read state mutation of filtered-out articles.
3. **Hybrid Search Injection & XSS Guardrails**:
   - `filterArticlesClientSide(query)` searches using `textContent` and toggles CSS `display`, avoiding any raw HTML injection or DOM XSS vectors.
   - Backend search queries in `build_graphql_filter` are sanitized via `serde_json::to_string`, securely escaping quotes, backslashes, and control characters to prevent GraphQL query injection.
   - Whitespace-only search queries are ignored safely.
4. **Detail Panel Mark Unread Synchronization**:
   - "Mark as Unread" action invokes `POST /hx/entries/{id}/read?status=unread`.
   - Correctly updates the backend status, replaces the list row item with unread dot indicator, and updates the global header unread count via HTMX out-of-band swap (`hx-swap-oob="true"`).
5. **Author Field Elimination**:
   - Verified that author metadata is completely removed from `crates/ui-components/templates/partials/entry_detail.html` while preserving feed title and publication date, matching design decisions.

---

## Quality

- **Result:** clean

### 5-Axis Quality Pass
1. **Correctness**:
   - Filter pill order conforms to `Unread` < `Today` < `Starred` < `All Titles`, with `Unread` active by default.
   - Hybrid search defaults to searching all articles (`status=all`) when submitting form via server query.
   - All 55 automated tests (20 in `miniflux-client`, 9 in `ui-components`, 24 in `server` lib, 2 in `server` e2e) pass cleanly.
2. **Readability & Simplicity**:
   - Frontend JavaScript functions (`setActiveFilter`, `filterArticlesClientSide`, `getVisibleEntryIds`) are concise, self-contained, and easy to maintain.
   - Template changes use standard Askama structures with clear semantic markup.
3. **Architecture & Separation of Concerns**:
   - GraphQL schema encapsulates batch status updates via `updateEntriesStatus`.
   - Axum handler layer cleanly extracts form data and coordinates schema execution with template rendering.
   - UI templates remain decoupled from server logic through typed view models.
4. **Performance**:
   - Client-side article filtering provides immediate response on keystroke without unnecessary server requests.
   - Batch mark-as-read executes in a single HTTP request to Miniflux rather than issuing individual per-article requests.
5. **Code Health**:
   - No unnecessary dependencies or dead code added.
   - Clean, idiomatic Rust with comprehensive error handling and logging via existing application error types.
