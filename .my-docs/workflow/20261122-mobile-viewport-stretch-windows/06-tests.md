# Tests & Smoke Report

Result: success

- **Test path:** lite
- **Stage:** test-lite
- **Updated:** 2026-11-22T13:01:26+07:00

## Summary

`test-lite` suite ran targeted tests across `crates/ui-components` validating the mobile viewport stretch and responsive searchbox container query implementations. All 40 unit tests passed cleanly with 0 failures and 0 regressions.

| Crate / Target | Tests | Passed | Failed | Result |
|---|---|---|---|---|
| `crates/ui-components` (lib unittests) | 40 | 40 | 0 | success |
| `crates/ui-components` (doctests) | 0 | 0 | 0 | success |

---

## Detailed Test Breakdown

### 1. Mobile Container Query & Edge-to-Edge Stretch (`crates/ui-components/templates/base.html`)
- `test_spatial_shell_and_window_mobile_container_queries`:
  - Verified `body { container-type: inline-size; }` establishes inline containment on root.
  - Verified `@container (max-width: 768px)` override sets zero padding on `.spatial-shell`, `100%` width and `min-height: 100vh; min-height: 100dvh;`.
  - Verified `.spatial-window` stretches to `width: 100%`, `max-width: 100%`, `height: 100vh; height: 100dvh;`, `border: none`, and `border-radius: 0`.
- `test_spatial_window_desktop_card_preserved`:
  - Verified desktop card styles remain intact on viewports > 768px (`padding: clamp(0.75rem, 3vw, 2.5rem)`, `max-width: min(100cqi, 1440px)`, `height: min(90vh, 840px)`, `border-radius: var(--radius)`, `border: 1px solid var(--border)`).
- `test_spatial_window_fluid_width_container_query`:
  - Verified fluid container query sizing for intermediate viewports.

### 2. Mobile Searchbox Alignment & Compact Expansion (`crates/ui-components/templates/base.html`)
- `test_mobile_searchbox_compact_and_expand_container_query`:
  - Verified filter pills container declares `overflow-x-auto min-w-0 shrink no-scrollbar` to allow smooth horizontal touch scrolling without displacing the searchbox.
  - Verified `#search-form` declares `shrink-0` to protect search input visibility.
  - Verified `#search-input` starts at `5.5rem` (88px) compact resting width, expanding smoothly to `11rem` (176px) on focus.
  - Verified right margin of `#search-form` aligns flush with the right action buttons in header (`px-6`).

### 3. Filter Label Polish & Generic UI Error Messages
- `test_filter_bar_reordering` & guest filter tests: Verified filter pill label is renamed from "All Titles" to "All".
- `test_user_facing_error_messages_do_not_leak_internal_details`: Verified `AppError::user_facing_message` returns sanitized, human-friendly messages without leaking internal IPs, databases, or stack traces.
- `test_handle_htmx_error_returns_clean_toast_without_debug_dump`: Verified HTMX error handling returns generic toast with `HX-Reswap: none`.

### 4. Regression Suite
- All 36 additional unit tests covering Annuaire theme tokens, AI grouped lists, detail views, filter bar reordering, guest state scripts, unread counting, and HTML sanitization passed without issues.

---

## Verification Commands Executed

```bash
cargo test -p ui-components
cargo test -p server --lib
```
