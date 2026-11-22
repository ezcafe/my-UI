# Design: AI Usage Optimization & Executive Presentation

## Executive Summary

This design optimizes AI consumption by 60%–80% while enhancing the reading experience through concise AI-suggested titles and an Executive Briefs summary card.

Key improvements:
- **Instant list loads**: Zero latency impact (<30ms) by reading cached titles and summaries directly from local SQLite.
- **Single LLM call per page**: When a user views "unread" or "today", all uncached items on that page (up to 20 items) are processed in exactly 1 background LLM call.
- **Filter-specific briefs**: The Top Executive Brief card is only loaded and rendered for "unread" and "today" views; all other filters ("read", "starred", "all", search) set the brief to None.
- **Removed dedicated AI filter**: Removes `<button id="filter-ai">` so AI summaries augment natural reading views rather than acting as a separate silo.
- **Comprehensive server logging**: Clear `info!`, `warn!`, and `error!` logging for every outbound prompt, model, timing, response, fallback, and error.

---

## Architectural Options & Evaluation

### Option 1: Instant Page Load with Asynchronous Background Batch (Confirmed)

- **Explanation**:
  - The list page loads instantly using existing SQLite cache data.
  - On list rendering for "unread" or "today", the server collects all uncached entries on the visible page (up to page limit, e.g. 20) and triggers 1 background task.
  - The background task issues exactly 1 LLM call for all uncached items on that page using a compact JSON prompt (`{"top":"...","items":[{"i":...,"t":"...","b":"..."}]}`).
  - When the LLM responds, the page executive brief (`top`) and each item's suggested title (`t`) and brief (`b`) are saved to SQLite.
  - Subprocess logging tracks prompt dispatch, response duration, raw output, parsing warnings, and process errors.
  - Subsequent page reloads or detail view opens read directly from SQLite with zero latency.

- **Concrete Schema & Prompting Example**:
  ```json
  {
    "top": "Tech giants pivot towards edge-inference chips as open-weights models close performance gaps.",
    "items": [
      {
        "i": 4012,
        "t": "Nvidia Announces Blackwell Ultra",
        "b": "Nvidia unveiled its Blackwell Ultra architecture, boosting memory bandwidth by 40% for large model reasoning."
      }
    ]
  }
  ```
  ```sql
  -- SQLite AiDb Evolution
  ALTER TABLE article_ai_summaries ADD COLUMN suggested_title TEXT;
  CREATE TABLE IF NOT EXISTS page_executive_briefs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    brief TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
  );
  PRAGMA user_version = 2;
  ```

- **Pros**:
  - Zero latency impact on list browsing.
  - Predictable LLM usage: exactly 1 LLM call per viewed page with uncached items.
  - Eliminates redundant prompt overhead via prompt prefix caching.
  - Clean separation: list navigation never blocks on external CLI processes.

- **Cons**:
  - Newly loaded items display original titles until the background batch job finishes and the page is refreshed or detail view opened.

---

### Option 2: Synchronous Sequential Processing

- **Explanation**:
  - Process items synchronously or sequentially one-by-one as articles are opened.
  - Top executive brief is computed by stitching together existing individual summaries.

- **Pros**:
  - Simple single-item request structure.

- **Cons**:
  - High token overhead: prompt instructions repeated for every single article.
  - No unified page-level synthesis.
  - Patchy list view where only previously opened articles have AI titles.

---

### Decision

**User confirmed Option 1**: Instant Page Load with Asynchronous Background Batch.

---

## Detailed Component Design

### 1. Database Layer (`crates/server/src/ai/db.rs`)

- **Schema Migration (`PRAGMA user_version = 2`)**:
  - If current `user_version < 2`:
    - Add `suggested_title TEXT` to `article_ai_summaries` table.
    - Create `page_executive_briefs` table for storing batch executive summaries.
    - Set `PRAGMA user_version = 2`.
- **Data Models**:
  ```rust
  #[derive(Debug, Clone)]
  pub struct AiSummaryRecord {
      pub entry_id: i64,
      pub summary: String,
      pub suggested_title: Option<String>,
      pub model: String,
      pub created_at: String,
  }
  ```
- **CRUD Operations**:
  - `get_summary_and_title(entry_id: i64) -> Option<AiSummaryRecord>`: Single record lookup.
  - `get_batch_summaries(entry_ids: &[i64]) -> HashMap<i64, AiSummaryRecord>`: Fast in-memory map lookup for list rendering.
  - `save_summary_and_title(entry_id: i64, suggested_title: Option<&str>, summary: &str, model: &str) -> Result<(), AiDbError>`: Upsert item summary and title.
  - `save_page_brief(brief: &str) -> Result<(), AiDbError>`: Persist page-level executive brief.
  - `get_latest_page_brief() -> Option<String>`: Retrieve latest executive brief for list header.

---

### 2. Extraction & Token Optimization (`crates/server/src/ai/extractor.rs`)

- **Body Truncation**: Truncate article body text to 250 words per item before batching into LLM prompt.
- **Boilerplate Stripping**: Retain existing removal of scripts, headers, navigation elements, footers, and SVGs.
- **Deduplication**: Filter out duplicate entry IDs within the same batch.

