# Design: Multi-Device Responsive UI (Desktop, Mobile & AR Glasses)

- **Workflow Slug:** `20261124-multi-device-responsive-ui-ar`
- **Mode:** full
- **Result:** done

---

## 1. Architectural Options & Recommendation

### Option 1 (Recommended): Unified Adaptive Container-Query Architecture with Named Containers
- **Explanation**:
  1. The main content area in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) is wrapped in a named container context: `container: workspace / inline-size;`.
  2. On wide containers (`@container workspace (min-width: 58rem)` / ~928px), the layout switches to a 2-pane master-detail split (`grid-template-columns: minmax(20rem, 24rem) 1fr`). `#entries-list` stays on the left, and `#detail-panel` displays on the right as a relative grid child (`position: relative !important; inset: auto !important; display: flex !important; border-left: 1px solid var(--border)`).
  3. When no article is selected yet on desktop, `#detail-panel` displays an Annuaire-styled empty placeholder card ("Select an article to read") with an inbox/newspaper icon.
  4. On narrow containers (`@container workspace (max-width: 58rem)`), the layout remains a single-column flow where `#detail-panel` functions as a full-screen slide-over modal (`absolute inset-0 z-30`) with a prominent "Back" button.
  5. In [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), all hardcoded Tailwind media query prefixes (`sm:`, `md:`, `lg:`) are completely removed and replaced by `@container` rules and fluid `clamp()` formulas.
  6. For AR smart glasses and optical waveguides, high-contrast dark surfaces (`rgba(18, 19, 22, 0.85)` with `backdrop-filter: blur(20px)`), 48px minimum touch/gaze targets, and central-cone button alignments are natively integrated into the dark theme and container queries.
- **Pros**:
  - Single source of truth across all device classes.
  - Zero hardcoded media query breakpoints; 100% compliant with project rules.
  - Completely resolves desktop master-detail occlusion.
  - Ready for optical waveguides without extra toggle buttons.
- **Cons**:
  - Requires updating both HTML templates and CSS recipes.

### Option 2: JavaScript ResizeObserver Driven Master-Detail
- **Explanation**: Use a JavaScript `ResizeObserver` on `#entries-list` and `#detail-panel` to dynamically append/remove inline style classes based on pixel widths.
- **Pros**: Works in older browsers that lack CSS container query support.
- **Cons**: Adds JavaScript runtime overhead, risks layout thrashing and flicker, and violates modern CSS best practices.

### Recommendation
**Option 1** is selected and recommended.

---

## 2. Technical Specifications

### A. Layout Structure in `crates/ui-components/templates/pages/index.html`

Replace the absolute flex layout in lines 160–169 with a named container:

```html
<!-- Main Content Area: Entries List + Detail Panel Container -->
<div class="workspace-layout flex-1 overflow-hidden relative">
  <!-- Loading Indicator -->
  <div id="entries-loading-bar" class="htmx-indicator h-0.5 bg-brand animate-pulse absolute top-0 left-0 right-0 z-20 pointer-events-none"></div>
  
  <!-- Left / Master: Entries List -->
  <div id="entries-list" class="entries-pane flex-1 overflow-y-auto p-4 flex flex-col gap-1.5 bg-card min-w-0">
    {{ entries_list_html|safe }}
  </div>

  <!-- Right / Detail: Slide-Over on Mobile, Static Pane on Desktop -->
  <div id="detail-panel" class="detail-pane detail-scrollbar hidden absolute inset-0 z-30 bg-card border-l border-border flex flex-col p-6 overflow-y-auto">
    <!-- Desktop Empty Placeholder (visible when wide and no entry selected) -->
    <div id="detail-empty-placeholder" class="hidden h-full flex-col items-center justify-center text-center text-muted-foreground p-8 select-none">
      <div class="p-4 rounded-full bg-muted/60 mb-3 text-muted-foreground/80">
        <i data-lucide="newspaper" class="w-8 h-8 stroke-1"></i>
      </div>
      <p class="text-sm font-semibold font-sans text-foreground/80">Select an article to read</p>
      <p class="text-xs text-muted-foreground mt-1 max-w-xs font-sans">Choose an item from the feed on the left to view the inverted pyramid summary and insights.</p>
    </div>
  </div>
</div>
```

