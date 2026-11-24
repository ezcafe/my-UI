# Analysis: Batch Rewrite & Detail Impact/Fallacy Analysis

## 1. Context & Objectives

Currently:
1. In the batch LLM request (`summarize_batch_structured`), each item in the batch returns a 1-sentence title (`t`) and a 3-sentence brief (`b`).
2. When the user opens an article detail view, an "Executive Brief" card is rendered at the top of the body (`{% include "brief_card.html" %}`).
3. Underneath the brief card, an HTMX trigger makes a separate request to `/hx/entries/:id/rewrite` to generate the rewrite, impact analysis, and fallacy check together. This introduces a perceived delay on the detail page while waiting for the rewrite to appear.

Under the new requirements (confirmed Option A1 and Option B1):
1. **Batch Request Optimization**:
   - Replace the 3-sentence brief (`b`) with the inverted-pyramid rewrite text (`r`).
   - The batch JSON schema outputs: `{"top": "...", "items": [{"i": ..., "t": "...", "r": "..."}]}`.
   - Batch results are stored in `article_ai_summaries`, populating `rewrite` and `summary`.
2. **Details Page Streamlining**:
   - Remove "Executive Brief" card (`brief_card.html`) completely from `entry_detail.html`.
   - The `rewrite` text is loaded and rendered immediately on initial detail view render (0ms delay, instantaneous local read).
3. **Decoupled Impact & Fallacy Analysis**:
   - The detail page renders an HTMX container for analysis below the rewrite text.
   - If `impact` and `fallacies` are already cached in `article_ai_summaries`, render them immediately.
   - If not cached, the HTMX container asynchronously requests `/hx/entries/:id/analysis`, which generates and caches `impact` and `fallacies`, then swaps the cards in.

---

## 2. Technical Nuances & Architecture

### A. Batch Prompt & Parsing
- In `LlmCliRunner::format_batch_prompt_structured`:
  - Clearly instruct the model:
    `2. "items": For each NEW article id, provide a punchy "t" (suggested title, strictly in one sentence) and "r" (inverted pyramid rewrite: lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context).`
  - Compact schema: `{"top":"...","items":[{"i":101,"t":"...","r":"..."}]}`.
- In `LlmCliRunner::parse_batch_response`:
  - `BatchAiItem` struct:
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
    Using `#[serde(rename = "r", alias = "b")]` provides backwards compatibility if any legacy response formats still output `"b"`.
  - Fallback regex `ITEM_REGEX`:
    Update to match either `"r"` or `"b"`.

### B. Database Persistence
- `article_ai_summaries` table schema:
  - `summary TEXT NOT NULL`: We save `item.rewrite` into `summary`.
  - `rewrite TEXT`: We save `item.rewrite` into `rewrite`.
  - `impact TEXT`: Nullable; populated when analysis completes.
  - `fallacies TEXT`: Nullable; populated when analysis completes.
- In `save_batch_summaries`:
  ```rust
  ArticleSummaryInput {
      entry_id: item.id,
      summary: item.rewrite.clone(),
      suggested_title: if item.suggested_title.is_empty() { None } else { Some(item.suggested_title.clone()) },
      rewrite: Some(item.rewrite.clone()),
      impact: None,
      fallacies: None,
      model: model.clone(),
  }
  ```

### C. Analysis Prompt & Endpoint
- We introduce `format_impact_and_fallacies_prompt(title: &str, content_or_rewrite: &str) -> String`:
  - Requests strictly `"impact"` and `"fallacies"` in JSON schema:
    ```json
    {
      "impact": "Analyze the practical consequences and real-world impact...",
      "fallacies": "Identify informal logical fallacies or state 'Không phát hiện lỗi ngụy biện...'"
    }
    ```
- Endpoint `/hx/entries/:id/analysis`:
  - Checks if `impact` and `fallacies` already exist in DB. If so, renders `analysis_cards.html` immediately.
  - If not, invokes `ai_service.analyze_impact_and_fallacies(id, title, content)`.
  - Saves analysis to DB via `AiDb::save_analysis(entry_id, impact, fallacies)`.
  - Returns the rendered `analysis_cards.html` fragment.

### D. Detail Page Template
- `crates/ui-components/templates/partials/entry_detail.html`:
  - Delete `{% include "brief_card.html" %}`.
  - Render rewrite immediately:
    ```html
    <div id="ai-rewrite-box" class="space-y-6">
      <div class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
        {{ rewrite }}
      </div>
    </div>
    ```
  - Analysis section:
    ```html
    {% if has_analysis %}
      {% include "analysis_cards.html" %}
    {% else %}
      <div id="ai-analysis-box" class="animate-pulse space-y-4 my-4" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="load" hx-swap="outerHTML">
        <div class="rounded-lg p-4 bg-muted/30 border border-border space-y-2">
          <div class="h-4 bg-muted rounded w-1/4"></div>
          <div class="h-3 bg-muted rounded w-3/4"></div>
        </div>
        <div class="rounded-lg p-4 bg-muted/30 border border-border space-y-2">
          <div class="h-4 bg-muted rounded w-1/3"></div>
          <div class="h-3 bg-muted rounded w-5/6"></div>
        </div>
      </div>
    {% endif %}
    ```

---

## 3. Risks & Mitigations

1. **Risk: Batch response output length increase**:
   - Writing an inverted-pyramid rewrite (~50-80 words) takes slightly longer than a 3-sentence brief (~40 words).
   - *Mitigation*: The CLI and HTTP timeouts are 90s, with semaphore-controlled concurrency. Token limit is well within typical 4k output limits.
2. **Risk: Missing rewrite on direct detail page open**:
   - If an article detail page is opened before the batch pre-request finishes, `rewrite` in DB might be empty.
   - *Mitigation*: Fall back to the article's teaser/summary or content if `rewrite` is not yet available, ensuring the view never renders blank.
3. **Risk: Duplicate or overlapping HTMX requests**:
   - In-flight deduplication in `AiService` ensures duplicate concurrent requests for the same entry do not cause multiple LLM calls.

## Result

Result: done
