# Test Execution Log

- **Result:** success
- **Test path:** lite
- **Stage:** test-lite
- **Date:** 2026-11-20

## Summary

The complete workspace test suite was executed using `cargo test --workspace`. All 83 unit, integration, and e2e tests passed with 0 failures and 0 regressions.

| Crate / Target | Tests | Passed | Failed | Result |
|---|---|---|---|---|
| `miniflux-client` (lib) | 22 | 22 | 0 | pass |
| `server` (lib) | 39 | 39 | 0 | pass |
| `server` (e2e integration) | 2 | 2 | 0 | pass |
| `ui-components` (lib) | 20 | 20 | 0 | pass |
| **Total** | **83** | **83** | **0** | **success** |

---

## Detailed Test Breakdown

### 1. `ui-components` (20 passed)
- `test_entry_item_wires_open_detail_with_skeleton`: Validates that `EntryItemTemplate` renders `onclick="openDetailWithSkeleton()"` instead of inline class toggling.
- `test_entry_list_wires_open_detail_with_skeleton`: Validates that `EntryListTemplate` renders list entries with `onclick="openDetailWithSkeleton()"` instead of inline class toggling.
- `test_entry_detail_wires_close_detail_panel`: Validates that `EntryDetailTemplate` Back button renders with `onclick="closeDetailPanel()"` instead of inline class toggling.
- `test_index_template_defines_detail_panel_helpers`: Validates that `IndexTemplate` defines `openDetailWithSkeleton` and `closeDetailPanel` helper functions in its `<script>` block.
- `test_index_template_skeleton_structure`: Validates that `openDetailWithSkeleton` generates a structured spatial skeleton with `animate-pulse`, an interactive Back button wired to `closeDetailPanel()`, and metadata/AI brief/prose placeholders.
- `test_index_template_renders_today_and_unread_counts`: Validates today and unread count badges.
- `test_entry_list_renders_pagination_when_multiple_pages`: Validates pagination navigation.
- `test_entry_list_hides_pagination_when_single_page`: Validates pagination hidden for single page.
- `test_spatial_window_fluid_width_container_query`: Asserts modern CSS container query rules.
- `test_render_title_only_entry_item`: Verifies entry row markup.
- `test_render_entry_list_with_title_only_items`: Verifies entry list rendering.
- `test_detail_template_omits_author`: Verifies author omission.
- `test_detail_template_has_mark_unread_button`: Verifies mark as unread button.
- `test_filter_bar_reordering`: Verifies filter pills ordering and active state.
- `test_index_template_renders_ai_button_when_enabled`: Verifies AI button rendering.
- `test_ai_grouped_list_template_renders_feed_headers_and_canonical_route`: Verifies AI group list markup.
- `test_render_toast_oob_template`: Verifies toast component rendering.
- `test_sanitizer_removes_dangerous_tags`: Verifies HTML tag sanitizer.
- `test_sanitizer_adds_noopener_to_links`: Verifies `rel="noopener noreferrer"`.
- `test_sanitizer_multiple_links_all_receive_target_blank`: Verifies `target="_blank"`.

### 2. `server` (39 lib + 2 e2e passed)
- All 39 unit tests (routes, auth, config, AI extractor, LLM, DB) and 2 e2e tests passed.

### 3. `miniflux-client` (22 passed)
- All 22 unit tests (client methods, GraphQL schema, queries, mutations) passed.
