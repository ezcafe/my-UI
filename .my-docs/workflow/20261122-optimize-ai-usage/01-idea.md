# Ideation: Optimize AI Usage & Token Efficiency

## Executive Summary

This feature enhances AI capabilities in My News when `AI_ENABLED=true`:
1. **Executive Brief**: Extracts what changed or why it matters in 1–2 sentences (<35 words) for each item, used in the details page and on the list page.
2. **Suggested Title**: AI generates an informative, punchy title for each item, replacing the original title on the list view (with original title available on hover / detail).
3. **List Page Brief Section**: Shows brief insights directly on the list page for all items on the current page.
4. **Structured Debug Logging**: Emits clear logs for AI operations (`success`, `warning`, `error`) with latency, cache status, and token/content size.
5. **Token Reduction Strategy**: Recommends high-impact techniques to minimize token consumption and cost.

---

## Scope & Boundaries

- **Goals**:
  - Store and cache AI-generated briefs and suggested titles per entry in SQLite (`AiDb`).
  - Generate brief and suggested title in a single unified prompt to minimize token usage and latency.
  - Render suggested titles and brief sections on the list page without breaking existing keyboard/click interactions or Annuaire design rules.
  - Display the generated Executive Brief in the details panel under `brief_card.html`.
  - Add structured debug logs for all AI lifecycle events (`tracing` with level control).
  - Provide a practical token reduction blueprint.
- **Non-Goals**:
  - Full-text translation or multi-turn chat about articles.
  - Altering Miniflux upstream data or modifying database entries on the Miniflux server.
- **Anti-Goals**:
  - Running separate LLM calls for title and brief (wasteful duplicate token overhead).
  - Blocking page loads if LLM CLI fails or times out (always gracefully fallback to original title and extractive teaser).

---

## Token Reduction Strategies

To reduce token consumption by 60%–80% while improving response speed:

1. **Combined Prompting (Single-Call Generation)**:
   - *Strategy*: Ask the model to return both the suggested title and the executive brief in a single JSON response or delimited format.
   - *Token Savings*: Avoids sending the article context twice. Saves ~50% of input tokens compared to two independent calls.
   - *Example prompt format*:
     ```text
     Analyze the article below. Output in exactly two lines:
     TITLE: <concise suggested title, max 10 words>
     BRIEF: <1-2 sentences on what changed or why it matters, under 35 words>
     ```

2. **Input Truncation & Lead Extraction**:
   - *Strategy*: News articles put the most critical facts in the first 2-3 paragraphs. Truncate article content to the first 250 words (down from 500 words) after cleaning HTML boilerplate.
   - *Token Savings*: Reduces input tokens by ~50% per article with virtually zero loss in brief quality.

3. **Persistent SQLite Caching**:
   - *Strategy*: Cache `(entry_id, suggested_title, brief, model, created_at)` in SQLite (`AiDb`). Never call the LLM twice for the same entry.
   - *Token Savings*: 100% token savings on subsequent page visits, pagination back/forward, and article re-opens.

4. **Batch Processing per Page**:
   - *Strategy*: For items on the current page, batch multiple article titles/excerpts into a single prompt (e.g. 5-10 articles per batch) when generating page-level briefs.
   - *Token Savings*: Amortizes system prompt and instructions across multiple items.

5. **Extractive Pre-Filtering for Short Notices**:
   - *Strategy*: If an article is already under 40 words (e.g. short status update or tweet), use the text directly as the brief without invoking LLM.

---

## Architectural Decisions & Trade-Offs

### Decision 1: Combined Prompt Format vs Separate Calls

- **Option A (Recommended): Combined Delimited / JSON Prompt**
  - **Explanation**: A single LLM invocation produces both `TITLE` and `BRIEF`. Parsed cleanly in Rust.
  - **Example**:
    ```text
    TITLE: Rust 1.85 Stabilizes Async Closures
    BRIEF: Rust 1.85 brings async closures to stable, simplifying asynchronous functional pipelines across the ecosystem.
    ```
  - **Pros**: Cuts token consumption by ~50%; halves external CLI process spawns; keeps database writes atomic.
  - **Cons**: Requires simple parsing logic with fallback if the model misses a delimiter.

