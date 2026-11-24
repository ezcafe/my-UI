Result: clean

# TDD Plan Review: Executive Brief Duplication, Detail Caching, DB Reuse, and Prompt Rules

## 1. Summary

This document evaluates the 6 test cases planned in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/04-tasks.md) against the architectural goals in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03-design.md) and strict Test-Driven Development (TDD) best practices.

**Verdict**: The planned test cases are **clean**, complete, and accurately scoped to enforce the Red-Green-Refactor cycle across all four functional areas.

---

## 2. Assessment of Planned Test Cases

### Test 1: `test_executive_brief_card_has_single_container`
- **Target File**: `crates/ui-components/src/templates.rs`
- **Scope**: Template rendering of `ExecutiveBriefCardTemplate`.
- **Behavior Verified**:
  - Renders `<div id="page-executive-brief">` container.
  - Does **not** render duplicate `<div id="list-executive-brief">`.
  - Does **not** contain `hx-swap-oob="true"`.
- **TDD Red State**: Fails on current `executive_brief_card.html` which contains the OOB swap block.
- **Evaluation**: **Pass (Clean)**. Eliminates over-the-wire payload duplication and ensures a single HTML tree.

### Test 2: `test_index_template_instant_detail_title_and_cache_validation`
- **Target File**: `crates/ui-components/src/templates.rs`
- **Scope**: Client-side JavaScript embedded in `IndexTemplate` (`pages/index.html`).
- **Behavior Verified**:
  - `isCacheValid` logic validates `cached && cached.html` without requiring the obsolete string check `cached.html.includes('Executive Brief')`.
  - `openEntryDetail` extracts title from `#entry-title-${numId}` and forwards it to `openDetailWithSkeleton`.
  - `openDetailWithSkeleton` renders the real title inside `<h1>` immediately rather than showing blank gray pulse boxes.
  - `htmx:afterSwap` listener copies the text from `#page-executive-brief` to `#list-executive-brief` to keep both views synchronized without server duplication.
- **TDD Red State**: Fails on current `index.html` where `openDetailWithSkeleton` accepts no title and `isCacheValid` contains the broken string check.
- **Evaluation**: **Pass (Clean)**. Directly targets the 30-day offline cache regression and fixes perceived latency on article click.

### Test 3: `test_get_entry_analysis_htmx_reuses_database_without_llm`
- **Target File**: `crates/server/src/routes/entries.rs`
- **Scope**: Axum route handler `/hx/entries/{id}/analysis`.
- **Behavior Verified**:
  - When the database has either `impact` or `fallacies` stored, the endpoint immediately returns `render_analysis_cards_fragment`.
  - Verifies that zero requests are dispatched to upstream Miniflux or the AI LLM runner when cached data exists.
  - Verifies graceful handling when either field is present alone (OR condition rather than AND).
- **TDD Red State**: Fails on current handler which requires both fields to be non-empty (`&&`) and does not guard against calling LLM if one is missing.
- **Evaluation**: **Pass (Clean)**. Prevents costly redundant LLM calls and speeds up analysis card rendering.

### Test 4: `test_ai_service_analyze_entry_reuses_existing_db_record`
- **Target File**: `crates/server/src/ai/mod.rs`
- **Scope**: `AiService::analyze_entry_impact_and_fallacies`.
- **Behavior Verified**:
  - Queries `db.get_summary_and_title(entry_id)` before calling `llm.analyze_impact_and_fallacies`.
  - If existing impact or fallacies exist in the database, returns them immediately and bypasses LLM execution.
- **TDD Red State**: Fails on current `AiService` where `llm.analyze_impact_and_fallacies` is invoked unconditionally at the start of the function.
- **Evaluation**: **Pass (Clean)**. Ensures service-layer protection against duplicate AI analysis.

### Test 5: `test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid`
- **Target File**: `crates/server/src/ai/llm.rs`
- **Scope**: `LlmCliRunner::format_batch_prompt_structured`.
- **Behavior Verified**:
  - Prompt instructions specify word limits: under 110 words total, sweet spot 30–60 words.
  - Prompt requires inverted pyramid structure (lead with outcome and impact in first 2 sentences).
  - Prompt requires 2–3 concise bullet points with essential context.
  - Prompt strictly forbids introductory filler and closing boilerplate.
  - Retains compact schema `{"top":"...","items":[{"i":101,"t":"...","r":"..."}]}`.
- **TDD Red State**: Fails because existing prompt lacks the word count boundaries (<110 words, 30–60 sweet spot) and explicit boilerplate bans.
- **Evaluation**: **Pass (Clean)**. Produces concise, scannable summaries that respect reading eye flow.

### Test 6: `test_rewrite_and_analyze_prompt_rules`
- **Target File**: `crates/server/src/ai/llm.rs`
- **Scope**: `LlmCliRunner::format_rewrite_and_analyze_prompt`.
- **Behavior Verified**:
  - Single-entry prompt instructions enforce identical concise rewrite rules: under 110 words, 30–60 sweet spot, inverted pyramid, 2–3 bullets, no filler.
  - Retains multilingual requirement and JSON schema matching `DetailAnalysisResult`.
- **TDD Red State**: Fails because existing prompt lacks the 110-word ceiling and 30–60 word target.
- **Evaluation**: **Pass (Clean)**. Guarantees consistency between batch generation and single-entry detail re-generation.

---

## 3. Coverage and Edge Cases

- **DOM Synchronization**: Test 1 and Test 2 verify that removing the server OOB tag does not break the list view brief. The client-side swap handler bridges the gap cleanly.
- **Partial DB Cache**: Test 3 explicitly tests that if only `impact` exists in the database (or only `fallacies`), the system does not re-run the entire LLM pipeline needlessly.
- **Word Limits & Conciseness**: Tests 5 and 6 verify that prompt guidelines give the LLM clear boundaries, eliminating filler and keeping summaries compact.
- **Regressions**: Existing test suites (`cargo test -p ui-components` and `cargo test -p server`) remain fully green throughout development.

---

## 4. TDD Execution Sequence

During the Build phase, implementation will strictly follow the Red-Green-Refactor cycle:

1. **Phase 1: Prompts & LLM (Backend Logic)**
   - Add Test 5 and Test 6 to `crates/server/src/ai/llm.rs` $\rightarrow$ **Verify Red**
   - Update prompt templates in `LlmCliRunner` $\rightarrow$ **Verify Green**
2. **Phase 2: Database Reuse & Routes (Backend Routing)**
   - Add Test 4 to `crates/server/src/ai/mod.rs` and Test 3 to `crates/server/src/routes/entries.rs` $\rightarrow$ **Verify Red**
   - Implement database cache checks before LLM invocation $\rightarrow$ **Verify Green**
3. **Phase 3: Templates & Client Scripts (Frontend & UI)**
   - Add Test 1 and Test 2 to `crates/ui-components/src/templates.rs` $\rightarrow$ **Verify Red**
   - Update `executive_brief_card.html` and `index.html` $\rightarrow$ **Verify Green**
4. **Phase 4: Full Suite Verification & Refactor**
   - Run full workspace test suite `cargo test` $\rightarrow$ **Verify All Green**

---

## 5. Conclusion

The TDD plan is solid, directly targets all identified root causes, and ensures comprehensive test coverage before production code changes.

**Result**: `clean`
