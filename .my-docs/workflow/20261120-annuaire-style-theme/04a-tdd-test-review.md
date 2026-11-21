# TDD test plan review

**Result:** clean

- **Work slug:** `20261120-annuaire-style-theme`
- **Review date:** 2026-11-20
- **Mode:** simple
- **Scope:** Review planned TDD tests in `04-tasks.md` section 1 against `03-design.md` specifications.

---

## 1. Planned Cases Evaluation

The test plan in `04-tasks.md` specifies 12 distinct unit test cases (10 existing tests updated/aligned and 2 new tests):

| # | Test Case | Target / Verification | Design Alignment | Status |
|---|-----------|-----------------------|------------------|--------|
| 1 | `test_filter_bar_reordering` | Active tab styling changed to ink pill (`bg-primary text-primary-foreground font-bold`); tab order preserved. | Aligns with Golden Rule 2 (Ink pills) and Section 6.3. | Clean |
| 2 | `test_render_title_only_entry_item` | 3px vertical red bar (`bg-brand`), dotted leader (`.leader`), tabular timestamp (`entry.published_at`), `title="Unread"`. | Aligns with Settled Decision Q1 (Option A) & Golden Rule 1 (Red marks position). | Clean |
| 3 | `test_spatial_window_fluid_width_container_query` | Container query styling (`container-type: inline-size; max-width: min(100cqi, 1440px)`). | Aligns with Section 6.1 (Container query architecture). | Clean |
| 4 | `test_index_template_skeleton_structure` | Skeleton pulse animation, token classes, `closeDetailPanel()` wiring. | Aligns with Section 6.5 (Detail drawer & skeleton). | Clean |
| 5 | `test_render_toast_oob_template` | Toast floating pill (`.shadow-float`, `rounded-full bg-foreground text-background`). | Aligns with Golden Rule 5 (Floating elevation) & Section 6.7. | Clean |
| 6 | `test_index_template_renders_ai_button_when_enabled` | AI filter button positioning, HTMX target, pill styling. | Aligns with Section 6.3. | Clean |
| 7 | `test_index_template_defines_detail_panel_helpers` | JavaScript helpers `openDetailWithSkeleton` and `closeDetailPanel`. | Aligns with Section 6.5. | Clean |
| 8 | `test_entry_detail_wires_close_detail_panel` | Detail view back trigger wires to `closeDetailPanel()`. | Aligns with Section 6.5. | Clean |
| 9 | `test_detail_template_has_mark_unread_button` | Mark unread & bookmark buttons with HTMX targets. | Aligns with Section 6.5. | Clean |
| 10 | `test_entry_list_renders_pagination_when_multiple_pages` | Pagination nav bar with disabled/active pill buttons. | Aligns with Section 6.1 & Golden Rule 2. | Clean |
| 11 | `test_annuaire_theme_tokens_and_toggle_present` (New) | OKLCH color tokens, fonts link, `.font-display`, `.font-sans`, `.tabular`, `.leader`, theme script. | Aligns with Section 2 (Option A), Section 4, Section 5, and Settled Decision Q2. | Clean |
| 12 | `test_login_page_renders_dual_panel_swiss_brand` (New) | Dual-panel layout: left `--brand-surface` with `Big Shoulders Display`, right card with Pocket-ID ink pill. | Aligns with Settled Decision Q3 (Option A) and Section 6.6. | Clean |

---

## 2. Coverage Analysis Against The 6 Golden Rules

1. **Red marks position, never action**: Verified by `test_render_title_only_entry_item` (3px active bar), `test_annuaire_theme_tokens_and_toggle_present` (wordmark styling), and `test_login_page_renders_dual_panel_swiss_brand` (brand surface panel). No actions use red.
2. **Primary action is an Ink pill**: Verified by `test_filter_bar_reordering`, `test_render_toast_oob_template`, `test_login_page_renders_dual_panel_swiss_brand`.
3. **Destructive actions are isolated**: Covered by existing `ErrorCardTemplate` isolation tests.
4. **Two typefaces with strict roles**: Verified by `test_annuaire_theme_tokens_and_toggle_present` and `test_login_page_renders_dual_panel_swiss_brand`.
5. **Zero page shadows**: Verified across item and layout tests; shadow assertions restricted to `.shadow-float` in `test_render_toast_oob_template`.
6. **Numbers are tabular**: Verified by `test_render_title_only_entry_item` (`.tabular` timestamp) and `test_annuaire_theme_tokens_and_toggle_present`.

---

## 3. Conclusion

The planned test cases provide comprehensive coverage of all architectural choices, settled decisions (Q1, Q2, Q3), and design system rules specified in `03-design.md`. The TDD flow is well-defined with explicit RED/GREEN transitions.

**Result: clean**
