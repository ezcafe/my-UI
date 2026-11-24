# Design: Article Detail AI Rewrite with Impact & Fallacy Analysis

- **Stage:** design
- **Status:** completed
- **Result:** `done`

---

## 1. Executive Summary

This feature transforms the article details drawer into a pure AI-synthesized reading experience by:
1. **Removing raw article content (`sanitized_content`) entirely** from the details page, leaving the brief, impact analysis, fallacy detection, and rewrite as the sole body content (with the external "Original" button retained for source verification).
2. **Extending the LLM rewrite request** into a single structured extraction pass returning:
   - Inverted pyramid rewrite (`rewrite`).
   - News impact analysis (`impact`).
   - Logical fallacy and bias evaluation (`fallacies`), defaulting to a reassuring indicator if the reporting is objective.
3. **Persisting analysis metadata** in dedicated PostgreSQL columns (`impact TEXT`, `fallacies TEXT`) in `article_ai_summaries`.
4. **Rendering two distinct highlight cards** styled identically to the Executive Brief for Impact Analysis and Fallacy & Objectivity Check (Option 3B), followed by the AI Rewrite.

---

## 2. Architecture & Decision Comparison

### Option A (Recommended & Selected): Structured JSON with Dedicated PostgreSQL Columns & Distinct Cards
- **LLM Contract:** Prompt enforces raw JSON output: `{"rewrite": "...", "impact": "...", "fallacies": "..."}` with language parity.
- **Database:** `article_ai_summaries` schema extended with `impact TEXT` and `fallacies TEXT`.
- **UI:** Two separate highlight cards (Impact and Fallacy) matching `brief_card.html` styling, placed before `#ai-rewrite-box`. Raw HTML content completely omitted.
- **Pros:**
  - High resilience and deterministic parsing using JSON.
  - Zero schema pollution; strongly typed in Rust domain models.
  - Clear visual demarcation of impact vs. rhetorical fallacy checks.
- **Cons:**
  - Requires updating database structs and test fixtures.

### Option B: Markdown Tagged Blocks in `rewrite` Column
- **LLM Contract:** Prompt requests `### Rewrite`, `### Impact`, `### Fallacies`.
- **Database:** Stored as raw markdown in `rewrite` column.
- **Pros:** No database schema modification.
- **Cons:** Fragile string parsing, prone to model formatting variations, cannot easily index or query fields independently.

---

## 3. Detailed Component Design

### 3.1 Database Layer (`crates/server/src/ai/db.rs`)

#### Schema Extension
```sql
ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;
ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;
```

