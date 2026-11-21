# Tasks: Client-Only Read & Bookmark (Guest Mode) and Server Mutation Rejection

Result: done

## Tasks Breakdown

### Phase 1: ViewModel & Template Wiring (UI Components & Server)
- [x] Task 1.1: Add `pub auth_enabled: bool` to `EntryDetailTemplate` in `crates/ui-components/src/templates.rs`.
- [x] Task 1.2: Update existing unit tests in `crates/ui-components/src/templates.rs` (`test_render_entry_detail_template`, `test_render_entry_detail_sanitized_content`, and `test_render_entry_detail_with_brief_card`) to supply `auth_enabled: true`.
- [x] Task 1.3: Update `get_entry_detail_htmx` in `crates/server/src/routes/entries.rs` to initialize `auth_enabled: config.auth_enabled`.

### Phase 2: Server Mutation Rejection Guardrails (Server)
- [x] Task 2.1: Add `if !config.auth_enabled { return (StatusCode::FORBIDDEN, ...).into_response(); }` check to `mark_entry_read_htmx` in `crates/server/src/routes/entries.rs`.
- [x] Task 2.2: Add `if !config.auth_enabled { return (StatusCode::FORBIDDEN, ...).into_response(); }` check to `toggle_bookmark_htmx` in `crates/server/src/routes/entries.rs` (replacing the previous fallback button render).
- [x] Task 2.3: Add `if !config.auth_enabled { return (StatusCode::FORBIDDEN, ...).into_response(); }` check to `mark_page_read_htmx` in `crates/server/src/routes/entries.rs`.
- [x] Task 2.4: Add `if !config.auth_enabled { return (StatusCode::FORBIDDEN, ...).into_response(); }` check to `mark_feed_read_htmx` and `mark_all_read_htmx` in `crates/server/src/routes/entries.rs`.

### Phase 3: HTML Templates & Client Script Updates (UI Components)
- [x] Task 3.1: Update `crates/ui-components/templates/partials/entry_detail.html` so that when `auth_enabled=false`, the mark read/unread button omits `hx-post` and uses `onclick="toggleLocalReadStatus({{ entry.id }})"`.
- [x] Task 3.2: Update `crates/ui-components/templates/partials/entry_detail.html` so that when `auth_enabled=false`, the bookmark button omits `hx-post` and uses `onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')"`.
- [x] Task 3.3: Update `crates/ui-components/templates/pages/index.html` so that `#mark-page-read-btn` omits `hx-post` when `auth_enabled=false` and uses `onclick="markVisiblePageLocallyRead()"`.
- [x] Task 3.4: In `crates/ui-components/templates/pages/index.html` under `{% if !auth_enabled %}`, implement `toggleLocalReadStatus(id)`, `markVisiblePageLocallyRead()`, and `triggerNativeBookmark(title, url)`.

### Phase 4: Automated Testing & Verification
- [x] Task 4.1: Add unit tests in `crates/ui-components/src/templates.rs` verifying rendered HTML for `EntryDetailTemplate` with `auth_enabled=true` and `auth_enabled=false`.
- [x] Task 4.2: Add integration tests in `crates/server/tests` or `crates/server/src/routes/entries.rs` tests verifying HTTP 403 Forbidden is returned for mutation endpoints when `auth_enabled=false`.
- [x] Task 4.3: Run `cargo check` and `cargo test` across workspace to ensure no regressions.

---

## Planned Tests

1. `test_entry_detail_renders_local_actions_when_auth_disabled`:
   - Validates that when `auth_enabled: false`, `EntryDetailTemplate` does not render `hx-post="/hx/entries/.../read"` or `hx-post="/hx/entries/.../bookmark"`, and instead renders `onclick="toggleLocalReadStatus(...)"` and `onclick="triggerNativeBookmark(...)"`.
2. `test_entry_detail_renders_htmx_actions_when_auth_enabled`:
   - Validates that when `auth_enabled: true`, `EntryDetailTemplate` renders standard `hx-post` actions for read and bookmark toggles.
3. `test_mark_entry_read_htmx_returns_403_when_auth_disabled`:
   - Validates that `POST /hx/entries/:id/read` returns `HTTP 403 Forbidden` when `config.auth_enabled == false`.
4. `test_toggle_bookmark_htmx_returns_403_when_auth_disabled`:
   - Validates that `POST /hx/entries/:id/bookmark` returns `HTTP 403 Forbidden` when `config.auth_enabled == false`.
5. `test_mark_page_read_htmx_returns_403_when_auth_disabled`:
   - Validates that `POST /hx/mark-page-read` returns `HTTP 403 Forbidden` when `config.auth_enabled == false`.
6. `test_mark_feed_read_htmx_returns_403_when_auth_disabled`:
   - Validates that `POST /hx/feeds/:id/read` returns `HTTP 403 Forbidden` when `config.auth_enabled == false`.
7. `test_mark_all_read_htmx_returns_403_when_auth_disabled`:
   - Validates that `POST /hx/mark-all-read` returns `HTTP 403 Forbidden` when `config.auth_enabled == false`.
8. `test_index_template_mark_page_read_button_auth_toggle`:
   - Validates that `IndexTemplate` renders `hx-post="/hx/mark-page-read"` when `auth_enabled: true`, and renders `onclick="markVisiblePageLocallyRead()"` without `hx-post` when `auth_enabled: false`.
