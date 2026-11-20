# Design: Title-Only List, External Link, Today Filter & Search/Filter Fixes

- **Result:** done

## 1. Architecture Overview

The solution spans three tiers within the existing architecture:
1. **Miniflux & GraphQL Tier (`crates/miniflux`)**: Add `published_after` support to `EntriesFilter`, `EntryFilterInput`, and `MinifluxClient::get_entries`.
2. **Server Route Tier (`crates/server/src/routes/entries.rs`)**:
   - Parse `search` parameter and escape it into the GraphQL `entries(filter: { search: "..." })` query.
   - Expand `status` handling:
     - `"all"`: no status restriction
     - `"unread"`: `status: UNREAD`
     - `"starred"`: `starred: true`
     - `"today"`: `publishedAfter: <midnight_utc_timestamp>` (all published today, read & unread per user decision 1B)
     - `"read"`: `status: READ`
3. **UI & Templates Tier (`crates/ui-components`)**:
   - `entry_item.html` & `entry_list.html`: Standardize on title-only spatial row matching `ar-list.html`.
   - `index.html`: Order filter pills (`All Titles`, `Unread`, `Today`, `Starred`), add active pill styling toggle, and wire search with `hx-include`.

```
[ Browser UI ]
   ├── Search Input (hx-get="/hx/entries" hx-trigger="keyup changed delay:300ms, search" hx-include="[name='status']")
   ├── Filter Pills: All Titles | Unread | Today | Starred (onclick=setActiveFilter(this) hx-get="/hx/entries?status=...")
   └── Title-Only Row
         ├── Title (line-clamp-2, click -> opens reader modal)
         ├── Unread dot indicator
         └── External Link Button (<a target="_blank" rel="noopener noreferrer" onclick="event.stopPropagation()">)
                   │
                   ▼ (HTTP GET /hx/entries)
[ Axum Route: list_entries_htmx ]
   ├── Maps status query to GraphQL filter (status, starred, publishedAfter)
   ├── Injects search query string into GraphQL filter
   └── Executes GraphQL query against AppSchema
                   │
                   ▼
[ GraphQL / Miniflux Client ]
   ├── Builds /v1/entries request with query params
   └── Returns entries & unread count
```

## 2. Component Specifications

### 2.1 Miniflux Schema & Client Extensions
- File: [crates/miniflux/src/models.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/models.rs)
  ```rust
  pub struct EntriesFilter {
      pub status: Option<String>,
      pub feed_id: Option<i64>,
      pub category_id: Option<i64>,
      pub starred: Option<bool>,
      pub search: Option<String>,
      pub published_after: Option<i64>,
      pub limit: Option<i64>,
      pub offset: Option<i64>,
      pub order: Option<String>,
      pub direction: Option<String>,
  }
  ```
- File: [crates/miniflux/src/client.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/client.rs)
  ```rust
  if let Some(published_after) = filter.published_after {
      req = req.query(&[("published_after", published_after.to_string())]);
  }
  ```
- File: [crates/miniflux/src/schema/types.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/schema/types.rs)
  ```rust
  #[derive(InputObject, Clone, Debug, Default)]
  pub struct EntryFilterInput {
      pub status: Option<EntryStatus>,
      pub feed_id: Option<ID>,
      pub category_id: Option<ID>,
      pub starred: Option<bool>,
      pub search: Option<String>,
      pub published_after: Option<i64>,
      pub limit: Option<i32>,
      pub offset: Option<i32>,
      pub order: Option<String>,
      pub direction: Option<OrderDirection>,
  }
  ```
- File: [crates/miniflux/src/schema/query.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/schema/query.rs)
  ```rust
  if let Some(pa) = f.published_after {
      domain_filter.published_after = Some(pa);
  }
  ```

