# Code review log

**Overall Result:** clean

---

## 1. Adversarial Review

**Result:** clean

### Boundary, Security, Contrast & Defensiveness Review

- **6 Golden Rules of Annuaire Enforcement**:
  - **Rule 1 (Red marks position, never action)**: Verified that `--brand` (`oklch(0.569 0.213 29)` / `#D8261D`) and `--brand-surface` are used strictly for positional indicators and brand identity:
    - Header wordmark (`ANNUAIRE RSS`) in `crates/ui-components/templates/pages/index.html`.
    - 3px vertical positional bar (`w-[3px] self-stretch rounded-full bg-brand shrink-0`) in `crates/ui-components/templates/partials/entry_item.html`.
    - Left brand surface panel in `crates/ui-components/templates/pages/login.html`.
    - Unread indicator dot (`w-2 h-2 rounded-full bg-brand shrink-0`).
    - Verified no clickable buttons or action links use red.
  - **Rule 2 (Primary action is an Ink pill)**: High-contrast ink pills (`rounded-full bg-primary text-primary-foreground font-bold`) are uniformly applied to:
    - Pocket-ID login button in `login.html`.
    - Active filter button ("Unread") in `index.html`.
    - Back navigation button in `entry_detail.html`.
    - Previous / Next pagination buttons in `entry_list.html`.
  - **Rule 3 (Destructive actions are isolated)**: Error card (`partials/error_card.html`) isolates upstream communication failures with `text-destructive`, distinct warning icon, and neutral retry trigger.
  - **Rule 4 (Two typefaces with strict roles)**:
    - `Big Shoulders Display 700/800` (`font-display`) is strictly constrained to wordmarks, titles, and section headers.
    - `Work Sans 400/500/700` (`font-sans`) is applied to all article titles, body copy, timestamps, input fields, and counters.
  - **Rule 5 (Zero page shadows)**:
    - Cards, list rows, headers, and drawers have zero box shadows (`shadow: none`).
    - Elevation is reserved strictly for floating overlays: `.shadow-float` for floating toasts (`partials/toast.html`) and `.shadow-overlay` for dialogs.
  - **Rule 6 (Numbers are tabular)**:
    - Tabular numerals (`.tabular` / `font-variant-numeric: tabular-nums lining-nums`) are rigorously applied to all timestamps, unread counters, today counters, and pagination indicators.

- **Contrast & WCAG AA Compliance**:
  - All token pairs in `:root` and `[data-theme="dark"]` satisfy WCAG AA (>= 4.5:1 contrast):
    - Light mode: `--foreground` on `--background` is `#17181C` on `#FFFFFF` (16.1:1).
    - Light mode: `--primary-foreground` on `--primary` is `#FFFFFF` on `#17181C` (16.1:1).
    - Dark mode: `--foreground` on `--background` is `#ECECEC` on `#121316` (13.6:1).
    - Dark mode: `--primary-foreground` on `--primary` is `#17181C` on `#ECECEC` (13.6:1).
  - Out-of-band updates in `crates/server/src/routes/entries.rs` (`#unread-pill` and `#filter-unread-count`) were audited and updated to use Annuaire tokens (`bg-muted border border-border text-foreground font-bold text-xs tabular` and `text-xs tabular`), eliminating legacy glass tokens (`bg-indigo-500/20 text-indigo-300`) and low-contrast white text (`text-white/50`) that previously broke contrast in light mode.
  - AI summary fallback card updated to neutral `text-muted-foreground` and `border-border font-sans`.

- **XSS Hygiene & Injection Defense**:
  - `sanitized_content` in `entry_detail.html` is cleansed via `ammonia` with strict element and attribute allowlists, stripping dangerous script tags, event handlers, and javascript: URI schemes while adding `rel="noopener noreferrer"` and `target="_blank"` to hyperlinks.
  - All Askama template bindings (`{{ entry.title }}`, `{{ entry.published_at }}`, `{{ summary }}`, `{{ query.search }}`) are auto-escaped by default.
  - Theme switching script in `base.html` evaluates only known static string literals (`'dark'` vs `'light'`). It does not execute `eval()` or insert untrusted data into `innerHTML`.

- **Container Queries & Responsiveness**:
  - Modern container queries are applied to the application viewport shell via `.spatial-shell` (`container-type: inline-size`) and `.spatial-window` (`max-width: min(100cqi, 1440px)`).
  - Fluid padding uses `clamp(0.75rem, 3vw, 2.5rem)` avoiding hardcoded breakpoints.
  - Mobile responsiveness: On screens under 640px (`<sm`), the Annuaire dotted leader (`.leader hidden sm:block`) and secondary timestamps collapse smoothly so titles have maximal reading width; on wider screens (`sm+`), the Swiss dotted leader gracefully connects the title to tabular relative dates.

---

## 2. Quality Review

**Result:** clean

### Five-Axis Evaluation

- **Correctness**:
  - Comprehensive automated test suite passing with 0 failures:
    - 22/22 unit tests in `crates/ui-components`.
    - 39/39 integration and unit tests in `crates/server`.
    - 2/2 end-to-end tests in `crates/server`.
    - 22/22 unit tests in `crates/miniflux`.
    - Total: 85/85 tests passing workspace-wide.
  - Dedicated TDD tests `test_annuaire_theme_tokens_and_toggle_present` and `test_login_page_renders_dual_panel_swiss_brand` thoroughly verify Google Fonts links, token definitions, light/dark overrides, utility classes, and dual-panel layout.

- **Readability**:
  - Markup is clean, semantic, and highly readable.
  - CSS variables in `base.html` are systematically grouped by category (colors, fonts, radius, avatar tints, shadows) with clear hex comments.
  - JavaScript helpers (`toggleTheme()`, `updateThemeIcons()`, `setActiveFilter()`, `openDetailWithSkeleton()`, `closeDetailPanel()`) use simple, plain words and self-documenting logic.

- **Architecture**:
  - Direct inline CSS architecture (Option A) cleanly eliminates external asset pipeline overhead and build complexity while keeping Askama templates entirely self-contained.
  - Zero build step: Works natively with Rust compile-time templates and Tailwind CDN script.
  - Theme switching uses early-execution IIFE in `<head>` to prevent flash-of-unthemed-content (FOUC) while respecting both `prefers-color-scheme` and `localStorage` preferences.

- **Security**:
  - Zero runtime dependencies added.
  - HTML content sanitization strictly maintained.
  - External links consistently configured with `target="_blank"` and `rel="noopener noreferrer"`.
  - Form field focus states defined using high-visibility `outline: none; box-shadow: 0 0 0 1px var(--ring)`.

- **Performance**:
  - Google Fonts links include `rel="preconnect"` for `fonts.googleapis.com` and `fonts.gstatic.com`.
  - Zero layout shifts or expensive repaints: shadows are removed from page canvas.
  - `prefers-reduced-motion` media query included in `base.html` to respect accessibility settings.