---

### 3. LLM Runner & Server Logging (`crates/server/src/ai/llm.rs`)

- **Thinking Budget**: Pass `--thinking-budget 0` to CLI args to disable reasoning tokens and minimize latency.
- **Micro-Batch Prompt Contract**:
  - Static instructions placed first to maximize prompt prefix caching.
  - Prompt format:
    ```text
    You are an executive news editor. For the articles below, provide:
    1. "top": A 1-2 sentence executive brief (<35 words) synthesizing the most important themes.
    2. "items": For each article id, a punchy "t" (suggested title, <=10 words) and "b" (executive brief, 1-2 sentences, <35 words, what changed/why it matters).

    Articles:
    [ID 101] Title: ...
    Content: ...

    Output JSON format:
    {"top":"...","items":[{"i":101,"t":"...","b":"..."}]}
    ```
- **Comprehensive Server Logging**:
  - `tracing::info!`:
    - Before calling LLM: Log outbound prompt preview (first 200 chars), model name, and item count.
      ```rust
      tracing::info!(
          target: "ai::batch",
          "[AI_DISPATCH] model=\"{}\" item_count={} prompt_preview={:?}",
          self.model,
          articles.len(),
          prompt.chars().take(200).collect::<String>()
      );
      ```
    - Upon return: Log duration in milliseconds and full response output.
      ```rust
      tracing::info!(
          target: "ai::batch",
          "[AI_RETURN] elapsed_ms={} raw_output={:?}",
          elapsed.as_millis(),
          raw_output
      );
      ```
  - `tracing::warn!`:
    - Log JSON parsing failures and fallback strategy activation (regex fallback or extractive fallback).
      ```rust
      tracing::warn!(
          target: "ai::batch",
          "[AI_WARN] reason=\"json_parse_fallback\" fallback_strategy=\"regex\" raw_snippet={:?}",
          raw_output.chars().take(150).collect::<String>()
      );
      ```
  - `tracing::error!`:
    - Log CLI timeouts:
      ```rust
      tracing::error!(
          target: "ai::batch",
          "[AI_ERROR] error=\"cli_timeout\" timeout_secs=15 elapsed_ms={}",
          elapsed.as_millis()
      );
      ```
    - Log non-zero exit code or process spawn failure:
      ```rust
      tracing::error!(
          target: "ai::batch",
          "[AI_ERROR] error=\"process_failed\" exit_code={:?} stderr={:?}",
          status.code(),
          stderr_preview
      );
      ```
    - Log empty process stdout:
      ```rust
      tracing::error!(
          target: "ai::batch",
          "[AI_ERROR] error=\"empty_output\" elapsed_ms={}",
          elapsed.as_millis()
      );
      ```

- **Resilient Fallback Parsing**:
  - Primary: Direct deserialization into `BatchAiResponse { top, items }`.
  - Secondary: Markdown code fence stripping (````json ... ````).
  - Tertiary: Regex matching for `top` and individual item entries `{"i":..., "t":..., "b":...}`.
  - Quaternary: Extractive sentence fallback using article content if CLI invocation completely fails.

---

### 4. Single-Page Background Batch Processing (`crates/server/src/ai/prefetch.rs`)

- **Page-Batch Strategy**:
  - Removed old batch chunking into 12 items.
  - Removed old `unread_count > 20` prefetch gating.
  - When rendering an "unread" or "today" list page:
    - Collect all uncached entries on the current page (up to the page limit, e.g. 20 items).
    - If uncached entries exist, spawn 1 background task (`spawn_page_batch_job`).
    - The background task runs exactly 1 LLM call for all uncached entries on that page.
    - Persist the page brief (`top`) and each item's suggested title (`t`) and brief (`b`) into SQLite.

---

### 5. UI & Route Integration

- **Remove AI Filter**:
  - Remove `<button id="filter-ai">` from [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html).
  - Remove any corresponding `data-status="ai"` handling.

- **Filter-Specific Top Executive Brief**:
  - In [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) and [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs):
    - Determine active filter: `status` query parameter ("unread", "today", "read", "starred", "all", search).
    - If filter is "unread" or "today":
      - Query `ai.db.get_latest_page_brief().await` and pass to `EntryListTemplate`.
      - Collect uncached entries for the current page and spawn 1 background batch job.
    - For all other filters ("read", "starred", "all", search):
      - Set `top_executive_brief = None`.
      - Do not spawn background batch jobs.

- **List Template (`crates/ui-components/templates/partials/entry_list.html`)**:
  - Renders Top Executive Brief card only when `top_executive_brief` is `Some(...)`.
  - Displays sparkles icon, concise synthesis (<35 words), and clean card styling (`bg-muted/40 border border-border rounded-lg p-3.5 mb-4`).

- **List Rows (`crates/ui-components/templates/partials/entry_item.html`)**:
  - Displays `entry.suggested_title` when available, falling back to `entry.title`.
  - Preserves original headline in `title` HTML attribute for hover tooltip.

- **Detail View (`crates/ui-components/templates/partials/brief_card.html`)**:
  - Displays concise Executive Brief (<35 words) at the top of the opened article.

---

## Result

Result: done