#### Domain Structs
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleSummaryInput {
    pub entry_id: i64,
    pub summary: String,
    pub suggested_title: Option<String>,
    pub rewrite: Option<String>,
    pub impact: Option<String>,
    pub fallacies: Option<String>,
    pub model: String,
}
```

#### Persistence Method
```rust
pub async fn save_summary_and_title_with_analysis(
    &self,
    entry_id: i64,
    suggested_title: Option<&str>,
    summary: &str,
    rewrite: Option<&str>,
    impact: Option<&str>,
    fallacies: Option<&str>,
    model: &str,
) -> Result<(), AiDbError>
```
PostgreSQL upsert:
```sql
INSERT INTO article_ai_summaries (entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
ON CONFLICT (entry_id) DO UPDATE SET
    summary = EXCLUDED.summary,
    suggested_title = COALESCE(EXCLUDED.suggested_title, article_ai_summaries.suggested_title),
    rewrite = COALESCE(EXCLUDED.rewrite, article_ai_summaries.rewrite),
    impact = COALESCE(EXCLUDED.impact, article_ai_summaries.impact),
    fallacies = COALESCE(EXCLUDED.fallacies, article_ai_summaries.fallacies),
    model = EXCLUDED.model;
```

---

### 3.2 LLM Extraction Layer (`crates/server/src/ai/llm.rs`)

#### Domain Models
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
pub struct DetailAnalysisResult {
    pub rewrite: String,
    pub impact: String,
    pub fallacies: String,
}
```

#### Prompt Specification
```rust
pub fn format_rewrite_and_analyze_prompt(title: &str, content: &str) -> String {
    format!(
        "Analyze and rewrite the following article. Output raw JSON only matching this schema:\n\
        {{\n  \
          \"rewrite\": \"Inverted pyramid style rewrite. Lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context.\",\n  \
          \"impact\": \"Analyze the practical consequences and real-world impact of this news (who is affected, industry or social significance, short/long-term stakes).\",\n  \
          \"fallacies\": \"Identify any informal logical fallacies or rhetorical biases in the text (e.g. ad hominem, straw man, false dilemma, hasty generalization, cherry-picking, emotional appeal). If the reporting is objective, factual, and logically sound, strictly output: 'Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu' (for Vietnamese) or 'No logical fallacies detected — Objective and well-supported reporting' (for English).\"\n\
        }}\n\n\
        CRITICAL MULTILINGUAL RULE:\n\
        Write your response in the EXACT SAME LANGUAGE as the source article text.\n\
        If an article is in Vietnamese, all fields MUST be in Vietnamese.\n\
        If an article is in English, all fields MUST be in English.\n\
        Do NOT translate articles into a different language.\n\n\
        Output raw JSON only.\n\n\
        Title: {}\n\
        Content:\n{}",
        title.trim(),
        content.trim()
    )
}
```

#### Dual-Pass Deserialization & Reassurance Normalization
1. Try direct `serde_json::from_str::<DetailAnalysisResult>`.
2. Try extracting from code fences (````json ... ```` or ```` ... ````).
3. Post-process `fallacies`:
   If `fallacies` is empty or matches tokens like "none", "không có", "không phát hiện", normalize to:
   - Vietnamese: `"Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"`
   - English: `"No logical fallacies detected — Objective and well-supported reporting"`
4. Fallback: if JSON parsing completely fails, treat output as `rewrite`, set fallback `impact` to the executive brief, and set standard reassuring fallacy message.

---

### 3.3 UI & Presentation Layer

#### Template Model (`crates/ui-components/src/templates.rs`)
```rust
#[derive(Template)]
#[template(path = "partials/entry_detail.html")]
pub struct EntryDetailTemplate {
    pub entry: EntryViewModel,
    pub summary: String,
    pub auth_enabled: bool,
    pub rewrite: String,
    pub impact: String,
    pub fallacies: String,
    pub is_rewrite: bool,
}
```

#### Template HTML (`crates/ui-components/templates/partials/entry_detail.html`)
```html
    <!-- AI Brief Card Partial -->
    {% include "brief_card.html" %}

    <div id="ai-detail-analysis-box">
      {% if is_rewrite %}
      <!-- Impact Analysis Highlight Card -->
      {% if !impact.is_empty() %}
      <div class="rounded-lg p-4 bg-muted/40 border border-border mb-6">
        <div class="flex items-center gap-2 mb-2">
          <div class="p-1 rounded bg-muted text-foreground">
            <i data-lucide="trending-up" class="w-4 h-4 text-brand"></i>
          </div>
          <span class="text-xs font-bold tracking-wide uppercase text-foreground font-sans">Impact Analysis</span>
        </div>
        <p class="text-sm text-foreground/90 leading-relaxed font-sans">
          {{ impact }}
        </p>
      </div>
      {% endif %}

      <!-- Logical Fallacy & Objectivity Check Highlight Card -->
      {% if !fallacies.is_empty() %}
      <div class="rounded-lg p-4 bg-muted/40 border border-border mb-6">
        <div class="flex items-center gap-2 mb-2">
          <div class="p-1 rounded bg-muted text-foreground">
            <i data-lucide="shield-alert" class="w-4 h-4 text-brand"></i>
          </div>
          <span class="text-xs font-bold tracking-wide uppercase text-foreground font-sans">Logical Fallacy & Objectivity Check</span>
        </div>
        <p class="text-sm text-foreground/90 leading-relaxed font-sans">
          {{ fallacies }}
        </p>
      </div>
      {% endif %}

      <!-- Inverted Pyramid Rewrite Content -->
      <div id="ai-rewrite-box" class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
        {{ rewrite }}
      </div>
      {% else %}
      <!-- Skeleton Loading State -->
      <div class="animate-pulse space-y-6 my-4" hx-get="/hx/entries/{{ entry.id }}/rewrite" hx-trigger="load" hx-swap="outerHTML">
        <div class="rounded-lg p-4 bg-muted/30 border border-border space-y-2">
          <div class="h-4 bg-muted rounded w-1/4"></div>
          <div class="h-3 bg-muted rounded w-3/4"></div>
        </div>
        <div class="rounded-lg p-4 bg-muted/30 border border-border space-y-2">
          <div class="h-4 bg-muted rounded w-1/3"></div>
          <div class="h-3 bg-muted rounded w-5/6"></div>
        </div>
        <div class="space-y-3">
          <div class="h-4 bg-muted rounded w-3/4"></div>
          <div class="h-4 bg-muted rounded w-5/6"></div>
          <div class="h-4 bg-muted rounded w-2/3"></div>
        </div>
      </div>
      {% endif %}
    </div>
```

---

### 3.4 HTMX Lazy Route (`/hx/entries/:id/rewrite`)
When on-demand rewrite is invoked:
1. Generate analysis via `ai_service.generate_rewrite_and_analysis(id, &title, &content).await`.
2. Cache `rewrite`, `impact`, and `fallacies` in `AiDb`.
3. Render HTML fragment returning the Impact Card, Fallacy Card, and Rewrite Box wrapped in `<div id="ai-detail-analysis-box">...</div>` to seamlessly replace the skeleton loader.

---

## 4. Verification & Testing Strategy
1. **Unit Tests (`crates/server/src/ai/llm.rs`)**:
   - Prompt format verification with multilingual rules.
   - Deserialization of valid JSON, fenced JSON, and fallback behavior.
   - Fallacy reassurance text normalization.
2. **Database Tests (`crates/server/src/ai/db.rs`)**:
   - Verify `save_summary_and_title_with_analysis` and retrieval in both PostgreSQL and mock DB.
   - Verify upsert preserves existing fields when optional updates occur.
3. **Template Tests (`crates/ui-components/src/templates.rs`)**:
   - Verify `EntryDetailTemplate` renders Brief, Impact Card, Fallacy Card, and Rewrite without raw content.
   - Verify no trace of `sanitized_content` in rendered output.
4. **Integration & Route Tests (`crates/server/src/routes/entries.rs`)**:
   - Verify `get_entry_detail_htmx` serves cached impact, fallacies, and rewrite.
   - Verify `/hx/entries/:id/rewrite` generates, caches, and returns the full analysis fragment.
5. **E2E Tests (`crates/server/tests/e2e_tests.rs`)**:
   - Verify end-to-end flow from list click to detail panel drawer rendering.
