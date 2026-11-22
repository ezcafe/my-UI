# Grill: Mobile Viewport Full Stretch Windows

Result: frontier-empty

## Frontier Rounds

### Round 1 (Settled during Ideation & Plan Review)

❓ **Q1** - **Edge-to-edge full stretch vs floating inset card on mobile viewports**:
On mobile devices, should the application window stretch completely to the viewport edges (Option 1: 0 padding, full 100dvh height, 0 radius, no border) or retain a compact floating card appearance with minimal margins (Option 2: 8px padding, 8px radius)?

➡️ **User decision**: Option 1 (True Native Edge-to-Edge) confirmed. Maximize reading area on mobile by removing all outer shell padding, eliminating window borders and rounded corners, and stretching to 100% width and 100dvh height.

---

❓ **Q2** - **Responsiveness strategy (Container Queries vs Viewport Breakpoints)**:
Per architectural guidelines, hardcoded viewport media query breakpoints (`@media (max-width: ...)`) must be avoided. Should the mobile layout be triggered via CSS container queries (`container-type: inline-size` on `body` paired with `@container (max-width: 768px)`)?

➡️ **User decision**: Yes, use modern CSS container queries. Establish `container-type: inline-size` on `body` and apply `@container (max-width: 768px)` overrides.

---

❓ **Q3** - **Viewport height dynamic scaling (`100dvh` with fallback)**:
Mobile browsers display dynamic toolbars and address bars that shrink or expand during scrolling. Should the height rule use `100dvh` (Dynamic Viewport Height) with a standard `100vh` fallback to ensure maximum screen utilization without toolbar clipping?

➡️ **User decision**: Yes, use `100dvh` with `100vh` fallback for both `.spatial-shell` and `.spatial-window`.

---

❓ **Q4** - **Preserving desktop floating spatial card styling**:
Should all existing desktop styles (`padding: clamp(0.75rem, 3vw, 2.5rem)`, `max-width: min(100cqi, 1440px)`, `height: min(90vh, 840px)`, `border-radius: var(--radius)`, `border: 1px solid var(--border)`) remain completely unchanged for larger viewports?

➡️ **User decision**: Yes, preserve all existing desktop card rules without modification.
