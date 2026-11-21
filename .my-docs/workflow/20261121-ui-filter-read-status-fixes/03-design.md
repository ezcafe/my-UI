# 03-design: Guest Unread Scope, Local All Titles (Option B), and UI Filter Fixes

- **Result:** done

## 1. Key Takeaways & Conclusions

- **Guest mode "Unread" scope**: In `crates/server/src/routes/entries.rs` (`build_graphql_filter`) and `crates/server/src/routes/pages.rs` (`index_page`), when `!auth_enabled` and status is `"unread"` or default, apply `publishedAfter: start_of_today` in the GraphQL filter to Miniflux. In guest mode, unread items are scoped to today's incoming news, and `unread_count` reflects `todayCounter`.
- **Show "All Titles" in guest mode via IndexedDB (Option B)**:
  - In `crates/ui-components/templates/pages/index.html`, `#filter-all` is displayed unconditionally. `#filter-starred` remains wrapped in `{% if auth_enabled %}`.
  - Bump `my_news_guest_db` to version 2, creating an object store `all_articles` with `keyPath: 'id'`.
  - Every batch of articles loaded into `#entries-list` (initial load or HTMX swap) is automatically cached into `all_articles`.
  - Clicking "All Titles" in guest mode intercepts network requests, retrieves all stored records from `all_articles`, sorts them by publish date descending, and renders them directly into `#entries-list` using `.annuaire-row` markup.
- **Mark as read on browser open**: The unified `markArticleRead(id)` helper optimistically clears unread dots/bars and updates `markLocallyRead(id)` (guest mode) or `/hx/entries/{id}/read` (auth mode). Wired to `<a title="Open in browser">` in `entry_item.html`, `entry_list.html`, and `entry_detail.html`.
- **Guest unread count decoupling**: `syncDomReadStates()` derives total unread as `Math.max(0, baseServerUnread - readSet.size)`. Visible row counting is strictly used for "Unread" tab row visibility and `#client-empty-state`.
- **Filter click closes detail panel**: All filter button handlers and search submissions invoke `closeDetailPanel()` to return cleanly to the list view.

---

## 2. Options Comparison

### Option A: Server-Backed All Titles in Guest Mode

- **Explanation**:
  - In guest mode, clicking "All Titles" sends a query to the server for all entries without status filter (`status: None`).
  - The server queries Miniflux for global entries, and client-side `readSet` applies read styles to visible rows.
- **Pros**:
  - Requires no client-side IndexedDB article storage or custom client-side row rendering.
- **Cons**:
  - Miniflux contains historical feeds that guest users cannot clear or mark read on the server.
  - Conflicts with guest user expectations of an offline/local-first reading history.
  - High bandwidth and query load for transient guest sessions.

### Option B: Local IndexedDB "All Titles" (Selected & Recommended)

- **Explanation**:
  - In guest mode, every article presented to the user is cached into IndexedDB (`my_news_guest_db`, store `all_articles`).
  - Clicking "All Titles" in guest mode bypasses server calls completely. It fetches all records from `all_articles`, renders them directly into `#entries-list`, marks them read/unread according to `readSet`, and attaches event handlers.
  - Scopes guest data entirely to the local browser instance, giving the user a persistent offline archive of titles they have interacted with.
- **Concrete Examples**:

  *IndexedDB Schema Initialization (`index.html`)*:
  ```javascript
  const DB_NAME = 'my_news_guest_db';
  const DB_VERSION = 2;
  const STORE_READ = 'read_entries';
  const STORE_ARTICLES = 'all_articles';

  function initGuestDb() {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = (e) => {
      const db = e.target.result;
      if (!db.objectStoreNames.contains(STORE_READ)) {
        db.createObjectStore(STORE_READ, { keyPath: 'id' });
      }
      if (!db.objectStoreNames.contains(STORE_ARTICLES)) {
        db.createObjectStore(STORE_ARTICLES, { keyPath: 'id' });
      }
    };
    req.onsuccess = (e) => {
      const db = e.target.result;
      const tx = db.transaction(STORE_READ, 'readonly');
      const getAllReq = tx.objectStore(STORE_READ).getAllKeys();
      getAllReq.onsuccess = (ev) => {
        readSet = new Set(ev.target.result);
        syncDomReadStates();
      };
      cacheLoadedArticlesFromDom();
    };
  }
  ```

  *Caching Loaded Articles into `all_articles`*:
  ```javascript
  function cacheLoadedArticlesFromDom() {
    const rows = document.querySelectorAll('#entries-list .annuaire-row');
    if (!rows.length) return;
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onsuccess = (e) => {
      const db = e.target.result;
      const tx = db.transaction(STORE_ARTICLES, 'readwrite');
      const store = tx.objectStore(STORE_ARTICLES);
      rows.forEach(row => {
        const id = Number(row.getAttribute('data-entry-id'));
        if (!id) return;
        const titleEl = row.querySelector('.entry-title-text') || row.querySelector('h3') || row.querySelector('a');
        const title = titleEl ? titleEl.textContent.trim() : '';
        const urlEl = row.querySelector('a[title="Open in browser"]');
        const url = urlEl ? urlEl.getAttribute('href') : '';
        const timeEl = row.querySelector('time');
        const publishedAt = timeEl ? timeEl.getAttribute('datetime') || timeEl.textContent.trim() : '';
        const readingTimeEl = row.querySelector('.reading-time');
        const readingTime = readingTimeEl ? readingTimeEl.textContent.trim() : '';
        const summaryEl = row.querySelector('.entry-summary');
        const summary = summaryEl ? summaryEl.textContent.trim() : '';

        store.put({
          id,
          title,
          url,
          publishedAt,
          readingTime,
          summary,
          cachedAt: Date.now()
        });
      });
    };
  }
  ```

  *Client-Side Rendering of "All Titles"*:
  ```javascript
  function renderLocalAllArticles() {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onsuccess = (e) => {
      const db = e.target.result;
      const tx = db.transaction(STORE_ARTICLES, 'readonly');
      const getAll = tx.objectStore(STORE_ARTICLES).getAll();
      getAll.onsuccess = (ev) => {
        const articles = ev.target.result || [];
        // Sort newest first
        articles.sort((a, b) => (b.id - a.id));
        const listEl = document.getElementById('entries-list');
        if (!listEl) return;

        if (articles.length === 0) {
          listEl.innerHTML = `
            <div id="client-empty-state" class="flex flex-col items-center justify-center p-12 text-center text-muted-foreground">
              <i data-lucide="inbox" class="w-12 h-12 mb-3 stroke-1 text-muted-foreground/60"></i>
              <p class="text-sm font-medium font-sans">No saved titles yet</p>
              <p class="text-xs text-muted-foreground mt-1 font-sans">Articles you browse will appear here.</p>
            </div>
          `;
        } else {
          listEl.innerHTML = articles.map(art => renderArticleRowHtml(art)).join('');
        }
        if (window.lucide && typeof lucide.createIcons === 'function') {
          lucide.createIcons();
        }
        syncDomReadStates();
      };
    };
  }
  ```

