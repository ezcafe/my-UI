# Implementation Tasks

## Overview
This task breakdown implements the design across templates, frontend caching scripts, database analysis reuse, and LLM prompt rules.

---

## Planned tests

1. **`test_executive_brief_card_has_single_container`**:
   Verify in `crates/ui-components/src/templates.rs` that `ExecutiveBriefCardTemplate` renders only `#page-executive-brief` and does not contain `#list-executive-brief` or `hx-swap-oob`.
2. **`test_index_template_instant_detail_title_and_cache_validation`**:
   Verify in `crates/ui-components/src/templates.rs` that `openEntryDetail` and `openDetailWithSkeleton` extract and render the title immediately, and that cache validation does not require `'Executive Brief'`.
3. **`test_get_entry_analysis_htmx_reuses_database_without_llm`**:
   Verify in `crates/server/src/routes/entries.rs` that when an entry has `impact` or `fallacies` stored in the database, `/hx/entries/{id}/analysis` returns the rendered cards without querying upstream Miniflux or calling the LLM.
4. **`test_ai_service_analyze_entry_reuses_existing_db_record`**:
   Verify in `crates/server/src/ai/mod.rs` that `analyze_entry_impact_and_fallacies` returns existing impact and fallacies from the database without invoking the LLM runner.
5. **`test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid`**:
   Verify in `crates/server/src/ai/llm.rs` that `format_batch_prompt_structured` includes rules: under 110 words, 30–60 words sweet spot, inverted pyramid (outcome and impacts in first 2 sentences), 2–3 bullet points, and no filler.
6. **`test_rewrite_and_analyze_prompt_rules`**:
   Verify in `crates/server/src/ai/llm.rs` that `format_rewrite_and_analyze_prompt` includes rules: under 110 words, 30–60 words sweet spot, inverted pyramid, and no boilerplate.

---

## Task Breakdown

### Task 1: Clean Executive Brief Card Markup
- [x] In `crates/ui-components/templates/partials/executive_brief_card.html`, remove `<div id="list-executive-brief" hx-swap-oob="true">`.
- [x] In `crates/ui-components/templates/pages/index.html`, update the `htmx:afterSwap` listener so that when `#page-executive-brief` swaps, its text is copied to `#list-executive-brief` (if present) and the skeleton pulse is removed.
- [x] Update and run unit tests in `crates/ui-components/src/templates.rs`.

### Task 2: Instant Detail Title & Brief from Cache
- [x] In `crates/ui-components/templates/pages/index.html` (`openEntryDetail`):
  - Fix `isCacheValid` check to remove the obsolete check for `'Executive Brief'`.
  - Extract the entry title from `#entry-title-${numId}` and pass it to `openDetailWithSkeleton`.
- [x] In `openDetailWithSkeleton`:
  - Render the extracted title in an `<h1>` tag immediately.
  - If cached brief/summary is available from DOM/cache, display it immediately in place of gray pulse bars.
- [x] Verify detail panel opening behavior and unit tests.

### Task 3: Database Reuse for Analysis
- [x] In `crates/server/src/routes/entries.rs` (`get_entry_analysis_htmx`):
  - Check if `record.impact` or `record.fallacies` exists and is non-empty. If so, return `render_analysis_cards_fragment` immediately.
- [x] In `crates/server/src/ai/mod.rs` (`analyze_entry_impact_and_fallacies`):
  - Query `self.db.get_summary_and_title(entry_id).await` first. If `impact` or `fallacies` is present, return immediately without calling LLM.
- [x] Add integration unit tests verifying zero LLM calls on cached analysis.

### Task 4: Inverted Pyramid & Concise Rewrite Prompts
- [x] In `crates/server/src/ai/llm.rs`:
  - Update `format_batch_prompt_structured` with rules: under 110 words, 30–60 sweet spot, inverted pyramid (lead with outcome/impact in first 2 sentences), 2–3 bullet points, no intro/closing filler.
  - Update `format_rewrite_and_analyze_prompt` with the same concise rewrite rules.
- [x] Update assertions in `ai::llm::tests` to verify prompt constraints and schemas.

### Task 5: Verification and Clean Runs
- [x] Run `cargo test -p ui-components`.
- [x] Run `cargo test -p server --lib ai::llm`.
- [x] Run full test suite verification.
