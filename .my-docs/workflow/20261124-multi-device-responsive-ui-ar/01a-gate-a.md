# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear and explicit.
  - The user selected **Option 1: Unified Adaptive Container-Query Architecture**.
  - Objectives:
    1. **Desktop Master-Detail Experience**: On wide container viewports (e.g., width > 58rem / 928px), display an adaptive 2-pane master-detail layout where the feed list stays on the left and the article reader displays on the right, eliminating the `absolute inset-0` occlusion.
    2. **Container Queries Over Hardcoded Breakpoints**: Eliminate all hardcoded Tailwind media queries (`sm:`, `md:`, `lg:`) from [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), replacing them with `@container` rules and fluid CSS (`repeat(auto-fit, minmax(...))`).
    3. **AR Smart Glasses Readiness**: Prepare the UI system with high-contrast backplates for optical waveguides, 48px hit targets for gaze-pinch raycasting, and central-cone (30°–45°) control anchoring.
    4. **Strict Design Alignment**: Maintain all Annuaire design rules (Big Shoulders + Work Sans, positional red only, Ink pill primary actions, zero page shadows, and tabular numbers).

- **Scope & Boundaries**:
  - Affects:
    1. `crates/ui-components/templates/base.html`: Ensure root and container styles define appropriate container context for master-detail and spatial modes.
    2. `crates/ui-components/templates/pages/index.html`: Update the main container and `#detail-panel` CSS rules to switch between single-column overlay (mobile/narrow) and side-by-side master-detail (desktop/wide).
    3. `crates/ui-components/templates/partials/entry_list.html`: Replace all `sm:`, `md:`, `lg:` classes with container-aware layout recipes.
    4. `crates/ui-components/templates/partials/entry_detail.html`: Ensure detail panel controls stay comfortable in both desktop split and spatial views.
    5. `annuaire-theme.css`: Add clean container query utility recipes and spatial ergonomics tokens.
  - Non-goals:
    - No changes to PostgreSQL database schemas or migrations.
    - No changes to server-side business logic, AI extraction, or Miniflux API proxying.

- **Feasibility & Constraints**:
  - Standard CSS container queries (`container-type: inline-size` and `@container`) are supported across all modern browsers (Chrome, Safari, Firefox, Edge, visionOS Safari, Android XR Chrome).
  - Preserves HTMX dynamic swaps and existing IndexedDB article caching without regression.

- **Risks & Mitigations**:
  - *Risk*: Detail panel opening logic in JavaScript might expect `hidden` class toggling.
  - *Mitigation*: Update `openEntryDetail()` and `closeDetailPanel()` in `index.html` to handle both full-overlay (narrow containers) and active split states (wide containers) seamlessly.

## Result

Result: ok
