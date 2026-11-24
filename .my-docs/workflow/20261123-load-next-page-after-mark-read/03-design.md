# System Design: Automatically Load Next Page After Marking Page as Read

## 1. Executive Summary & Core Milestones

When the user clicks the "Mark This Page as Read" button (double-check icon in the header), the application marks all visible articles on the current page as read and automatically transitions to the next batch of articles:
- **Milestone 1**: In Online mode (`auth_enabled = true`), pass the current pagination offset from the client to the server, and calculate filter-aware progression:
  - For `status == "unread"`, load `offset: 0` (which immediately yields the next 50 unread articles as the previous ones leave the unread set). If no unread articles remain, render the empty "All caught up!" state.
  - For `status != "unread"` (`today`, `all`, `starred`), advance `next_offset = offset + limit`. If `next_offset < total`, load `next_offset`. If `next_offset >= total` (the user was on the final page), stay on the final page and display a toast: `"Page marked as read. You are on the last page."`.
- **Milestone 2**: In Guest mode (`auth_enabled = false`), mark visible articles as read in IndexedDB and programmatically click the pagination Next button (`#pagination-next-btn`) if present. If on the final page, display the toast notification `"Page marked as read. You are on the last page."`.
- **Milestone 3**: In both modes, reset the scroll position of `#entries-list` to `0` whenever a new page is swapped into the view, providing a clean, top-of-page reading experience.

---

## 2. Design Options & Trade-Offs

### Option 1: Filter-Aware Progression & Out-of-Band Toast (Recommended)

- **Explanation**:
  The server determines the next offset based on the active filter.
  In `unread` mode, marking articles read removes them from the unread query; querying `offset: 0` loads the next 50 unread articles from the top of the unread pool.
  In `today`, `all`, and `starred` modes, marked articles remain visible, so the server queries the total count and advances `next_offset = offset + limit`. If `next_offset >= total`, the user is on the final page: the server stays on the current offset, marks the articles read, and sends an out-of-band toast notification (`ToastTemplate` with `hx-swap-oob="beforeend"`).
  In guest mode, the client marks items read and clicks the Next button, or shows the toast if no Next button exists.

