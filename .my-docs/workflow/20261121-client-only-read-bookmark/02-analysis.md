# Analysis: Client-Only Read & Bookmark (Guest Mode) and Server Mutation Rejection

Result: done

## Context & Architecture

The application is an AR Spatial RSS reader built with:
- **Backend**: Rust using Axum, Askama templates, async-graphql, and Miniflux client.
- **Frontend**: Server-rendered HTML with Tailwind CSS, HTMX for reactive updates, Lucide icons, and IndexedDB for local persistence.
- **Authentication**: Configured via `AUTH_ENABLED` (boolean). When `AUTH_ENABLED=false` (Guest Mode), user authentication is disabled, and readers browse feeds without logging into an upstream Miniflux account.

When running in Guest Mode (`AUTH_ENABLED=false`), upstream Miniflux write operations (such as marking an entry read or toggling bookmarks) cannot or should not modify server-side state. The user requested:
1. Both mark as read/unread and bookmarking should work locally on the client when `AUTH_ENABLED=false`.
2. The server must reject write/mutation requests with `HTTP 403 Forbidden` when `AUTH_ENABLED=false`.

---

## Detailed Technical Analysis

### 1. Template ViewModel Updates
- **File**: `crates/ui-components/src/templates.rs`
- **Current State**:
  - `EntryDetailTemplate` contains `pub entry: EntryViewModel`, `pub summary: String`, `pub sanitized_content: String`.
  - It does not contain an `auth_enabled: bool` field.
- **Target State**:
  - Add `pub auth_enabled: bool` to `EntryDetailTemplate`.
  - Update all test cases in `crates/ui-components/src/templates.rs` that construct `EntryDetailTemplate` to provide `auth_enabled`.
  - In `crates/server/src/routes/entries.rs::get_entry_detail_htmx`, populate `auth_enabled: config.auth_enabled`.

### 2. Client-Side Read & Bookmark Behavior (`AUTH_ENABLED=false`)
- **File**: `crates/ui-components/templates/partials/entry_detail.html`
  - **Mark as read/unread**:
    - When `auth_enabled=true`: retains `hx-post="/hx/entries/{{ entry.id }}/read?status=unread"`, `hx-target="#entry-{{ entry.id }}"`, `hx-swap="outerHTML"`.
    - When `auth_enabled=false`: omits `hx-post`; renders `onclick="toggleLocalReadStatus({{ entry.id }})"`.
  - **Bookmark button**:
    - When `auth_enabled=true`: retains `hx-post="/hx/entries/{{ entry.id }}/bookmark"`, `hx-target="this"`, `hx-swap="outerHTML"`.
    - When `auth_enabled=false`: omits `hx-post`; renders `onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')"`.
- **File**: `crates/ui-components/templates/pages/index.html`
  - **Mark Page Read button (`#mark-page-read-btn`)**:
    - When `auth_enabled=true`: retains `hx-post="/hx/mark-page-read"`.
    - When `auth_enabled=false`: omits `hx-post`; renders `onclick="markVisiblePageLocallyRead()"`.
  - **Guest Mode Client Scripts (`{% if !auth_enabled %}`)**:
    - `index.html` already maintains an IndexedDB store (`my_news_guest_db`, object store `read_entries`) and a memory `Set` (`readSet`).
    - Add `toggleLocalReadStatus(id)`: checks `readSet.has(numId)`. If present, calls `unmarkLocallyRead(numId)`; if absent, calls `markLocallyRead(numId)`. Toggles the visual state of the detail toggle button and list item indicators.
    - Add `markVisiblePageLocallyRead()`: fetches visible entry IDs via `getVisibleEntryIds()`, adds them to `readSet` and IndexedDB, and invokes `syncDomReadStates()`.
    - Add `triggerNativeBookmark(title, url)`: checks `navigator.share`. If available, calls `navigator.share({ title, url })`. Otherwise, falls back to copying the URL to clipboard and displaying a floating toast: `"Press Cmd/Ctrl+D to bookmark this article"`.

### 3. Server Mutation Rejection
- **File**: `crates/server/src/routes/entries.rs`
- **Target Endpoints**:
  1. `mark_entry_read_htmx` (`POST /hx/entries/:id/read`)
  2. `toggle_bookmark_htmx` (`POST /hx/entries/:id/bookmark`)
  3. `mark_page_read_htmx` (`POST /hx/mark-page-read`)
  4. `mark_feed_read_htmx` (`POST /hx/feeds/:id/read`)
  5. `mark_all_read_htmx` (`POST /hx/mark-all-read`)
- **Requirement**:
  - When `!config.auth_enabled`, each of these endpoints must immediately return `HTTP 403 Forbidden` (`StatusCode::FORBIDDEN`).
  - For HTMX callers, returning `(StatusCode::FORBIDDEN, "Mutations require authentication").into_response()` ensures the request fails cleanly without modifying server data or executing upstream Miniflux mutations.

---

## Risks & Mitigations

1. **DOM Sync Consistency**:
   - *Risk*: Marking an item as read or unread locally inside `entry_detail` might not immediately reflect in the open background list row `#entry-{id}`.
   - *Mitigation*: `toggleLocalReadStatus` directly calls `syncDomReadStates()` and updates both the row classes (`unread` / `read`) and unread dot indicator in `#entries-list`.
2. **Special Characters in Bookmark URL/Title**:
   - *Risk*: Unescaped quotes in title or url passed to `triggerNativeBookmark('...', '...')` could break JavaScript syntax.
   - *Mitigation*: Ensure template variables use Askama escaping or JSON serialization where appropriate.
3. **Template Regression in Existing Tests**:
   - *Risk*: Adding a field to `EntryDetailTemplate` breaks existing unit tests in `templates.rs`.
   - *Mitigation*: Update all unit tests instantiating `EntryDetailTemplate` to set `auth_enabled: true` or `false`.
