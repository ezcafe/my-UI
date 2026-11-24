# Codebase Skim: Multi-Device Responsive UI (Desktop, Mobile & AR Glasses)

## Touchpoint Analysis

1. **Root Templates & Spatial Container ([crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html))**:
   - `body`: Defines `container-type: inline-size;`.
   - `.spatial-shell`: Wraps the entire application. Has `container-type: inline-size;` and `padding: clamp(0.75rem, 3vw, 2.5rem);`.
   - `.spatial-window`: Has `max-width: min(100cqi, 1440px); height: min(90vh, 840px);`.
   - Mobile override: Currently `@container (max-width: 768px)` removes border and padding for edge-to-edge mobile screens.
   - Target update: Refine container definitions so children can cleanly query container width without media queries.

2. **Main Layout & Detail Panel ([crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html))**:
   - Lines 160–169: Container holds `#entries-list` and `#detail-panel`.
   - Current: `#detail-panel` has `absolute inset-0 z-30 hidden`.
   - Target update:
     - Wrap the main workspace in a container query grid:
       ```css
       .workspace-container {
         container-type: inline-size;
         display: grid;
         grid-template-columns: 1fr;
         position: relative;
         height: 100%;
         width: 100%;
         overflow: hidden;
       }
       @container (min-width: 58rem) {
         .workspace-container {
           grid-template-columns: minmax(20rem, 24rem) 1fr;
         }
         #detail-panel {
           position: relative !important;
           inset: auto !important;
           z-index: 10 !important;
           display: flex !important;
           border-left: 1px solid var(--border);
         }
       }
       ```
     - Detail Panel empty state on desktop: When no article is selected yet on wide screen, display a clean Annuaire-styled prompt ("Select an article to read") with an inbox/newspaper icon.
     - Mobile behavior: When width < 58rem, `#detail-panel` acts as an absolute overlay (`absolute inset-0 z-30`) with a prominent "Back" button, maintaining single-column focus on smartphones.

3. **Feed List & Magazine View Breakpoints ([crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html))**:
   - Lines 65–180: Multiple hardcoded media queries exist:
     - `lg:grid-cols-[minmax(220px,1fr)_minmax(380px,2.5fr)_minmax(220px,1.1fr)]`
     - `sm:flex-row`, `sm:w-1/2`, `md:p-6`, `md:text-xl`
   - Target update:
     - Replace Tailwind media queries with modern container queries using Tailwind arbitrary variants `@container` or semantic CSS classes in `annuaire-theme.css`:
       - `@container (min-width: 48rem)` for multi-column magazine grid.
       - Fluid typography: `clamp(1rem, 0.9rem + 1cqi, 1.25rem)`.
       - Fluid card image split: `@container (min-width: 36rem)` for horizontal priority-1 cards.

4. **AR Smart Glasses & Optical Waveguide Compatibility ([annuaire-theme.css](file:///Users/ptquang86/ws/my-ui/annuaire-theme.css))**:
   - Optical see-through (OST) glasses constraints:
     - Pure black is transparent on waveguides; text must have high-contrast backplates or soft luminescence.
     - Interactive elements need minimum 44px–48px hit areas for gaze/pinch raycasting.
     - Controls must stay within central 30°–45° cone.
   - Target update:
     - Add `.ar-mode` / `.spatial-ready` tokens:
       - Slightly enlarged touch/gaze targets: `.field-button-hit-area` and `min-h-[44px]`.
       - Waveguide-safe high-contrast card background with subtle translucent border.
       - Gaze-hover micro-elevation transitions (`scale(1.02)` on gaze focus).

## Result

Result: done
