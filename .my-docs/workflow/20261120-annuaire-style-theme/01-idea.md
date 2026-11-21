# Framing & Intent: Annuaire Theme Application

- **Work size:** Project with written design (UI template & styling overhaul)
- **Has UI (tentative):** yes

## Takeaway & Intent Summary
We are replacing the current dark AR/spatial glass styling across the application (`crates/ui-components/templates/`) with the **Annuaire Design System** defined in `ANNUAIRE_STYLE_GUIDE.md` and `annuaire-theme.css`. The new interface will adopt an editorial, Swiss directory aesthetic featuring high information density, dual light/dark OKLCH tokens, Big Shoulders display and Work Sans body typography, ink pill actions, zero page shadows, and dotted-leader rows.

## Framing

### Goals
- **Token integration:** Embed `annuaire-theme.css` tokens and utilities (OKLCH palette, light/dark theme support, dotted leaders, custom focus rings) into `base.html`.
- **Typography:** Load `Big Shoulders` (Display) and `Work Sans` (Body) without breaking existing CDN-based zero-build architecture.
- **Template transformation:** Update `base.html`, `pages/index.html`, `pages/login.html`, and `partials/*.html` to follow Annuaire's 6 Golden Rules:
  - Red is strictly positional (wordmark, active 3px indicator, brand login panel), never action.
  - Primary actions use high-contrast ink pill buttons (`rounded-full`).
  - Page-level box shadows are removed, keeping shadows only for floating layers (toasts, menus).
  - Dates, counts, and numbers use tabular numerals (`tabular-nums lining-nums`).
  - Dotted leaders (`.leader`) connect list entries with trailing timestamps or counts.
- **Preserve interactions:** Keep all HTMX partial swapping, detail drawer navigation, mark-as-read, search, and pagination working without disruption.
- **Test alignment:** Update `crates/ui-components` template tests to verify Annuaire classes and tokens while ensuring all 20 tests pass.

### Non-Goals
- Changing backend server APIs or database models in `crates/server`.
- Modifying static showcase prototypes at the repo root (`index.html`, `ar-list.html`, `option1-workspace.html`).

### Anti-Goals
- Adding complex npm or Node.js build pipelines when standard web fonts and Tailwind CDN suffice.
- Using red for buttons, badges, or error links (violating Rule 1).
- Hardcoding viewport breakpoints instead of fluid container queries.

### Constraints
- Retain container queries (`container-type: inline-size`) for responsive layout.
- Strictly adhere to WCAG AA contrast compliance provided by `annuaire-theme.css`.

---

## Skip Option
If this framing matches your vision, you can simply say **"just start"** (or confirm) to proceed directly to the design phase.

## Shared understanding

- [x] User confirmed (brainstorm hard-gate)
- **Theme support decision:** Both Light and Dark modes with auto-detect and header toggle.
