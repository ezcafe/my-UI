# Test Report — Paging Loading, Mark-Page-Read Toast, and Lighter Read Blur

- **Slug:** `20261125-paging-loading-toast-blur`
- **Smoke source:** `Build-verify`
- **Test path:** `lite`
- **Result:** `success`

## Smoke

- `cargo test -p ui-components`: **77 passed, 0 failed** (includes `test_paging_loading_overlay_and_read_blur_removed`)
- `cargo test -p server`: **140 unit tests passed, 8 e2e tests passed, 0 failed** (includes `test_hx_mark_page_read_advances_offset_when_more_pages_exist`, `test_hx_mark_page_read_stays_on_last_page_with_toast`, `test_hx_mark_page_read_loads_offset_zero_for_unread`)

## Test-lite

- `cargo test -p ui-components`: **77 passed, 0 failed**
- `cargo test -p server`: **140 unit tests passed, 8 e2e tests passed, 0 failed** (unit + e2e suite)
