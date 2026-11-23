# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear and well-bounded. The user requested enhancements to the AI executive brief and UI interactions in My News:
  1. **Source Grouping**: In `/hx/executive-brief` and prefetch, group unread items by source (`feed_title`), sorting items within each group by date (latest first) before passing to the LLM.
  2. **Cache Reuse for Item Briefs**: Only request LLM generation for new items that lack summaries. For items that already have summaries in SQLite (`article_summaries`), reuse their existing brief.
  3. **Page-level Top Brief Synthesis**: The page-level executive brief ("top") synthesizes the entire page's items, combining reused existing briefs from DB with newly generated article briefs.
  4. **Strict 3-Sentence Brief Length**: The executive brief for items (shown in list and details pages) must strictly have 3 sentences. Update LLM prompt instructions and unit/integration tests accordingly.
  5. **Row Click Interaction**: On the list page, clicking anywhere on an item row (the whitespace around title in `.annuaire-row`) navigates to or opens the details page.
  6. **Paging Bar Styling**: Remove the top border on the pagination navigation bar (remove `border-t border-border`).

- **Scope & Boundaries**:
  - In scope:
    - `src/my_news/ai/executive.py` (prompt formatting, source grouping, DB summary reuse, top brief synthesis).
    - `src/my_news/ai/db.py` (summary retrieval & persistence).
    - `src/my_news/web/routes.py` (executive brief route & prefetch integration).
    - `src/my_news/templates/items.html` or `.annuaire-row` markup & CSS for clickable row whitespace.
    - `src/my_news/templates/pagination.html` or pagination nav container styling (removing `border-t border-border`).
    - Unit tests in `tests/test_ai_executive.py` and web route tests.
  - Out of scope:
    - Changes to third-party RSS scraping/fetching logic.
    - Schema changes that break backward compatibility (reuse existing `article_summaries` table).
    - Non-unread feeds (briefs remain scoped to unread).

- **Feasibility & Constraints**:
  - High feasibility. The existing architecture (`AiDb`, `LlmCliRunner`, Jinja2 templates, and Tailwind/CSS classes) already supports summary caching, prompt generation, and HTMX navigation.
  - Database table `article_summaries` already stores article summaries keyed by URL or article ID.
  - Annuaire styling can cleanly accommodate row-level click handling while preserving sub-element links (e.g., star button, tags).

- **Risks & Mitigations**:
  - *Risk*: LLM might not adhere strictly to the 3-sentence constraint.
    - *Mitigation*: Emphasize constraint explicitly in prompt system/user instructions with concrete formatting rules; sanitize or validate in tests.
  - *Risk*: Row click interfering with nested interactive buttons (like star or external link).
    - *Mitigation*: Ensure click handler ignores clicks originating on buttons or nested links (e.g. using event target checks or CSS pointer-events).
  - *Risk*: LLM hallucinating missing articles when synthesizing full page.
    - *Mitigation*: Pass both existing briefs and new articles clearly identified by source in the prompt so the LLM has all context needed for the top synthesis.

## Result

Result: ok