- **Pros**:
  - Zero server load for "All Titles" in guest mode.
  - Matches user mental model: guest mode acts as a self-contained local workspace.
  - Resilient to network outages; works offline for browsed articles.
- **Cons**:
  - Adds ~50 lines of client-side DOM builder and IndexedDB transaction handling to `index.html`.

---

## 3. Technical Architecture & Changes

### 3.1 Backend Server Changes (`crates/server`)

1. **`crates/server/src/routes/entries.rs` (`build_graphql_filter`)**:
   ```rust
   Some("unread") => {
       if auth_enabled {
           filter_parts.push("status: UNREAD".to_string());
       } else {
           let start_of_today = chrono::Utc::now()
               .date_naive()
               .and_hms_opt(0, 0, 0)
               .unwrap()
               .and_utc()
               .timestamp();
           filter_parts.push(format!("publishedAfter: {}", start_of_today));
       }
   },
   _ => {
       if auth_enabled {
           filter_parts.push("status: UNREAD".to_string());
       } else {
           let start_of_today = chrono::Utc::now()
               .date_naive()
               .and_hms_opt(0, 0, 0)
               .unwrap()
               .and_utc()
               .timestamp();
           filter_parts.push(format!("publishedAfter: {}", start_of_today));
       }
   }
   ```

2. **`crates/server/src/routes/pages.rs` (`index_page`)**:
   - In guest mode query:
     ```rust
     let start_of_today = chrono::Utc::now()
         .date_naive()
         .and_hms_opt(0, 0, 0)
         .unwrap()
         .and_utc()
         .timestamp();
     let gql = format!(r#"query {{
         unreadCounter
         todayCounter
         entriesResult(filter: {{ limit: 50, publishedAfter: {start_of_today} }}) {{
             total
             entries {{
                 id title summary publishedAt readingTime isBookmarked url status
             }}
         }}
     }}"#);
     ```
   - In guest mode, `unread_count` is set from `todayCounter`.

3. **`crates/server/src/routes/entries.rs` (`render_entries_list`)**:
   - Query both `unreadCounter` and `todayCounter`.
   - Set `unread_count = if auth_enabled { unreadCounter } else { todayCounter }`.

### 3.2 UI Templates (`crates/ui-components`)

1. **`crates/ui-components/templates/pages/index.html`**:
   - Unconditionally render `#filter-all`.
   - Only wrap `#filter-starred` in `{% if auth_enabled %}`:
     ```html
     {% if auth_enabled %}
     <button id="filter-starred" data-status="starred" ...>Starred</button>
     {% endif %}
     <button id="filter-all" data-status="all" ...>All Titles</button>
     ```
   - Update `handleFilterClick(btn, event)`:
     - Always call `closeDetailPanel()`.
     - In guest mode (`!auth_enabled`), if `btn.dataset.status === 'all'`:
       - `event.preventDefault()`;
       - `setActiveFilter(btn)`;
       - `renderLocalAllArticles()`.
   - On `DOMContentLoaded` and `htmx:afterSwap`, call `cacheLoadedArticlesFromDom()` in guest mode.
   - Retain `markArticleRead(id)` and `syncDomReadStates()` decoupling with `data-base-unread`.

2. **`crates/ui-components/templates/partials/entry_item.html` & `entry_list.html`**:
   - Wire `<a title="Open in browser" onclick="event.stopPropagation(); markArticleRead({{ entry.id }})">`.
   - Ensure `data-base-unread="{{ unread }}"` is present on `#unread-pill`.

3. **`crates/ui-components/templates/partials/entry_detail.html`**:
   - Wire "Original" link `<a onclick="markArticleRead({{ entry.id }})">`.

---

## 4. Has Flags & Lifecycle Signals

- **Has UI**: `yes`
- **Has API**: `yes` (Modifies query parameters in `crates/server/src/routes/entries.rs` and `pages.rs`; candidate `tiny-contract-tweak` applies as blast radius is internal route query construction).
- **Has DB**: `no` (IndexedDB is browser client-side storage, no server database involved).
- **Has auth/PII/schema**: `no`

---

## 5. Verification & References

- [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) — Filter buttons, IndexedDB client stores, and client renderers.
- [entry_item.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html) — External link wire-up.
- [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html) — List item external link wire-up.
- [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html) — Detail panel link wire-up.
- [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) — GraphQL query builder and route handler.
- [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs) — Initial page load query.
- [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) — Template unit tests.
