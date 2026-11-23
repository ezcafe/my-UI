# Tasks: AI Source Grouping, 3-Sentence Brief & UI Polish

## Planned Tests

- [x] `test_format_prompt_strictly_3_sentences`: Verifies that `format_prompt` specifies strictly 3 sentences in prompt instructions.
- [x] `test_format_batch_prompt_strictly_3_sentences`: Verifies that `format_batch_prompt` specifies strictly 3 sentences for item briefs.
- [x] `test_format_batch_prompt_source_grouping_and_date_sorting`: Verifies that `format_batch_prompt` groups articles under source headers (`### Source: <Feed Title>`) and orders items within each group by date descending (latest first).
- [x] `test_format_batch_prompt_handles_cached_briefs_and_new_articles`: Verifies that cached articles are labeled as existing briefs and the prompt instructs the LLM to generate items only for new articles while synthesizing the overall top brief.
- [x] `test_extractive_fallback_3_sentences`: Verifies that `fallback_extractive` in `llm.rs` extracts strictly 3 sentences using `miniflux_client::summary::extract_summary(content, 3)`.
- [x] `test_executive_brief_reuses_db_summaries_and_synthesizes_top`: Verifies that `/hx/executive-brief` reuses existing summaries from `AiDb` and synthesizes the page-level brief across all page items.
- [x] `test_annuaire_row_click_target`: Verifies template rendering where `onclick="openEntryDetail(id)"` is attached to the outer `.annuaire-row` container.
- [x] `test_pagination_nav_no_top_border`: Verifies pagination navigation in `entry_list.html` has no `border-t border-border`.

---

## Milestone 1: LLM Prompt Directives & 3-Sentence Length

- [x] **Task 1.1: TDD tests for 3-sentence prompt instructions and extractive fallback**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Add tests `test_format_prompt_strictly_3_sentences`, `test_format_batch_prompt_strictly_3_sentences`, and `test_extractive_fallback_3_sentences`.
- [x] **Task 1.2: Update prompt formatters and fallback extraction**
  - Path: `crates/server/src/ai/llm.rs`
  - Action:
    - Update `format_prompt` to mandate "strictly 3 concise sentences".
    - Update `format_batch_prompt` item brief instruction to mandate "strictly 3 sentences".
    - In `fallback_extractive` and single article fallback, change sentence count from 2 to 3.
  - Verification: `cargo test --package server ai::llm` passes.

---

## Milestone 2: Source Grouping & Date Sorting in Batch Prompt

- [x] **Task 2.1: TDD tests for source grouping, date sorting, and cache reuse in batch prompt**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Add `test_format_batch_prompt_source_grouping_and_date_sorting` and `test_format_batch_prompt_handles_cached_briefs_and_new_articles`.
- [x] **Task 2.2: Implement structured batch article input and prompt formatting**
  - Path: `crates/server/src/ai/llm.rs`
  - Action:
    - Introduce `BatchArticleInput` representing article id, title, source/feed name, published_at timestamp, content/brief, and is_cached flag.
    - Implement source grouping under `### Source: <Name>` headers.
    - Sort articles within each source group by `published_at` descending (latest first).
    - Format cached items as `[ID x] (Existing Brief) Title: ...\nBrief: ...` and new items as `[ID y] (New Article) Title: ...\nContent: ...`.
    - Provide backward-compatible adapter for existing call sites.
  - Verification: `cargo test --package server ai::llm` passes.

---

## Milestone 3: Route Orchestration & Cache Reuse in `/hx/executive-brief` and Prefetch

- [x] **Task 3.1: TDD test for cache reuse and whole-page synthesis in executive brief route**
  - Path: `crates/server/tests/e2e_tests.rs` or `crates/server/src/routes/entries.rs`
  - Action: Add test verifying `/hx/executive-brief` passes existing DB summaries to prompt and only saves new summaries.
- [x] **Task 3.2: Update `/hx/executive-brief` route logic**
  - Path: `crates/server/src/routes/entries.rs`
  - Action:
    - Query `publishedAt` and `feed { title }` in GraphQL.
    - Group by source, sort by `publishedAt` descending.
    - Partition entries into cached (via `AiDb::get_batch_summaries`) and uncached.
    - Pass full page context (cached briefs + uncached content) to `summarize_batch`.
    - Filter returned items to only persist new article summaries in `AiDb`.
    - Store synthesized page brief under `page_key`.
- [x] **Task 3.3: Update prefetch worker**
  - Path: `crates/server/src/ai/prefetch.rs`
  - Action: Align candidate batch processing with source grouping and date sorting.
  - Verification: `cargo test --package server ai::prefetch` passes.

---

## Milestone 4: UI Row Click Target Expansion & Pagination Border Polish

- [x] **Task 4.1: TDD tests for `.annuaire-row` click handler and pagination styling**
  - Path: `crates/ui-components/src/templates.rs`
  - Action: Add `test_annuaire_row_click_target` and `test_pagination_nav_no_top_border`.
- [x] **Task 4.2: Update templates**
  - Path:
    - `crates/ui-components/templates/partials/entry_item.html`
    - `crates/ui-components/templates/partials/entry_list.html`
    - `crates/ui-components/templates/pages/index.html`
  - Action:
    - Move `onclick="openEntryDetail({{ entry.id }})"` to the outer `.annuaire-row` element.
    - Remove `onclick` from inner child container.
    - Ensure external link button preserves `event.stopPropagation()`.
    - In `entry_list.html`, remove `border-t border-border` on `<nav aria-label="Pagination">`.
  - Verification: `cargo test --package ui-components` passes.

---

## Milestone 5: Full Workspace Verification

- [x] **Task 5.1: Run full workspace tests and linter**
  - Action: `cargo test --workspace` and `cargo clippy --workspace --all-targets`.
