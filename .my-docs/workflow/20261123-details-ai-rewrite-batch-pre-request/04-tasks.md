# Tasks: Details Page AI Rewrite with Batch Pre-request

## Task Overview

This document defines the ordered Test-Driven Development (TDD) phases for implementing AI Rewrites on the article details page with batch pre-requesting. Every feature is preceded by failing tests before implementation.

---

## Planned Tests Index

| Phase | Test Identifier | Scope & Assertions |
| :--- | :--- | :--- |
| **1. Database Evolution** | `test_migration_v3_to_v4_adds_rewrite_column` | Verifies migration from v3 to v4 adds `rewrite TEXT` to `article_ai_summaries`, sets `user_version = 4`, and preserves existing rows with `None`. |
| **1. Database Evolution** | `test_save_and_get_summary_with_rewrite` | Verifies `AiDb` upserts and retrieves `rewrite` for single and batch queries, testing `COALESCE` retention on partial updates. |
| **2. Prompt & Truncation** | `test_format_batch_prompt_with_rewrite_schema_and_rules` | Verifies batch prompt contains compact schema with `"r"`, word limit (<110 words, sweet spot 30-60 words), inverted pyramid rules, and language parity. |
| **2. Prompt & Truncation** | `test_batch_prompt_input_truncation_300_words` | Verifies article input text exceeding 300 words is safely truncated at 300 words without breaking whitespace boundaries. |
| **3. Response Parsing** | `test_parse_batch_response_with_rewrite_json` | Verifies `parse_batch_response` deserializes `"r"` from direct JSON and fenced markdown blocks into `BatchAiItem`. |
| **3. Response Parsing** | `test_parse_batch_response_with_rewrite_regex_fallback` | Verifies fallback regex extracts `"r"` when JSON has minor formatting anomalies or when `"r"` is omitted. |
| **4. Prefetch Service** | `test_prefetch_service_persists_rewrites` | Verifies `AiPrefetchService` background job invokes batch LLM and persists `rewrite` into `AiDb`. |
| **5. Detail View UI** | `test_entry_detail_template_renders_rewrite` | Verifies `EntryDetailTemplate` renders `rewrite` in prose container and completely excludes raw article HTML (`sanitized_content`). |
| **5. Detail View UI** | `test_entry_detail_template_fallback_when_rewrite_missing` | Verifies `EntryDetailTemplate` falls back to 3-sentence brief / teaser when rewrite is pending or missing. |
| **5. Detail View UI** | `test_get_entry_detail_route_loads_rewrite_from_ai_db` | Integration test verifying `GET /hx/entries/{id}` queries `AiDb` and serves the precomputed rewrite in `< 5ms`. |

---

## Phase 1: Database Evolution (`AiDb` Migration v4)

### Task 1.1: Schema Migration & Model Extension
- **Test**: `crates/server/src/ai/db.rs` &rarr; `tests::test_migration_v3_to_v4_adds_rewrite_column`
  - Set up an in-memory or temporary SQLite database at `user_version = 3`.
  - Insert sample records without a `rewrite` column.
  - Run `init_schema()`.
  - Assert `get_user_version() == 4`.
  - Assert existing rows are preserved with `rewrite == None`.
- **Implementation**:
  - Update `init_schema()` in [crates/server/src/ai/db.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/db.rs) to check `version < 4`.
  - Execute `ALTER TABLE article_ai_summaries ADD COLUMN rewrite TEXT;` and `PRAGMA user_version = 4;`.
  - Add `pub rewrite: Option<String>` to `AiSummaryRecord` and `ArticleSummaryInput`.

### Task 1.2: CRUD Operations for Rewrites
- **Test**: `crates/server/src/ai/db.rs` &rarr; `tests::test_save_and_get_summary_with_rewrite`
  - Call `save_summary_and_title_with_rewrite` (or extended `save_summary_and_title`).
  - Verify `get_summary_and_title` returns `record.rewrite == Some("...")`.
  - Test `save_batch_results` with multiple items containing rewrites.
  - Verify `get_batch_summaries` returns all items with their respective rewrites.
  - Verify that updating an article without specifying a rewrite retains the previously saved rewrite via `COALESCE`.
- **Implementation**:
  - Update `SELECT` statements in `get_summary_and_title` and `get_batch_summaries` to include the `rewrite` column.
  - Update `INSERT ... ON CONFLICT` in `save_summary_and_title` and `save_batch_results` to handle `rewrite = COALESCE(excluded.rewrite, article_ai_summaries.rewrite)`.

---

## Phase 2: Batch Prompt Formatting & 300-Word Truncation

