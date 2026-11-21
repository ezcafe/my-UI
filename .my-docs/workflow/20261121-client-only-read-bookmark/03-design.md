# Design: Client-Only Read & Bookmark (Guest Mode) and Server Mutation Rejection

Result: done

## Executive Summary

This design implements client-side guest interactions and server-side mutation protection for `AUTH_ENABLED=false`:
1. **ViewModel update**: `EntryDetailTemplate` receives `auth_enabled: bool` to enable template branching.
2. **Local client actions in `entry_detail.html`**:
   - Mark read/unread button omits `hx-post`; calls `onclick="toggleLocalReadStatus({{ entry.id }})"` to update IndexedDB and DOM indicators locally.
   - Bookmark button omits `hx-post`; calls `onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')"` to invoke the Web Share API or display a clipboard toast.
3. **Local bulk read in `index.html`**:
   - `#mark-page-read-btn` omits `hx-post` when `auth_enabled=false`; calls `onclick="markVisiblePageLocallyRead()"`.
4. **Server mutation rejection**:
   - `crates/server/src/routes/entries.rs` endpoints (`mark_entry_read_htmx`, `toggle_bookmark_htmx`, `mark_page_read_htmx`, `mark_feed_read_htmx`, and `mark_all_read_htmx`) reject with `HTTP 403 Forbidden` when `!config.auth_enabled`.

---

## Architectural Options

### Option A: Direct Template Branching & Strict HTTP 403 Server Guardrails (Recommended)

- **Explanation**:
  - `EntryDetailTemplate` receives `auth_enabled: bool` from `config.auth_enabled`.
  - In `entry_detail.html`, `{% if auth_enabled %}` renders standard HTMX attributes (`hx-post`), while `{% else %}` renders pure client-side `onclick` handlers (`toggleLocalReadStatus` and `triggerNativeBookmark`).
  - In `index.html`, `#mark-page-read-btn` uses `{% if auth_enabled %}` for `hx-post` and `{% else %}` for `onclick="markVisiblePageLocallyRead()"`.
  - In `crates/server/src/routes/entries.rs`, all five write endpoints check `if !config.auth_enabled { return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response(); }`.

- **Concrete Examples**:

  ```rust
  // crates/ui-components/src/templates.rs
  #[derive(Template)]
  #[template(path = "partials/entry_detail.html")]
  pub struct EntryDetailTemplate {
      pub entry: EntryViewModel,
      pub summary: String,
      pub sanitized_content: String,
      pub auth_enabled: bool,
  }
  ```

  ```html
  <!-- crates/ui-components/templates/partials/entry_detail.html -->
  {% if auth_enabled %}
  <button 
    hx-post="/hx/entries/{{ entry.id }}/read?status=unread" 
    hx-target="#entry-{{ entry.id }}" 
    hx-swap="outerHTML" 
    class="w-8 h-8 inline-flex items-center justify-center rounded-full border border-border bg-card hover:bg-muted text-muted-foreground hover:text-foreground transition-colors shrink-0"
    title="Mark as unread">
    <i data-lucide="circle-dot" class="w-4 h-4 shrink-0"></i>
  </button>
  <button 
    hx-post="/hx/entries/{{ entry.id }}/bookmark" 
    hx-target="this" 
    hx-swap="outerHTML" 
    class="w-8 h-8 inline-flex items-center justify-center rounded-full border border-border bg-card hover:bg-muted text-muted-foreground hover:text-foreground transition-colors shrink-0"
    title="Toggle bookmark">
    {% if entry.is_bookmarked %}
    <i data-lucide="bookmark" class="w-4 h-4 fill-foreground text-foreground shrink-0"></i>
    {% else %}
    <i data-lucide="bookmark" class="w-4 h-4 shrink-0"></i>
    {% endif %}
  </button>
  {% else %}
  <button 
    id="detail-toggle-read-btn"
    onclick="toggleLocalReadStatus({{ entry.id }})" 
    class="w-8 h-8 inline-flex items-center justify-center rounded-full border border-border bg-card hover:bg-muted text-muted-foreground hover:text-foreground transition-colors shrink-0"
    title="Toggle read status">
    <i data-lucide="circle-dot" class="w-4 h-4 shrink-0"></i>
  </button>
  <button 
    onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')" 
    class="w-8 h-8 inline-flex items-center justify-center rounded-full border border-border bg-card hover:bg-muted text-muted-foreground hover:text-foreground transition-colors shrink-0"
    title="Bookmark article">
    <i data-lucide="bookmark" class="w-4 h-4 shrink-0"></i>
  </button>
  {% endif %}
  ```

  ```rust
  // crates/server/src/routes/entries.rs
  pub async fn mark_entry_read_htmx(
      _user: AuthenticatedUser,
      State(schema): State<AppSchema>,
      State(config): State<Arc<Config>>,
      headers: HeaderMap,
      Path(id): Path<i64>,
      Query(query): Query<ReadStatusQuery>,
  ) -> Response {
      if !config.auth_enabled {
          return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
      }
      // ...
  }
  ```

