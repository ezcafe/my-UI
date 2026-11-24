# Grill: Automatically Load Next Page After Marking Page as Read

## Settled Decisions & Architectural Alignment

### 1. Filter-Aware Progression vs. Always-Advance Offset (Settled)
- **Decision**: Filter-Aware Progression (Option 1).
- **Behavior**:
  - In `unread` filter view: Marking items as read removes them from the unread pool in Miniflux. Requesting `offset: 0` loads the subsequent batch of unread articles. If no unread articles remain, the server renders the "All caught up!" empty state.
  - In `today`, `all`, and `starred` filter views: Articles remain in the query results even after being marked read. The server checks the total count and advances `next_offset = current_offset + limit` when `next_offset < total`.
- **User Confirmation**: Confirmed in Ideation (`user: "Option 1 confirmed"`).

### 2. Final Page Feedback & Toast Handling (Settled)
- **Decision**: When the user is on the final page of `today`, `all`, or `starred`, clicking "Mark This Page as Read" keeps the user on the final page, marks all articles read, and displays an out-of-band toast: `"Page marked as read. You are on the last page."`.
- **Behavior**:
  - In Online mode, the server appends `ToastTemplate` (with `id="toast-container" hx-swap-oob="beforeend"`) to the entries list HTML. HTMX swaps the list into `#entries-list` and appends the toast to `#toast-container`.
  - In Guest mode, `markVisiblePageLocallyRead()` checks for a Next button. If none is found, it calls `showToast('Page marked as read. You are on the last page.')`.
- **User Confirmation**: Confirmed in Ideation (`user: "Option 1 confirmed"`).

### 3. Guest Mode Next Page Triggering (Settled)
- **Decision**: In `markVisiblePageLocallyRead()`, after saving the visible article IDs to `readSet` and IndexedDB, check for the pagination Next button (`nav[aria-label="Pagination"] button[hx-get*="offset="]`). If present, trigger it (`nextBtn.click()`).
- **Behavior**: Programmatically clicks the Next button, which initiates HTMX request to fetch and swap the next page.
- **User Confirmation**: Confirmed in Ideation.

### 4. Viewport Scroll Position Reset (Settled)
- **Decision**: Reset `scrollTop = 0` on `#entries-list` in `htmx:afterSwap`.
- **Behavior**: Whenever `#entries-list` receives new content (whether from pagination, filter change, or mark-page-read), the scroll position resets to the top smoothly.
- **User Confirmation**: Confirmed in Ideation.

---

## Frontier Status

All design decisions, edge cases, and architectural contracts have been resolved and settled. No open frontier questions remain.

## Result

Result: frontier-empty
