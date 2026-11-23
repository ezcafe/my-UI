# System Design: Unread Count, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

## 1. Executive Summary & Core Milestones

This design resolves four key user experience and system issues:
- **Milestone 1**: Fall back to `total` unread articles when server counters return 0 or null, ensuring the Unread badge accurately reflects visible articles.
- **Milestone 2**: Update `filterCache` when `#page-executive-brief` finishes loading, and re-bind HTMX via `htmx.process(entriesList)` upon cache restoration to eliminate frozen loading skeletons.
- **Milestone 3**: Update the AI title generation prompt from `<=10 words` to `strictly in one sentence`.
- **Milestone 4**: Throttle executive brief generation on the Unread filter so fresh briefs are generated only after reading 10 or more articles, otherwise displaying the latest existing brief.

---

## 2. Design Options & Trade-Offs

### Option 1: URL Query Parameter with Client-Side Read-Set Tracking (Recommended)

- **Explanation**: The client tracks articles read since the last brief in an in-memory `Set` (`readSinceLastBriefSet`). When clicking the Unread filter button, an HTMX request interceptor checks if `readSinceLastBriefSet.size >= 10`. If true, it sends `request_brief=true` and clears the set. If false, it sends `request_brief=false`. The server parses `request_brief: Option<bool>` in `EntriesFilterQuery`. When `false`, the server sets `is_brief_eligible = false` and serves the latest existing brief from SQLite (`top_executive_brief`).
- **Concrete Examples**:
  - *Client HTMX configuration (`crates/ui-components/templates/pages/index.html`)*:
    ```javascript
    const readSinceLastBriefSet = new Set();

    function recordReadArticle(id) {
      const numId = Number(id);
      if (numId) readSinceLastBriefSet.add(numId);
    }

    document.addEventListener('htmx:configRequest', (evt) => {
      const trigger = evt.detail && evt.detail.elt;
      if (trigger && trigger.id === 'filter-unread') {
        if (readSinceLastBriefSet.size >= 10) {
          evt.detail.parameters['request_brief'] = 'true';
          readSinceLastBriefSet.clear();
        } else {
          evt.detail.parameters['request_brief'] = 'false';
        }
      }
    });
    ```
  - *Server query parsing (`crates/server/src/routes/entries.rs`)*:
    ```rust
    #[derive(Debug, Deserialize, Default)]
    pub struct EntriesFilterQuery {
        pub status: Option<String>,
        pub feed_id: Option<i64>,
        pub category_id: Option<i64>,
        pub search: Option<String>,
        pub limit: Option<i64>,
        pub offset: Option<i64>,
        pub request_brief: Option<bool>,
    }

    let filter_eligible = is_brief_eligible_filter(query.status.as_deref(), query.search.as_deref());
    let request_brief = query.request_brief.unwrap_or(true);
    let is_eligible = filter_eligible && request_brief;

    let (ai_summaries, top_executive_brief) = if let Some(ai) = ai_service {
        let entry_ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
        let summaries = ai.db.get_batch_summaries(&entry_ids).await;
        let page_brief = if filter_eligible && !is_eligible {
            ai.db.get_latest_page_brief().await
        } else {
            None
        };
        (summaries, page_brief)
    } else {
        (std::collections::HashMap::new(), None)
    };
    ```
- **Pros**:
  - **Stateless backend**: The server remains completely stateless and does not need session cookies or server-side counters.
  - **Deduplication safety**: Using `Set.add()` ensures opening or clicking the same article multiple times does not falsely inflate the counter.
  - **Accurate context retention**: Serves existing `top_executive_brief` statically without showing a blank space or unnecessary loading spinner.
  - **Testability**: Query parameters can be easily tested with unit tests in `entries.rs`.
- **Cons**:
  - The counter resets if the browser tab undergoes a full reload (which is acceptable and expected for first-load briefing).

---

### Option 2: Server-Side Session / Cookie Tracking with Custom Header

- **Explanation**: The client sends an HTTP custom header `X-Read-Count` or the server increments a session cookie every time an article is marked read. When the Unread filter is clicked, the server inspects the cookie or header value. If `< 10`, it suppresses brief generation.
- **Concrete Examples**:
  - *Axum handler with HeaderMap*:
    ```rust
    pub async fn render_entries_list(
        headers: HeaderMap,
        Query(query): Query<EntriesFilterQuery>,
        ...
    ) {
        let read_count = headers
            .get("X-Read-Count")
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let request_brief = read_count >= 10;
        ...
    }
    ```
- **Pros**:
  - URL queries remain slightly shorter without `&request_brief=...`.
- **Cons**:
  - **State synchronization issues**: Cookies or custom headers complicate guest mode vs auth mode, creating inconsistency across tabs.
  - **Caching interference**: Custom headers are bypassed or stripped by intermediate reverse proxies and standard HTTP client caches.
  - **Higher test complexity**: Requires custom header mocking in tests rather than simple query deserialization.

---

### Recommended Option

> **Recommendation**: **Option 1**. It maintains server statelessness, provides deterministic unit testing, prevents duplicate counts via `Set`, and cleanly integrates with HTMX `configRequest`.