### 2.2 Server Route Filter & Search Resolution
- File: [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
- Build GraphQL filter fragments:
  ```rust
  let mut filter_parts = Vec::new();

  // Status & Filter mapping
  match query.status.as_deref() {
      Some("all") => {},
      Some("read") => filter_parts.push("status: READ".to_string()),
      Some("starred") => filter_parts.push("starred: true".to_string()),
      Some("today") => {
          let start_of_today = chrono::Utc::now()
              .date_naive()
              .and_hms_opt(0, 0, 0)
              .unwrap()
              .and_utc()
              .timestamp();
          // User decision 1B: show all published today (both read and unread)
          filter_parts.push(format!("publishedAfter: {}", start_of_today));
      },
      _ => filter_parts.push("status: UNREAD".to_string()), // Default: unread
  }

  // Search mapping
  if let Some(ref search) = query.search {
      let trimmed = search.trim();
      if !trimmed.is_empty() {
          let escaped = trimmed.replace('\\', "\\\\").replace('"', "\\\"");
          filter_parts.push(format!(r#"search: "{}""#, escaped));
      }
  }

  filter_parts.push(format!("limit: {}", limit_val));
  filter_parts.push(format!("offset: {}", offset_val));
  let filter_str = filter_parts.join(", ");
  ```

### 2.3 UI & Templates Updates
- File: [crates/ui-components/templates/partials/entry_item.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html) & [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
  ```html
  <div class="ar-list-item px-4 py-3 rounded-2xl flex items-center justify-between gap-4 cursor-pointer select-none group" id="entry-{{ entry.id }}">
    <div class="flex items-center gap-2.5 flex-1 min-w-0" hx-get="/hx/entries/{{ entry.id }}" hx-target="#detail-panel" onclick="document.getElementById('detail-panel').classList.remove('hidden')">
      {% if !entry.is_read %}
      <span class="w-2 h-2 rounded-full bg-indigo-400 ring-4 ring-indigo-500/20 shrink-0 shadow-sm shadow-indigo-400" title="Unread"></span>
      {% endif %}
      <span class="text-sm font-medium leading-snug line-clamp-2 text-slate-100 group-hover:text-indigo-200 transition-colors flex-1">
        {{ entry.title }}
      </span>
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <a 
        href="{{ entry.url }}" 
        target="_blank" 
        rel="noopener noreferrer" 
        onclick="event.stopPropagation()" 
        class="p-1.5 rounded-full hover:bg-white/10 text-slate-400 hover:text-indigo-300 transition-colors"
        title="Open in browser">
        <i data-lucide="external-link" class="w-4 h-4"></i>
      </a>
      <i data-lucide="chevron-right" class="w-4 h-4 text-slate-500 group-hover:text-slate-300 transition-colors"></i>
    </div>
  </div>
  ```

- File: [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - Quick Filter toolbar with order: `All Titles`, `Unread`, `Today`, `Starred`:
  ```html
  <div class="flex items-center gap-2">
    <button 
      id="filter-all"
      hx-get="/hx/entries?status=all" 
      hx-target="#entries-list" 
      hx-swap="innerHTML" 
      onclick="setActiveFilter(this)"
      class="filter-pill px-3 py-1 rounded-full text-xs font-medium bg-white/5 text-slate-400 hover:text-white border border-white/10 transition-colors">
      All Titles
    </button>
    <button 
      id="filter-unread"
      hx-get="/hx/entries?status=unread" 
      hx-target="#entries-list" 
      hx-swap="innerHTML" 
      onclick="setActiveFilter(this)"
      class="filter-pill px-3 py-1 rounded-full text-xs font-medium bg-indigo-500/20 text-indigo-300 border border-indigo-400/30 hover:bg-indigo-500/30 transition-colors">
      Unread
    </button>
    <button 
      id="filter-today"
      hx-get="/hx/entries?status=today" 
      hx-target="#entries-list" 
      hx-swap="innerHTML" 
      onclick="setActiveFilter(this)"
      class="filter-pill px-3 py-1 rounded-full text-xs font-medium bg-white/5 text-slate-400 hover:text-white border border-white/10 transition-colors">
      Today
    </button>
    <button 
      id="filter-starred"
      hx-get="/hx/entries?status=starred" 
      hx-target="#entries-list" 
      hx-swap="innerHTML" 
      onclick="setActiveFilter(this)"
      class="filter-pill px-3 py-1 rounded-full text-xs font-medium bg-white/5 text-slate-400 hover:text-white border border-white/10 transition-colors">
      Starred
    </button>
  </div>
  ```
  - Active filter script:
  ```html
  <script>
    function setActiveFilter(btn) {
      document.querySelectorAll('.filter-pill').forEach(el => {
        el.classList.remove('bg-indigo-500/20', 'text-indigo-300', 'border-indigo-400/30', 'bg-white/10', 'text-white', 'border-white/20');
        el.classList.add('bg-white/5', 'text-slate-400', 'border-white/10');
      });
      btn.classList.add('bg-indigo-500/20', 'text-indigo-300', 'border-indigo-400/30');
      btn.classList.remove('bg-white/5', 'text-slate-400');
    }
  </script>
  ```

## 3. Verification & Safety Guarantees

- **TDD Compliance**: Unit tests in `ui-components` verify HTML rendering of title-only format, external link attributes (`target="_blank"`, `rel="noopener noreferrer"`), and chevron.
- **Route Tests**: Route unit tests verify status and search parameter mapping to GraphQL filter strings.
- **Miniflux Client Tests**: Tests verify `published_after` query parameter generation in `MinifluxClient`.
