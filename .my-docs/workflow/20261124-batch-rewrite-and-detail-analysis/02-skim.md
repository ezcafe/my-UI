# Codebase Skim: Batch Rewrite & Detail Impact/Fallacy Analysis

## Touchpoint Analysis

1. **LLM Engine & Prompts (`crates/server/src/ai/llm.rs`)**:
   - `BatchAiItem`:
     - Change field `#[serde(rename = "r")] pub rewrite: String` (replacing or aliasing `b`).
   - `format_batch_prompt_structured`:
     - Update prompt prompt text from:
       `2. "items": For each NEW article id, provide a punchy "t" (suggested title, strictly in one sentence) and "b" (executive brief, strictly 3 sentences, what changed/why it matters).`
       to:
       `2. "items": For each NEW article id, provide a punchy "t" (suggested title, strictly in one sentence) and "r" (inverted pyramid rewrite: lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context).`
     - Compact schema: `{"top":"...","items":[{"i":101,"t":"...","r":"..."}]}`.
   - `ITEM_REGEX`:
     - Update regex to capture `"i"`, `"t"`, and `"r"`.
   - `parse_batch_response`:
     - Parse `rewrite` directly from `"r"` (with fallback if `"b"` is present).
   - `format_impact_and_fallacies_prompt` & `parse_impact_and_fallacies_response`:
     - Focused prompt taking `title` and `content` (or `rewrite`), generating strictly `"impact"` and `"fallacies"`.
     - Standard fallacy normalization ("Không phát hiện lỗi ngụy biện..." / "No logical fallacies detected...").

2. **Database & Service Layer (`crates/server/src/ai/db.rs` & `crates/server/src/ai/mod.rs`)**:
   - `article_ai_summaries` table already has columns `rewrite`, `impact`, `fallacies`.
   - `save_batch_summaries`:
     - When saving batch items, store `item.rewrite` into both `rewrite` and `summary` (to fulfill `summary NOT NULL` and keep article summaries fresh).
   - `AiDb::update_analysis(entry_id, impact, fallacies)`:
     - Method to update `impact` and `fallacies` for an entry when background analysis returns.
   - `AiService::analyze_entry_impact_and_fallacies(entry_id, title, content)`:
     - Calls LLM with the focused impact/fallacies prompt and saves the result to `AiDb`.

3. **Routes (`crates/server/src/routes/entries.rs`)**:
   - `get_page_executive_brief_htmx`:
     - In batch processing loop, construct `ArticleSummaryInput` using `item.rewrite` for `summary` and `rewrite`.
   - `get_entry_detail_htmx`:
     - Fetch `AiSummaryRecord` from `AiDb`.
     - Remove `brief_card.html` inclusion.
     - Provide `rewrite` directly in `EntryDetailTemplate`.
     - Pass `has_analysis = !impact.is_empty() && !fallacies.is_empty()`.
   - `/hx/entries/:id/analysis`:
     - If `impact` and `fallacies` are cached in `AiDb`, return `render_analysis_fragment(impact, fallacies)`.
     - Otherwise, extract content, invoke `ai_service.analyze_entry_impact_and_fallacies(id, title, content)`, and return the analysis fragment.
     - Keep `/hx/entries/:id/rewrite` as an alias or redirect to `/hx/entries/:id/analysis` for backwards compatibility.

4. **Templates & UI (`crates/ui-components/src/templates.rs` & `entry_detail.html`)**:
   - `EntryDetailTemplate`:
     - Pass `rewrite`, `impact`, `fallacies`, and `has_analysis`.
   - `crates/ui-components/templates/partials/entry_detail.html`:
     - Remove `{% include "brief_card.html" %}`.
     - Render `rewrite` immediately on load:
       ```html
       <div id="ai-rewrite-box" class="space-y-6">
         <div class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
           {{ rewrite }}
         </div>
       </div>
       ```
     - Underneath, render analysis:
       ```html
       {% if has_analysis %}
         {% include "analysis_cards.html" %}
       {% else %}
         <div id="ai-analysis-box" class="animate-pulse space-y-4 my-4" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="load" hx-swap="outerHTML">
           <!-- Skeleton for Impact and Fallacies -->
         </div>
       {% endif %}
       ```

## Result

Result: done
