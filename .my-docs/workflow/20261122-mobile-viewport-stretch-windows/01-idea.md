# 01 Idea & Intent

- **Goal:** On mobile viewport size, make the application windows (`.spatial-window` and outer `.spatial-shell`) stretch 100% viewport width and height.
- **Decision:** Option 1 (True Native Edge-to-Edge) confirmed by user.
- **Scope & Behavior:**
  - On mobile screens (using container queries without hardcoded breakpoints):
    - `.spatial-shell`: `padding: 0`, `min-height: 100dvh`.
    - `.spatial-window`: `width: 100%`, `max-width: 100%`, `height: 100dvh`, `border-radius: 0`, `border: none`.
  - On desktop screens: Preserve existing floating card layout (`max-width: min(100cqi, 1440px)`, `height: min(90vh, 840px)`, border, radius, shell padding).
