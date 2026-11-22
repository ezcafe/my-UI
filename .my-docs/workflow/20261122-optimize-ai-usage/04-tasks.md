# Tasks: AI Usage Optimization & Executive Presentation

## Task Overview

This implementation executes strict Test-Driven Development (TDD) across 5 modular phases:
1. Database evolution & batch caching (`AiDb`)
2. Content extraction & 250-word truncation limit (`ArticleExtractor`)
3. Compact single-page batch LLM runner & structured server logging (`LlmCliRunner`)
4. Single-page background batch processing service (`AiPrefetchService` / page batch runner)
5. UI and route integration (removing AI filter button, restricting top brief to "unread" & "today", rendering AI titles)

---

## Phase 1: Database Evolution & Batch Caching

### Task 1.1: Schema Migration & `AiSummaryRecord`
- **TDD Test**: `crates/server/src/ai/db.rs` -> `tests::test_migration_v1_to_v2()`
  - Verify migration from `v1` to `v2` adds `suggested_title` column to `article_ai_summaries` and creates `page_executive_briefs` table without data loss.
  - Verify `get_user_version() == 2`.
- **Implementation**:
  - Update `init_schema()` in `crates/server/src/ai/db.rs` to handle migration from version 1 to 2.
  - Define `pub struct AiSummaryRecord`.

### Task 1.2: Batch Querying & Saving Methods
- **TDD Test**: `crates/server/src/ai/db.rs` -> `tests::test_batch_summaries_and_page_brief()`
  - Test `save_summary_and_title`, `get_summary_and_title`, `get_batch_summaries`, `save_page_brief`, and `get_latest_page_brief`.
  - Ensure missing IDs return empty map/None without error.
- **Implementation**:
  - Implement `get_summary_and_title()`, `get_batch_summaries()`, `save_summary_and_title()`, `save_page_brief()`, and `get_latest_page_brief()` in `AiDb`.

---

## Phase 2: Content Extraction & Truncation

### Task 2.1: 250-Word Truncation Limit & Deduplication
- **TDD Test**: `crates/server/src/ai/extractor.rs` -> `tests::test_word_truncation_250()`
  - Verify `parse_html` truncates extracted text to at most 250 words while preserving title.
- **Implementation**:
  - Update truncation in `crates/server/src/ai/extractor.rs` to 250 words.

---

## Phase 3: Compact Single-Page Batch LLM Runner & Server Logging

### Task 3.1: Micro-Batch Prompt Formatting & Zero Thinking Budget
- **TDD Test**: `crates/server/src/ai/llm.rs` -> `tests::test_batch_prompt_formatting_and_args()`
  - Verify prompt contains compact keys schema (`{"top":"...","items":[{"i":...,"t":...,"b":...}]}`) for up to 20 items.
  - Verify CLI args include `--thinking-budget 0`.
- **Implementation**:
  - Implement `format_batch_prompt(articles: &[(i64, &str, &str)]) -> String` in `crates/server/src/ai/llm.rs`.
  - Ensure `--thinking-budget 0` is added to CLI invocation args.

### Task 3.2: Comprehensive Server Logging
- **TDD Test**: `crates/server/src/ai/llm.rs` -> `tests::test_server_logging_format()`
  - Verify `info!` logs outbound prompt preview, model, and item count before dispatch.
  - Verify `info!` logs elapsed duration and full raw output upon return.
  - Verify `warn!` logs JSON parsing issues and fallback activations (regex, extractive).
  - Verify `error!` logs CLI timeouts, process failure exits, or empty outputs.
- **Implementation**:
  - Add structured `tracing::info!`, `tracing::warn!`, and `tracing::error!` calls inside `crates/server/src/ai/llm.rs`.

### Task 3.3: Resilient JSON & Fallback Parsing
- **TDD Test**: `crates/server/src/ai/llm.rs` -> `tests::test_parse_batch_response()`
  - Test valid JSON parsing into `BatchAiResponse { top, items }`.
  - Test markdown code fences stripping (````json ... ````).
  - Test fallback regex parsing when conversational text surrounds JSON.
  - Test extractive fallback when parsing fails completely.
- **Implementation**:
  - Implement `parse_batch_response(raw_output: &str) -> BatchAiResponse` in `crates/server/src/ai/llm.rs`.

---

## Phase 4: Single-Page Background Batch Processing

### Task 4.1: Single-Page Batch Logic (No Chunking, No Unread Threshold)
- **TDD Test**: `crates/server/src/ai/prefetch.rs` -> `tests::test_page_batch_processing()`
  - Verify chunking into 12 is removed; all uncached entries for the current page (up to 20) are processed in 1 batch.
  - Verify prefetch gate `unread_count > 20` is removed.
  - Verify `run_page_batch` makes exactly 1 LLM call for all uncached entries and persists page brief (`top`) and item records (`t`, `b`) to SQLite.
- **Implementation**:
  - Refactor `crates/server/src/ai/prefetch.rs` to process all uncached entries on the rendered page in a single LLM call.
  - Persist `top` executive brief via `ai.db.save_page_brief` and items via `ai.db.save_summary_and_title`.

---

## Phase 5: UI & Route Integration

### Task 5.1: Remove AI Filter Button
- **TDD Test**: `crates/ui-components/src/templates.rs` -> `tests::test_index_template_no_ai_filter()`
  - Verify `<button id="filter-ai">` is completely removed from `index.html`.
- **Implementation**:
  - Remove `<button id="filter-ai">` from `crates/ui-components/templates/pages/index.html`.

### Task 5.2: Filter-Specific Top Executive Brief & Batch Trigger
- **TDD Test**: `crates/server/src/routes/entries.rs` & `pages.rs` -> `tests::test_filter_specific_brief()`
  - For filter "unread" and "today":
    - Verify `top_executive_brief` is queried from `ai.db.get_latest_page_brief()` and passed to template.
    - Verify background page batch is triggered for uncached items.
  - For other filters ("read", "starred", "all", search):
    - Verify `top_executive_brief` is set to `None`.
    - Verify background page batch is NOT triggered.
- **Implementation**:
  - Update `list_entries_html` in `crates/server/src/routes/entries.rs` and `serve_index` in `crates/server/src/routes/pages.rs`.

### Task 5.3: Template Rendering & Suggested Titles
- **TDD Test**: `crates/ui-components/templates` tests:
  - Verify `entry_list.html` renders top card only when `top_executive_brief.is_some()`.
  - Verify `entry_item.html` renders `suggested_title` when available, falling back to original `title` in tooltip.
  - Verify `brief_card.html` renders concise brief.
- **Implementation**:
  - Update `entry_list.html`, `entry_item.html`, and `brief_card.html`.

---

## Verification & Acceptance Criteria

1. `cargo test` passes for all crates.
2. Server logs clearly display outbound prompt preview, model, item count, duration, raw output, warnings, and errors.
3. No `<button id="filter-ai">` exists in `index.html`.
4. Top executive brief card renders only on "unread" and "today" views; absent on "read", "starred", "all", and search.
5. All uncached items on an "unread" or "today" page are sent in a single LLM call.
6. List browsing latency remains under 30ms with instant cache lookups.