### B. Container Query Recipes in `annuaire-theme.css`

Add the following container query rules:

```css
/* Container Query Master-Detail Workspace */
.workspace-layout {
  container-type: inline-size;
  container-name: workspace;
  display: grid;
  grid-template-columns: 1fr;
  height: 100%;
  width: 100%;
  position: relative;
  overflow: hidden;
}

@container workspace (min-width: 58rem) {
  .workspace-layout {
    grid-template-columns: minmax(20rem, 24rem) 1fr;
  }

  .workspace-layout .detail-pane {
    position: relative !important;
    inset: auto !important;
    z-index: 10 !important;
    display: flex !important;
    border-left: 1px solid var(--border);
  }

  /* Show empty placeholder when detail-panel has no active article loaded */
  .workspace-layout .detail-pane:not(.has-content) #detail-empty-placeholder {
    display: flex !important;
  }

  /* Hide back button on desktop split view */
  .workspace-layout .detail-pane #detail-back-btn {
    display: none !important;
  }
}

/* AR Optical Waveguide & Gaze Target Ergonomics */
[data-theme="dark"] .spatial-window,
:root.dark .spatial-window {
  background: rgba(18, 19, 22, 0.92);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
}

.ar-list-item {
  min-height: 48px; /* Gaze & pinch touch target size */
}
```

### C. Breakpoint Migration in `crates/ui-components/templates/partials/entry_list.html`

Remove all `sm:`, `md:`, `lg:` prefixes:
1. Magazine 3-column grid:
   - Change: `grid grid-cols-1 lg:grid-cols-[minmax(220px,1fr)_minmax(380px,2.5fr)_minmax(220px,1.1fr)]`
   - To: `magazine-3col-grid` styled via `@container (min-width: 48rem)`.
2. Priority-1 magazine card:
   - Change: `sm:flex-row`, `sm:w-1/2`, `md:p-6`, `md:text-xl`
   - To: `magazine-card priority-1` styled via `@container (min-width: 36rem)` with fluid typography `clamp(1rem, 0.9rem + 1cqi, 1.25rem)`.
3. Priority-2 & Priority-3 cards:
   - Change: `md:p-5`, `md:text-sm`
   - To: Container-based spacing and typography.

### D. JavaScript Master-Detail Synchronization in `index.html`

1. Update `openEntryDetail(id)`:
   - Highlight active row in list: Add `bg-brand` to `#entry-${numId}-bar`, remove from other rows.
   - Add `.has-content` class to `#detail-panel`.
   - Hide `#detail-empty-placeholder`.
2. Update `closeDetailPanel()`:
   - On mobile: Add `hidden` to `#detail-panel`.
   - On desktop: Remove `.has-content` from `#detail-panel`, displaying `#detail-empty-placeholder`.
   - Remove active row highlight.
3. In initial load:
   - Check if an entry is in the URL query param (`?entry=123`). If yes, open it.
   - If no entry is in the URL, initialize `#detail-empty-placeholder` on desktop without marking any article read.

---

## 3. Annuaire Design System & AR Compliance Checklist

- [x] **Rule 1: Red marks position, never action** (Active list item retains 3px vertical red bar).
- [x] **Rule 2: Primary action is an Ink pill** ("Back" and original link use high-contrast ink pill).
- [x] **Rule 4: Two typefaces** (Big Shoulders for titles, Work Sans for body).
- [x] **Rule 5: Zero page shadows** (Flat 1px border between master and detail).
- [x] **Rule 6: Numbers are tabular** (Timestamps and counts use `tabular-nums lining-nums`).
- [x] **No hardcoded breakpoints** (Container queries and `clamp()` used exclusively).
- [x] **AR optical waveguide safe** (High-contrast translucent backing in dark mode, 48px hit areas).

## Result

Result: done
