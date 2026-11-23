# Test Log: AI Magazine View Toggle & Hot Topic Highlighting

- **Run slug:** `20261123-ai-magazine-view-toggle`
- **Updated:** 2026-11-23T09:22:30+07:00
- **Test path:** lite
- **Result:** success
- **Smoke source:** Smoke-Build

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 105 | 105 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 55 | 55 | 0 | PASSED |
| **Total** | **189** | **189** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Milestone Test Details

### Milestone 1: Data Model & Extraction Helper
- Verified `EntryViewModel` supports optional `category_title` and `image_url` fields with backwards-compatible serde deserialization defaults.
- Verified regex extraction of image src from HTML content via `extract_first_image_url` handles standard `img`, uppercase `IMG`, single/double quotes, and returns `None` when absent.
- Tests:
  - `templates::tests::test_entry_view_model_category_and_image_url`
  - `routes::entries::tests::test_extract_first_image_url`

### Milestone 2: Magazine Grid Layout in `entry_list.html`
- Verified `EntryListTemplate` renders both `#entries-list-view` and `#entries-magazine-view`.
- Verified magazine cards display photo banner when `image_url` is present, category badge, publication date, brief text, feed title, external link, and bottom pagination.
- Verified read status in magazine view renders grey text (`text-muted-foreground`).
- Tests:
  - `templates::tests::test_entry_list_template_renders_magazine_view_and_cards`
  - `templates::tests::test_render_entry_list_with_title_only_items`

### Milestone 3: Header AI Toggle Button & Client Scripts
- Verified `#ai-toggle-btn` positioned immediately following `#theme-toggle-btn` in header.
- Verified client-side scripts define `initAiViewMode`, `toggleAiView`, `applyAiViewMode`, `recordSourceClick`, `updateHotTopicHighlighting`, and `openEntryFromMagazine`.
- Verified default view mode is `'ai'` when `localStorage.getItem('my_news_view_mode')` is unset.
- Tests:
  - `templates::tests::test_index_template_renders_ai_toggle_button_and_scripts`
