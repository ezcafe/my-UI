# System Design: AI Source Grouping, 3-Sentence Brief & UI Polish

## 1. Executive Summary & Core Milestones

This design enhances AI executive briefings and reading UX across five key pillars:
- **Source Grouping & Chronological Order**: Groups unread articles by feed title and sorts them by publish date descending (latest first) before sending them to the LLM.
- **Intelligent Summary Reuse**: Identifies articles already summarized in SQLite (`article_ai_summaries`) and reuses their briefs rather than requesting new LLM generations.
- **Full-Page Synthesis**: Feeds both existing cached summaries and uncached article content into the LLM prompt so the top executive brief (`"top"`) accurately synthesizes the entire page.
- **Strict 3-Sentence Brief Length**: Updates prompt directives across single and batch summaries to enforce strictly 3 concise sentences.
- **UI Interaction & Styling Polish**: Expands click targets so clicking anywhere on `.annuaire-row` opens the detail drawer, and removes the top border from the pagination navigation bar.

---

## 2. Design Options & Trade-Offs

### Option 1: Single-Pass Hybrid Batch with Source Clusters & Existing Brief Re-Use (Recommended)

- **Explanation**: A single unified LLM prompt presents all articles on the page, grouped under their feed titles and sorted chronologically (latest first). Cached articles are marked as `[Existing Brief]` with their stored title and 3-sentence summary, while uncached articles are marked as `[New Article]` with their title and extracted content. The prompt directs the LLM to:
  1. Synthesize `"top"` across the entire page (all sources, all articles).
  2. Output `"items"` containing suggested titles (`"t"`) and 3-sentence briefs (`"b"`) strictly for the `[New Article]` IDs.
- **Concrete Example**:
  ```text
  You are an executive news editor. For the articles below, provide:
  1. "top": A concise executive brief synthesizing the most important themes across all sources on this page.
  2. "items": For each NEW article id, provide a punchy "t" (suggested title, <=10 words) and "b" (executive brief, strictly 3 sentences, what changed/why it matters). Do not regenerate items for existing briefs.

  Articles by Source:

  ### Source: Rust Blog
  [ID 101] (New Article) Title: Rust 1.85 Released
  Content: Rust 1.85 is now available with async closures...

  [ID 102] (Existing Brief) Title: Cargo Workspace Improvements
  Brief: Cargo now supports faster resolver caching. Builds finish 25% faster. Developers can enable it via config.

  ### Source: Hacker News
  [ID 103] (New Article) Title: SQLite 3.48 Features
  Content: SQLite announced new query planner optimizers...
  ```
- **Pros**:
  - Exactly 1 LLM request per page view.
  - Zero redundant token generation for already summarized articles.
  - Full-page synthesis takes all articles into account without information gaps.
  - Sub-second processing latency for partially cached pages.
- **Cons**:
  - Requires prompt parsing to handle mixed article states (reference vs generation targets).

### Option 2: Two-Pass Split Execution (New Item Generation then Top Synthesis)

- **Explanation**: Pass 1 invokes the LLM on only the uncached items to generate their suggested titles and briefs. Pass 2 collects all briefs (old + new) and issues a second LLM request to synthesize `"top"`.
- **Concrete Example**:
  ```rust
  let new_items = llm.summarize_batch(&uncached_articles).await;
  let all_summaries = combine(cached_summaries, new_items);
  let top_brief = llm.synthesize_page_brief(&all_summaries).await;
  ```
- **Pros**: Cleaner separation between item brief generation and overall page synthesis.
- **Cons**:
  - Two sequential LLM calls per page view (doubling latency from ~1.2s to ~2.5s+).
  - Double billing / token consumption for external API providers.
  - Increased failure surface if either call times out.

> **Recommendation**: **Option 1**. Single-pass hybrid batch achieves 100% full-page fidelity with lowest latency and minimal token usage.

---

## 3. Detailed Component Architecture

### 3.1 Prompt Formatting & Grouping (`crates/server/src/ai/llm.rs`)

1. **Structured Source Item Definition**:
   ```rust
   pub struct BatchArticleInput<'a> {
       pub id: i64,
       pub title: &'a str,
       pub source: &'a str,
       pub published_at: &'a str,
       pub content_or_brief: &'a str,
       pub is_cached: bool,
   }
   ```
2. **Grouping & Sorting**:
   - Collect items by `source`.
   - Sort groups or keep deterministic order.
   - Within each source group, sort items by `published_at` descending (latest first).
3. **Prompt Directives**:
   - Explicit rule: `"b" (executive brief, strictly 3 sentences, what changed/context/why it matters)`.
   - Single article prompt directive: `Summarize what changed or why this news matters in strictly 3 concise sentences. Do not include introductory filler.`
   - Multilingual rule: Preserve source article language per item and in overall synthesis.

### 3.2 Route Endpoint Orchestration (`crates/server/src/routes/entries.rs`)

1. **GraphQL Fetch**:
   - Query `publishedAt` and `feed { title }` alongside `id`, `title`, `summary`, `url`.
2. **Partitioning**:
   - Fetch existing summaries via `ai_service.db.get_batch_summaries(&entry_ids).await`.
   - If an entry has a summary with `model != "extractive-fallback"`, mark as cached.
   - If uncached, spawn concurrent content extraction via `ai_service.extractor.extract_content`.
3. **Execution**:
   - If all cached and page brief exists in SQLite: return cached card + OOB titles immediately (0ms LLM).
   - If all cached and page brief missing: pass cached summaries to LLM for `"top"` synthesis.
   - If mixed or all uncached: pass source-grouped batch input to `summarize_batch`.
   - Save new article summaries into `article_ai_summaries`.
   - Save `"top"` into `page_executive_briefs_v2` under `page_key`.
   - Return rendered `ExecutiveBriefCardTemplate` and OOB title updates.

### 3.3 UI Click Interactions & Styling (`crates/ui-components/`)

1. **`.annuaire-row` Click Target**:
   - In `entry_item.html`, `entry_list.html`, and `index.html`:
     - Move `onclick="openEntryDetail({{ entry.id }})"` to the outer `<div class="annuaire-row ..."` element.
     - Remove `onclick` from the inner child `<div>` to avoid nested handlers.
     - Retain `onclick="event.stopPropagation(); markArticleRead({{ entry.id }})"` on the external link button.
2. **Pagination Top Border Removal**:
   - In `entry_list.html`:
     - Change `<nav class="flex items-center justify-between pt-4 pb-2 border-t border-border ...">`
     - To `<nav class="flex items-center justify-between pt-4 pb-2 text-xs text-muted-foreground tabular font-sans" aria-label="Pagination">`.

---

## 4. Verification & Testing Strategy

1. **Unit Tests (`crates/server/src/ai/llm.rs`)**:
   - Verify prompt construction with source headers (`### Source: ...`) and descending date sorting.
   - Verify prompt explicitly specifies strictly 3 sentences.
   - Verify extractive fallback extracts 3 sentences.
2. **Route Integration Tests (`crates/server/src/routes/entries.rs` & `tests/e2e_tests.rs`)**:
   - Test cache reuse: existing article summaries in SQLite are not overwritten or re-requested.
   - Test full-page synthesis: page brief is generated and stored with `page_key`.
3. **Template Rendering Tests (`crates/ui-components/src/templates.rs`)**:
   - Verify `.annuaire-row` contains `onclick="openEntryDetail(...)"`.
   - Verify pagination nav lacks `border-t border-border`.

## Result

Result: done
