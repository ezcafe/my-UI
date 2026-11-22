# Design: Mobile Viewport Full Stretch Windows

Result: done

## Executive Summary

This design implements **Option 1 (True Native Edge-to-Edge)** for the application window on mobile viewports. On smaller screens, the reader expands to 100% viewport width and height with zero outer padding and seamless flat edges. On desktop screens, it preserves the existing floating spatial card design.

The responsiveness is implemented strictly using modern CSS **container queries** (`@container`) and dynamic viewport units (`100dvh`), avoiding hardcoded media queries in full compliance with project rules.

---

## Visual & Layout Specifications

| Viewport Mode | Container Rule | `.spatial-shell` Styling | `.spatial-window` Styling |
|---|---|---|---|
| **Mobile Viewport** | `@container (max-width: 768px)` | `padding: 0`<br>`min-height: 100vh`<br>`min-height: 100dvh` | `width: 100%`<br>`max-width: 100%`<br>`height: 100vh`<br>`height: 100dvh`<br>`border-radius: 0`<br>`border: none` |
| **Desktop Viewport** | Default / `> 768px` | `padding: clamp(0.75rem, 3vw, 2.5rem)`<br>`min-height: 100vh` | `width: 100%`<br>`max-width: min(100cqi, 1440px)`<br>`height: min(90vh, 840px)`<br>`border-radius: var(--radius)`<br>`border: 1px solid var(--border)` |

---

## Architectural Options

### Option A: Desktop-First Container Query with Mobile Override (Recommended)

- **Explanation**:
  - Keep the existing desktop styles as base definitions for `.spatial-shell` and `.spatial-window`.
  - Add `container-type: inline-size` to `body`.
  - Use an `@container (max-width: 768px)` block to override padding, height, width, borders, and border-radius for mobile screens.
- **Concrete Examples**:
  ```css
  /* Body establishes the root inline container */
  body {
    container-type: inline-size;
  }

  /* Desktop defaults (Existing spatial card) */
  .spatial-shell {
    container-type: inline-size;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: clamp(0.75rem, 3vw, 2.5rem);
    background-color: var(--background);
    color: var(--foreground);
  }

  .spatial-window {
    width: 100%;
    max-width: min(100cqi, 1440px);
    height: min(90vh, 840px);
    display: flex;
    flex-direction: column;
    border-radius: var(--radius);
    overflow: hidden;
    border: 1px solid var(--border);
    background-color: var(--card);
  }

  /* Mobile Edge-to-Edge Container Override */
  @container (max-width: 768px) {
    .spatial-shell {
      padding: 0;
      min-height: 100vh;
      min-height: 100dvh;
    }

    .spatial-window {
      width: 100%;
      max-width: 100%;
      height: 100vh;
      height: 100dvh;
      border-radius: 0;
      border: none;
    }
  }
  ```
- **Pros**:
  - **Minimal diff & zero regression risk**: Preserves all existing desktop properties and assertions.
  - **Clean separation**: Mobile rules are cleanly isolated inside the container query block.
  - **Full backward compatibility**: Retains `100vh` fallback before `100dvh`.
- **Cons**:
  - Mobile styles require overriding multiple properties (`border: none`, `border-radius: 0`).

---

### Option B: Mobile-First Base with Desktop Container Query Enhancement

- **Explanation**:
  - Define mobile edge-to-edge styles as the unconstrained base styles for `.spatial-shell` and `.spatial-window`.
  - Use `@container (min-width: 769px)` to wrap the desktop floating card styling (`padding`, `max-width`, `height`, `radius`, `border`).
- **Concrete Examples**:
  ```css
  body {
    container-type: inline-size;
  }

  /* Mobile-first base styles */
  .spatial-shell {
    container-type: inline-size;
    min-height: 100vh;
    min-height: 100dvh;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--background);
    color: var(--foreground);
  }

  .spatial-window {
    width: 100%;
    max-width: 100%;
    height: 100vh;
    height: 100dvh;
    display: flex;
    flex-direction: column;
    border-radius: 0;
    border: none;
    overflow: hidden;
    background-color: var(--card);
  }

  /* Desktop Enhancement */
  @container (min-width: 769px) {
    .spatial-shell {
      padding: clamp(0.75rem, 3vw, 2.5rem);
    }
    .spatial-window {
      max-width: min(100cqi, 1440px);
      height: min(90vh, 840px);
      border-radius: var(--radius);
      border: 1px solid var(--border);
    }
  }
  ```
- **Pros**:
  - Adheres strictly to mobile-first CSS architecture principles.
  - Avoids `border: none` and `border-radius: 0` overrides.
- **Cons**:
  - In environments where container queries are partially supported or unparsed, the desktop view would degrade to a full-screen mobile layout.
  - Requires updating existing Rust unit test assertions that check for desktop properties directly on the base class.

---

### Decision & Recommendation

**Option A (Desktop-First with Mobile Container Query Override) is recommended.**
- Preserves the existing desktop spatial layout without risking regressions.
- Keeps existing unit tests passing without modification while allowing targeted assertions for the mobile container query block.
- Delivers the exact edge-to-edge mobile experience requested in ideation.

---

## Detailed Implementation Plan

### 1. Template CSS Update ([crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html))
- Add `body { container-type: inline-size; }` within the embedded `<style>` block.
- Append `@container (max-width: 768px)` block overriding `.spatial-shell` and `.spatial-window` as described in Option A.

### 2. Rust Unit Tests ([crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs))
- Maintain `test_spatial_window_fluid_width_container_query`.
- Add `test_mobile_viewport_edge_to_edge_container_queries` to verify:
  - `body` declares `container-type: inline-size;`.
  - `@container (max-width: 768px)` is defined.
  - Inside the container query:
    - `.spatial-shell` has `padding: 0;` and `min-height: 100dvh;`.
    - `.spatial-window` has `max-width: 100%;`, `height: 100dvh;`, `border-radius: 0;`, and `border: none;`.

---

## Non-Functional Requirements

- **Performance**: Pure CSS rules evaluated natively by the browser layout engine. No JavaScript event listeners (`resize` / `matchMedia`) or layout thrashing.
- **Accessibility & Touch**: Removing borders on mobile maximizes touch target space for article rows and navigation buttons.
- **Browser Compatibility**: Fully supported in modern browsers (Chrome 105+, Safari 16+, Firefox 110+). Double declaration of `100vh` and `100dvh` provides reliable graceful fallback.

---

## References

- [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L241-L263)
- [MDN CSS Container Queries](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries)
- [MDN Viewport Concepts & dvh](https://developer.mozilla.org/en-US/docs/Web/CSS/length#dvh)