- **Concrete Examples**:
  - *Server Form & Progression (`crates/server/src/routes/entries.rs`)*:
    ```rust
    #[derive(Debug, Deserialize, Default)]
    pub struct MarkPageReadForm {
        pub entry_ids: Option<String>,
        pub status: Option<String>,
        pub search: Option<String>,
        pub feed_id: Option<i64>,
        pub category_id: Option<i64>,
        pub offset: Option<i64>,
    }

    pub async fn mark_page_read_htmx(...) -> Response {
        // ... mark entries as READ in Miniflux mutation ...

        let current_offset = form.offset.unwrap_or(0);
        let limit = 50;
        let is_unread = form.status.as_deref() == Some("unread") || form.status.is_none();

        let (target_offset, show_last_page_toast) = if is_unread {
            (0, false)
        } else {
            // Count total items under current filter
            let count_filter = build_graphql_filter(&EntriesFilterQuery {
                status: form.status.clone(),
                search: form.search.clone(),
                feed_id: form.feed_id,
                category_id: form.category_id,
                limit: Some(1),
                offset: Some(0),
                request_brief: None,
            }, 1, 0, config.auth_enabled);
            let gql = format!(
                r#"query {{
                    entriesResult(filter: {{ {count_filter} }}) {{
                        total
                    }}
                }}"#
            );
            let res = schema.execute(&gql).await;
            let total = res.data.into_json()
                .ok()
                .and_then(|d| d.get("entriesResult").and_then(|r| r.get("total")).and_then(|v| v.as_i64()))
                .unwrap_or(0);

            let next_offset = current_offset + limit;
            if next_offset < total {
                (next_offset, false)
            } else {
                (current_offset, true)
            }
        };

        let filter_query = EntriesFilterQuery {
            status: form.status,
            search: form.search,
            feed_id: form.feed_id,
            category_id: form.category_id,
            limit: Some(limit),
            offset: Some(target_offset),
            request_brief: None,
        };

        let html = render_entries_list_html(&schema, None, None, &headers, &filter_query, config.auth_enabled).await;
        match html {
            Ok(entries_html) => {
                if show_last_page_toast {
                    let toast = ToastTemplate {
                        message: "Page marked as read. You are on the last page.".to_string(),
                        is_error: false,
                    };
                    let toast_html = toast.render().unwrap_or_default();
                    Html(format!("{entries_html}{toast_html}")).into_response()
                } else {
                    Html(entries_html).into_response()
                }
            }
            Err(resp) => resp,
        }
    }
    ```

  - *Client Button & Offset Template (`templates/pages/index.html` & `templates/partials/entry_list.html`)*:
    ```html
    <!-- In entry_list.html -->
    <input type="hidden" id="active-offset" name="offset" value="{{ offset }}">

    <!-- Next button in entry_list.html pagination -->
    {% if has_next %}
    <button id="pagination-next-btn"
            hx-get="/hx/entries?status={{ status }}&search={{ search }}&offset={{ next_offset }}&limit={{ limit }}"
            hx-target="#entries-list"
            hx-swap="innerHTML"
            class="px-3 py-1.5 rounded-full border border-border bg-card hover:bg-muted text-foreground font-bold transition-colors">
      Next
    </button>
    {% endif %}

    <!-- In index.html (auth_enabled branch) -->
    <button 
      id="mark-page-read-btn"
      type="button" 
      hx-post="/hx/mark-page-read" 
      hx-target="#entries-list" 
      hx-swap="innerHTML" 
      hx-disabled-elt="this"
      hx-include="#active-status, #active-offset, [name='search']"
      hx-vals="js:{entry_ids: getVisibleEntryIds()}"
      class="p-2 rounded-full border border-border bg-card hover:bg-muted text-foreground transition-colors" 
      title="Mark This Page as Read"
      aria-label="Mark this page as read">
      <i data-lucide="check-check" class="w-4 h-4"></i>
    </button>

    <!-- In index.html (guest branch) -->
    <button 
      id="mark-page-read-btn"
      type="button" 
      onclick="markVisiblePageLocallyRead()" 
      class="p-2 rounded-full border border-border bg-card hover:bg-muted text-foreground transition-colors" 
      title="Mark This Page as Read"
      aria-label="Mark this page as read">
      <i data-lucide="check-check" class="w-4 h-4"></i>
    </button>
    ```

  - *Client Guest Mode Progression (`templates/pages/index.html`)*:
    ```javascript
    let isMarkingPageRead = false;

    function markVisiblePageLocallyRead() {
      if (isMarkingPageRead) return;
      isMarkingPageRead = true;
      const btn = document.getElementById('mark-page-read-btn');
      if (btn) btn.disabled = true;

      try {
        const ids = (typeof getVisibleEntryIds === 'function') ? getVisibleEntryIds() : '';
        const idList = typeof ids === 'string' ? ids.split(',').filter(Boolean) : (Array.isArray(ids) ? ids : []);
        if (idList.length > 0) {
          idList.forEach(id => markLocallyRead(id));
        } else {
          document.querySelectorAll('.annuaire-row').forEach(row => {
            const id = row.getAttribute('data-entry-id');
            if (id) markLocallyRead(id);
          });
        }
        if (typeof invalidateFilterCache === 'function') {
          invalidateFilterCache();
        }

        const nextBtn = document.getElementById('pagination-next-btn');
        if (nextBtn) {
          nextBtn.click();
        } else {
          showToast('Page marked as read. You are on the last page.');
        }
      } finally {
        setTimeout(() => {
          isMarkingPageRead = false;
          if (btn) btn.disabled = false;
        }, 300);
      }
    }
    ```

  - *Client Scroll Reset on Swap (`templates/pages/index.html`)*:
    ```javascript
    document.addEventListener('htmx:afterSwap', (evt) => {
      if (evt.detail && evt.detail.target && evt.detail.target.id === 'entries-list') {
        evt.detail.target.scrollTop = 0;
      }
      // ... existing cache & mode handling ...
    });
    ```

- **Pros**:
  - Delivers seamless user progression in all filter modes without skipping articles or jumping unexpectedly.
  - Automatically handles the final page with a helpful toast rather than an empty screen or error.
  - Preserves HTMX idiomatic out-of-band swap (`ToastTemplate`) without introducing additional polling or WebSocket connections.
  - Smooth scroll to top guarantees consistent reading ergonomics.
- **Cons**:
  - Requires a lightweight count query when advancing in `status != "unread"`.

### Option 2: Always Advance Offset (`offset + limit`) Regardless of Filter

- **Explanation**: Always increment `offset` by `limit` without checking the active filter.
- **Pros**: Minimal code on the server.
- **Cons**: Broken in `unread` mode. When 50 items are marked read, they leave the unread set. Items 50–99 become the new 0–49. Advancing to offset 50 queries items 100–149, completely skipping 50 unread articles!

**Recommended**: Option 1 is selected and confirmed.

---

## 3. Detailed Component Architecture

### 3.1 DOM & Template Wiring

