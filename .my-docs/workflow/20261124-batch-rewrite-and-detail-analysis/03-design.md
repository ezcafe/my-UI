# Design: Batch Rewrite & Decoupled Detail Analysis

- **Workflow Slug:** `20261124-batch-rewrite-and-detail-analysis`
- **Mode:** full
- **Result:** done

---

## 1. Architectural Options & Recommendation

### Option 1 (Recommended): Batch Rewrite Generation + On-Demand Background Analysis
- **Explanation**:
  1. In the batch LLM prompt, replace the 3-sentence brief (`b`) with the full inverted-pyramid rewrite (`r`). The batch response outputs: `{"top":"...","items":[{"i":...,"t":"...","r":"..."}]}`.
  2. Batch results are persisted in `article_ai_summaries`, populating `rewrite` and `summary`.
  3. In `entry_detail.html`, remove the "Executive Brief" card (`brief_card.html`). Render the `rewrite` text directly on load (0ms waiting time).
  4. An HTMX container triggers `/hx/entries/:id/analysis` on load (`hx-trigger="load" hx-swap="outerHTML"`). The server computes `impact` and `fallacies` via a focused prompt, saves them to `article_ai_summaries`, and returns the analysis fragment. If already cached in DB, the analysis cards are rendered on initial page load without any network roundtrip.
- **Concrete Example**:
  ```html
  <!-- Instant rewrite on load -->
  <div id="ai-rewrite-box" class="space-y-6">
    <div class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
      {{ rewrite }}
    </div>
  </div>

  <!-- Lazy-loaded impact and fallacies -->
  {% if has_analysis %}
    {% include "analysis_cards.html" %}
  {% else %}
    <div id="ai-analysis-box" class="animate-pulse space-y-4 my-4" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="load" hx-swap="outerHTML">
      <!-- Lightweight skeleton -->
    </div>
  {% endif %}
  ```
- **Pros**:
  - Instant reading experience: Readers immediately see the full rewritten article.
  - Zero duplicate "Executive Brief" cards.
  - Clean separation: Batch handles high-volume generation (titles + rewrites); interactive analysis is computed asynchronously as needed.
  - Efficient token use: System prompt overhead is minimized.
- **Cons**: Requires adding the `/analysis` endpoint and updating batch parsers.

### Option 2: All-in-Batch (Batch generates title, rewrite, impact, and fallacies)
- **Explanation**: Ask the batch LLM call to return title, rewrite, impact, and fallacies for all 20 articles at once.
- **Pros**: No separate analysis endpoint needed.
- **Cons**: Drastically increases batch output tokens (over 4,000 output tokens for 20 articles), causing frequent LLM timeouts or token limits. Wastes tokens on articles the user never opens.

### Recommendation
**Option 1** is selected and recommended. It directly honors the user's requirement: "replace brief with rewrite text in llm batch request. In details page, remove 'Executive Brief', show rewrite on load, then request llm for impact and fallacies".

---

## 2. Component Specifications

