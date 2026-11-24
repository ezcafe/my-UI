# Test Log

- **Stage:** test-full
- **Result:** success
- **Smoke source:** Build-verify
- **Test path:** full
- **Timestamp:** 2026-11-24T14:37:00+07:00

## Test Summary

- `ui_components` unit tests: 64 passed, 0 failed
- Total: 64 tests passing in the modified UI domain with 0 failures.

## Key Verifications

1. **CSS Container Queries for Master-Detail**:
   - `test_workspace_layout_container_queries_and_ar_tokens_in_css`: Verified `base.html` and `annuaire-theme.css` define `.workspace-layout` with `container-name: workspace;`, `@container workspace (min-width: 58rem)` split view, 48px gaze targets, and waveguide-safe dark backdrop.
2. **Strict Elimination of Hardcoded Viewport Media Queries**:
   - `test_entry_list_no_hardcoded_viewport_media_queries`: Verified complete absence of `lg:grid-cols`, `sm:flex-row`, `sm:w-1/2`, `md:p-6`, and `md:text-xl` in `crates/ui-components/templates/partials/entry_list.html`.
3. **Workspace Layout & Desktop Empty Placeholder**:
   - `test_workspace_layout_and_empty_placeholder_rendered`: Verified `index.html` structure correctly wraps content in `.workspace-layout` and provides `#detail-empty-placeholder` with "Select an article to read".
4. **Master-Detail JavaScript State Synchronization**:
   - JavaScript state synchronization cleanly toggles `.has-content`, maintains active row highlight without premature mark-read, and gracefully handles URL parameters.
