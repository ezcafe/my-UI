# Tasks: Multi-Device Responsive UI (Desktop, Mobile & AR Glasses)

- **Workflow Slug:** `20261124-multi-device-responsive-ui-ar`
- **Result:** done

---

## Task List (TDD Steps)

### Task 1: CSS Design Tokens & Container Query Recipes
- **Files**: `annuaire-theme.css`, `crates/ui-components/templates/base.html`
- **Tests First**:
  - `test_theme_css_contains_container_query_master_detail_recipes`: Unit/integration test ensuring `annuaire-theme.css` has `.workspace-layout`, container query `@container workspace`, optical waveguide dark translucent backplate tokens, and minimum 48px gaze targets.
  - `test_base_template_includes_container_recipes`: Verify `base.html` properly links and injects the updated CSS.
- **Implementation**:
  - Add `.workspace-layout` with `container-type: inline-size; container-name: workspace;`.
  - Add `@container workspace (min-width: 58rem)` split-pane master-detail rules.
  - Add waveguide contrast tokens and 48px gaze target rules.

### Task 2: Remove Hardcoded Media Query Breakpoints from `entry_list.html`
- **Files**: `crates/ui-components/templates/partials/entry_list.html`
- **Tests First**:
  - `test_entry_list_no_hardcoded_viewport_media_queries`: Test scanning `entry_list.html` asserting zero occurrences of `lg:grid-cols`, `sm:flex-row`, `sm:w-1/2`, `md:p-6`, `md:text-xl`.
  - `test_entry_list_renders_with_container_query_classes`: Assert magazine view renders with container classes (`magazine-3col-grid`, `magazine-card priority-1`).
- **Implementation**:
  - Replace all hardcoded Tailwind media query prefixes with container query classes and fluid `clamp()` formulas.
  - Ensure magazine cards adapt smoothly to any container width.

### Task 3: Workspace Layout & Desktop Empty Placeholder in `index.html`
- **Files**: `crates/ui-components/templates/pages/index.html`
- **Tests First**:
  - `test_index_template_renders_workspace_layout_and_placeholder`: Assert `index.html` renders `<div class="workspace-layout">` and includes `#detail-empty-placeholder`.
- **Implementation**:
  - Update the main content container to `.workspace-layout`.
  - Add `#detail-empty-placeholder` with newspaper icon and Annuaire styling.
  - Update `#detail-panel` class definitions for responsive container handling.

### Task 4: Master-Detail JavaScript Synchronization & State Management
- **Files**: `crates/ui-components/templates/pages/index.html`
- **Tests First**:
  - `test_e2e_or_template_js_master_detail_handlers`: Verify JavaScript functions (`openEntryDetail`, `closeDetailPanel`) properly update active selection markers, toggle `.has-content`, and maintain URL synchronization without accidental mark-read.
- **Implementation**:
  - Update `openEntryDetail(id)` to add `.has-content` to `#detail-panel`, highlight active row with 3px red bar, and render content.
  - Update `closeDetailPanel()` to handle mobile slide-over dismissal vs. desktop split reset.
  - Update initial page load to honor URL `?entry=123` or show `#detail-empty-placeholder`.
