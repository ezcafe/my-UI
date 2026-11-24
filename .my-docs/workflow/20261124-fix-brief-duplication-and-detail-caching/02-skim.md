# Codebase Skim

**Result:** done

---

## 1. Relevant Code Paths and Locations

### Executive Brief Cards & Duplication
- **`crates/ui-components/templates/partials/executive_brief_card.html`**:
  Contains two top-level divs: `#page-executive-brief` and `#list-executive-brief hx-swap-oob="true"`. Both contain the exact same brief text.
- **`crates/ui-components/templates/partials/entry_list.html`**:
  Has `#list-executive-brief` in `#entries-list-view` (lines 11-34) and `#page-executive-brief` in `#entries-magazine-view` (lines 70-99).
- **`crates/server/src/routes/entries.rs` (`executive_brief_htmx`)**:
  Renders `ExecutiveBriefCardTemplate { brief: &brief }`. Returning both cards over HTTP doubles the payload and creates redundant DOM trees.

### Detail Panel Opening & Client Caching
- **`crates/ui-components/templates/pages/index.html` (`openEntryDetail`)**:
  - Line 630: `const isCacheValid = cached && cached.html && (!hasAiEnabled || cached.html.includes('Executive Brief'));`. Because detail articles no longer contain the literal string `'Executive Brief'`, this check always evaluates to false when `ai_enabled` is true.
  - Line 675 (`openDetailWithSkeleton`): Empties the detail panel and shows generic gray placeholders for title, brief, and content.
  - Existing DOM data: The list item has `#entry-title-{id}` with the exact article title or suggested title.

### Database Reuse for Analysis
- **`crates/server/src/routes/entries.rs` (`get_entry_analysis_htmx`)**:
  - Lines 1528–1536: Checks `ai_service.db.get_summary_and_title(id).await`. Requires both `impact` and `fallacies` to be `Some` and non-empty. If one is missing or empty, it proceeds to make a Miniflux GraphQL query and runs LLM extraction.
- **`crates/server/src/ai/mod.rs` (`analyze_entry_impact_and_fallacies`)**:
  - Does not check if analysis is already stored before calling `self.llm.analyze_impact_and_fallacies`.

### Prompt Guidelines for Rewrites
- **`crates/server/src/ai/llm.rs`**:
  - `format_batch_prompt_structured` (lines 83–96): Needs explicit constraints for under 110 words (30–60 words sweet spot), inverted pyramid structure (first 2 sentences lead with outcome and impacts), 2–3 concise bullet points, and plain language with no introductory/closing filler.
  - `format_rewrite_and_analyze_prompt` (lines 162–180): Update the single article prompt schema instructions to match these rewrite rules.
