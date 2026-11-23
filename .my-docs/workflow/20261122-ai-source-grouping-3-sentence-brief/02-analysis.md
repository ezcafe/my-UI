# System Analysis: Source-Grouped 3-Sentence AI Briefs & UI Interactions

## 1. Context & Objectives

This change enhances the AI executive briefing and reading UX in My News across three dimensions:
- **Source Grouping & Chronological Ordering**: Before dispatching to the LLM in `/hx/executive-brief` and prefetch, unread items must be grouped by source (`feed.title`) and sorted by publish date descending (latest first).
- **Intelligent Cache Reuse & Full-Page Synthesis**:
  - Only new items without summaries are requested for generation from the LLM.
  - Items with existing summaries in SQLite (`article_ai_summaries`) reuse their stored brief.
  - The page-level executive brief (`"top"`) synthesizes the entire page's items, taking both existing briefs and new articles into account.
- **Strict 3-Sentence Brief Length**:
  - Executive briefs across both list items and the detail drawer must strictly follow a 3-sentence constraint.
- **UI Interaction & Styling Polish**:
  - The entire `.annuaire-row` container (whitespace around the title, badges, and trailing chevron) becomes clickable to open the detail view.
  - The pagination bar loses its top border (`border-t border-border`) for cleaner visual flow.

---

## 2. Technical & Architectural Breakdown

### A. Source Grouping & Date Sorting (`crates/server/src/routes/entries.rs` & `crates/server/src/ai/`)

1. **Data Retrieval**:
   - The GraphQL query in `/hx/executive-brief` is updated:
     ```graphql
     query {
       entriesResult(filter: { ... }) {
         entries {
           id
           title
           summary
           url
           publishedAt
           feed {
             title
           }
         }
       }
     }
     ```
2. **Grouping & Sorting**:
   - Items are grouped by feed title: `feed.title` (or `"Unknown Source"` if empty).
   - Within each feed group, articles are sorted by `publishedAt` descending (latest first).
   - The prompt presents items grouped under feed headings:
     ```text
     ### Source: TechCrunch
     [ID 101] (New Article) Title: AI Breakthrough
     Content:
     ...
     [ID 102] (Existing Brief) Title: Market Shifts
     Brief: First sentence. Second sentence. Third sentence.
     ```

### B. Cache Partitioning & Full-Page Synthesis

1. **Partitioning**:
   - Given page `entry_ids`, query `AiDb::get_batch_summaries(&entry_ids)`.
   - Cached entries: record exists in DB with `model != "extractive-fallback"` and non-empty `summary`.
   - Uncached entries: absent from DB or only have fallback summaries.
2. **LLM Dispatch Scenarios**:
   - **Scenario 1: All Cached**:
     - No new article summaries needed.
     - If page brief (`page_key`) is missing: pass all cached article titles + briefs to the LLM to synthesize the whole-page `"top"` brief.
   - **Scenario 2: All Uncached**:
     - Extract full contents for all items.
     - Pass all items to LLM to generate `"top"` and `"items"` (suggested title + 3-sentence brief).
   - **Scenario 3: Mixed (Some Cached, Some Uncached)**:
     - Extract contents ONLY for uncached items.
     - Construct prompt including both existing briefs (for context) and new articles (for generation).
     - Instruct LLM to:
       1. Synthesize `"top"` covering the entire page's items.
       2. Generate `"items"` ONLY for the new article IDs.
     - Save new summaries and the synthesized page brief to `AiDb`.

### C. Strict 3-Sentence Brief Length (`crates/server/src/ai/llm.rs`)

1. **Prompt Updates**:
   - `format_batch_prompt`:
     - Instruction: `"b": executive brief, strictly 3 sentences (what changed, context, why it matters).`
   - `format_prompt` (single article detail view):
     - Instruction: `Summarize what changed or why this news matters in strictly 3 concise sentences. Do not include introductory filler.`
2. **Extractive Fallbacks**:
   - Update fallback extraction calls in `llm.rs` from 2 sentences to 3 sentences (`miniflux_client::summary::extract_summary(content, 3)`).
3. **Tests**:
   - Update prompt assertions in `test_llm_cli_prompt_formatting_and_args` and `test_multilingual_batch_prompt_and_robust_parsing`.

### D. UI Click Target Expansion (`crates/ui-components/`)

1. **Row Click Behavior**:
   - In `entry_item.html` and `entry_list.html`, move `onclick="openEntryDetail({{ entry.id }})"` to the outer `.annuaire-row` container.
   - The inner external link button `<a ... onclick="event.stopPropagation(); markArticleRead(...)">` preserves stop propagation so clicking external link opens the URL in a new tab without opening details.
   - In `pages/index.html` (guest local mode), update the template literal similarly.

### E. Pagination Styling

- In `entry_list.html`:
  - Change `<nav class="flex items-center justify-between pt-4 pb-2 border-t border-border ...">`
  - To `<nav class="flex items-center justify-between pt-4 pb-2 ...">` (removing `border-t border-border`).

---

## 3. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| LLM returns items for cached IDs even when instructed not to | Server filters `batch_res.items` against the uncached ID set before inserting into DB, ensuring cached summaries are never overwritten with duplicates. |
| Mixed language feeds on the same page | Multilingual instructions remain strict: each item retains its source language, and the top brief synthesizes in the primary language of the page. |
| Nested click propagation causing duplicate actions | Explicit `event.stopPropagation()` on interactive child elements (`<a>`, bookmark buttons) ensures clean separation from row-level click. |

## Result

Result: done
