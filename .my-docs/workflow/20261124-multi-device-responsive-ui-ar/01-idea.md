# Ideation: Multi-Device Responsive UI (Desktop, Mobile & AR Smart Glasses)

## 1. Executive Summary

This review analyzes the current UI implementation across `crates/ui-components/`, `annuaire-theme.css`, and the spatial prototypes (`ar-list.html`, `ar-detail.html`, `index.html`). 

The current system has strong typographic and brand foundations based on the **Annuaire Design System** (Swiss high-density editorial style). However, it faces three key structural gaps:
1. **Desktop Master-Detail Occlusion**: Opening an article detail panel uses `absolute inset-0 z-30`, completely hiding the list on wide screens instead of providing a fluid split-pane master-detail experience.
2. **Hardcoded Viewport Breakpoints**: Several components in `crates/ui-components/templates/partials/entry_list.html` rely on Tailwind media query breakpoints (`sm:`, `md:`, `lg:`) rather than CSS container queries and fluid `minmax()` grids.
3. **AR Smart Glasses Readiness**: While standalone static prototypes exist (`ar-list.html` and `ar-detail.html`), the production Rust/Askama templates lack a unified adaptive layout that supports optical waveguides, gaze-friendly sizing, and central-cone ergonomics.

---

## 2. Codebase Audit Findings

### A. Breakpoints & Layout Encapsulation
- **Problem**: In [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L65), layout rules use `grid-cols-1 lg:grid-cols-[minmax(220px,1fr)_minmax(380px,2.5fr)_minmax(220px,1.1fr)]` and multiple `sm:` / `md:` utility classes.
- **Impact**: Violates strict design rules against hardcoded viewport breakpoints. When embedded inside a desktop side panel, split screen, or a floating AR spatial window, media queries respond to the device screen width instead of the window container width.
- **Fix**: Replace media query prefixes with container query classes (e.g. `@container (min-width: 48rem)`) and fluid CSS grid formulas (`repeat(auto-fit, minmax(min(100%, 320px), 1fr))`).

### B. Desktop Reading Ergonomics
- **Problem**: In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L168), `#detail-panel` is styled with `hidden absolute inset-0 z-30 bg-card`.
- **Impact**: On a 1440px desktop display, opening an article replaces the entire feed list with a single reading column. The user loses list context and must click "Back" every time to pick the next article.
- **Fix**: Implement an adaptive 2-pane layout using CSS container queries: when container width is wide (> 960px), display List on the left (e.g. 380px–420px) and Reader on the right. When container width is narrow (< 960px), keep the full-screen slide-over.

### C. AR Smart Glasses Ergonomics & Optical Characteristics
- **Visual Contrast on Waveguides**: Optical See-Through (OST) displays (e.g., Android XR glasses, Meta Orion, XREAL) cannot project true black. Dark pixels become transparent. Pure white text without backplates is washed out by ambient daylight.
- **Central Comfort Cone**: Smart glasses have a narrow Field of View (typically 30°–50°). Controls pinned to extreme window corners require uncomfortable eye/neck rotation. Primary actions must stay in the central 30° cone.
- **Hit Targets for Gaze & Pinch / Raycast**: While mouse clicks require 24–32px targets, gaze-and-pinch or spatial raycast requires minimum 44px–48px interactive boundaries with visual gaze highlights.

---

## 3. Reference Standards

- **W3C WebXR Device API**: [W3C Recommendation](https://www.w3.org/TR/webxr/)
- **Apple visionOS Human Interface Guidelines**: [Spatial Layout & Materials](https://developer.apple.com/design/human-interface-guidelines/spatial-layout)
- **Google Android XR & Jetpack Compose XR Guidelines**: [Spatial Design Principles](https://developer.android.com/guide/topics/large-screens/spatial)
- **visx by Airbnb**: [Official Documentation](https://visx.airbnb.tech/docs)
- **Lightweight Charts by TradingView**: [Documentation](https://tradingview.github.io/lightweight-charts/docs)
- **CSS Container Queries (W3C Spec)**: [MDN CSS Container Queries](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries)
