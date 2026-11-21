# Tasks: Annuaire Theme Implementation

- **Work slug:** `20261120-annuaire-style-theme`
- **Mode:** simple
- **Has UI / API / DB / auth:** yes | no | no | no
- **Result:** done

---

## 1. Planned Tests (TDD Contract)

All implementation steps strictly follow test-driven development (TDD). The test suite is in `crates/ui-components/src/templates.rs`.

### Existing Unit Tests to Update / Align
1. **`test_filter_bar_reordering`**:
   - **Assertion updates**: Change active filter button classes from legacy glass tokens (`bg-white/20`) to Annuaire ink pill tokens (`bg-primary text-primary-foreground font-bold`).
   - **Verification**: Verifies tab ordering (`Unread < Today < Starred < All Titles`) and ink pill styling.
2. **`test_render_title_only_entry_item`**:
   - **Assertion updates**: Verify 3px vertical brand red bar (`bg-brand`), dotted leader (`.leader`), tabular timestamp (`entry.published_at`), external link, and `title="Unread"`.
3. **`test_spatial_window_fluid_width_container_query`**:
   - **Verification**: Verifies container query wrapper (`container-type: inline-size;` and `max-width: min(100cqi, 1440px);`).
4. **`test_index_template_skeleton_structure`**:
   - **Verification**: Verifies skeleton pulse animation, Annuaire token placeholders, and interactive back trigger (`closeDetailPanel()`).
5. **`test_render_toast_oob_template`**:
   - **Assertion updates**: Verifies toast floating pill classes (`.shadow-float`, `rounded-full bg-foreground text-background`).
6. **`test_index_template_renders_ai_button_when_enabled`**:
   - **Verification**: Verifies AI filter button positioning, HTMX attributes, and pill shape.
7. **`test_index_template_defines_detail_panel_helpers`**:
   - **Verification**: Verifies JavaScript helpers `openDetailWithSkeleton` and `closeDetailPanel`.
8. **`test_entry_detail_wires_close_detail_panel`**:
   - **Verification**: Verifies detail view back button triggers `closeDetailPanel()`.
9. **`test_detail_template_has_mark_unread_button`**:
   - **Verification**: Verifies mark unread and bookmark buttons with correct HTMX targets.
10. **`test_entry_list_renders_pagination_when_multiple_pages`**:
    - **Verification**: Verifies pagination nav bar rendering and disabled state on first page.

### New Unit Tests to Add
11. **`test_annuaire_theme_tokens_and_toggle_present`**:
    - **RED**: Fails until `base.html` includes:
      - Annuaire OKLCH color tokens for `:root` and `[data-theme="dark"]`.
      - Google Fonts links for `Big Shoulders Display` and `Work Sans`.
      - Helper utilities `.font-display`, `.font-sans`, `.tabular`, `.leader`.
      - Theme auto-detection and `toggleTheme()` inline script.
    - **GREEN**: Passes once tokens, fonts, and theme toggle are embedded in `base.html`.
12. **`test_login_page_renders_dual_panel_swiss_brand`**:
    - **RED**: Fails until `login.html` contains:
      - Left brand panel with `--brand-surface` background and `Big Shoulders Display` wordmark.
      - Right login panel with high-contrast ink pill button for Pocket-ID.
    - **GREEN**: Passes once `pages/login.html` is updated to the Annuaire dual-panel layout.

---

## 2. Implementation Tasks (Step-by-Step TDD)

### Task 1: Design Tokens, Typography & Shell (`base.html`)
- **TDD Step**:
  1. Add `test_annuaire_theme_tokens_and_toggle_present` in `crates/ui-components/src/templates.rs` (RED).
  2. Embed Annuaire OKLCH tokens, Google Fonts links, utility classes, and theme switcher script in `crates/ui-components/templates/base.html` (GREEN).
  3. Verify with `cargo test -p ui-components -- test_annuaire_theme_tokens_and_toggle_present`.

### Task 2: Header, Filter Bar & Dashboard Shell (`pages/index.html`)
- **TDD Step**:
  1. Update assertions in `test_filter_bar_reordering` for active tab ink pill classes (RED).
  2. Update `crates/ui-components/templates/pages/index.html`:
     - Wordmark "ANNUAIRE RSS" in `font-display text-brand`.
     - Tabular unread badge.
     - Header theme toggle button calling `toggleTheme()`.
     - Filter tabs as ink pills (`bg-primary text-primary-foreground`).
     - Search input with `.field` styling.
  3. Run `cargo test -p ui-components` to confirm tests pass (GREEN).

### Task 3: Entry List & Dotted Leader Row Layout (`partials/entry_item.html`, `partials/entry_list.html`)
- **TDD Step**:
  1. Update `test_render_title_only_entry_item` to assert dotted leader `.leader` and tabular timestamp (RED).
  2. Update `crates/ui-components/templates/partials/entry_item.html`:
     - 3px vertical positional red bar (`bg-brand`).
     - Title in `font-sans text-sm font-medium text-foreground`.
     - Dotted leader span (`.leader hidden sm:block`).
     - Relative publication timestamp with `.tabular`.
  3. Update `crates/ui-components/templates/partials/entry_list.html` for clean tabular pagination.
  4. Run `cargo test -p ui-components` (GREEN).

### Task 4: Detail Drawer & Executive Brief (`partials/entry_detail.html`, `partials/brief_card.html`)
- **TDD Step**:
  1. Verify existing detail drawer tests `test_entry_detail_wires_close_detail_panel` and `test_detail_template_has_mark_unread_button`.
  2. Update `crates/ui-components/templates/partials/entry_detail.html`:
     - Back button styled as ink pill (`rounded-full bg-primary text-primary-foreground`).
     - Metadata in `Work Sans` with tabular figures.
     - Zero page shadows on detail container (`border-l border-border bg-muted`).
  3. Update `crates/ui-components/templates/partials/brief_card.html` with Annuaire typography and subtle border.
  4. Run `cargo test -p ui-components` (GREEN).

### Task 5: Swiss Red Brand Panel Login Screen (`pages/login.html`)
- **TDD Step**:
  1. Add `test_login_page_renders_dual_panel_swiss_brand` in `crates/ui-components/src/templates.rs` (RED).
  2. Update `crates/ui-components/templates/pages/login.html`:
     - Dual-panel grid (`grid-cols-1 md:grid-cols-2`).
     - Left brand panel: deep red `--brand-surface` with `Big Shoulders Display` wordmark and directory subtitle.
     - Right panel: clean card with high-contrast ink pill button for Pocket-ID.
  3. Run `cargo test -p ui-components -- test_login_page_renders_dual_panel_swiss_brand` (GREEN).

### Task 6: Floating Toasts & Error States (`partials/toast.html`, `partials/error_card.html`)
- **TDD Step**:
  1. Update `test_render_toast_oob_template` to assert Annuaire `.shadow-float` floating pill classes.
  2. Update `crates/ui-components/templates/partials/toast.html` to `.shadow-float` pill.
  3. Update `crates/ui-components/templates/partials/error_card.html` to neutral card with retry pill button.
  4. Run `cargo test -p ui-components` (GREEN).

### Task 7: Full Test Suite Verification & Visual Sanity Check
- **Verification**:
  - Run all unit tests: `cargo test -p ui-components`.
  - Ensure all 22 tests pass cleanly with zero warnings or errors.
