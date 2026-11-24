# Analysis: Multi-Device Responsive UI (Desktop, Mobile & AR Glasses)

## 1. Context & Objectives

The application currently provides a clean Annuaire-styled RSS reader interface, but has three architectural bottlenecks:
1. **Desktop Master-Detail Occlusion**: When reading on wide desktop monitors (e.g., 1440px), opening an article completely covers the feed list (`absolute inset-0 z-30`), forcing the user to hit "Back" repeatedly to pick subsequent articles.
2. **Hardcoded Media Query Breakpoints**: Child components like `crates/ui-components/templates/partials/entry_list.html` utilize Tailwind viewport media query classes (`sm:`, `md:`, `lg:`), breaking container-query encapsulation when embedded in resizable desktop windows or spatial panes.
3. **AR Smart Glasses Readiness**: Optical see-through (OST) glasses (such as Android XR smart glasses, Meta Orion, or XREAL) cannot display black pixels (black is 100% transparent), and their narrow 30°–50° Field of View requires centralized controls and generous (44px–48px) hit targets for gaze and pinch interaction.

Following the user's selection of **Option 1 (Unified Adaptive Container-Query Architecture)**, we are structuring a unified responsive frontend that scales naturally from mobile to desktop and prepares for AR smart glasses.

---

## 2. Technical Nuances & Architecture

### A. Fluid Master-Detail via CSS Container Queries
- Instead of using `@media (min-width: 1024px)`, we designate the inner content workspace as a container:
  ```css
  .workspace-layout {
    container-type: inline-size;
    container-name: workspace;
    display: grid;
    grid-template-columns: 1fr;
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
  }
  ```
- At wide container widths (`@container workspace (min-width: 58rem)` / ~928px):
  - `.workspace-layout` switches to `grid-template-columns: minmax(20rem, 24rem) 1fr`.
  - `#entries-list` occupies column 1.
  - `#detail-panel` occupies column 2 as a static grid child (`position: relative !important; inset: auto !important; display: flex !important; border-left: 1px solid var(--border);`).
  - Active article in the list shows the Annuaire 3px positional red bar (`bg-brand`) to highlight the active master row.
- At narrow container widths (`@container workspace (max-width: 58rem)`):
  - The layout behaves as a single-column flow.
  - `#detail-panel` acts as an absolute slide-over (`absolute inset-0 z-30`) when active, and hides with `hidden` when closed.
  - The "Back" button in the detail panel is visible and allows effortless return to the list.

### B. Eliminating Hardcoded Breakpoints in `entry_list.html`
- In `entry_list.html`:
  - Replace `grid grid-cols-1 lg:grid-cols-[minmax(220px,1fr)_minmax(380px,2.5fr)_minmax(220px,1.1fr)]` with container query rules:
    ```css
    .magazine-3col-grid {
      display: grid;
      grid-template-columns: 1fr;
      gap: 1.25rem;
    }
    @container (min-width: 48rem) {
      .magazine-3col-grid {
        grid-template-columns: minmax(14rem, 1fr) minmax(22rem, 2.5fr) minmax(14rem, 1.1fr);
      }
    }
    ```
  - Replace `sm:flex-row`, `sm:w-1/2`, `md:p-6`, `md:text-xl` with fluid container query classes or fluid typography using `clamp()`.

### C. AR Smart Glasses Adaptation (Waveguide & Spatial Ergonomics)
- **Optical Waveguide Contrast**:
  - Waveguides cannot create black. Dark surfaces in dark mode must have a subtle backing opacity (`rgba(18, 19, 22, 0.85)` with `backdrop-filter: blur(20px)`) to guarantee contrast against bright real-world rooms.
- **Central Comfort Cone**:
  - In spatial mode, controls are positioned away from the extreme top-right / top-left corners and grouped in the central 30° viewing area.
- **Hit Targets for Gaze & Pinch / Raycasting**:
  - Interactive buttons and filter pills have a minimum touch/gaze height of 44px (`min-h-[44px]`).
  - Rows have a comfortable hit area of at least 48px.

### D. JavaScript Master-Detail Synchronization
- In `openEntryDetail(id)`:
  - Updates the active row indicator in `#entries-list` (adding the Annuaire 3px red bar to the selected item).
  - On desktop (wide container), it renders into `#detail-panel` without hiding `#entries-list`.
  - On mobile (narrow container), it shows `#detail-panel` as a full overlay.
- In `closeDetailPanel()`:
  - On mobile, hides `#detail-panel`.
  - On desktop, clears the active selection or shows a placeholder ("Select an article to view").

---

## 3. Risks & Mitigations

1. **Layout Jitter on Container Resize**:
   - *Risk*: Dragging or resizing windows could cause rapid shifting between 1-pane and 2-pane modes.
   - *Mitigation*: Use CSS container queries with smooth CSS transitions on column width (`transition: grid-template-columns 0.2s ease-out`).
2. **HTMX Dynamic Replacement**:
   - *Risk*: When filtering or changing pages, HTMX swaps `#entries-list`.
   - *Mitigation*: Ensure the active article ID is tracked in a data attribute or URL query param (`?entry=123`), re-highlighting the active row when the list re-renders.

---

## 4. Open Questions for Grill

1. On desktop wide view before any article is clicked, should `#detail-panel` display an empty placeholder card or auto-select the first unread article?
2. Should the spatial / AR waveguide styling be triggered automatically via a media/display query (e.g. `@media (hover: none) and (pointer: fine)`) or via a dedicated user toggle button in the header?

## Result

Result: done