### Task 2.1: Prompt Formatting with Rewrite Rules
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_format_batch_prompt_with_rewrite_schema_and_rules`
  - Call `LlmCliRunner::format_batch_prompt_structured`.
  - Assert the prompt contains instructions for `"r"`:
    - Under 110 words total, target 30–60 words in 2–3 bullet points or short paragraphs.
    - Inverted Pyramid: conclusion and primary impact in the first two sentences.
    - Strict language parity (Vietnamese source &rarr; Vietnamese rewrite, English source &rarr; English rewrite).
  - Assert compact schema includes `{"i":...,"t":"...","b":"...","r":"..."}`.
- **Implementation**:
  - Update prompt template in `crates/server/src/ai/llm.rs` to include the rewrite specifications and schema keys.

### Task 2.2: 300-Word Input Truncation
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_batch_prompt_input_truncation_300_words`
  - Pass an article with 500 words of content to `format_batch_prompt_structured`.
  - Assert that the formatted article content in the prompt contains at most 300 words.
- **Implementation**:
  - Implement a whitespace-aware `truncate_words(s: &str, max_words: usize) -> &str` helper and apply it to `content_or_brief` prior to appending to prompt.

---

## Phase 3: Response Parsing & Regex Fallback

### Task 3.1: JSON Deserialization with Rewrite Field
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_parse_batch_response_with_rewrite_json`
  - Test raw JSON with `"top"` and `"items"` containing `"i"`, `"t"`, `"b"`, `"r"`.
  - Test fenced markdown code blocks (` ```json ... ``` `).
  - Assert `parsed.items[0].rewrite == Some("...")`.
- **Implementation**:
  - Add `#[serde(default, rename = "r")] pub rewrite: Option<String>` to `BatchAiItem`.

### Task 3.2: Regex Extraction Fallback for Rewrite
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_parse_batch_response_with_rewrite_regex_fallback`
  - Test output wrapped in conversational text or slightly invalid JSON where direct serde fails.
  - Verify regex successfully extracts `"r"` as well as `"i"`, `"t"`, and `"b"`.
  - Test fallback when `"r"` is omitted by LLM (defaults to `None` without parse failure).
- **Implementation**:
  - Update `ITEM_REGEX` in `crates/server/src/ai/llm.rs` to optionally capture the `"r"` group.

---

## Phase 4: Server Background Prefetching (`AiPrefetchService`)

### Task 4.1: Prefetch Pipeline Persistence of Rewrites
- **Test**: `crates/server/src/ai/prefetch.rs` &rarr; `tests::test_prefetch_service_persists_rewrites`
  - Configure mock CLI returning batch JSON with `r` fields.
  - Trigger `spawn_prefetch_job` with uncached candidate entries.
  - Await background completion.
  - Query `AiDb` and assert that `rewrite` is persisted for all processed entries.
- **Implementation**:
  - In `AiPrefetchService::run_prefetch`, map `item.rewrite` into `ArticleSummaryInput` and pass to `AiDb::save_batch_results` (or `save_summary_and_title`).

---

## Phase 5: Detail View Template & Route Integration

### Task 5.1: Detail View Template Rendering (Option B & 1A)
- **Test**: `crates/ui-components/src/templates.rs` &rarr; `tests::test_entry_detail_template_renders_rewrite`
  - Instantiate `EntryDetailTemplate` with `is_rewrite: true` and rewrite text.
  - Render HTML.
  - Assert the rewrite text is present in the DOM.
  - Assert raw `sanitized_content` is **not** present in the DOM.
  - Assert external "Original" link button is present.
- **Test**: `crates/ui-components/src/templates.rs` &rarr; `tests::test_entry_detail_template_fallback_when_rewrite_missing`
  - Instantiate `EntryDetailTemplate` with `is_rewrite: false` (fallback scenario).
  - Assert the fallback brief card is rendered.
- **Implementation**:
  - Update `EntryDetailTemplate` in [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) to include `rewrite: String` and `is_rewrite: bool`.
  - Update [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html) to render `rewrite` and omit raw HTML content.

### Task 5.2: Detail Route Loading from `AiDb`
- **Test**: `crates/server/src/routes/entries.rs` &rarr; `tests::test_get_entry_detail_route_loads_rewrite_from_ai_db`
  - Populate `AiDb` with a record having `rewrite = Some("High impact conclusion...")`.
  - Perform GET request on `/hx/entries/{id}`.
  - Assert HTTP 200 and response body contains the AI rewrite.
- **Implementation**:
  - In `crates/server/src/routes/entries.rs` (`get_entry_detail`):
    - Check `ai_service.db.get_summary_and_title(id).await`.
    - If `record.rewrite` is present and non-empty, pass `rewrite` and `is_rewrite = true` to `EntryDetailTemplate`.
    - Else, pass `summary` (teaser / brief) and `is_rewrite = false` as graceful fallback.

---

## Verification Checklist

1. `cargo test -p server test_migration_v3_to_v4` passes.
2. `cargo test -p server test_batch_prompt` passes.
3. `cargo test -p server test_parse_batch_response` passes.
4. `cargo test -p server test_prefetch_service` passes.
5. `cargo test -p ui-components test_entry_detail` passes.
6. `cargo clippy --all-targets` passes with zero warnings.