- **Option B: Two Separate LLM Invocations**
  - **Explanation**: One prompt for title (`LlmCliRunner::suggest_title`), one prompt for summary (`LlmCliRunner::summarize`).
  - **Pros**: Trivial parsing.
  - **Cons**: Doubles token usage, doubles process execution overhead, increases latency significantly.

- **Option C: JSON Mode Prompting**
  - **Explanation**: Ask for strict JSON `{"title": "...", "brief": "..."}`.
  - **Pros**: Structured format.
  - **Cons**: CLI models without enforced JSON mode might add markdown code blocks (````json ... ````), requiring regex stripping.

> **Recommendation**: **Option A**. Simple key-prefix lines (`TITLE:` and `BRIEF:`) work reliably across all CLI LLMs without JSON formatting quirks and minimize token overhead.

---

### Decision 2: UI Presentation of Briefs on the List Page

- **Confirmed Decision**: Top "Executive Briefs" summary card + AI-suggested title on list rows (no row subtitles)
  - **Details**:
    1. **List Rows**: Replace original article title with the AI-suggested punchy title. Keep the row height compact and clean (no secondary subtitle on list items). The original title remains accessible on hover / in the details view.
    2. **List Page Top**: Add a dedicated "Page Executive Brief" summary card at the top of the entries list highlighting the key takeaways / summary of the current page.
    3. **Details Pane**: Show the full "Executive Brief" (1–2 sentences, <35 words, what changed/why it matters) in the details panel under `brief_card.html`.

---

### Decision 3: Debug Logging Level and Output Format

- **Option A (Recommended): Structured `tracing` Logs with Specific Target and Tags**
  - **Explanation**: Use `tracing::info!`, `tracing::warn!`, and `tracing::error!` with target `ai::llm` and tags:
    - Success: `[AI_SUCCESS] entry_id=123 model="gpt-4o-mini" latency_ms=450 input_words=180 output_chars=140`
    - Warning: `[AI_WARN] entry_id=123 msg="CLI timed out after 5s; falling back to extractive summary"`
    - Error: `[AI_ERROR] entry_id=123 error="failed to execute CLI command: NotFound"`
  - **Pros**: Standard Rust logging pattern; easily filtered in console or production log collectors; non-blocking.
  - **Cons**: Requires `tracing-subscriber` env filter configuration (e.g., `RUST_LOG=info,server::ai=debug`).

- **Option B: Custom In-Memory or SQLite Debug Log Table**
  - **Explanation**: Store the last 50 AI operations in an in-memory ring buffer or SQLite table with an admin UI endpoint `/debug/ai-logs`.
  - **Pros**: Can view logs in the web browser.
  - **Cons**: Extra schema complexity and UI maintenance for internal debug logs.

> **Recommendation**: **Option A**. Standard Rust `tracing` logs provide zero overhead and integrate with terminal and container logging.

---

## Database Schema Evolution

Update `AiDb` table `article_ai_summaries`:
```sql
ALTER TABLE article_ai_summaries ADD COLUMN suggested_title TEXT;
```
When `init_schema` runs:
```sql
CREATE TABLE IF NOT EXISTS article_ai_summaries (
  entry_id INTEGER PRIMARY KEY,
  summary TEXT NOT NULL,
  suggested_title TEXT,
  model TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

---

## Tentative Lens & Card Flags

- **Has UI**: yes (List item title replacement, list brief snippet, brief card in details)
- **Has API**: yes (Endpoints returning suggested titles & briefs)
- **Has DB**: yes (`AiDb` schema update for `suggested_title`)
- **Has auth/PII/schema**: yes (Database schema migration)

---

## Alignment Confirmed

- **Ideation Status**: Confirmed by user.
- **User Preference**: Top "Executive Briefs" summary card + AI-suggested title on list rows (no subtitle).
- **Optimization Guidelines**:
  1. Pre-Processing: Deduplicate before sending, strip boilerplate & truncate (title + first 300–500 words).
  2. Batching Strategy: Micro-batches of 10–15 articles per call with prompt caching enabled.
  3. Token-Efficient Prompting: Compact JSON keys (`i`, `t`, `b`), constrained summary angle (1–2 sentences, <35 words), reasoning/thinking tokens disabled (budget = 0).
  4. App-Level Caching & Lazy Loading: Deduplicate by URL/Hash in database, summarize on scroll/viewport if real-time.
