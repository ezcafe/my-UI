# System Analysis: AI Optimization & Presentation

## Context & Objectives

The goal is to optimize AI usage for token efficiency and latency while elevating article insights in My News:
1. **Token Efficiency**: Cut token consumption by 60%–80% using combined single-call generation (suggested title + brief together), tighter input truncation (250 words), and persistent SQLite caching.
2. **Presentation**:
   - List rows show punchy AI-suggested titles (clean, compact height, no subtitles) with original titles in tooltips.
   - Top of list page features an "Executive Briefs" summary card.
   - Article details panel renders the 1–2 sentence (<35 words) Executive Brief in `brief_card.html`.
3. **Observability**: Structured debug logging with `tracing` capturing latency, token/character counts, and cache hits/misses.

---

## Architectural & Technical Analysis

### 1. Database Schema Evolution (`AiDb`)

- **Current State**:
  - `article_ai_summaries` table: `(entry_id INTEGER PRIMARY KEY, summary TEXT NOT NULL, model TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT ...)`.
  - `PRAGMA user_version = 1`.
- **Target State**:
  - Add column `suggested_title TEXT`.
  - `PRAGMA user_version = 2`.
  - Schema upgrade logic: if `user_version == 1`, run `ALTER TABLE article_ai_summaries ADD COLUMN suggested_title TEXT; PRAGMA user_version = 2;`. Fresh installs create table with `suggested_title` directly and set `user_version = 2`.
- **Query Capabilities**:
  - `get_summary_and_title(entry_id: i64) -> Option<AiSummaryRecord>`
  - `get_batch_summaries(entry_ids: &[i64]) -> HashMap<i64, AiSummaryRecord>` to avoid N+1 queries when rendering the entries list.
  - `save_summary_and_title(entry_id: i64, suggested_title: Option<&str>, summary: &str, model: &str)`

### 2. Prompting & LLM Runner (`LlmCliRunner`)

- **Current State**:
  - `format_prompt` asks only for summary.
  - CLI output returned directly without structured parsing.
- **Combined Prompt**:
  ```text
  Analyze the article below. Output in exactly two lines:
  TITLE: <concise suggested title, max 10 words>
  BRIEF: <1-2 sentences on what changed or why it matters, strictly under 35 words>

  Article Title: ...
  Article Content:
  ...
  ```
- **Parsing Strategy**:
  - Scan lines for `TITLE:` and `BRIEF:` prefixes (case-insensitive, trimming whitespace).
  - If `TITLE:` line is present and non-empty, extract as `suggested_title`. Otherwise `None`.
  - If `BRIEF:` line is present, extract as `brief`.
  - Fallback: If delimiters are missing, fallback cleanly to original title and extractive teaser.
- **Observability**:
  - Emit structured `tracing::info!(target: "ai::llm", ...)` on success with elapsed time in ms.
  - Emit `tracing::warn!(target: "ai::llm", ...)` on timeout / parse fallback.
  - Emit `tracing::error!(target: "ai::llm", ...)` on CLI process errors.

### 3. Article Content Extraction (`ArticleExtractor`)

- Reduce word truncation from 500 words to 250 words (`Self::truncate_words(&extracted_text, 250)`).
- Boilerplate removal already discards scripts, navigation, headers, footers, and SVGs.
- Fallback chain remains resilient (article tag -> body -> teaser).

### 4. UI & Template Integration

- **View Models (`ui-components/src/templates.rs`)**:
  - `EntryViewModel`:
    - `pub suggested_title: Option<String>`
    - In `entry_list.html` and `entry_item.html`:
      ```html
      <span class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors"
            title="{{ entry.title }}">
        {% if let Some(suggested) = entry.suggested_title %}
          {{ suggested }}
        {% else %}
          {{ entry.title }}
        {% endif %}
      </span>
      ```
  - `EntryListTemplate`:
    - `pub page_brief: Option<String>`
    - In `entry_list.html`: render Executive Briefs summary card at the top when present.

---

## Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
|------|--------|---------------------|
| LLM CLI format drift | Parse error | Resilient prefix extraction; fallback to original title and extractive teaser |
| Database migration lock | Temporary startup stall | `PRAGMA busy_timeout = 5000; PRAGMA journal_mode = WAL;` already configured |
| List page response latency | Slower navigation if blocking | Query SQLite cache in batch (`get_batch_summaries`); do not block on cold LLM CLI during list render |
| Style inconsistency | Annuaire visual degradation | Strictly use existing CSS tokens, `brand` red bar for position only, no hardcoded breakpoints |

---

## Open Questions for Design & Grilling

1. **List Page Executive Brief generation**:
   - For the top summary card on the list page: should it display the brief of the top highlighted article, or a synthesized overview of the page's unread items?
2. **Cache population strategy for list titles**:
   - Should suggested titles for list rows be loaded exclusively from SQLite cache (populated in background / on article access), or should a background worker / lazy HTMX trigger populate un-cached entries?

## Result

Result: done
