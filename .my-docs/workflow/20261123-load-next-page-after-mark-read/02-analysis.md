# System Analysis: Automatically Load Next Page After Marking Page as Read

## 1. Context & Objectives

When a user clicks the "Mark This Page as Read" button (the double-check icon in the header), the application marks all visible articles on the current page as read. Currently, this leaves the user in an unproductive or confusing state:
- In **Guest mode** (`auth_enabled = false`), articles are marked read and hidden. When all articles on the page disappear, the UI displays the empty "All caught up!" placeholder, even when subsequent pages contain unread articles. The user is stuck on an empty screen and must manually find and click pagination.
- In **Authenticated mode** (`auth_enabled = true`), the server marks articles as read in Miniflux and re-renders the list starting at hardcoded `offset: 0`. For views like **Today**, **All**, or **Starred** where articles remain visible after being marked read, the server re-renders the exact same articles instead of advancing to Page 2. If the user was on Page 2 or 3, they are unexpectedly reset back to Page 1.
- In both modes, the list scroll position is preserved from before the swap, leaving the viewport at the bottom or middle of the list instead of at the top of the newly loaded page.

### Objectives
1. **Authenticated Mode Progression**: Add pagination awareness to `mark_page_read_htmx`. For `unread`, fetch `offset: 0` (the next 50 unread items); for `today`, `all`, and `starred`, advance `offset` by `limit`. If on the final page of `today`/`all`/`starred`, stay on the final page and show an out-of-band toast notification: `"Page marked as read. You are on the last page."`.
2. **Guest Mode Progression**: In `markVisiblePageLocallyRead()`, after marking the visible items read, trigger the pagination Next button if present. If no Next button exists (last page), show the toast: `"Page marked as read. You are on the last page."`.
3. **Viewport Scroll Reset**: On any swap targeting `#entries-list`, automatically reset `scrollTop = 0` so the newly arrived articles start at the top of the list.
4. **Reliable DOM Wiring**: Expose `#active-offset` in `entry_list.html` and include it in `#mark-page-read-btn`'s `hx-include`.

---

## 2. Technical & Architectural Breakdown

### A. Online Mode (`crates/server/src/routes/entries.rs`, `crates/ui-components`)

1. **DOM Exposure of Active Offset**:
   - `entry_list.html` currently lacks a hidden input for the active offset.
   - We inject `<input type="hidden" id="active-offset" name="offset" value="{{ offset }}">` at the top of `templates/partials/entry_list.html`.
   - In `templates/pages/index.html`, `#mark-page-read-btn`'s `hx-include` is updated:
     ```html
     hx-include="#active-status, #active-offset, [name='search']"
     ```
   - When the button is clicked, HTMX serializes `offset` along with `status` and `search`.

2. **Form Structure & Handler Progression**:
   - `MarkPageReadForm` in `crates/server/src/routes/entries.rs` is expanded with:
     ```rust
     pub offset: Option<i64>,
     ```
   - In `mark_page_read_htmx`, progression is calculated according to the active filter:
     - **For `status == "unread"` (or omitted)**:
       Since marking articles read removes them from the unread pool in Miniflux, items 51–100 immediately slide down to positions 0–49. Querying `offset: 0` loads the next batch of unread articles. If no unread items remain, `total` is 0 and `entry_list.html` renders the clean "All caught up!" empty state.
     - **For `status != "unread"` (`today`, `all`, `starred`)**:
       Marked articles remain in the result set. We query the total count for the current filter. If `offset + limit < total`, we advance `offset` to `offset + limit`. If `offset + limit >= total` (the user is already on the final page), we stay at the current `offset` and append `ToastTemplate` with `"Page marked as read. You are on the last page."`.
   - `ToastTemplate` utilizes `hx-swap-oob="beforeend"` on `#toast-container`. When appended to the HTML response, HTMX updates `#entries-list` with the page articles and simultaneously displays the toast notification.

### B. Guest Mode (`crates/ui-components/templates/pages/index.html`)

1. **Next Button Trigger**:
   - In `markVisiblePageLocallyRead()`:
     - Visible article IDs are retrieved and marked read in `readSet` and IndexedDB.
     - The pagination navigation is inspected for an active Next button:
       ```javascript
       const nextBtn = document.querySelector('nav[aria-label="Pagination"] button[hx-get*="offset="]');
       if (nextBtn) {
         nextBtn.click();
       } else {
         showToast('Page marked as read. You are on the last page.');
       }
       ```
     - If `nextBtn` is present, clicking it triggers HTMX to fetch and swap the next page.
     - If no Next button exists (last page or single page), `showToast('Page marked as read. You are on the last page.')` is displayed.
2. **Type Discrepancy Fix in `markVisiblePageLocallyRead()`**:
   - `getVisibleEntryIds()` returns a comma-separated string (`"1,2,3"`).
   - `markVisiblePageLocallyRead()` previously checked `Array.isArray(ids)`, failing every time.
   - We parse `ids` whether string or array, ensuring every visible ID is recorded in IndexedDB.

### C. Scroll Position Reset (`crates/ui-components/templates/pages/index.html`)

- In `document.addEventListener('htmx:afterSwap', ...)`, whenever `evt.detail.target.id === 'entries-list'`:
  ```javascript
  evt.detail.target.scrollTop = 0;
  ```
- This ensures any new page of articles loaded via HTMX starts scrolled to the very top.

---

## 3. Risks & Mitigations

- **Risk 1: Double advancement or offset overflow in Online mode**:
  - *Mitigation*: The server queries `total` before advancing `offset`. If `current_offset + limit >= total`, the server caps the offset at `current_offset` and renders the last-page toast.
- **Risk 2: Multiple `#active-offset` inputs in the DOM**:
  - *Mitigation*: `#active-offset` is placed strictly inside `entry_list.html`. Since `entry_list.html` is the swapped inner HTML of `#entries-list`, each swap cleanly replaces the previous input, maintaining exactly one instance.
- **Risk 3: Unread filter skipping unread articles**:
  - *Mitigation*: For `unread`, the offset is kept at `0` because Miniflux shifts unread items upward as they are marked read.

---

## Result

Result: done
