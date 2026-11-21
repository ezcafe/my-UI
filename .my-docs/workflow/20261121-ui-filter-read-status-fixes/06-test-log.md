# Test Execution Log

- **Result:** success
- **Test path:** lite
- **Stage:** test-lite
- **Date:** 2026-11-21

## Summary

`test-lite` suite ran targeted tests across `server` and `ui-components` validating Option B (Guest unread scoped to today + all titles stored in local IndexedDB). All 84 unit tests (51 server + 33 ui-components) and 4 server e2e tests passed cleanly with 0 failures or regressions.

| Crate / Target | Tests | Passed | Failed | Result |
|---|---|---|---|---|
| `server` (lib unittests) | 51 | 51 | 0 | success |
| `server` (e2e tests) | 4 | 4 | 0 | success |
| `ui-components` (lib unittests) | 33 | 33 | 0 | success |

---

## Detailed Test Breakdown

### 1. Backend Verification (`crates/server`)
- `test_build_graphql_filter_guest_unread_scopes_to_today`:
  - Verified `build_graphql_filter` applies `publishedAfter: <start_of_today>` and omits `status: UNREAD` when `auth_enabled: false` and `status: Some("unread")`.
  - Verified `build_graphql_filter` applies `publishedAfter: <start_of_today>` and omits `status: UNREAD` when `auth_enabled: false` and `status: None` (default view).
  - Verified `build_graphql_filter` outputs `status: UNREAD` and no `publishedAfter` restriction when `auth_enabled: true`.
- `test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled`:
  - Verified guest mode entry detail fetch does not mutate upstream Miniflux read status.
- `test_mutation_endpoints_reject_with_403_when_auth_disabled`:
  - Verified upstream mutating actions are rejected with 403 when authentication is disabled.
- `test_e2e_guest_mode_flow`:
  - Full e2e guest scenario passing with unread scoped to today and read tracking handled locally.

### 2. UI Verification (`crates/ui-components`)
- `test_index_template_hides_starred_but_shows_all_titles_when_auth_disabled`:
  - Verified `#filter-starred` is omitted in guest mode while `#filter-all` is visible.
  - Verified both `#filter-starred` and `#filter-all` are rendered in authenticated mode.
- `test_index_template_defines_all_articles_store_and_helpers`:
  - Verified `all_articles` object store is configured in `my_news_guest_db` (version 2).
  - Verified `renderLocalAllArticles` and `cacheLoadedArticlesFromDom` helper functions are defined in `IndexTemplate`.
  - Verified `#filter-unread-count` and `#unread-pill` preserve `data-base-unread`.
- `test_open_in_browser_link_wires_mark_article_read`:
  - Verified external browser link in `EntryItemTemplate`, `EntryListTemplate`, and `EntryDetailTemplate` wires `markArticleRead`.
- `test_filter_click_closes_detail_panel`:
  - Verified `closeDetailPanel` is called in filter click and search submit handlers.

### 3. Verification Commands Executed

```bash
cargo test -p server
cargo test -p ui-components
```
