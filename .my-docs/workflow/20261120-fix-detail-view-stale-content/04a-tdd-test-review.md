Result: clean

# TDD Test Plan Review: Fix Stale Content in Article Detail View

## Key Takeaway

The planned unit tests in `04-tasks.md` thoroughly cover all architectural requirements specified in `03-design.md`. They test both **positive wiring** (verifying that templates invoke `openDetailWithSkeleton()` and `closeDetailPanel()`, and define proper skeleton DOM structures) and **negative regressions** (confirming that legacy inline `classList.remove('hidden')` and `classList.add('hidden')` handlers are eradicated).

---

## Detailed Test Case Evaluation

1. **`test_entry_item_wires_open_detail_with_skeleton`**
   - **Target**: `crates/ui-components/templates/partials/entry_item.html`
   - **Positive check**: Verifies `onclick="openDetailWithSkeleton()"` is rendered on article click container.
   - **Negative check**: Verifies `document.getElementById('detail-panel').classList.remove('hidden')` is completely removed.
   - **Verdict**: **Complete**.

2. **`test_entry_list_wires_open_detail_with_skeleton`**
   - **Target**: `crates/ui-components/templates/partials/entry_list.html`
   - **Positive check**: Verifies `onclick="openDetailWithSkeleton()"` is rendered across all list items.
   - **Negative check**: Verifies legacy inline `classList.remove('hidden')` code is absent.
   - **Verdict**: **Complete**.

3. **`test_entry_detail_wires_close_detail_panel`**
   - **Target**: `crates/ui-components/templates/partials/entry_detail.html`
   - **Positive check**: Verifies Back button renders `onclick="closeDetailPanel()"`.
   - **Negative check**: Verifies `document.getElementById('detail-panel').classList.add('hidden')` is completely removed.
   - **Verdict**: **Complete**.

4. **`test_index_template_defines_detail_panel_helpers`**
   - **Target**: `crates/ui-components/templates/pages/index.html`
   - **Positive check**: Validates that `function openDetailWithSkeleton()` and `function closeDetailPanel()` are defined within the `<script>` block.
   - **Negative check**: Verifies functions guard against missing `#detail-panel` (`if (!panel) return;`).
   - **Verdict**: **Complete**.

5. **`test_index_template_skeleton_structure`**
   - **Target**: `crates/ui-components/templates/pages/index.html`
   - **Positive check**: Validates skeleton placeholder structure:
     - Root container contains `animate-pulse select-none`.
     - Interactive Back button contains `onclick="closeDetailPanel()"` and `pointer-events-auto`.
     - Spatial glassmorphic placeholders for header metadata, title, AI brief card, and prose content.
   - **Verdict**: **Complete**.

---

## Traceability & Coverage Matrix

| Design Requirement (`03-design.md`) | Planned Test in `04-tasks.md` | Positive Check | Negative Regression Check | Status |
| :--- | :--- | :--- | :--- | :--- |
| Immediate skeleton on entry item click | `test_entry_item_wires_open_detail_with_skeleton` | `openDetailWithSkeleton()` | No `classList.remove` | Verified |
| Immediate skeleton on entry list items | `test_entry_list_wires_open_detail_with_skeleton` | `openDetailWithSkeleton()` | No `classList.remove` | Verified |
| Detail panel reset on Back click | `test_entry_detail_wires_close_detail_panel` | `closeDetailPanel()` | No `classList.add` | Verified |
| Client helper function definitions | `test_index_template_defines_detail_panel_helpers` | Function declarations present | Guard checks | Verified |
| Spatial pulse skeleton DOM | `test_index_template_skeleton_structure` | Skeleton elements & classes | None needed | Verified |

---

## Conclusion

The planned test suite provides 100% coverage for the bug fix without gaps. All tests can be implemented as Askama template rendering unit tests in `crates/ui-components/src/templates.rs`.
