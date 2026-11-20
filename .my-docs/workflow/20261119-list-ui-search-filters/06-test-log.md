# Test log

- **Result:** success
- **Test path last run:** lite
- **Smoke source:** Build-verify
- **Updated:** 2026-11-19

## Smoke

- **Verification Command:** `cargo test --workspace`
- **Result:** Pass (50 passed, 0 failed, 0 ignored across 3 crates)
- **Summary of Verifications:**
  - `miniflux-client` (19 passed):
    - `test_miniflux_client_get_entries_published_after`: verifies `published_after` query parameter serialization in `MinifluxClient::get_entries`.
    - `test_graphql_query_entries_published_after_and_search`: verifies GraphQL schema queries with `publishedAfter` and `search` propagate properly.
  - `server` (22 unit + 2 e2e passed):
    - `test_entries_filter_query_mapping`: validates GraphQL filter mapping for `status=today`, `status=starred`, `status=all`, `status=unread`, `status=read`, and escaped search queries.
    - `test_hx_entries_filter_today_and_search`: validates end-to-end route handling for today filter and search.
    - `test_app_starts_and_responds_to_health_check` & `test_e2e_authenticated_session_flow`: health check and full authenticated session flow.
  - `ui-components` (7 passed):
    - `test_render_title_only_entry_item`: verifies title-only spatial row, omission of summary/reading time/feed title, presence of external link button with `target="_blank"`, `rel="noopener noreferrer"`, and `onclick="event.stopPropagation()"`, and unread dot indicator logic.
    - `test_render_entry_list_with_title_only_items`: verifies list container rendering title-only rows with external link buttons.
    - `test_render_index_filter_bar`: verifies filter buttons sequence (`All Titles`, `Unread`, `Today`, `Starred`), `/hx/entries?status=today` endpoint, and active filter script hook.

## Suite / e2e (test-lite)

- **Execution Command:** `cargo test --workspace`
- **Result:** success (50 passed; 0 failed; 0 ignored)
  - `miniflux-client`: 19 passed (19 unit, 0 doc)
  - `server`: 24 passed (22 unit, 2 e2e, 0 doc)
  - `ui-components`: 7 passed (7 unit, 0 doc)
- All unit, template, and WireMock integration tests passed cleanly.
