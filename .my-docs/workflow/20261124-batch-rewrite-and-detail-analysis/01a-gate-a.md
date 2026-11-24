# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear and explicit.
  - In batch LLM request: replace the 3-sentence brief (`b`) with the inverted-pyramid rewrite text (`r`). Output schema: `{"top": "...", "items": [{"i": ..., "t": "...", "r": "..."}]}`.
  - In detail page: Remove the "Executive Brief" card partial entirely. Render the `rewrite` text immediately on initial load without any pulse or skeleton delay.
  - Lazy-load impact and fallacies: If not already present in `article_ai_summaries`, trigger an HTMX request to `/hx/entries/:id/analysis` which queries the LLM for impact analysis and fallacy & objectivity checks, persists the result, and swaps the cards into place.
  - User explicitly confirmed **Option A1** (replace `"b"` with `"r"` in batch schema) and **Option B1** (inline rewrite on load + dedicated `/analysis` endpoint for impact and fallacies).
- **Scope & Boundaries**:
  - Affects:
    1. `crates/server/src/ai/llm.rs`: Update `BatchAiItem` (`r` replaces `b`), update `format_batch_prompt_structured` to ask for inverted-pyramid rewrite `r` instead of brief `b`, update `parse_batch_response` (regex + JSON), and add `format_impact_and_fallacies_prompt` / parsing if needed.
    2. `crates/server/src/ai/mod.rs` & `db.rs`: Batch insertion stores `item.rewrite` into `rewrite` and `summary`; analysis update method to store impact and fallacies.
    3. `crates/server/src/routes/entries.rs`: Update batch processor mapping, update `get_entry_detail_htmx` to remove `brief_card` and supply `rewrite` on load, add/update `/hx/entries/:id/analysis` route.
    4. `crates/ui-components/`: Remove `brief_card.html` from `entry_detail.html`, inline `rewrite` on load, add analysis skeleton/placeholder and rendered fragment.
  - Non-goals:
    - No changes to page-level executive brief (`top`) displayed on the feed / magazine view.
    - No changes to Postgres database table schema (columns `rewrite`, `impact`, `fallacies` already exist).
- **Feasibility & Constraints**:
  - Database schema: `summary` is `NOT NULL` in `article_ai_summaries`. Storing rewrite into both `rewrite` and `summary` satisfies the constraint and keeps list views rich and informative.
  - Multilingual fidelity: Strict requirement that rewrite, impact, and fallacies must be in the same language as the source article text.
  - HTMX pattern: Standard `hx-get="/hx/entries/:id/analysis" hx-trigger="load" hx-swap="outerHTML"`.
- **Risks & Mitigations**:
  - LLM parsing issues with batch items: Comprehensive unit tests and regex fallbacks for `t` and `r`.
  - Detail view opening before batch runs: Fall back to teaser/summary gracefully, ensuring zero broken pages.

## Result

Result: ok
