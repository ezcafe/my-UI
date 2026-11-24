# Codebase Skim: Details Page AI Rewrite with Batch Pre-request

## Core Modules & Touchpoints

1. **Database Schema & Storage (`crates/server/src/ai/db.rs`)**:
   - `init_schema`:
     - Inspect `user_version`. If `version < 4`, execute:
       ```sql
       ALTER TABLE article_ai_summaries ADD COLUMN rewrite TEXT;
       PRAGMA user_version = 4;
       ```
   - `AiSummaryRecord` & `ArticleSummaryInput`:
     - Add `pub rewrite: Option<String>` to both structs.
   - `get_summary_and_title` & `get_batch_summaries`:
     - Update `SELECT entry_id, summary, suggested_title, rewrite, model, created_at FROM article_ai_summaries...`
   - `save_summary_and_title` / new `save_summary_record`:
     - Support persisting `rewrite` along with title, summary/brief, and model into SQLite with `ON CONFLICT(entry_id) DO UPDATE SET rewrite = excluded.rewrite, ...`.

2. **LLM Engine & Response Contracts (`crates/server/src/ai/llm.rs`)**:
   - `BatchAiItem`:
     - Add `#[serde(rename = "r", default)] pub rewrite: Option<String>` to the item schema.
   - `format_batch_prompt_structured`:
     - Update instructions to request `"r"`: An Inverted Pyramid rewrite under 110 words (target 30–60 words in 2–3 scannable bullet points or paragraphs), placing the core conclusion in the first two sentences.
     - Update compact JSON schema example:
       `{"top":"...","items":[{"i":101,"t":"...","b":"...","r":"..."}]}`
     - Reiterate language parity rules: match the exact language of source articles.
   - Response Parsing & Regex Fallback:
     - Update `parse_batch_response` to extract `"r"` in direct `serde_json` parsing.
     - Extend `ITEM_REGEX` fallback to optionally capture `"r"` if LLM outputs non-standard JSON wrapper.
   - Extractive Fallback:
     - In `fallback_extractive_structured`, populate `rewrite` with extracted lead text when LLM fails or is unavailable.

3. **Background Prefetch Service (`crates/server/src/ai/prefetch.rs`)**:
   - `run_prefetch`:
     - When receiving `batch_res.items`, extract `item.rewrite` alongside `item.suggested_title` and `item.brief`.
     - Pass `item.rewrite` into `ai.db.save_summary_record(...)` so that batch-computed rewrites are immediately queryable.

4. **Detail Route & Data Loading (`crates/server/src/routes/entries.rs`)**:
   - `get_entry_detail_htmx`:
     - Fetch `record` via `ai_service.db.get_summary_and_title(id).await`.
     - Read `record.rewrite`. If present and non-empty, use it as the primary article body.
     - If absent (e.g. LLM prefetch pending or failed), fall back gracefully to `summary` / extractive brief so the view renders cleanly without breaking.
     - Pass `rewrite` into `EntryDetailTemplate`.

5. **Template & View Model (`crates/ui-components/src/templates.rs` & `entry_detail.html`)**:
   - `EntryDetailTemplate`:
     - Add `pub rewrite: Option<String>` (or pass rendered body).
   - `crates/ui-components/templates/partials/entry_detail.html`:
     - Implement user-confirmed **Option B**: Replace `sanitized_content` in the article body with the AI Rewrite.
     - Retain external source link ("Original") in the header.
     - Render rewrite in clean Annuaire typography with Inverted Pyramid lead and scannable bullet formatting.

## Test Infrastructure

- Unit tests in `crates/server/src/ai/db.rs` for `user_version = 4` migration and record saving/reading.
- Prompt formatting and JSON parsing tests in `crates/server/src/ai/llm.rs`.
- Prefetch integration tests in `crates/server/src/ai/prefetch.rs`.
- Detail view rendering tests in `crates/ui-components/src/templates.rs`.
- Integration tests in `crates/server/src/routes/entries.rs`.

## Result

Result: done
