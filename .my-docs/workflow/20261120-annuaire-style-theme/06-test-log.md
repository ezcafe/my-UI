# Test Execution Log

- **Result:** success
- **Test path:** lite
- **Stage:** test-lite
- **Smoke source:** Build-verify
- **Date:** 2026-11-21

## Summary

The full test suite was executed across all workspace crates after implementing the Annuaire Design System theme and components. All 85 tests passed cleanly with 0 failures and 0 regressions.

| Crate / Target | Tests | Passed | Failed | Result |
|---|---|---|---|---|
| `miniflux-client` (lib) | 22 | 22 | 0 | pass |
| `server` (lib) | 39 | 39 | 0 | pass |
| `server` (e2e integration) | 2 | 2 | 0 | pass |
| `ui-components` (lib) | 22 | 22 | 0 | pass |
| **Total** | **85** | **85** | **0** | **success** |

---

## Annuaire TDD Test Suite Breakdown (`ui-components`)

All 12 planned TDD test cases from `04-tasks.md` are actively executing and passing:

1. **`test_annuaire_theme_tokens_and_toggle_present`** (NEW):
   - Asserts Google Fonts links for `Big Shoulders Display` and `Work Sans` in `base.html`.
   - Asserts Annuaire OKLCH color tokens (`--brand`, `--primary`, `--background`, `--foreground`) in `:root` and `[data-theme="dark"]`.
   - Asserts utility classes (`.font-display`, `.font-sans`, `.tabular`, `.leader`, `.shadow-float`).
   - Asserts theme toggle JavaScript (`toggleTheme()`, `localStorage`, `data-theme`).
2. **`test_login_page_renders_dual_panel_swiss_brand`** (NEW):
   - Asserts dual-panel grid layout (`grid-cols-1 md:grid-cols-2`).
   - Asserts left brand panel with rich Annuaire red background (`--brand-surface`) and `Big Shoulders Display` wordmark (`ANNUAIRE RSS`).
   - Asserts right login card with high-contrast ink pill button (`bg-primary text-primary-foreground`) for Pocket-ID.
3. **`test_filter_bar_reordering`**:
   - Asserts filter buttons in strict order: Unread < Today < Starred < All Titles.
   - Asserts active tab styled as an ink pill (`bg-primary text-primary-foreground`).
4. **`test_render_title_only_entry_item`**:
   - Asserts 3px vertical positional red bar (`bg-brand`).
   - Asserts omission of dotted leader and published_at date time per user preference.
   - Asserts omission of summary, reading time, and feed title.
   - Asserts unread indicator dot (`title="Unread"`), external link, and chevron.
5. **`test_render_toast_oob_template`**:
   - Asserts Annuaire floating elevation (`.shadow-float`).
   - Asserts ink pill floating toast styling (`rounded-full bg-foreground text-background`).
6. **`test_index_template_skeleton_structure`**:
   - Asserts `animate-pulse` placeholder animation.
   - Asserts interactive Back button wired to `closeDetailPanel()`.
   - Asserts Annuaire design token classes (`bg-muted`, `bg-card`).
7. **`test_spatial_window_fluid_width_container_query`**:
   - Asserts container query rule `container-type: inline-size;`.
   - Asserts fluid container max width `max-width: min(100cqi, 1440px);`.
8. **`test_entry_detail_wires_close_detail_panel`**:
   - Asserts detail view Back button triggers `closeDetailPanel()`.
9. **`test_detail_template_has_mark_unread_button`**:
   - Asserts mark as unread button with HTMX target `#entry-{id}` preceding bookmark toggle.
10. **`test_detail_template_omits_author`**:
    - Asserts author string is omitted from article detail view while feed title and publication date remain intact.
11. **`test_entry_list_renders_pagination_when_multiple_pages`**:
    - Asserts pagination nav bar, disabled Previous button on page 1, and active Next link.
12. **`test_entry_list_hides_pagination_when_single_page`**:
    - Asserts pagination controls are hidden when `total_pages <= 1`.

---

## Server & Miniflux Integration Tests

- `server` root route test (`test_get_root_page_serves_html_shell`): Verified that root endpoint serves the Annuaire RSS HTML shell with `#unread-pill`.
- `server` e2e tests (`test_app_starts_and_responds_to_health_check`, `test_e2e_authenticated_session_flow`): All end-to-end Axum integration flows pass.
- `miniflux-client` (22 tests): All GraphQL queries, mutations, and caching functions pass without regression.
