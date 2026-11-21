# 04-tasks: Implementation Plan & Planned Tests

Result: done

## 1. Planned Tests (TDD)

### 1.1 Backend Route Tests (`crates/server`)

- `test_build_graphql_filter_guest_unread_scopes_to_today`:
  - When `auth_enabled: false` and `status: Some("unread")`, verify that `build_graphql_filter` outputs `publishedAfter: <timestamp>` and does not output `status: UNREAD`.
  - When `auth_enabled: false` and status is default (`None` with empty search), verify `publishedAfter: <timestamp>` is applied.
  - When `auth_enabled: true` and `status: Some("unread")`, verify that `build_graphql_filter` outputs `status: UNREAD`.

### 1.2 UI Component & Template Tests (`crates/ui-components`)

- `test_index_template_hides_starred_but_shows_all_titles_when_auth_disabled`:
  - When `auth_enabled: false`:
    - Asserts rendered HTML does **not** contain `id="filter-starred"`.
    - Asserts rendered HTML **does** contain `id="filter-all"`.
    - Asserts rendered HTML contains `id="filter-unread"` and `id="filter-today"`.
  - When `auth_enabled: true`:
    - Asserts rendered HTML contains both `id="filter-starred"` and `id="filter-all"`.

- `test_open_in_browser_link_wires_mark_article_read`:
  - Renders `EntryItemTemplate` and verifies that the `<a>` link with `title="Open in browser"` contains `markArticleRead(42)`.
  - Renders `EntryListTemplate` and verifies that the `<a>` link with `title="Open in browser"` contains `markArticleRead(42)`.
  - Renders `EntryDetailTemplate` and verifies that the "Original" `<a>` link contains `markArticleRead(42)`.

- `test_index_template_defines_all_articles_store_and_helpers`:
  - Verifies that `IndexTemplate` defines `all_articles` store in IndexedDB.
  - Verifies that `renderLocalAllArticles` and `cacheLoadedArticlesFromDom` are defined.
  - Verifies that `#filter-unread-count` and `#unread-pill` include `data-base-unread`.

- `test_filter_click_closes_detail_panel`:
  - Verifies `closeDetailPanel` is called in filter click and search submit handlers.

---

## 2. Tasks

### Task 1: Backend Route Scoping for Guest Mode Unread
- **Red**: Add test `test_build_graphql_filter_guest_unread_scopes_to_today` in `crates/server/src/routes/entries.rs`. Run `cargo test -p server` (fails).
- **Green**:
  - In `crates/server/src/routes/entries.rs`, update `build_graphql_filter` to apply `publishedAfter: start_of_today` for unread/default filters when `!auth_enabled`.
  - In `crates/server/src/routes/pages.rs`, update `index_page` guest query to apply `publishedAfter: start_of_today` and map `unread_count` to `todayCounter`.
  - In `render_entries_list`, query both `unreadCounter` and `todayCounter`, returning `todayCounter` as unread count when `!auth_enabled`.
- **Verify**: Run `cargo test -p server` (passes).

### Task 2: Filter Toolbar Visibility - Show All Titles in Guest Mode
- **Red**: Update `test_index_template_hides_starred_but_shows_all_titles_when_auth_disabled` in `templates.rs`. Run `cargo test -p ui-components` (fails).
- **Green**:
  - In `crates/ui-components/templates/pages/index.html`, wrap only `#filter-starred` in `{% if auth_enabled %}`, leaving `#filter-all` rendered unconditionally.
- **Verify**: Run `cargo test -p ui-components` (passes).

### Task 3: Local IndexedDB All Titles (Option B) Storage & Client Rendering
- **Red**: Add test `test_index_template_defines_all_articles_store_and_helpers` in `templates.rs`. Run `cargo test -p ui-components` (fails).
- **Green**:
  - Bump `my_news_guest_db` to version 2; create object store `all_articles` in `initGuestDb()`.
  - Add `cacheLoadedArticlesFromDom()` to parse and upsert DOM rows into `all_articles`.
  - Add `renderLocalAllArticles()` to retrieve records from `all_articles`, build `.annuaire-row` markup, update `#entries-list`, and re-apply `syncDomReadStates()`.
  - Intercept `#filter-all` click in guest mode to call `renderLocalAllArticles()` without triggering HTMX network request.
- **Verify**: Run `cargo test -p ui-components` (passes).

### Task 4: Mark as Read on Browser Open
- **Red**: Add test `test_open_in_browser_link_wires_mark_article_read` in `templates.rs`. Run `cargo test -p ui-components` (fails).
- **Green**:
  - Add `markArticleRead(id)` helper in `index.html`.
  - Wire `onclick="event.stopPropagation(); markArticleRead({{ entry.id }})"` in `entry_item.html`, `entry_list.html`, and `entry_detail.html`.
- **Verify**: Run `cargo test -p ui-components` (passes).

### Task 5: Decouple Guest Unread Count from Filtered Rows
- **Red**: Verify `data-base-unread` assertions in `test_index_template_defines_all_articles_store_and_helpers`.
- **Green**:
  - Add `data-base-unread="{{ unread_count }}"` to `#filter-unread-count` and `#unread-pill` in `index.html` and `entry_list.html`.
  - Derive effective unread count in `syncDomReadStates()` as `Math.max(0, baseServerUnread - readSet.size)`.
- **Verify**: Run `cargo test -p ui-components` (passes).

### Task 6: Filter Button Clicks and Search Close Detail Panel
- **Red**: Assert `closeDetailPanel` presence in filter and search handlers in `templates.rs`.
- **Green**:
  - Invoke `closeDetailPanel()` inside `handleFilterClick(btn, event)`, `setActiveFilter(btn)`, and `onSearchSubmit(event)` in `index.html`.
- **Verify**: Run `cargo test -p ui-components` (passes).

---

## 3. Verification Command

```bash
cargo test -p server -p ui-components
```
