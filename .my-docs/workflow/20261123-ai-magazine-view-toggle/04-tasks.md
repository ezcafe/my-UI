# Tasks: AI Magazine View Toggle & Hot Topic Highlighting

## Planned Tests

- [x] `test_entry_view_model_category_and_image_url`: Verifies that `EntryViewModel` supports optional `category_title` and `image_url` fields and serializes/deserializes properly.
- [x] `test_extract_first_image_url`: Verifies regex extraction of image src from various HTML formats (standard `img`, uppercase, with single/double quotes, and returns `None` when absent).
- [x] `test_entry_list_template_renders_magazine_view_and_cards`: Verifies that `EntryListTemplate` renders both list view and magazine view containers, including photo elements, category badges, briefs, and title click handlers.
- [x] `test_index_template_renders_ai_toggle_button_and_scripts`: Verifies that `index.html` includes the `#ai-toggle-btn` positioned directly after `#theme-toggle-btn`, and defines `toggleAiView`, `applyAiViewMode`, `recordSourceClick`, and hot topic styling logic.

---

## Milestone 1: Data Model & Extraction Helper

- [x] **Task 1.1: TDD tests for `EntryViewModel` and image extraction**
  - Path: `crates/ui-components/src/templates.rs` & `crates/server/src/routes/entries.rs`
  - Action: Write tests verifying `category_title`, `image_url` fields, and `extract_first_image_url` helper.
- [x] **Task 1.2: Update `EntryViewModel` definition**
  - Path: `crates/ui-components/src/templates.rs`
  - Action: Add `pub category_title: Option<String>` and `pub image_url: Option<String>` to `EntryViewModel` with `#[serde(default)]`.
- [x] **Task 1.3: Implement image & category extraction in server routes**
  - Path: `crates/server/src/routes/entries.rs` & `crates/server/src/routes/pages.rs`
  - Action: Extract `image_url` and `category_title` when mapping entries to `EntryViewModel`.

---

## Milestone 2: Magazine Grid Layout in `entry_list.html`

- [x] **Task 2.1: TDD test for dual-view template output**
  - Path: `crates/ui-components/src/templates.rs`
  - Action: Add `test_entry_list_template_renders_magazine_view_and_cards`.
- [x] **Task 2.2: Implement magazine view markup in `entry_list.html`**
  - Path: `crates/ui-components/templates/partials/entry_list.html`
  - Action:
    - Wrap list rows in `#entries-list-view`.
    - Add `#entries-magazine-view` with responsive grid of magazine cards.
    - Render image header if present, category badge, publication date, title, brief text, source title, and external link.
    - Ensure bottom pagination bar sits below both view containers.

---

## Milestone 3: Header AI Toggle Button & Client-Side Interaction

- [x] **Task 3.1: TDD test for AI toggle button and view mode scripts**
  - Path: `crates/ui-components/src/templates.rs`
  - Action: Add `test_index_template_renders_ai_toggle_button_and_scripts`.
- [x] **Task 3.2: Add AI button to header**
  - Path: `crates/ui-components/templates/pages/index.html`
  - Action: Add `<button id="ai-toggle-btn" ...>` immediately following `#theme-toggle-btn`.
- [x] **Task 3.3: Implement view toggle, hot topic tracking, and read state color change in `index.html`**
  - Path: `crates/ui-components/templates/pages/index.html`
  - Action:
    - Implement `initAiViewMode()`, `toggleAiView()`, `applyAiViewMode(mode)`.
    - Default to `'ai'` view mode if not set in `localStorage`.
    - Implement `recordSourceClick(feedTitle)` and `updateHotTopicHighlighting()`.
    - Implement `openEntryFromMagazine(id, feedTitle)` marking title grey (`text-muted-foreground`) while staying on page.
    - Integrate magazine read state into `syncDomReadStates()`.

---

## Milestone 4: End-to-End Verification & Lints

- [x] **Task 4.1: Run all workspace unit and integration tests**
  - Verification: `cargo test` passing with zero failures (189 tests passing).
- [x] **Task 4.2: Verify with Clippy**
  - Verification: `cargo clippy --workspace --all-targets` with 0 warnings.
