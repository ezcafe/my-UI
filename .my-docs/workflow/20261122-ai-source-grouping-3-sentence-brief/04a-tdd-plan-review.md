clean

# TDD Plan Review: AI Source Grouping, 3-Sentence Brief & UI Polish

## 1. Review Summary & Verdict

- **Verdict:** `clean`
- **Scope reviewed:** `04-tasks.md` planned tests against `01a-gate-a.md`, `02-analysis.md`, `03-design.md`, and TDD Iron Law.
- **TDD Compliance:** Strict Red-Green-Refactor sequence is enforced across all 4 implementation milestones. No production code is modified without first establishing a failing test.

---

## 2. Test Coverage Matrix

| Requirement | Planned Test Case | Test File | Assessment |
| :--- | :--- | :--- | :--- |
| **Strict 3-Sentence Prompt** | `test_format_prompt_strictly_3_sentences` | `crates/server/src/ai/llm.rs` | Complete. Asserts directive mandates strictly 3 sentences. |
| **Strict 3-Sentence Batch Prompt** | `test_format_batch_prompt_strictly_3_sentences` | `crates/server/src/ai/llm.rs` | Complete. Asserts item brief directive specifies strictly 3 sentences. |
| **Source Grouping & Chronology** | `test_format_batch_prompt_source_grouping_and_date_sorting` | `crates/server/src/ai/llm.rs` | Complete. Verifies `### Source: <Feed>` grouping and date descending order within groups. |
| **Hybrid Cache Reuse & Whole-Page Synthesis** | `test_format_batch_prompt_handles_cached_briefs_and_new_articles` | `crates/server/src/ai/llm.rs` | Complete. Verifies existing briefs are labeled for reference and new articles for generation. |
| **Extractive Fallback Length** | `test_extractive_fallback_3_sentences` | `crates/server/src/ai/llm.rs` | Complete. Asserts 3 sentences are extracted on failure / fallback. |
| **Route Cache Reuse & Top Synthesis** | `test_executive_brief_reuses_db_summaries_and_synthesizes_top` | `crates/server/tests/e2e_tests.rs` / `routes/entries.rs` | Complete. Verifies route reuses SQLite summaries and writes page-level brief. |
| **Annuaire Row Click Target** | `test_annuaire_row_click_target` | `crates/ui-components/src/templates.rs` | Complete. Asserts outer `.annuaire-row` has `onclick` and child container does not. |
| **Pagination Nav Border Polish** | `test_pagination_nav_no_top_border` | `crates/ui-components/src/templates.rs` | Complete. Asserts absence of `border-t border-border` on `<nav aria-label="Pagination">`. |

---

## 3. Edge Cases & Safety Checks

1. **All Articles Cached**: When all articles on a page already have summaries in `article_ai_summaries`, the prompt must still synthesize `"top"` without asking the LLM to regenerate item briefs.
2. **Missing / Malformed Dates**: If an entry has an empty or unparseable `published_at`, sorting must safely fall back (e.g. secondary sort by ID or treat as oldest/newest consistently) without panics.
3. **Empty Source Title**: If `feed.title` is empty or missing, format gracefully under a default header (e.g., `### Source: Uncategorized`).
4. **Short Content Fallback**: When an article has fewer than 3 sentences, `fallback_extractive` must safely return all available sentences without truncation errors or panics.
5. **Click Event Bubbling**: External link button inside `.annuaire-row` must preserve `event.stopPropagation()` so opening the link in a new tab does not inadvertently trigger drawer expansion.

---

## 4. Execution Readiness

The test plan in `04-tasks.md` satisfies all design and architectural constraints. The suite is ready for execution as soon as Gate B approval is settled.

## Result

Result: clean
