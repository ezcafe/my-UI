clean

# Design Review (Overall): AI Source Grouping, 3-Sentence Brief & UI Polish

**Stage:** `design-review`  
**Result:** `clean`  
**Date:** 2026-11-23  

---

## 1. Executive Summary

This comprehensive design review evaluated the technical architecture, user experience, error handling, performance, and accessibility specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-source-grouping-3-sentence-brief/03-design.md).

Both isolated prerequisite reviews have completed clean:
- **API Contract Review** ([03a-api-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-source-grouping-3-sentence-brief/03a-api-review.md)): `clean`
- **Database Design Review** ([03a-db-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-source-grouping-3-sentence-brief/03a-db-review.md)): `clean`

**Overall Verdict: `clean`** (Zero Critical, zero Major blockers). The proposed architecture is well-factored, delivers significant latency and token-cost reductions, maintains backward compatibility, and adheres to accessibility and UI styling principles.

---

## 2. Comprehensive Review by Domain

### 2.1 User Interface & Interaction Design
- **Expanded `.annuaire-row` Click Target**:
  - **Assessment**: Moving `onclick="openEntryDetail({{ entry.id }})"` to the outer `<div class="annuaire-row ...">` container eliminates dead zones on the vertical positional bar and whitespace padding.
  - **Event Isolation**: Retaining `onclick="event.stopPropagation(); markArticleRead({{ entry.id }})"` on the external link button `<a ...>` properly shields external navigation from unintentionally triggering the side drawer.
  - **Consistency**: Client-side article card generation in `index.html` must mirror the outer click handler placement to maintain parity between server-rendered and IndexedDB/client-rendered rows.
- **Pagination Navigation Styling**:
  - **Assessment**: Removing `border-t border-border` on `<nav ... aria-label="Pagination">` removes redundant visual separation, establishing a unified reading flow.
  - **Regression Safety**: Existing automated test assertions in `crates/ui-components/src/templates.rs` verify `aria-label="Pagination"` and navigation buttons, ensuring this styling change causes no test regressions.

### 2.2 System Architecture & Component Interaction
- **Hybrid Single-Pass Batch (`Option 1`)**:
  - **Assessment**: Sending all articles on the page in a structured source-grouped format (`### Source: ...`) sorted chronologically descending (`published_at` desc) gives the LLM complete context for executive page synthesis (`"top"`).
  - **Cache Discrimination**: Explicitly labeling cached entries as `[ID <id>] (Existing Brief)` and uncached entries as `[ID <id>] (New Article)` gives the LLM clear boundaries: synthesize `"top"` across all entries, but generate suggested titles (`"t"`) and briefs (`"b"`) strictly for new items.
  - **Data Flow Separation**: Cleanly divides responsibilities between GraphQL retrieval, cache partitioning, concurrent content extraction, LLM prompt generation, and SQLite persistence.

### 2.3 Performance & Token Efficiency
- **Prompt Token Reduction**: Reusing stored 3-sentence briefs (~40 tokens) instead of full article bodies (~300-500 tokens) for cached items cuts input prompt size by up to 60-80% on active reading pages.
- **Generation Token Savings**: Constraining item generation to uncached IDs eliminates redundant generation tokens, speeding up LLM time-to-first-token (TTFT) and total generation time by up to 90%.
- **Single-Roundtrip Latency**: Avoids the 2.5s+ latency penalty of two-pass architectures by completing page synthesis and item generation in exactly 1 LLM request.
- **Zero Extractor Waste**: Uncached entries only trigger `extract_content` when missing from SQLite, preventing wasteful background HTTP fetches and HTML parsing.

### 2.4 Error Handling & System Resilience
- **LLM Timeout & Degradation**:
  - If the LLM call times out or fails, fallback to 3-sentence extractive summarization ensures users still receive an executive summary.
  - If parsing fails completely, returning `<div id="page-executive-brief" class="hidden"></div>` with HTTP 200 OK guarantees HTMX swap stability without throwing client-side JavaScript or CSS layout errors.
- **Defensive Server Filtering**:
  - The server verifies returned item IDs against the uncached article ID set before updating SQLite, preventing hallucinations or unwanted overwrites of previously cached briefs.
- **GraphQL Field Robustness**:
  - Handles missing or null `feed` (`feed: None`) by defaulting to `"Unknown Source"`.
  - Handles missing or unparseable `publishedAt` by defaulting to epoch fallback during chronological sorting.

### 2.5 Accessibility (a11y) & Semantic Markup
- **Keyboard & Screen Reader Support**:
  - The outer row element `.annuaire-row` should include `role="button"` and `tabindex="0"` with an `onkeydown` handler (Enter / Space) to enable full keyboard accessibility for non-pointer users.
  - `title="{{ entry.title }}"` on `#entry-title-{{ entry.id }}` provides accessible tooltips for truncated or rewritten titles.
  - `aria-label="Pagination"` is preserved on the pagination `<nav>`.

---

## 3. Findings & Recommendations Summary

| # | Domain | Finding | Severity | Recommendation |
|---|--------|---------|----------|----------------|
| 1 | Accessibility | `.annuaire-row` keyboard navigation | Minor (Advisory) | Add `tabindex="0"` and keyboard activation (Enter/Space) to `.annuaire-row` |
| 2 | Prompting | Empty items when 100% cached | Minor (Advisory) | Explicitly instruct LLM that `"items"` can be empty `[]` when all entries on the page are `[Existing Brief]` |
| 3 | Robustness | Fallback sentence segmentation | Minor (Advisory) | Use regex/sentence tokenizer in extractive fallback to guarantee strictly 3 sentences are extracted |

**Blockers:** 0  
**Critical:** 0  
**Major:** 0  
**Minor:** 3 (all advisory / non-blocking)

---

## 4. Conclusion & Next Step

The architecture in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-source-grouping-3-sentence-brief/03-design.md) satisfies all functional requirements, architectural invariants, performance targets, and safety standards.

**Final Result:** `clean`. Design review is complete. Ready for parent to proceed with Gate B approval and TDD task execution.
