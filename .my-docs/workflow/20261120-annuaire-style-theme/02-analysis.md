# Analysis: Annuaire Theme Migration

- **Work slug:** `20261120-annuaire-style-theme`
- **Mode:** simple
- **Has UI / API / DB / auth:** yes | no | no | no
- **Result:** done

---

## 1. Executive Summary

We are migrating the UI component templates (`crates/ui-components/templates/`) from the dark AR/spatial glass design to the **Annuaire Design System**. The new visual identity is an editorial Swiss directory aesthetic inspired by European telephone directories: high information density, dual light/dark OKLCH tokens, Big Shoulders display and Work Sans body typography, ink pill actions, zero page shadows, and dotted-leader rows.

All existing HTMX interactions (search, filter tabs, detail drawer with skeleton, mark-as-read, pagination) will remain intact. All 20 tests in `crates/ui-components` will be preserved and updated to assert Annuaire design system classes.

---

## 2. Current Architecture & Baseline

### Template Structure
- `crates/ui-components/templates/base.html`: Main shell loading Tailwind CDN, HTMX, Lucide icons, and embedding custom CSS.
- `crates/ui-components/templates/pages/index.html`: Main dashboard with header, filter tabs, hybrid search, entry list container, and detail drawer.
- `crates/ui-components/templates/pages/login.html`: Login prompt with Pocket-ID button.
- `crates/ui-components/templates/partials/`:
  - `entry_list.html`: Iterates over `entries`, renders pagination navigation when `total_pages > 1`, and out-of-band unread count.
  - `entry_item.html`: Single entry row with read/unread indicator, title, external link, and chevron.
  - `entry_detail.html`: Article reading view with back button, mark unread, bookmark, original link, and embedded `brief_card.html`.
  - `brief_card.html`: AI executive brief callout.
  - `ai_summary_card.html`: Streamed/revealed AI summary.
  - `ai_grouped_list.html`: Feeds grouped by source for AI view.
  - `toast.html`: Out-of-band notification pill.
  - `error_card.html`: Inline error state with retry button.

### Current Test Suite
- `cargo test -p ui-components` currently passes 20/20 unit tests in `crates/ui-components/src/templates.rs` and `sanitizer.rs`.
- Key template tests verify:
  - Title-only entry item rendering and unread dot (`test_render_title_only_entry_item`).
  - Filter bar sequence: Unread < Today < Starred < All Titles (`test_filter_bar_reordering`).
  - Active filter classes (`bg-indigo-500/20 text-indigo-300 border-indigo-400/30`).
  - Fluid container query: `container-type: inline-size;` and `max-width: min(100cqi, 1440px);` (`test_spatial_window_fluid_width_container_query`).
  - Detail drawer open/close helper bindings and skeleton placeholder (`test_index_template_skeleton_structure`).
  - Out-of-band toast container swap (`test_render_toast_oob_template`).
  - Pagination visibility based on page count (`test_entry_list_renders_pagination_when_multiple_pages`).

---

## 3. Annuaire Design System Mapping

### The 6 Golden Rules vs Current Implementation

| Golden Rule | Annuaire Specification | Current Implementation | Action Required |
|---|---|---|---|
| **1. Positional Red** | Red marks position only (wordmark, 3px active bar, brand panel). Never action/delete/badge. | Indigo accents (`indigo-400`, `indigo-500`) used for unread dots, active pills, icons. | Remove indigo. Use `--brand` (`#D8261D`) strictly for wordmark, 3px active selection bar, and login brand panel. |
| **2. Ink Pill Actions** | Primary actions are high-contrast ink pills (`rounded-full`, `--primary`, `--primary-foreground`). | Glass/indigo buttons with custom borders and shadows. | Convert primary buttons to `rounded-full` ink pills with dark/light dynamic tokens. |
| **3. Isolated Destructive** | Destructive actions use dedicated red with both icon + text label. | Not currently present in RSS views; error states use rose accents. | Ensure error badges/toasts follow isolated styling without violating Rule 1. |
| **4. Strict Type Roles** | Display (`Big Shoulders 700`) for wordmark and titles; Body (`Work Sans`) for all running text. | System sans-serif font stack. | Load Google Fonts for `Big Shoulders Display` and `Work Sans`; apply `.font-display` and `.font-sans`. |
| **5. Zero Page Shadows** | Canvas elements have zero box shadows (`shadow: none`). Shadows strictly for floating layers. | Heavy glass shadows (`box-shadow: 0 24px 60px ...`). | Strip box shadows from page cards and rows; keep elevation only on `.shadow-float` (toasts, menus) and `.shadow-overlay` (modals). |
| **6. Tabular Numbers** | Dates, timestamps, phone numbers, counts use `font-variant-numeric: tabular-nums lining-nums`. | Standard variable-width numerals. | Apply `.tabular` and `tabular-nums lining-nums` to dates, unread counts, and page indicators. |

---

## 4. Theme System & Token Integration

- **CSS Variables**: Embed tokens from `annuaire-theme.css` directly into `base.html`:
  - Light mode: `:root` with `--background: oklch(1 0 0)`, `--foreground: oklch(0.21 0.008 274.5)`, etc.
  - Dark mode: `[data-theme="dark"], :root.dark` with `--background: oklch(0.187 0.006 271.1)`, etc.
- **Theme Toggle & Detection**:
  - Auto-detection script in `<head>` executes before render to prevent flash of wrong theme (FOUC).
  - Persists preference to `localStorage.getItem('theme')`.
  - Toggle button in header flips `data-theme` attribute on `<html>`.

---

## 5. Risk & Impact Analysis

- **Risk 1: Test Regressions**: Several tests in `templates.rs` check for specific Tailwind class strings (e.g. `bg-indigo-500/20 text-indigo-300`, `indigo-400`).
  - *Mitigation*: Update assertions in `templates.rs` to match Annuaire tokens/classes in lockstep with template updates, keeping all 20 tests green.
- **Risk 2: Broken HTMX Bindings**: Modifying template markup could accidentally break `hx-target`, `hx-get`, `hx-post`, or JavaScript helper functions (`openDetailWithSkeleton`, `closeDetailPanel`, `setActiveFilter`, `getVisibleEntryIds`).
  - *Mitigation*: Strictly preserve all HTML IDs, data attributes, HTMX attributes, and JavaScript function signatures.
- **Risk 3: External Dependencies**: Fontsource npm packages cannot be used in a zero-build CDN setup.
  - *Mitigation*: Use Google Fonts CDN links for `Big Shoulders Display` and `Work Sans` with `display=swap`.
