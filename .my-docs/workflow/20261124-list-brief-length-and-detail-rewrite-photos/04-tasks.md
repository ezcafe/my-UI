# Implementation Tasks

## Overview
This task breakdown implements the 3 to 5 sentence brief length requirement across prompts and templates, alongside the detail page rewrite with original photos (Option 2 magazine hero image + body photos).

---

## Planned tests

1. **`test_extract_article_images_hero_and_body`**:
   Verify in `crates/server/src/routes/entries.rs` that `extract_article_images` correctly extracts the first image as `hero_image` and remaining valid images as `body_images`, while ignoring 1x1 tracking pixels and feedburner gifs.
2. **`test_extract_article_images_empty_and_no_images`**:
   Verify in `crates/server/src/routes/entries.rs` that `extract_article_images` returns `(None, vec![])` when the content has no images or empty string.
3. **`test_entry_detail_template_renders_hero_image_with_overlaid_title`**:
   Verify in `crates/ui-components/src/templates.rs` that `EntryDetailTemplate` with `hero_image = Some("https://example.com/hero.jpg".to_string())` renders the hero image container with `bg-gradient-to-t`, `relative z-10`, and overlaid title text.
4. **`test_entry_detail_template_renders_without_hero_image`**:
   Verify in `crates/ui-components/src/templates.rs` that `EntryDetailTemplate` with `hero_image = None` renders the clean typography title header without any hero container or broken image tags.
5. **`test_entry_detail_template_renders_body_images`**:
   Verify in `crates/ui-components/src/templates.rs` that `EntryDetailTemplate` with `body_images` renders the responsive article photo grid beneath the rewrite content.
6. **`test_format_prompt_requires_strictly_3_to_5_sentences`**:
   Verify in `crates/server/src/ai/llm.rs` that `format_prompt` prompt text explicitly includes "strictly 3 to 5 concise sentences".
7. **`test_format_batch_prompt_requires_strictly_3_to_5_sentences`**:
   Verify in `crates/server/src/ai/llm.rs` that `format_batch_prompt_structured` prompt text requires strictly 3 to 5 sentences for both the top executive brief and item rewrites.
8. **`test_magazine_view_brief_line_clamping`**:
   Verify in `crates/ui-components/src/templates.rs` that rendered magazine cards contain `line-clamp-6` on Priority 1 and `line-clamp-4` on Priority 2.
9. **`test_detail_panel_slide_in_animation_and_no_split_on_load`**:
   Verify in `crates/ui-components/src/templates.rs` that `base.html` defines `.animate-slide-in-right`, does not force wide desktop split on initial load, and keeps `#detail-back-btn` accessible.


---

## Task Breakdown

### Task 1: Prompt Guidelines and Fallback Extractor (3 to 5 Sentences)
- [x] In `crates/server/src/ai/llm.rs`:
  - Update `format_prompt` to require "strictly 3 to 5 concise sentences".
  - Update `format_batch_prompt_structured` to require "strictly 3 to 5 sentences" for executive brief and item rewrites.
  - Update existing tests in `crates/server/src/ai/llm.rs` to assert "3 to 5 sentences".
- [x] In `crates/server/src/ai/extractor.rs`:
  - Ensure teaser extraction uses `miniflux_client::summary::extract_summary(teaser, 3)`.
- [x] In `crates/miniflux/src/summary.rs`:
  - Verify extractive summary unit tests support extracting 3 sentences.

### Task 2: List View Line Clamping
- [x] In `crates/ui-components/templates/partials/entry_list.html`:
  - Change Priority 1 card brief line clamping from `line-clamp-4` to `line-clamp-6`.
  - Change Priority 2 card brief line clamping from `line-clamp-3` to `line-clamp-4`.
- [x] Add unit test in `crates/ui-components/src/templates.rs` verifying the updated line-clamp classes in rendered markup.

### Task 3: Image Extraction Logic in Server Routes
- [x] In `crates/server/src/routes/entries.rs`:
  - Implement `extract_article_images(html: &str) -> (Option<String>, Vec<String>)` with tracking pixel and gif filtering.
  - In `get_entry_detail`, call `extract_article_images(content)` to obtain `(hero_image, body_images)`.
  - Pass `hero_image` and `body_images` into `EntryDetailTemplate`.
- [x] Add unit tests in `crates/server/src/routes/entries.rs` covering image extraction with multiple images, tracking pixels, and empty content.

### Task 4: Detail Panel Template (Option 2 Hero & Body Photos)
- [x] In `crates/ui-components/src/templates.rs`:
  - Add `pub hero_image: Option<String>` and `pub body_images: Vec<String>` to `EntryDetailTemplate`.
  - Update all existing test instances of `EntryDetailTemplate` with `hero_image: None, body_images: Vec::new()`.
- [x] In `crates/ui-components/templates/partials/entry_detail.html`:
  - If `hero_image` is present: Render Option 2 magazine hero container with gradient scrim and overlaid title + metadata.
  - If `hero_image` is absent: Render standard clean typography header.
  - If `body_images` is non-empty: Render responsive image grid below the rewrite box with lazy loading and error handlers.
- [x] Add unit tests in `crates/ui-components/src/templates.rs` for `EntryDetailTemplate` with hero image, without hero image, and with body images.

### Task 5: Wide Desktop Detail Panel on Click & Slide-from-Right Animation
- [x] In `crates/ui-components/templates/base.html`:
  - Define `@keyframes slideInFromRight` and `.animate-slide-in-right`.
  - Remove forced split `grid-template-columns: minmax(20rem, 24rem) 1fr` and `display: flex !important` under `@container workspace (min-width: 58rem)`.
  - Ensure `#detail-back-btn` is not hidden on wide desktop so users can close the detail view.
- [x] In `crates/ui-components/templates/pages/index.html`:
  - When opening `#detail-panel`, apply `animate-slide-in-right` so the panel slides in smoothly from the right edge.
  - On closing, remove the animation class and hide the panel.
- [x] Add unit test in `crates/ui-components/src/templates.rs` verifying the animation class and wide desktop rules.

### Task 6: End-to-End Build and Verification
- [x] Run `cargo test -p miniflux-client`.
- [x] Run `cargo test -p ui-components`.
- [x] Run `cargo test -p server`.
- [x] Verify clean builds and tests passing.
