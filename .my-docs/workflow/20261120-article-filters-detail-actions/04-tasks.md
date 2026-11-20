# Implementation Tasks (TDD)

## Planned Tests
1. **`test_filter_bar_reordering`** in `crates/ui-components/src/templates.rs`:
   - Verify rendered `IndexTemplate` has pills in order: `Unread`, `Today`, `Starred`, `All titles`.
   - Verify `Unread` is styled as the active pill (`bg-indigo-500/20 text-indigo-300`).
2. **`test_detail_template_omits_author`** in `crates/ui-components/src/templates.rs`:
   - Verify rendered `EntryDetailTemplate` does not contain `By <Author>` when `author` is `Some(...)`.
   - Verify `entry.feed_title` and `entry.published_at` are still rendered.
3. **`test_detail_template_has_mark_unread_button`** in `crates/ui-components/src/templates.rs`:
   - Verify rendered `EntryDetailTemplate` contains `hx-post="/hx/entries/{id}/read?status=unread"`.
   - Verify button appears before bookmark button.
4. **`test_build_graphql_filter_search_defaults_to_all`** in `crates/server/src/routes/entries.rs`:
   - Verify query with `search="test"` and no `status` does not append `status: UNREAD`.
5. **`test_graphql_mutation_update_entries_status`** in `crates/miniflux/src/schema/mod.rs`:
   - Verify GraphQL mutation `updateEntriesStatus(entryIds: ["101", "102"], status: READ)` invokes client batch update.
6. **`test_hx_mark_page_read_endpoint`** in `crates/server/src/routes/mod.rs`:
   - Wiremock test verifying `POST /hx/mark-page-read` with `entry_ids=101,102` calls Miniflux batch update and returns rendered `EntryListTemplate` with updated `unread-pill`.

---

## Tasks

### Phase 1: Templates & Frontend (TDD)
- [x] **Task 1.1: Write failing template tests**:
  - Update `test_render_index_filter_bar` in `crates/ui-components/src/templates.rs` for new pill order (`Unread` < `Today` < `Starred` < `All titles`).
  - Add `test_detail_template_omits_author` and `test_detail_template_has_mark_unread_button`.
- [x] **Task 1.2: Implement template updates**:
  - Update `crates/ui-components/templates/pages/index.html`:
    - Reorder filter pills.
    - Implement hybrid search form and client-side filter function `filterArticlesClientSide(query)`.
    - Replace mark-all-read with mark-this-page-as-read button and `getVisibleEntryIds()` helper.
  - Update `crates/ui-components/templates/partials/entry_detail.html`:
    - Remove author display.
    - Add "Mark as Unread" icon button before bookmark button.
  - Update `crates/ui-components/templates/partials/entry_list.html` and `entry_item.html` to include `data-entry-id="{{ entry.id }}"`.
- [x] **Task 1.3: Verify template tests pass**:
  - Run `cargo test -p ui-components` to confirm all template tests are green.

### Phase 2: GraphQL Mutation & Server Handlers (TDD)
- [x] **Task 2.1: Write failing server & schema tests**:
  - Add test for `updateEntriesStatus` mutation in `crates/miniflux/src/schema/mod.rs`.
  - Add unit test for `build_graphql_filter` search status default in `crates/server/src/routes/entries.rs`.
  - Add test for `POST /hx/mark-page-read` in `crates/server/src/routes/mod.rs`.
- [x] **Task 2.2: Implement GraphQL batch update**:
  - Add `update_entries_status` mutation to `crates/miniflux/src/schema/mutation.rs`.
- [x] **Task 2.3: Implement server routes**:
  - Update `build_graphql_filter` in `crates/server/src/routes/entries.rs` to default to all when search query is present.
  - Implement `mark_page_read_htmx` in `crates/server/src/routes/entries.rs`.
  - Register route in `crates/server/src/app.rs`.
- [x] **Task 2.4: Verify backend tests pass**:
  - Run `cargo test -p miniflux-client` and `cargo test -p server` to verify all pass.

### Phase 3: End-to-End Verification
- [x] **Task 3.1: Run full test suite**:
  - Execute `cargo test` across workspace.
