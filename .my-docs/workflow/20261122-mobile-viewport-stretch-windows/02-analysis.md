# Analysis: Mobile Viewport Full Stretch Windows

Result: done

## Context & Architecture

The application is an AR Spatial RSS reader built with Rust (Axum, Askama templates, async-graphql) and server-rendered HTML using Tailwind CSS and native CSS container queries.

Current layout structure in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html):
- **`.spatial-shell`**: Centers content on the screen with `padding: clamp(0.75rem, 3vw, 2.5rem)` and `min-height: 100vh`.
- **`.spatial-window`**: Acts as a floating spatial card with `max-width: min(100cqi, 1440px)`, `height: min(90vh, 840px)`, `border-radius: var(--radius)`, and `border: 1px solid var(--border)`.

On desktop and widescreen monitors, this floating card layout creates a clean spatial computing aesthetic. However, on mobile devices with narrow screens, this layout wastes valuable screen real estate. The margins around `.spatial-shell` and the rounded corners of `.spatial-window` compress article lists and reading views unnecessarily.

---

## Technical Problem & Requirements

### Problem Statement
Mobile users need full-width and full-height edge-to-edge reading. Preserving outer padding and borders on mobile viewports causes:
- **Wasted horizontal space**: Outer shell padding (`clamp(0.75rem, 3vw, 2.5rem)`) reduces readable line length for article content.
- **Wasted vertical space**: Height limit `min(90vh, 840px)` plus shell padding leaves dead space above and below the window, and `vh` does not account for dynamic browser chrome (navigation bars, address bars) on mobile iOS/Android.
- **Unnatural mobile aesthetics**: Floating cards with rounded borders feel out of place on handheld devices compared to native edge-to-edge mobile app screens.

### Target Requirements
1. **Mobile Viewport (`@container (max-width: 768px)` or similar compact container size)**:
   - **`.spatial-shell`**: Set `padding: 0` and `min-height: 100dvh` (with fallback to `100vh`).
   - **`.spatial-window`**: Set `width: 100%`, `max-width: 100%`, `height: 100dvh` (with fallback to `100vh`), `border-radius: 0`, and `border: none`.
2. **Desktop Viewport**:
   - Preserve existing floating spatial card styles: `padding: clamp(0.75rem, 3vw, 2.5rem)`, `min-height: 100vh`, `max-width: min(100cqi, 1440px)`, `height: min(90vh, 840px)`, `border-radius: var(--radius)`, and `border: 1px solid var(--border)`.
3. **No Hardcoded Viewport Media Queries**:
   - Must use modern CSS container queries (`@container`) rather than viewport media queries (`@media (max-width: ...)`).
   - Establish a container context on `body` (e.g. `body { container-type: inline-size; }`) so `.spatial-shell` and `.spatial-window` can query inline dimensions adaptively.

---

## Technical Feasibility & Container Context

### Container Context Hierarchy
In CSS container queries, an element can only query an ancestor container:
1. **`body`**: Define `container-type: inline-size`. This allows `.spatial-shell` and `.spatial-window` to query the overall available inline width.
2. **`.spatial-shell`**: Continues to declare `container-type: inline-size` for internal elements (such as child components inside `.spatial-window`).
3. **`.spatial-window`**: Responds to container queries by resetting margins, padding, borders, and radius when the container is compact (mobile).

### Viewport Height Units (`dvh`)
- Mobile browsers frequently show and hide URL bars and toolbar controls as users scroll.
- Standard `100vh` represents the largest possible viewport height, often causing content overflow under browser bars.
- `100dvh` (Dynamic Viewport Height) adapts dynamically to the actual visible height.
- Providing `min-height: 100vh; min-height: 100dvh;` ensures full backward compatibility with older browsers while delivering native behavior on modern mobile browsers.

---

## Risks & Mitigations

- **Risk: Nested container query evaluation**:
  - *Mitigation*: Ensure `body` has `container-type: inline-size;` so `.spatial-shell` is inside a registered container. When evaluating `@container (max-width: 768px)`, child elements query the nearest container ancestor.
- **Risk: Breaking existing Rust template unit tests**:
  - *Mitigation*: Existing unit test `test_spatial_window_fluid_width_container_query` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L645-L666) asserts `container-type: inline-size;` and `max-width: min(100cqi, 1440px);`. Desktop styles remain intact so existing assertions pass, and new tests will assert mobile container query rules.
