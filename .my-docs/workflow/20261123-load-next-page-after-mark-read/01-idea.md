# 01-idea: Automatically Load Next Page After Marking Page as Read

## 1. Executive Summary & Goals

- **Core Goal**: When the user clicks the "Mark This Page as Read" button (the double-check icon in the header), the application should mark the current visible page of articles as read and **immediately load the next page of articles automatically**.
- **Current Problem**:
  - In **Guest / Offline mode**, clicking "Mark This Page as Read" marks articles in local IndexedDB and hides them from the screen. When all articles on the page disappear, the UI displays an empty "All caught up!" placeholder, **leaving the user on an empty screen even when there are more pages with unread articles**.
  - In **Authenticated / Online mode**, clicking "Mark This Page as Read" sends an HTMX POST to `/hx/mark-page-read`. The server marks the articles read in Miniflux and re-renders the list starting at `offset: 0`. For filters like **Today**, **All**, or **Starred** where read articles remain visible, **the exact same page is re-rendered instead of advancing to Page 2**. If the user was on Page 2 or 3, it forcefully jumps them back to Page 1.
  - In both modes, the scroll position is not reset to the top when the new page arrives.
- **Key Takeaways**:
  - Automatically load the next page in both authenticated online mode and guest offline mode.
  - Intelligently handle pagination depending on the active filter (`unread` consumes items, while `today`/`all`/`starred` keeps items and increments `offset`).
  - Reset list scroll to the top (`scrollTop = 0`) smoothly upon loading the new page.
  - If the user is on the final page or no articles remain, cleanly present the "All caught up!" state or end-of-feed indication.

---

## 2. Root Cause Analysis in Codebase

### A. Online Mode (`auth_enabled = true`)
1. **Server Hardcodes `offset: 0`**:
   - In [`crates/server/src/routes/entries.rs#L851-L916`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L851-L916), `mark_page_read_htmx` receives `MarkPageReadForm` and re-renders the entries list with:
     ```rust
     let filter_query = EntriesFilterQuery {
         status: form.status,
         search: form.search,
         feed_id: form.feed_id,
         category_id: form.category_id,
         limit: Some(50),
         offset: Some(0), // Hardcoded to 0
         request_brief: None,
     };
     ```
   - For `status = "all"` or `status = "today"`, articles remain in the query even after being marked read. Re-rendering `offset: 0` causes the server to return the exact same Page 1 articles, failing to load Page 2.
2. **Form Missing Pagination Context**:
   - In [`crates/server/src/routes/entries.rs#L32-L39`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L32-L39), `MarkPageReadForm` only defines `entry_ids`, `status`, `search`, `feed_id`, and `category_id`. It does not accept `offset`.
   - In [`crates/ui-components/templates/pages/index.html#L41-L53`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L41-L53), `#mark-page-read-btn` uses `hx-include="#active-status, [name='search']"`. It does not include the current offset.

### B. Guest / Offline Mode (`auth_enabled = false`)
1. **Never Triggers Next Page**:
   - In [`crates/ui-components/templates/pages/index.html#L1158-L1171`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1158-L1171), `markVisiblePageLocallyRead()` adds all visible entry IDs into `readSet` and IndexedDB.
   - It then calls `syncDomReadStates()`. On the `unread` filter, all rows receive `style.display = 'none'` ([`index.html#L1026-L1028`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1026-L1028)).
   - Once all rows are hidden, `visibleUnreadRowCount === 0`, and it immediately injects `#client-empty-state` ("All caught up!").
   - It does not attempt to trigger the pagination "Next" button ([`crates/ui-components/templates/partials/entry_list.html#L210-L221`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L210-L221)) or fetch the next page.
2. **Type Discrepancy in `markVisiblePageLocallyRead`**:
   - `getVisibleEntryIds()` returns a comma-separated string (`"1,2,3"`), but `markVisiblePageLocallyRead()` checks `Array.isArray(ids)`, which always evaluates to `false` and falls back to a DOM query selector.

### C. Scroll Position & Feedback
- When a new page is swapped into `#entries-list`, there is no automatic scroll reset (`entriesList.scrollTop = 0`). The container stays scrolled to wherever the user was.

---

## 3. Proposed Solution

### A. Online Mode (`auth_enabled = true`)
1. **Pass Current Pagination in HTMX Request**:
   - Expose current offset in the DOM (e.g., hidden input `<input type="hidden" id="active-offset" name="offset" value="{{ offset }}">` inside `entry_list.html`).
   - Update `#mark-page-read-btn` `hx-include` to include `#active-offset`.
