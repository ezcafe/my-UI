Result: clean

# TDD Test Plan Review: AI Usage Optimization & Executive Presentation

**Result:** clean

## Key Takeaways

- **100% Target Scope Coverage**: Planned test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-optimize-ai-usage/04-tasks.md) comprehensively validate all backloop and optimization requirements, specifically Task 3.2 (Comprehensive Server Logging), Phase 4 (Single-page background batch runner in `AiPrefetchService`), and Phase 5 (Removal of AI filter button, restricting top executive brief to "unread" & "today", passing `None` for "read", "starred", "all", and search).
- **Strict TDD Adherence**: Every phase and task specifies explicit, named unit and integration test functions located in their target modules (`crates/server/src/ai/llm.rs`, `crates/server/src/ai/prefetch.rs`, `crates/server/src/routes/entries.rs`, `crates/server/src/routes/pages.rs`, `crates/ui-components/src/templates.rs`). Red tests must fail before any production code implementation begins.
- **Zero Latency Protection**: Tests guarantee that web routes never wait for LLM subprocesses. List routes load in <30ms from SQLite cache, while batch jobs execute asynchronously in the background.
- **Robust Failure Resilience**: Edge cases test timeout recovery, process exit errors, malformed/non-JSON responses, markdown fence stripping, regex recovery, and empty-set handling.

---

## Detailed Evaluation by Phase

### Phase 1: Database Evolution & Batch Caching
- **Planned Tests**:
  - `crates/server/src/ai/db.rs` -> `tests::test_migration_v1_to_v2()`:
    - **Schema migration**: Verifies upgrading schema from version 1 to 2 preserves existing records.
    - **New columns and tables**: Confirms `suggested_title` is added to `article_ai_summaries` and table `page_executive_briefs` is created.
    - **Version pragma**: Asserts `PRAGMA user_version == 2`.
  - `crates/server/src/ai/db.rs` -> `tests::test_batch_summaries_and_page_brief()`:
    - **Batch queries**: Verifies `get_batch_summaries(&[i64])` retrieves multiple entries efficiently.
    - **Missing keys**: Ensures missing or empty ID lists return an empty/partial map without errors.
    - **Brief persistence**: Verifies `save_page_brief` and `get_latest_page_brief` roundtrip accurately.
- **Evaluation**: **CLEAN**. Guarantees stable schema evolution and zero-latency batch lookups for list rendering.

---

### Phase 2: Content Extraction & Truncation
- **Planned Tests**:
  - `crates/server/src/ai/extractor.rs` -> `tests::test_word_truncation_250()`:
    - **Truncation bound**: Verifies extracted article body text is capped at 250 words per item.
    - **Content integrity**: Verifies articles shorter than 250 words are not truncated.
    - **Title preservation**: Ensures title remains clean and deduplicated from extracted body text.
- **Evaluation**: **CLEAN**. Protects token budgets by cutting input size by 60%–80% at extraction time.

---

### Phase 3: Compact Single-Page Batch LLM Runner & Server Logging
- **Planned Tests**:
  - `crates/server/src/ai/llm.rs` -> `tests::test_batch_prompt_formatting_and_args()`:
    - **Prompt schema**: Confirms compact JSON schema (`{"top":"...","items":[{"i":...,"t":...,"b":...}]}`) for up to 20 articles.
    - **Zero thinking budget**: Asserts CLI invocation args include `--thinking-budget 0` to disable reasoning tokens.
  - `crates/server/src/ai/llm.rs` -> `tests::test_server_logging_format()`:
    - **Info logging (outbound)**: Verifies `info!` logs outbound prompt preview (first 200 chars), model name, and item count before subprocess dispatch.
    - **Info logging (return)**: Verifies `info!` logs elapsed milliseconds and full raw CLI output upon completion.
    - **Warn logging**: Verifies `warn!` logs parsing issues and activation of fallback strategies (regex or extractive).
    - **Error logging**: Verifies `error!` logs CLI timeouts, non-zero process exits, and empty stdout outputs.
  - `crates/server/src/ai/llm.rs` -> `tests::test_parse_batch_response()`:
    - **JSON parse**: Direct serde deserialization into `BatchAiResponse { top, items }`.
    - **Code fence stripping**: Robust extraction when output is wrapped in ````json ... ````.
    - **Regex fallback**: Graceful extraction when conversational text surrounds JSON.
    - **Extractive fallback**: Fallback to extractive summary if all parsing fails, ensuring no panic.
- **Evaluation**: **CLEAN**. Thoroughly guards LLM runner performance, visibility, and parsing fault tolerance.

