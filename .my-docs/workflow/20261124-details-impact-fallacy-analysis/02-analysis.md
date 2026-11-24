# Context & Risk Analysis: Article Detail AI Rewrite with Impact & Fallacy Analysis

- **Stage:** analyze
- **Status:** completed
- **Result:** `done`

---

## 1. Deep Context & Architecture

### Current Flow
1. When a user clicks an article in the list:
   - `get_entry_detail_htmx` queries Miniflux for entry metadata and checks `AiDb` for cached summaries.
   - If `record.rewrite` exists, `is_rewrite = true` and `rewrite` is passed to `EntryDetailTemplate`.
   - If `record.rewrite` is None, `is_rewrite = false`, and `entry_detail.html` renders:
     - The Executive Brief card (`brief_card.html`).
     - A skeleton loader `#ai-rewrite-box` configured with `hx-get="/hx/entries/{{ entry.id }}/rewrite" hx-trigger="load" hx-swap="outerHTML"`.
     - Previously, it also rendered `sanitized_content` (the full raw article body).
2. When `/hx/entries/:id/rewrite` fires:
   - It extracts the article text and calls `ai_service.generate_rewrite`.
   - `generate_rewrite` calls `llm.rewrite(title, content)`.
   - The result is cached in `AiDb` and returned as an HTML snippet replacing `#ai-rewrite-box`.

### Target Flow
1. **Details Page Viewport**:
   - `sanitized_content` is removed from `entry_detail.html` entirely.
   - When cached (`is_rewrite = true` with analysis data):
     - Executive Brief card.
     - Impact Analysis highlight card (Option 3B: styled like brief, with `trending-up` icon).
     - Fallacy & Objectivity Check highlight card (Option 3B: styled like brief, with `shield-alert` or `scale` icon, displaying reassuring indicator if none detected).
     - Inverted Pyramid AI Rewrite box (`#ai-rewrite-box`).
   - When uncached (`is_rewrite = false`):
     - Executive Brief card.
     - Skeleton container `#ai-detail-analysis-box` with `hx-get="/hx/entries/{{ entry.id }}/rewrite" hx-trigger="load" hx-swap="outerHTML"`.
2. **On-demand HTMX Route `/hx/entries/:id/rewrite`**:
   - Invokes `ai_service.generate_rewrite_and_analysis`.
   - Calls `llm.rewrite_and_analyze(title, content)`.
   - Returns HTML containing the Impact card, Fallacy card, and Rewrite box.

---

## 2. LLM Prompt & Parsing Specification

### System & User Prompt Structure
Following 2026 prompt engineering guidelines for structured extraction and fallacy resistance:
```text
Analyze and rewrite the following article. Output raw JSON only matching this schema:
{
  "rewrite": "Inverted pyramid style rewrite. Lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context.",
  "impact": "Analyze the practical consequences and real-world impact of this news (who is affected, industry or social significance, short/long-term stakes).",
  "fallacies": "Identify any informal logical fallacies or rhetorical biases in the text (e.g. ad hominem, straw man, false dilemma, hasty generalization, cherry-picking, emotional appeal). If the reporting is objective, factual, and logically sound, strictly output: 'Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu' (for Vietnamese) or 'No logical fallacies detected — Objective and well-supported reporting' (for English)."
}

CRITICAL RULES:
1. Multilingual parity: Match the exact source language of the article (Vietnamese -> Vietnamese, English -> English).
2. Output valid, raw JSON only. Do not add markdown fences or extra explanations.

Title: {title}
Content: {content}
```

### Parsing Pipeline
1. Direct `serde_json::from_str::<DetailAnalysisResponse>`.
2. Fallback fenced block extraction (````json ... ```` or ```` ... ````).
3. Graceful degradation: if parsing fails, treat raw text as rewrite, generate standard fallback impact, and set reassuring fallacy indicator.

---

## 3. Database & Storage Architecture

### PostgreSQL
- `article_ai_summaries`:
  ```sql
  ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;
  ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;
  ```
- Update `AiSummaryRecord`:
  ```rust
  pub struct AiSummaryRecord {
      pub entry_id: i64,
      pub summary: String,
      pub suggested_title: Option<String>,
      pub rewrite: Option<String>,
      pub impact: Option<String>,
      pub fallacies: Option<String>,
      pub model: String,
      pub created_at: String,
  }
  ```
- Update `save_summary_and_title_with_rewrite` to accept `impact: Option<&str>` and `fallacies: Option<&str>`.
- In-memory mock (`MockData`): store and return `impact` and `fallacies`.

---

## 4. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **LLM JSON formatting errors** | Missing fields or malformed response | Robust parser with fence extraction; safe fallback defaults. |
| **Existing database records lack impact/fallacies** | NULL values on cached records | Conditional rendering: if `impact` is present, show cards; if missing on old cache, HTMX route or detail route handles gracefully without crashing. |
| **Drawer vertical scrolling fatigue** | Three cards (Brief, Impact, Fallacy) before rewrite | Use compact typography, tight padding (`p-4`), and clear scannable visual hierarchy adhering to Annuaire theme. |
| **Clippy / Test breakages** | Breaking changes in template struct | Update all test instances of `EntryDetailTemplate` across `ui-components` and `server`. |

---

## 5. Verdict

- **Analysis Result:** `done`
- **Proceed to:** `grill`
