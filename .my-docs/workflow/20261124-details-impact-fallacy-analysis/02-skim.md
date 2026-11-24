# Codebase Skim: Article Detail AI Rewrite with Impact & Fallacy Analysis

- **Stage:** skim
- **Status:** completed
- **Result:** `done`

---

## 1. Architectural Map

### UI & Templates
- `crates/ui-components/templates/partials/entry_detail.html`:
  - Contains the details drawer layout.
  - Currently renders `brief_card.html`, then `#ai-rewrite-box`, and when `!is_rewrite`, renders `{{ sanitized_content|safe }}`.
  - Must remove `sanitized_content` completely.
  - Must render Impact and Fallacy highlight cards (Option 3B: two separate cards styled like brief).
- `crates/ui-components/src/templates.rs`:
  - `EntryDetailTemplate` struct: currently has `entry`, `summary`, `sanitized_content`, `auth_enabled`, `rewrite`, `is_rewrite`.
  - Must add `impact: String` and `fallacies: String`.

### Database Layer
- `crates/server/src/ai/db.rs`:
  - `AiSummaryRecord` and `ArticleSummaryRow` currently contain `entry_id`, `summary`, `suggested_title`, `rewrite`, `model`, `created_at`.
  - Schema in `INIT_SCHEMA_SQL`:
    - `ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;`
    - `ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;`
  - `save_summary_and_title_with_rewrite` (or `save_summary_and_title_with_analysis`): update PostgreSQL upsert query and mock in-memory store (`MockData`) to persist and retrieve `impact` and `fallacies`.

### LLM Client & Service
- `crates/server/src/ai/llm.rs`:
  - `rewrite` method currently sends `"Rewrite the following article in an inverted pyramid style: ..."`.
  - Update to request structured JSON:
    ```json
    {
      "rewrite": "...",
      "impact": "...",
      "fallacies": "..."
    }
    ```
  - Parse JSON response with fallback handling (fenced markdown code blocks and graceful fallbacks).
  - Include reassuring default for fallacy if LLM indicates objective reporting without fallacies.
- `crates/server/src/ai/mod.rs`:
  - Update `generate_rewrite` (or `generate_rewrite_and_analysis`) to return `(rewrite, impact, fallacies)`.
  - Persist all three fields into `AiDb`.

### Routing & HTMX
- `crates/server/src/routes/entries.rs`:
  - `get_entry_detail_htmx`: loads `record` from `AiDb`. If `rewrite` (and `impact`/`fallacies`) are present, passes them to `EntryDetailTemplate`.
  - `get_entry_rewrite_htmx`: when invoked on-demand via HTMX `/hx/entries/:id/rewrite`, generates rewrite + analysis, caches them, and returns the rendered HTML block containing the Impact card, Fallacy card, and Rewrite.

### Tests
- `crates/ui-components/src/templates.rs`: update `EntryDetailTemplate` instantiation in unit tests.
- `crates/server/src/routes/entries.rs`: unit tests for `get_entry_rewrite_htmx` and `get_entry_detail_htmx`.
- `crates/server/tests/e2e_tests.rs`: e2e tests for drawer lazy load and cached states.

---

## 2. Verdict

- **Skim Result:** `done`
- **Proceed to:** `analyze`