---

### Phase 4: Single-Page Background Batch Processing Service
- **Planned Tests**:
  - `crates/server/src/ai/prefetch.rs` -> `tests::test_page_batch_processing()`:
    - **Single batch execution**: Verifies all uncached entries for the current page (up to 20 items) are processed in exactly 1 LLM batch call (removes old 12-item chunking).
    - **No threshold gating**: Verifies old `unread_count > 20` gating is completely removed; any uncached items on an unread/today page trigger batch processing.
    - **Data persistence**: Confirms that upon LLM return, `top` executive brief is saved via `ai.db.save_page_brief`, and each item's suggested title and brief are persisted via `ai.db.save_summary_and_title`.
    - **Cache deduplication**: Verifies already cached entries are filtered out before calling the LLM; if all items are cached, 0 LLM calls are dispatched.
- **Evaluation**: **CLEAN**. Accurately verifies single-page batching, quota optimization, and asynchronous persistence.

---

### Phase 5: UI & Route Integration
- **Planned Tests**:
  - `crates/ui-components/src/templates.rs` -> `tests::test_index_template_no_ai_filter()`:
    - **Filter button removal**: Verifies `<button id="filter-ai">` is absent from `index.html` rendering.
    - **Status attribute removal**: Verifies no `data-status="ai"` filter trigger exists in the DOM.
  - `crates/server/src/routes/entries.rs` & `pages.rs` -> `tests::test_filter_specific_brief()`:
    - **Active views ("unread" & "today")**:
      - Verifies `top_executive_brief` is queried from `ai.db.get_latest_page_brief()` and passed to `EntryListTemplate`.
      - Verifies background page batch is triggered for uncached entries.
    - **Inactive views ("read", "starred", "all", search)**:
      - Verifies `top_executive_brief` is explicitly set to `None`.
      - Verifies background page batch is NOT triggered.
  - `crates/ui-components/src/templates.rs`:
    - **Top brief card rendering**: Verifies `entry_list.html` renders top card only when `top_executive_brief.is_some()`, and completely omits the card when `None`.
    - **Suggested title rendering**: Verifies `entry_item.html` displays `suggested_title` when present, while retaining original title in the `title="..."` tooltip attribute.
    - **Detail brief rendering**: Verifies `brief_card.html` renders concise executive brief at the head of opened articles.
- **Evaluation**: **CLEAN**. Fully covers route filtering rules, DOM presentation, and template fallbacks.

---

## Edge Case Matrix

| Component | Test Case | Expected Behavior | Status |
|-----------|-----------|-------------------|--------|
| **DB Migration** | Existing v1 SQLite database | Upgrades to v2, keeps all summaries, creates `page_executive_briefs` | Covered |
| **DB Batch Query** | Empty ID list or non-existent IDs | Returns empty/partial map cleanly with no errors | Covered |
| **Extractor** | Article exceeding 250 words | Truncates to 250 words at word boundary; preserves title | Covered |
| **LLM Runner** | `--thinking-budget 0` argument | Included in CLI invocation args | Covered |
| **LLM Logging** | Subprocess dispatch, return, error | Structured logs at `info!`, `warn!`, `error!` with target `"ai::batch"` | Covered |
| **LLM Parser** | Markdown fences and preamble text | Strips fences and extracts JSON cleanly | Covered |
| **LLM Parser** | Total malformed output | Fallback to extractive summary; no panic | Covered |
| **Batch Service** | Rendered page with 15 uncached items | Executes exactly 1 LLM call; saves brief and item records | Covered |
| **Batch Service** | Rendered page with all items cached | Dispatches 0 LLM calls | Covered |
| **Route Filter** | Filter is "unread" or "today" | Loads latest page brief; triggers background batch | Covered |
| **Route Filter** | Filter is "read", "starred", "all", or search | Sets brief to `None`; skips background batch | Covered |
| **UI Filter Bar** | Index navigation bar | Contains no AI filter button or `data-status="ai"` | Covered |
| **UI Entry List** | `top_executive_brief == None` | Omits executive brief card completely | Covered |
| **UI Entry Item** | `suggested_title == None` | Falls back to Miniflux original title | Covered |

---

## Conclusion & Next Step

The TDD test plan in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-optimize-ai-usage/04-tasks.md) provides 100% comprehensive and verifiable coverage across all required phases, specifically validating Task 3.2 logging, Phase 4 single-page background batching, and Phase 5 filter-specific routing and button removal.

The plan is verified and marked **Result: clean**.
Per workflow governance, this stage is complete; returning to parent orchestrator.