2. **Smart Server Progression in `mark_page_read_htmx`**:
   - Add `pub offset: Option<i64>` to `MarkPageReadForm`.
   - Compute the next offset based on the active filter:
     - **For `status == "unread"`**:
       - When Page 1 (`offset == 0`) is marked read, those 50 items are removed from the unread set. The server fetches `offset: 0` from the updated unread set, which now contains the next 50 unread items.
       - When Page > 1 is marked read (e.g. user was on Page 2 at `offset: 50`), marking them read removes them. If remaining unread count > 50, fetch `offset: 50` (former Page 3). If remaining count <= 50, reset to `offset: 0`.
       - If total unread remaining is 0, the server renders the clean "All caught up!" empty state.
     - **For `status == "all"`, `status == "today"`, or `status == "starred"`**:
       - Marked items stay in the list. The server advances to `next_offset = current_offset + limit`.
       - If `next_offset < total`, load `next_offset`.
       - If `next_offset >= total` (user was on the final page), stay on the final page with all items displayed as read.

### B. Guest / Offline Mode (`auth_enabled = false`)
1. **Automatic Next Page Trigger in `markVisiblePageLocallyRead`**:
   - Mark visible articles read in `readSet` and IndexedDB.
   - Look for the pagination Next button in the DOM (`document.querySelector('nav[aria-label="Pagination"] button[hx-get*="offset="]')` that contains "Next").
   - If a Next button exists:
     - Programmatically trigger it (e.g., `nextBtn.click()` or `htmx.trigger(nextBtn, 'click')`).
     - The next page of articles is fetched via HTMX and swapped into `#entries-list`.
   - If no Next button exists (user was on the final page or only 1 page existed):
     - If viewing `unread`, display `#client-empty-state` ("All caught up!").
     - If viewing `today` or `all`, keep items visible and marked read.

### C. UX & Scroll Polishing
1. **Scroll to Top**:
   - In `htmx:afterSwap`, whenever `#entries-list` is swapped due to `mark-page-read` or pagination, scroll the container back to the top:
     ```javascript
     const listEl = document.getElementById('entries-list');
     if (listEl) listEl.scrollTop = 0;
     ```
2. **Visual Feedback**:
   - Ensure the loading bar (`#entries-loading-bar`) activates during the mark-read and page-load transition.

---

## 4. Architectural & Design Decisions

### Decision: Filter-Aware Progression vs. Always-Advance Offset

- **Option 1 (Recommended): Filter-Aware Progression**
  - **Explanation**: The server checks the active filter. For `unread`, marking articles read removes them from the unread pool, so loading offset 0 brings in the next unread items. For `today`, `all`, and `starred`, articles remain visible, so the offset advances by `+limit` (e.g., from 0 to 50).
  - **Example**:
    ```rust
    let next_offset = match form.status.as_deref() {
        Some("unread") | None => {
            // Unread items leave the pool; offset 0 holds the next batch
            form.offset.unwrap_or(0).min(remaining_total.saturating_sub(1) / limit * limit)
        },
        _ => {
            // Items remain in pool; advance offset to next page
            let current = form.offset.unwrap_or(0);
            if current + limit < total { current + limit } else { current }
        }
    };
    ```
  - **Pros**:
    - Intuitive behavior in every view.
    - Prevents skipping unread articles in `unread` mode.
    - Prevents staying stuck on Page 1 in `all`/`today` mode.
  - **Cons**: Small amount of conditional logic on the server to check filter status.

- **Option 2: Always Advance Offset (`offset + limit`) Regardless of Filter**
  - **Explanation**: Always increment the offset by 50 when "Mark This Page as Read" is clicked, regardless of whether articles were removed or kept.
  - **Example**:
    ```rust
    let next_offset = current_offset + limit;
    ```
  - **Pros**: Simple, single formula without branching.
  - **Cons**: In `unread` mode, when 50 items are marked read, they vanish from the unread query. Items 51–100 shift down to indices 0–49. If we query offset 50, we skip items 51–100 entirely and jump straight to items 101–150! Users would miss unread articles.

---

## 5. Invariants & Scope

- **Has UI (tentative)**: yes (updates to `#mark-page-read-btn`, `index.html` guest handling, `entry_list.html` offset input, scroll reset)
- **Has API**: no (internal HTMX endpoint `/hx/mark-page-read` parameter addition, no breaking API change)
- **Has DB**: no
- **Has auth/PII/schema**: no
- **TDD Test Plan**:
  1. `crates/server`: Wiremock test for `POST /hx/mark-page-read` in `status=all` asserting that next offset is loaded.
  2. `crates/server`: Wiremock test for `POST /hx/mark-page-read` in `status=unread` asserting that remaining unread items are returned.
  3. `crates/ui-components`: Template test ensuring `#active-offset` is rendered in `entry_list.html`.
  4. Client-side test / verification: Verify `markVisiblePageLocallyRead()` triggers next page load in guest mode and scrolls to top.

---

## 6. Questions & Confirmed Decision

- **User Decision**: Option 1 confirmed. When on the final page of articles, clicking "Mark This Page as Read" in `Today` or `All` view will keep the user on the final page, mark all items read, and show a brief toast: `"Page marked as read. You are on the last page."`. In `Unread` view, it displays the "All caught up!" empty state.
- **Status**: Ideation confirmed by user.