---

## 3. Detailed Component Architecture

### 3.1 Unread Count Fallback (`crates/server/src/routes/pages.rs` & `entries.rs`)

- In `pages.rs`:
  ```rust
  let mut unread_count = if config.auth_enabled {
      data.get("unreadCounter")
          .and_then(|v| v.as_i64())
          .unwrap_or(0) as i32
  } else {
      data.get("todayCounter")
          .and_then(|v| v.as_i64())
          .unwrap_or(0) as i32
  };

  // If counter is 0 but entriesResult contains unread entries, fall back to total
  if unread_count == 0 && total > 0 {
      unread_count = total as i32;
  }
  ```
- In `entries.rs`:
  ```rust
  let mut unread_count = if auth_enabled {
      data.get("unreadCounter").and_then(|v| v.as_i64()).map(|c| c as i32)
  } else {
      data.get("todayCounter").and_then(|v| v.as_i64()).map(|c| c as i32)
  };

  if unread_count.unwrap_or(0) == 0 && total > 0 && query.status.as_deref() == Some("unread") {
      unread_count = Some(total as i32);
  }
  ```
- In `index.html` (`syncDomReadStates()`):
  ```javascript
  let effectiveUnread = Math.max(0, baseServerUnread - readSet.size);
  if (isUnreadFilter && effectiveUnread === 0 && visibleUnreadRowCount > 0) {
    effectiveUnread = visibleUnreadRowCount;
  }
  ```

### 3.2 Brief Cache Updates & HTMX Processing (`crates/ui-components/templates/pages/index.html`)

- In `htmx:afterSwap`:
  ```javascript
  const targetId = evt.detail.target.id || '';
  const isEntriesSwap = targetId === 'entries-list';
  const isBriefSwap = targetId === 'page-executive-brief' || (evt.detail.target && evt.detail.target.closest('#entries-list') && evt.detail.target.id !== 'detail-panel');

  if (isEntriesSwap || isBriefSwap) {
    const entriesListEl = document.getElementById('entries-list');
    const activeStatusInput = document.getElementById('active-status');
    const status = activeStatusInput ? activeStatusInput.value : null;
    const searchInput = document.getElementById('search-input');
    const searchQuery = (searchInput && searchInput.value) ? searchInput.value.trim() : '';

    if (status && !searchQuery && entriesListEl) {
      filterCache.set(status, {
        timestamp: Date.now(),
        html: entriesListEl.innerHTML,
      });
    }
    if (typeof syncDomReadStates === 'function') {
      syncDomReadStates();
    }
  }
  ```
- In `applyCachedList()`:
  ```javascript
  function applyCachedList(cachedHtml) {
    const entriesList = document.getElementById('entries-list');
    if (entriesList) {
      entriesList.innerHTML = cachedHtml;
      if (window.lucide && typeof lucide.createIcons === 'function') {
        lucide.createIcons({ root: entriesList });
      }
      if (window.htmx && typeof htmx.process === 'function') {
        htmx.process(entriesList);
      }
      if (typeof syncDomReadStates === 'function') {
        syncDomReadStates();
      }
    }
  }
  ```

### 3.3 Suggested Title One Sentence Restriction (`crates/server/src/ai/llm.rs`)

- In `format_batch_prompt_structured`:
  Update line 78 prompt template:
  ```rust
  "2. \"items\": For each NEW article id, provide a punchy \"t\" (suggested title, strictly in one sentence) and \"b\" (executive brief, strictly 3 sentences, what changed/why it matters). Do not regenerate items for existing briefs.\n\n"
  ```
- Add unit test asserting prompt contains `strictly in one sentence` and does not contain `<=10 words`.

### 3.4 Brief Throttling (`crates/server/src/routes/entries.rs`, `index.html`)

- Add `request_brief: Option<bool>` to `EntriesFilterQuery`.
- In `render_entries_list`:
  - Determine `filter_eligible = is_brief_eligible_filter(...)`.
  - Check `request_brief = query.request_brief.unwrap_or(true)`.
  - `is_eligible = filter_eligible && request_brief`.
  - When `filter_eligible && !is_eligible`, load `ai.db.get_latest_page_brief().await` into `top_executive_brief`.
  - Pass `top_executive_brief` to `EntryListTemplate`.

---

## 4. Verification & Testing Strategy

1. **Unit Tests (`crates/server/src/routes/entries.rs`)**:
   - `test_entries_filter_query_request_brief_deserialization`: Verifies that `request_brief` parses `true`, `false`, and `None`.
   - `test_unread_count_fallback_to_total_when_zero`: Verifies fallback to `total` when counter is 0.
2. **Unit Tests (`crates/server/src/ai/llm.rs`)**:
   - `test_format_batch_prompt_suggested_title_one_sentence`: Verifies prompt requires title "strictly in one sentence".
3. **Template & Client-Side Verification (`crates/ui-components/src/templates.rs`)**:
   - Verify `EntryListTemplate` renders static `top_executive_brief` when provided and `is_brief_eligible` is false.
   - Verify `filter-unread` button and script logic in `index.html`.

## Result

Result: done
