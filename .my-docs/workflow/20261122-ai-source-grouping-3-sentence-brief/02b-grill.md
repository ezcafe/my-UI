# Grill: AI Source Grouping, 3-Sentence Brief & UI Polish

## Settled Decisions & Architectural Alignment

### 1. Source Grouping & Chronological Ordering (Settled)
- **Decision**: Group unread items by source (`feed.title`), then sort by publish date (`published_at`) descending (latest first) before requesting the LLM.
- **Behavior**:
  - The GraphQL query in `/hx/executive-brief` retrieves `publishedAt` and `feed { title }`.
  - Articles are clustered under their respective source titles. Within each source cluster, articles are ordered latest first.
  - The generated prompt clearly reflects these source clusters so the LLM synthesizes articles with proper context and chronological awareness.

### 2. Selective LLM Request & Full-Page Synthesis (Settled)
- **Decision**: Only request LLM generation for uncached new items; reuse existing summaries from SQLite (`article_ai_summaries`); synthesize the top page brief across the entire page's items.
- **Behavior**:
  - Partition visible page entries into cached (already summarized in DB) and uncached (new).
  - Include both cached summaries (as reference briefs) and new articles (for generation) in the prompt.
  - Instruct the LLM to synthesize `"top"` covering all items across all sources on the page, while outputting `"items"` strictly for the new article IDs.
  - Re-save new summaries into `article_ai_summaries` and store the full page synthesis in `page_executive_briefs_v2`.
- **User Confirmation**: Explicitly confirmed in Ideation (`user: "entire page's items. Show me example when you finish"`).

### 3. Strict 3-Sentence Brief Length (Settled)
- **Decision**: Executive briefs across both list items and the detail drawer must strictly follow a 3-sentence length constraint.
- **Behavior**:
  - Update `format_batch_prompt` instruction for `"b"` to strictly 3 sentences.
  - Update `format_prompt` for single article summaries to strictly 3 sentences.
  - Update extractive fallback in `llm.rs` to extract 3 sentences (`miniflux_client::summary::extract_summary(content, 3)`).
  - Update test fixtures and assertions accordingly.

### 4. Row Click Interaction on `.annuaire-row` (Settled)
- **Decision**: Clicking anywhere on `.annuaire-row` (including whitespace, badge, and trailing chevron) triggers `openEntryDetail(id)`.
- **Behavior**:
  - Move `onclick="openEntryDetail({{ entry.id }})"` to the outer `.annuaire-row` container.
  - The external link button `<a ...>` retains `event.stopPropagation();` so opening the source link never triggers the detail view.

### 5. Borderless Pagination Navigation Bar (Settled)
- **Decision**: Remove the top border (`border-t border-border`) from the pagination `<nav>`.
- **Behavior**: The pagination bar flows seamlessly beneath the article list without a separating border line.

---

## Frontier Status

No open frontier decisions remain. All technical approaches and user requirements are fully settled.

## Result

Result: frontier-empty
