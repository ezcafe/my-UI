Result: clean

# API Contract Review: AI Usage Optimization & Executive Presentation

## Key Takeaways
- **Zero blocking issues**: All API, service, data model, CLI prompt, server logging, and UI template interface contracts in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-optimize-ai-usage/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-optimize-ai-usage/04-tasks.md) are comprehensive, consistent, and backward-compatible.
- **Structured Server Logging**: Robust logging contracts with `info!`, `warn!`, and `error!` levels across `LlmCliRunner` capture outbound prompt previews, model info, item count, timing, raw output, parsing fallbacks, timeouts, and CLI failures without leaking credentials.
- **Filter-Specific Top Executive Brief**: Restricting `top_executive_brief` to "unread" and "today" views guarantees relevance, eliminates stale executive cards on search/archive views, and safely removes `<button id="filter-ai">`.
- **Single-Call Page Batch Prefetch**: Replacing the chunking loop and `unread_count > 20` gating with a single background LLM call per rendered page ensures bounded O(1) external CLI invocations and instant (<30ms) page loads.
- **Resilient Fallback Parsing**: Multi-tiered parsing (JSON deserialization -> markdown code fence strip -> regex fallback -> extractive fallback) guarantees graceful degradation under any LLM output format.

---

## Detailed Evaluation Checklist

### 1. Server Logging Contracts in `LlmCliRunner`
- **Design & Task Specification**:
  - `tracing::info!`:
    - Dispatch: `[AI_DISPATCH] model="..." item_count=N prompt_preview="..."` (first 200 chars).
    - Return: `[AI_RETURN] elapsed_ms=N raw_output="..."`.
  - `tracing::warn!`:
    - Fallback: `[AI_WARN] reason="json_parse_fallback" fallback_strategy="..." raw_snippet="..."` (first 150 chars).
  - `tracing::error!`:
    - Timeout: `[AI_ERROR] error="cli_timeout" timeout_secs=15 elapsed_ms=N`.
    - Process failure: `[AI_ERROR] error="process_failed" exit_code=... stderr="..."`.
    - Empty output: `[AI_ERROR] error="empty_output" elapsed_ms=N`.
- **Assessment**:
  - **Standardized formatting**: Uses standard `tracing` macros with clear target `ai::batch` and consistent `[AI_*]` message headers.
  - **Observability without noise**: Truncating prompt previews to 200 characters and raw snippets to 150 characters prevents log file bloat while providing enough context to debug model behavior.
  - **Security & Privacy**: Only public article feed summaries are processed; no API secrets or private tokens are logged.
- **Status**: **PASS** (Clean).

### 2. Removal of AI Filter & Filter-Specific Top Executive Brief
- **Design & Task Specification**:
  - Remove `<button id="filter-ai">` and corresponding `data-status="ai"` handling in `crates/ui-components/templates/pages/index.html`.
  - Filter parameter check in routes (`crates/server/src/routes/entries.rs` and `pages.rs`):
    - For "unread" and "today": query `ai.db.get_latest_page_brief().await`, set `top_executive_brief = Some(...)`, and spawn background page batch for uncached items.
    - For all other filters ("read", "starred", "all", search query): set `top_executive_brief = None` and do not trigger background batch.
  - Template contract: `EntryListTemplate` renders the executive brief card conditionally via `{% if let Some(brief) = top_executive_brief %}`.
- **Assessment**:
  - **Seamless UX**: Removing the dedicated AI button integrates AI summaries naturally into standard reading workflows.
  - **Context-aware briefs**: Preventing executive briefs from showing up on search queries or read archives prevents confusing or stale summaries.
  - **Backward compatibility**: `Option<String>` in `EntryListTemplate` preserves template rendering when no brief exists.
- **Status**: **PASS** (Clean).

### 3. Single-Call Page Batch Prefetch (`AiPrefetchService`)
- **Design & Task Specification**:
  - Removal of old `unread_count > 20` gating.
  - Removal of 12-item chunking loop.
  - When rendering an "unread" or "today" list page:
    - Collect all uncached entries on the visible page (up to 20 items).
    - If uncached entries exist, spawn 1 background task.
    - The background task executes exactly 1 LLM call for all uncached entries.
    - Persist the page brief (`top`) and item records (`t`, `b`) into SQLite.
- **Assessment**:
  - **Bounded resource consumption**: Replaces repeated sequential chunk calls with exactly 1 call per page load.
  - **Zero latency impact**: Executed asynchronously via `tokio::spawn`; HTTP handlers return immediately.
  - **Inclusive experience**: Users with fewer than 20 items now receive AI summaries and titles without hitting arbitrary count gates.
- **Status**: **PASS** (Clean).

### 4. Data Models & SQLite Interface Contracts
- **Design & Task Specification**:
  - `AiSummaryRecord { entry_id: i64, summary: String, suggested_title: Option<String>, model: String, created_at: String }`.
  - `BatchAiResponse { top: String, items: Vec<BatchAiItem> }` where `BatchAiItem { i: i64, t: String, b: String }`.
  - SQLite methods: `get_summary_and_title`, `get_batch_summaries`, `save_summary_and_title`, `save_page_brief`, `get_latest_page_brief`.
- **Assessment**:
  - Types and field names cleanly match across DB queries, JSON deserialization schemas, and UI view models.
  - Non-blocking queries via `tokio::task::spawn_blocking` prevent async thread starvation.
- **Status**: **PASS** (Clean).

---

## Conclusion

The API, service, logging, and template contracts in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-optimize-ai-usage/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-optimize-ai-usage/04-tasks.md) satisfy all functional, observability, and performance requirements.

**Final Assessment:** **clean**. Zero Critical or Major issues identified.
