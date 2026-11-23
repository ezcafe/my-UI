# Codebase Skim: AI Source Grouping, 3-Sentence Brief & UI Polish

## Core Modules & Touchpoints

1. **AI Subsystem (`crates/server/src/ai/`)**:
   - `llm.rs`:
     - Update `format_batch_prompt` to accept structured source groups with date ordering (latest first), existing DB briefs, and uncached new items.
     - Update prompt instructions: `b` (brief) must be strictly 3 sentences.
     - Update `format_prompt` (single article): summary must be strictly 3 sentences.
     - Update `fallback_extractive` to extract 3 sentences via `miniflux_client::summary::extract_summary(content, 3)`.
     - Tests in `llm.rs`: update prompt formatting tests and mock responses.
   - `db.rs`:
     - Existing `get_batch_summaries` and `get_unprocessed_entry_ids` provide efficient retrieval of cached article summaries.
     - Ensure existing summaries are reused directly without re-querying the LLM.
   - `prefetch.rs`:
     - Align background prefetch worker to group unread items by source and sort by date latest first, passing existing briefs and new articles.

2. **Routes & Endpoints (`crates/server/src/routes/entries.rs`)**:
   - `executive_brief_htmx` (`GET /hx/executive-brief`):
     - Expand GraphQL query to fetch `publishedAt` and `feed { title }`.
     - Group unread entries by source (`feed.title`), sort within each group by date (latest first).
     - Separate entries into cached (already in SQLite `article_ai_summaries`) and uncached (new items).
     - Send the prompt with source grouping, reusing existing briefs and requesting LLM only for new items.
     - Synthesize the page-level executive brief (`top`) from all items on the page (both cached briefs and new articles).
     - Save new summaries and the synthesized page brief in `AiDb`.

3. **Templates & UI Components (`crates/ui-components/`)**:
   - `templates/partials/entry_item.html`:
     - Move `onclick="openEntryDetail({{ entry.id }})"` to the outer `.annuaire-row` element so clicking whitespace around the title opens details.
   - `templates/partials/entry_list.html`:
     - Move `onclick="openEntryDetail({{ entry.id }})"` to the outer `.annuaire-row` element.
     - Remove `border-t border-border` on pagination `<nav>`.
   - `templates/pages/index.html`:
     - Move `onclick="openEntryDetail(${art.id})"` to the outer `.annuaire-row` container in client-side guest rendering.

4. **Testing Infrastructure**:
   - `crates/server/src/ai/llm.rs`: Unit tests for prompt instructions (strictly 3 sentences), source grouping, and parsing.
   - `crates/server/tests/e2e_tests.rs`: Integration tests for `/hx/executive-brief` with source-grouped prompts and cache reuse.
   - `crates/ui-components/src/templates.rs`: Tests for template rendering verifying row click markup and borderless pagination nav.

## Result

Result: done