- **Pros**:
  - **Zero unnecessary network traffic**: Guest mode does not make round-trip requests to the server for read/bookmark actions that the server cannot persist anyway.
  - **Immediate feedback**: Local state updates in IndexedDB and DOM happen instantly with zero latency.
  - **Robust security posture**: Server endpoints explicitly reject unauthorized mutation attempts with standard `HTTP 403 Forbidden`.
  - **Clean template readability**: Clear separation between server HTMX calls and local client handlers.

- **Cons**:
  - Requires updating `EntryDetailTemplate` usages across route handlers and unit tests to pass `auth_enabled`.

---

### Option B: Keep HTMX Handlers Everywhere and Return Client-Side JavaScript in Server Responses

- **Explanation**:
  - Keep `hx-post` on buttons even when `auth_enabled=false`.
  - The server endpoints check `if !config.auth_enabled` and return `HTTP 200 OK` with `<script>` tags or `HX-Trigger` headers directing the client to update IndexedDB.

- **Pros**:
  - Templates remain identical between auth-enabled and guest modes.

- **Cons**:
  - **Unnecessary latency**: Every local action requires an HTTP roundtrip to a server that cannot persist the data.
  - **Contradicts user requirement**: The user explicitly requested that `entry_detail.html` buttons omit `hx-post` and server reject mutations with `HTTP 403 Forbidden`.
  - **Flawed security semantics**: Returns 200 OK for operations that are forbidden in guest mode.

---

### Recommendation

**Option A** is strongly recommended. It directly matches the user's explicit design requirements, prevents useless network roundtrips in guest mode, provides immediate UI responsiveness, and protects backend resources with explicit `HTTP 403 Forbidden` responses.

---

## Detailed Component Specifications

### 1. `EntryDetailTemplate` & Route Wiring
- **File**: `crates/ui-components/src/templates.rs`
  - Add `pub auth_enabled: bool` to `EntryDetailTemplate`.
  - Update tests `test_render_entry_detail_template`, `test_render_entry_detail_sanitized_content`, and `test_render_entry_detail_with_brief_card` to include `auth_enabled: true` (or `false`).
- **File**: `crates/server/src/routes/entries.rs`
  - In `get_entry_detail_htmx`, construct `EntryDetailTemplate` with `auth_enabled: config.auth_enabled`.

### 2. Template Updates (`entry_detail.html` & `index.html`)
- **File**: `crates/ui-components/templates/partials/entry_detail.html`
  - Render read/unread and bookmark buttons based on `auth_enabled`.
  - When `auth_enabled=false`:
    - Read button: `onclick="toggleLocalReadStatus({{ entry.id }})"`.
    - Bookmark button: `onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')"`.
- **File**: `crates/ui-components/templates/pages/index.html`
  - Update `#mark-page-read-btn`:
    - `{% if auth_enabled %}`: maintains `hx-post="/hx/mark-page-read"` and associated HTMX attributes.
    - `{% else %}`: renders `onclick="markVisiblePageLocallyRead()"`.
  - Inside the `{% if !auth_enabled %}` script block, add:
    - `toggleLocalReadStatus(id)`: toggles entry presence in `readSet` and IndexedDB; updates DOM.
    - `markVisiblePageLocallyRead()`: marks all visible rows read in `readSet` and IndexedDB; syncs DOM.
    - `triggerNativeBookmark(title, url)`: invokes `navigator.share` if available, or copies `url` to clipboard and shows toast.

### 3. Server Endpoints Guardrail (`crates/server/src/routes/entries.rs`)
- In each of the following endpoints, add an early rejection check:
  - `mark_entry_read_htmx`
  - `toggle_bookmark_htmx`
  - `mark_page_read_htmx`
  - `mark_feed_read_htmx`
  - `mark_all_read_htmx`
- Check implementation:
  ```rust
  if !config.auth_enabled {
      return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
  }
  ```

---

## Reference Sources

- Route handlers: [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
- Template struct definitions: [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L73-L80)
- Entry detail template: [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)
- Index template: [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
- Server error handler: [crates/server/src/error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs)