1. **`crates/ui-components/templates/partials/entry_list.html`**:
   - Add `<input type="hidden" id="active-offset" name="offset" value="{{ offset }}">` at the very top of the template.
   - Because `offset: i64` is already a field in `EntryListTemplate`, this binds directly with no struct changes.
   - Add `id="pagination-next-btn"` to the Next button:
     ```html
     {% if has_next %}
     <button id="pagination-next-btn"
             hx-get="/hx/entries?status={{ status }}&search={{ search }}&offset={{ next_offset }}&limit={{ limit }}"
             ...
     ```
2. **`crates/ui-components/templates/pages/index.html`**:
   - In `#mark-page-read-btn` (`auth_enabled` branch):
     - Add `hx-disabled-elt="this"` to prevent rapid consecutive duplicate requests during in-flight HTMX operations.
     - Add `aria-label="Mark this page as read"` for accessibility.
     - Update `hx-include`:
       ```html
       hx-include="#active-status, #active-offset, [name='search']"
       ```
   - In `#mark-page-read-btn` (guest branch):
     - Add `aria-label="Mark this page as read"`.
   - In `markVisiblePageLocallyRead()`:
     - Add in-flight guard `isMarkingPageRead` and temporarily disable the button.
     - Correctly extract IDs whether `getVisibleEntryIds()` returns a string or array.
     - Call `markLocallyRead(id)` for each visible article.
     - Query `document.getElementById('pagination-next-btn')`. If found, call `nextBtn.click()`. If not found, call `showToast('Page marked as read. You are on the last page.')`.
   - In `htmx:afterSwap`:
     - If `evt.detail.target.id === 'entries-list'`, set `evt.detail.target.scrollTop = 0;`.

### 3.2 Server-Side Route Logic (`crates/server/src/routes/entries.rs`)

1. **`MarkPageReadForm`**:
   - Add `pub offset: Option<i64>`.
2. **`render_entries_list_html` Helper**:
   - Refactor `render_entries_list` into `render_entries_list_html` which returns `Result<String, Response>`, with `render_entries_list` calling it and returning `Html(html).into_response()`.
3. **`mark_page_read_htmx` Handler**:
   - Execute Miniflux status update mutation for `form.entry_ids`.
   - Invalidate `"entries:"` and `"index_page_entries"` caches.
   - If `status == "unread"` or None:
     - Set `target_offset = 0`, `show_last_page_toast = false`.
   - If `status != "unread"`:
     - Query `entriesResult(filter: { ... }) { total }` with `limit: 1, offset: 0`.
     - `let next_offset = current_offset + 50;`
     - If `next_offset < total`: `target_offset = next_offset`, `show_last_page_toast = false`.
     - Else: `target_offset = current_offset`, `show_last_page_toast = true`.
   - Call `render_entries_list_html(...)`.
   - If `show_last_page_toast`:
     - Render `ToastTemplate { message: "Page marked as read. You are on the last page.".to_string(), is_error: false }`.
     - Concatenate `toast_html` to `entries_html`.

---

## 4. Verification & Testing Strategy

### 4.1 Unit & Template Tests (`crates/ui-components/src/templates.rs`)
- Test that `EntryListTemplate` renders `<input type="hidden" id="active-offset" name="offset" value="X">` matching the template's `offset` field.
- Test that `EntryListTemplate` with `has_next: true` renders the Next button with `id="pagination-next-btn"`.
- Test that `IndexTemplate` with `auth_enabled: true` renders `#mark-page-read-btn` with `hx-include` containing `#active-offset`, `hx-disabled-elt="this"`, and `aria-label="Mark this page as read"`.
- Test that `IndexTemplate` with `auth_enabled: false` renders `#mark-page-read-btn` with `aria-label="Mark this page as read"`.

### 4.2 Server Route & Integration Tests (`crates/server/src/routes/entries.rs` & `crates/server/src/routes/mod.rs`)
- `test_mark_page_read_form_offset_deserialization`: Test that `MarkPageReadForm` deserializes `offset` from urlencoded form bodies.
- `test_hx_mark_page_read_advances_offset_when_more_pages_exist`: WireMock test for `status=all` with `total = 100`, `offset = 0`. Expect Miniflux GET with `offset=50` and verify response rendered offset 50.
- `test_hx_mark_page_read_stays_on_last_page_with_toast`: WireMock test for `status=all` with `total = 50`, `offset = 0`. Expect stay on offset 0 and verify response contains the last-page toast.
- `test_hx_mark_page_read_loads_offset_zero_for_unread`: WireMock test for `status=unread` with `offset = 50`. Expect query at `offset=0` to fetch the shifted unread batch.

### 4.3 Workspace Full Verification
- Run `cargo test --workspace`
- Run `cargo clippy --workspace --all-targets`

---

## Result

Result: done
