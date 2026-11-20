# Tasks: Fix Stale Content in Article Detail View

Result: done

## Tasks Breakdown

### Phase 1: Test Specification (TDD)
- [x] Task 1.1: Add failing unit tests in `crates/ui-components/src/templates.rs` verifying that `EntryItemTemplate` renders `onclick="openDetailWithSkeleton()"` instead of inline class toggling.
- [x] Task 1.2: Add unit tests in `crates/ui-components/src/templates.rs` verifying that `EntryListTemplate` renders `onclick="openDetailWithSkeleton()"` on entry items.
- [x] Task 1.3: Add unit tests in `crates/ui-components/src/templates.rs` verifying that `EntryDetailTemplate` renders `onclick="closeDetailPanel()"` on the Back button.
- [x] Task 1.4: Add unit tests in `crates/ui-components/src/templates.rs` verifying that `IndexTemplate` defines `openDetailWithSkeleton`, `closeDetailPanel`, and renders the skeleton placeholder structure.

### Phase 2: Template Implementation
- [x] Task 2.1: Add `openDetailWithSkeleton()` and `closeDetailPanel()` helper functions and the glassmorphic skeleton markup to `crates/ui-components/templates/pages/index.html`.
- [x] Task 2.2: Update `crates/ui-components/templates/partials/entry_item.html` to call `openDetailWithSkeleton()`.
- [x] Task 2.3: Update `crates/ui-components/templates/partials/entry_list.html` to call `openDetailWithSkeleton()`.
- [x] Task 2.4: Update `crates/ui-components/templates/partials/entry_detail.html` to call `closeDetailPanel()`.

### Phase 3: Verification & Polish
- [x] Task 3.1: Run `cargo test -p ui-components` to verify all template and helper tests pass.
- [x] Task 3.2: Inspect rendered HTML and spatial glassmorphism visual attributes.

---

## Planned tests

1. `test_entry_item_wires_open_detail_with_skeleton`:
   - Validates that `EntryItemTemplate` markup contains `onclick="openDetailWithSkeleton()"` and does not contain `document.getElementById('detail-panel').classList.remove('hidden')`.
2. `test_entry_list_wires_open_detail_with_skeleton`:
   - Validates that `EntryListTemplate` renders list entries with `onclick="openDetailWithSkeleton()"`.
3. `test_entry_detail_wires_close_detail_panel`:
   - Validates that `EntryDetailTemplate` Back button renders with `onclick="closeDetailPanel()"` and does not contain `document.getElementById('detail-panel').classList.add('hidden')`.
4. `test_index_template_defines_detail_panel_helpers`:
   - Validates that `IndexTemplate` defines `function openDetailWithSkeleton()` and `function closeDetailPanel()` in its `<script>` block.
5. `test_index_template_skeleton_structure`:
   - Validates that `openDetailWithSkeleton` generates a structured spatial skeleton featuring `animate-pulse`, a functioning Back button wired to `closeDetailPanel()`, and metadata/AI brief/prose placeholders.
