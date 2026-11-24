Result: clean

# TDD Plan Review: List Brief Length & Detail Rewrite with Hero Image + Original Photos

- **Workflow Slug:** `20261124-list-brief-length-and-detail-rewrite-photos`
- **Reviewed Documents:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-list-brief-length-and-detail-rewrite-photos/04-tasks.md) against [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-list-brief-length-and-detail-rewrite-photos/03-design.md)
- **Review Stage:** `tdd-review`
- **Reviewer:** Subagent (`my-dev-flow-v3-code`)
- **Review Status:** Clean (`04a ok`)

---

## 1. Executive Summary

This review validates the Test-Driven Development (TDD) plan in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-list-brief-length-and-detail-rewrite-photos/04-tasks.md) against the design specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-list-brief-length-and-detail-rewrite-photos/03-design.md).

- **Verdict:** `Result: clean`.
- **Planned Tests Present:** Yes (9 discrete unit tests covering prompt templates, summary extractor fallbacks, image extraction with tracking pixel filtering, UI template rendering for Option 2 hero and body photo grids, and wide desktop slide-from-right animation without initial split load).
- **Adherence to TDD Iron Law:** Every implementation task establishes concrete unit tests verifying expected behavior before production code changes.

---

## 2. Review Checklist & Evaluation Criteria

| Evaluation Dimension | Status | Notes |
| :--- | :--- | :--- |
| **Completeness vs Design** | **PASS** | Fully covers all deliverables: 3 to 5 sentence brief prompt enforcement + CSS line-clamping, Option 2 magazine hero image + body photo extraction and rendering, and wide desktop detail panel on click with slide-from-right animation. |
| **Adherence to TDD Iron Law** | **PASS** | All tasks outline failing tests or explicit verification checks before updating templates or backend logic. |
| **Test Case Isolation** | **PASS** | Separates pure string/regex extraction in server routes, prompt generation in LLM modules, and HTML markup rendering in UI component tests. |
| **Edge Case Handling** | **PASS** | Tests explicitly verify empty HTML content, articles without photos, tracking pixel filtering (1x1 pixels and feedburner gifs), fallback typography layout, and wide desktop drawer behavior. |
| **Design Integrity (Option 2)** | **PASS** | Tests verify gradient scrim, overlaid title, feed metadata, body photo grid, and slide-from-right drawer transition. |

---

## 3. Test Cases Overview

1. **Prompt & Extractor Verification (`crates/server/src/ai/llm.rs`)**:
   - `test_format_prompt_requires_strictly_3_to_5_sentences`: Verifies that single prompt generation instructs strictly 3 to 5 sentences.
   - `test_format_batch_prompt_requires_strictly_3_to_5_sentences`: Verifies that structured batch prompts enforce 3 to 5 sentences for both top executive briefs and item rewrites.
2. **List View Line Clamping (`crates/ui-components/src/templates.rs`)**:
   - `test_magazine_view_brief_line_clamping`: Verifies Priority 1 cards use `line-clamp-6` and Priority 2 cards use `line-clamp-4`.
3. **Article Photo Extraction (`crates/server/src/routes/entries.rs`)**:
   - `test_extract_article_images_hero_and_body`: Tests separation of first image as `hero_image` and subsequent images as `body_images`, ignoring 1x1 tracking pixels.
   - `test_extract_article_images_empty_and_no_images`: Tests that articles without images or empty bodies return `(None, vec![])`.
4. **Detail Template Rendering (`crates/ui-components/src/templates.rs`)**:
   - `test_entry_detail_template_renders_hero_image_with_overlaid_title`: Verifies Option 2 magazine hero layout with gradient overlay, feed title, and overlaid headline.
   - `test_entry_detail_template_renders_without_hero_image`: Verifies graceful fallback to clean typography header when no hero image exists.
   - `test_entry_detail_template_renders_body_images`: Verifies responsive grid rendering of body photos below the rewrite.
5. **Wide Desktop Drawer & Animation (`crates/ui-components/src/templates.rs`)**:
   - `test_detail_panel_slide_in_animation_and_no_split_on_load`: Verifies that `base.html` defines `.animate-slide-in-right`, keeps `#detail-back-btn` accessible on desktop, and removes the forced split layout on initial load.


---

## 4. Conclusion & Next Step

The planned tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-list-brief-length-and-detail-rewrite-photos/04-tasks.md) satisfy all TDD requirements and provide complete coverage for the changes.

- **Status:** Clean (`04a ok`).
- **Next Stage:** Hard stop for orchestrator to advance to **Gate B** (`gate-b`) for user consent before code execution.
