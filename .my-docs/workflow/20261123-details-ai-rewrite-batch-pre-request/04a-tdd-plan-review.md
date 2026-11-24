Result: clean

# TDD Plan Review: Details Page AI Rewrite with Batch Pre-request

- **Workflow Slug:** `20261123-details-ai-rewrite-batch-pre-request`
- **Review Stage:** `tdd-review`
- **Result:** clean
- **Target Tasks Document:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/04-tasks.md)
- **Target Design Document:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03-design.md)

---

## 1. Executive Summary & Verdict

This review verifies the planned test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/04-tasks.md) against the requirements, contracts, and edge cases defined in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03-design.md), [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03a-api-contract-review.md), and [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03a-db-design-review.md).

- **Verdict:** `Result: clean`.
- **Planned Tests Present:** Yes (10 discrete test cases spanning 5 development phases).
- **Coverage Assessment:** All core features, boundary conditions, data evolution migrations, schema backwards compatibility, regex fallbacks, and UI rendering rules are thoroughly covered with explicit pre-implementation test definitions.

---

## 2. Test Coverage & Edge Case Verification

### Phase 1: Database Evolution (`AiDb` Migration v4 & CRUD)
- **Target File:** [crates/server/src/ai/db.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/db.rs)
- **Planned Tests:**
  1. `test_migration_v3_to_v4_adds_rewrite_column`:
     - **Verification**: Sets up a v3 schema database, runs `init_schema()`, verifies `user_version = 4`, and confirms existing records are retained with `rewrite = None`.
     - **Edge Cases**: Verifies non-destructive DDL and backward compatibility.
  2. `test_save_and_get_summary_with_rewrite`:
     - **Verification**: Tests upsert and retrieval for both single article records and multi-item batches.
     - **Edge Cases**: Validates `COALESCE(excluded.rewrite, article_ai_summaries.rewrite)` retention behavior when partial updates or summaries without rewrites are inserted.

### Phase 2: Batch Prompt Formatting & 300-Word Truncation
- **Target File:** [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs)
- **Planned Tests:**
  1. `test_format_batch_prompt_with_rewrite_schema_and_rules`:
     - **Verification**: Validates prompt inclusion of compact schema key `"r"`, length constraint (<110 words total, sweet spot 30–60 words in 2–3 bullets or paragraphs), inverted pyramid rules (conclusion in first two sentences), and strict source language parity.
     - **Edge Cases**: Prevents cross-language translation leakage and ambiguous instructions.
  2. `test_batch_prompt_input_truncation_300_words`:
     - **Verification**: Tests passing >300 words input text to `format_batch_prompt_structured`.
     - **Edge Cases**: Confirms safe word boundary truncation without mid-word or UTF-8 character splitting.

### Phase 3: Response Parsing & Regex Fallback
- **Target File:** [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs)
- **Planned Tests:**
  1. `test_parse_batch_response_with_rewrite_json`:
     - **Verification**: Deserializes batch JSON containing `"i"`, `"t"`, `"b"`, `"r"` directly and inside markdown code blocks (` ```json ... ``` `).
     - **Edge Cases**: Validates Serde default handling when `"r"` is omitted.
  2. `test_parse_batch_response_with_rewrite_regex_fallback`:
     - **Verification**: Verifies regex extraction of `"r"` alongside `"i"`, `"t"`, and `"b"` when upstream LLM output contains conversational preamble or slight syntax anomalies.
     - **Edge Cases**: Ensures graceful fallback without crashing the batch worker.

### Phase 4: Server Background Prefetching (`AiPrefetchService`)
- **Target File:** [crates/server/src/ai/prefetch.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs)
- **Planned Tests:**
  1. `test_prefetch_service_persists_rewrites`:
     - **Verification**: Mocks batch CLI output containing rewrites and executes `spawn_prefetch_job`.
     - **Edge Cases**: Asserts background task writes rewrites into `AiDb` alongside briefs and titles in a single atomic transaction.

### Phase 5: Detail View Template & Route Integration
- **Target Files:** [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) & [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
- **Planned Tests:**
  1. `test_entry_detail_template_renders_rewrite`:
     - **Verification**: Renders `EntryDetailTemplate` with `is_rewrite: true` and verifies rewrite text is present in the DOM while raw `sanitized_content` is excluded.
     - **Edge Cases**: Asserts external "Original" link button is present and functional.
  2. `test_entry_detail_template_fallback_when_rewrite_missing`:
     - **Verification**: Renders `EntryDetailTemplate` with `is_rewrite: false` (settled Option 1A fallback).
     - **Edge Cases**: Asserts fallback brief card is rendered instead of empty content.
  3. `test_get_entry_detail_route_loads_rewrite_from_ai_db`:
     - **Verification**: Integration test for `GET /hx/entries/{id}` asserting that cached rewrites are served from `AiDb` in `< 5ms`.
     - **Edge Cases**: Confirms seamless integration between route handler and template rendering.

---

## 3. Review Checklist & Criteria

| Requirement | Design Specification | Planned Test Identifier | Status |
| :--- | :--- | :--- | :---: |
| SQLite Schema Migration | Add `rewrite TEXT`, user_version = 4 | `test_migration_v3_to_v4_adds_rewrite_column` | **PASS** |
| Upsert Data Retention | `COALESCE` retention on partial updates | `test_save_and_get_summary_with_rewrite` | **PASS** |
| Structured Prompt Schema | Schema `"r"`, Inverted Pyramid, Language parity | `test_format_batch_prompt_with_rewrite_schema_and_rules` | **PASS** |
| Safe Input Truncation | Whitespace-aware 300 words cap | `test_batch_prompt_input_truncation_300_words` | **PASS** |
| Serde JSON Deserialization | `BatchAiItem` with `#[serde(default, rename = "r")]` | `test_parse_batch_response_with_rewrite_json` | **PASS** |
| Resilient Regex Extraction | Robust fallback regex capturing `"r"` | `test_parse_batch_response_with_rewrite_regex_fallback` | **PASS** |
| Prefetch Pipeline Write | Background persistence into `AiDb` | `test_prefetch_service_persists_rewrites` | **PASS** |
| Detail View Rendering | Render rewrite, omit raw HTML, keep external link | `test_entry_detail_template_renders_rewrite` | **PASS** |
| Graceful Fallback (1A) | Render brief card when rewrite missing | `test_entry_detail_template_fallback_when_rewrite_missing` | **PASS** |
| Axum / HTMX Endpoint | Instant `< 5ms` response from SQLite | `test_get_entry_detail_route_loads_rewrite_from_ai_db` | **PASS** |

---

## 4. Next Step

The TDD plan in `04-tasks.md` satisfies all design contracts and review criteria. The workflow can proceed to **Gate B** confirmation before entering the Build phase (`build-test-draft`).