### A. LLM Prompts & Parsing (`crates/server/src/ai/llm.rs`)
1. **`BatchAiItem` struct**:
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
   pub struct BatchAiItem {
       #[serde(rename = "i")]
       pub id: i64,
       #[serde(rename = "t")]
       pub suggested_title: String,
       #[serde(rename = "r", alias = "b")]
       pub rewrite: String,
   }
   ```
2. **Batch prompt instruction (`format_batch_prompt_structured`)**:
   ```text
   You are an executive news editor. For the articles below, provide:
   1. "top": A concise executive brief synthesizing the most important themes across all sources on this page.
   2. "items": For each NEW article id, provide a punchy "t" (suggested title, strictly in one sentence) and "r" (inverted pyramid rewrite: lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context). Do not regenerate items for existing articles.
   
   CRITICAL MULTILINGUAL RULE:
   Write your response in the EXACT SAME LANGUAGE as the source text.
   
   Output raw JSON only using this compact schema:
   {"top":"...","items":[{"i":101,"t":"...","r":"..."}]}
   ```
3. **Regex fallback**:
   `ITEM_REGEX`: Match either `"r"` or `"b"`.
4. **Focused Analysis Prompt (`format_impact_and_fallacies_prompt`)**:
   ```text
   Analyze the following article. Output raw JSON only matching this schema:
   {
     "impact": "Analyze the practical consequences and real-world impact of this news (who is affected, industry or social significance, short/long-term stakes).",
     "fallacies": "Identify any informal logical fallacies or rhetorical biases in the text (e.g. ad hominem, straw man, false dilemma, hasty generalization, cherry-picking, emotional appeal). If the reporting is objective, factual, and logically sound, strictly output: 'Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu' (for Vietnamese) or 'No logical fallacies detected — Objective and well-supported reporting' (for English)."
   }
   
   CRITICAL MULTILINGUAL RULE:
   Write your response in the EXACT SAME LANGUAGE as the source article text.
   ```

### B. Database Operations (`crates/server/src/ai/db.rs`)
1. In `save_batch_summaries` / `save_summary_and_title_with_rewrite`:
   - Store `item.rewrite` into `summary` (NOT NULL) and `rewrite` (TEXT).
2. New method `save_analysis(&self, entry_id: i64, impact: &str, fallacies: &str) -> Result<(), AiDbError>`:
   - Updates `impact = excluded.impact, fallacies = excluded.fallacies` for `entry_id`.

### C. Server Routes (`crates/server/src/routes/entries.rs`)
1. Batch synthesis loop:
   - Construct `ArticleSummaryInput` using `item.rewrite` for `summary` and `rewrite`.
2. `get_entry_detail_htmx`:
   - Read `record = ai_service.db.get_summary_and_title(id).await`.
   - Extract `rewrite = record.rewrite.unwrap_or(record.summary)`.
   - Check `has_analysis = !record.impact.is_empty() && !record.fallacies.is_empty()`.
   - Pass `rewrite`, `impact`, `fallacies`, `has_analysis` to `EntryDetailTemplate`.
3. New handler `get_entry_analysis_htmx`:
   - Route: `/hx/entries/:id/analysis`.
   - If `impact` and `fallacies` already cached, return rendered `analysis_cards.html` fragment.
   - Otherwise, extract content, call `ai_service.analyze_impact_and_fallacies(id, title, content)`.
   - Save to DB and return the rendered HTML fragment.
   - Retain `/hx/entries/:id/rewrite` as an alias that returns the analysis fragment or rewrite container for backward compatibility.

### D. UI Templates (`crates/ui-components/`)
1. `crates/ui-components/templates/partials/entry_detail.html`:
   - Remove `{% include "brief_card.html" %}`.
   - Render `rewrite` immediately on load.
   - If `has_analysis` is true, render the Impact & Fallacy highlight cards.
   - Else, render `<div id="ai-analysis-box" class="animate-pulse space-y-4 my-4" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="load" hx-swap="outerHTML">...</div>`.
2. `crates/ui-components/templates/partials/analysis_cards.html`:
   - Clean partial rendering the Impact Analysis card and Logical Fallacy card.

---

## 3. Verification Plan
- Unit tests in `crates/server/src/ai/llm.rs` testing batch prompt generation with `"r"`, parsing with `"r"`, and fallback regex.
- Unit tests for impact & fallacy prompt formatting and parsing.
- Database tests in `crates/server/src/ai/db.rs` ensuring batch rewrites and on-demand analysis are correctly stored and retrieved.
- Detail route tests in `crates/server/src/routes/entries.rs` verifying:
  - Details page does NOT contain "Executive Brief".
  - Details page immediately contains the rewrite text on initial load.
  - `/hx/entries/:id/analysis` serves impact and fallacies.
