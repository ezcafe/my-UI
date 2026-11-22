# Tasks: Mobile Viewport Full Stretch Windows

Result: done

## Tasks Breakdown

### Phase 1: TDD Unit Tests (UI Components Crate)
- [x] Task 1.1: Add unit test `test_spatial_shell_and_window_mobile_container_queries` in `crates/ui-components/src/templates.rs` asserting `body { container-type: inline-size; }` and mobile container query declarations.
- [x] Task 1.2: Add unit test `test_spatial_window_desktop_card_preserved` in `crates/ui-components/src/templates.rs` ensuring desktop spatial card rules are untouched.

### Phase 2: Template CSS Implementation (Base Template)
- [x] Task 2.1: Update `crates/ui-components/templates/base.html` to declare `body { container-type: inline-size; }` inside the base `<style>` block.
- [x] Task 2.2: Add `@container (max-width: 768px)` CSS block in `crates/ui-components/templates/base.html` setting:
  - `.spatial-shell`: `padding: 0; min-height: 100vh; min-height: 100dvh;`
  - `.spatial-window`: `width: 100%; max-width: 100%; height: 100vh; height: 100dvh; border-radius: 0; border: none;`

### Phase 3: Verification
- [x] Task 3.1: Run `cargo test -p ui-components` to verify all template rendering assertions pass.
- [x] Task 3.2: Verify that existing desktop fluid container query tests pass without regressions.

### Phase 4: Mobile Searchbox Alignment & Compact Focus Expansion (Option 2)
- [x] Task 4.1: Add TDD unit test `test_mobile_searchbox_compact_and_expand_container_query` in `crates/ui-components/src/templates.rs` verifying mobile `#search-input` resting width, focus expansion, and filter pill shrink/scroll behavior.
- [x] Task 4.2: Update `crates/ui-components/templates/base.html` within `@container (max-width: 768px)` with `#search-form { flex-shrink: 0; }` and `#search-input { width: 5.5rem; transition: width ...; } #search-input:focus { width: 11rem; }`, plus `.no-scrollbar` utility.
- [x] Task 4.3: Update `crates/ui-components/templates/pages/index.html` to add `overflow-x-auto min-w-0 shrink no-scrollbar` on the filter pill row and `shrink-0` on `#search-form`.
- [x] Task 4.4: Verify full test suite passes with `cargo test -p ui-components`.

### Phase 5: Filter Label Polish & Generic UI Error Messages
- [x] Task 5.1: Rename filter label "All Titles" to "All" in `crates/ui-components/templates/pages/index.html` and update unit tests in `crates/ui-components/src/templates.rs`.
- [x] Task 5.2: Replace technical debug dumps with general user-friendly error messages in `crates/server/src/error.rs` (`AppError::user_facing_message`), detail panel `ErrorCardTemplate` in `crates/server/src/routes/entries.rs`, and client-side error fallback in `index.html`.
- [x] Task 5.3: Add unit tests for generic error responses in `crates/server/src/error.rs` and verify workspace tests pass.

---

## Planned Tests

1. `test_spatial_shell_and_window_mobile_container_queries`:
   - Validates that `IndexTemplate` renders `body { container-type: inline-size; }` (or equivalent container context).
   - Validates that the rendered HTML contains `@container (max-width: 768px)`.
   - Validates that within the mobile container query, `.spatial-shell` has `padding: 0;` and `min-height: 100dvh;`.
   - Validates that `.spatial-window` has `max-width: 100%;`, `height: 100dvh;`, `border-radius: 0;`, and `border: none;`.

2. `test_spatial_window_desktop_card_preserved`:
   - Validates that `IndexTemplate` retains desktop card styles:
     - `.spatial-shell` has `padding: clamp(0.75rem, 3vw, 2.5rem);`.
     - `.spatial-window` has `max-width: min(100cqi, 1440px);` and `height: min(90vh, 840px);`.
