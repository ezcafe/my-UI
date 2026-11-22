# System Design: Batch Page AI Brief & Multilingual Executive Briefs

## 1. Executive Summary & Core Milestones

This design optimizes AI usage, eliminates UI layout shifts, and ensures accurate multilingual summaries in My News.

- **Multilingual fidelity**: Prompts strictly force the LLM to write in each article's native source language without cross-translation.
- **Single batch call per page**: Summarizes all visible articles and generates a page-level executive brief in exactly 1 LLM CLI invocation.
- **Entry-fingerprint caching**: Caches page briefs by hashing visible entry IDs (`unread:sha256(...)`), preventing stale summaries when new feed items arrive.
- **Instant OOB updates**: Renders an Annuaire pulsing skeleton on page load, which lazily swaps via HTMX with the real brief and live row titles via `hx-swap-oob="outerHTML"`.
- **Graceful skeleton failure collapse**: Returns a 200 OK with an empty/hidden element on LLM timeout or error so the skeleton collapses cleanly rather than pulsing indefinitely.
- **Decoupled AI & auth**: Enables AI capabilities when `AI_ENABLED=true` regardless of whether `AUTH_ENABLED` is true or false.
- **Server prefetch & zero-LLM drawer**: Automatically batches un-summarized items when a page threshold is met, while the article detail drawer reads strictly from cache (fallback to teaser, 0 CLI spawns).

---

## 2. Design Options & Trade-Offs

### Option 1: Lazy HTMX Swap with OOB Title Injection (Recommended)

- **Explanation**: The initial HTML response returns in <50ms with original titles and an Annuaire pulsing skeleton card for the brief. The skeleton card fires an HTMX request forwarding all active query parameters (`hx-get="/hx/executive-brief?offset=0&limit=15&status=unread&feed_id=12"`, `hx-trigger="load"`, `hx-swap="outerHTML"`). On the server, the endpoint executes 1 batch LLM call for uncached items, saves results to `AiDb` in a single transaction, and responds with the completed brief card accompanied by Out-Of-Band (`hx-swap-oob`) elements for each article title. If the LLM times out or errors, it returns 200 OK with an empty container, cleanly removing the skeleton.
- **Concrete Example**:
  ```html
  <!-- Initial Page Load (sub-50ms) -->
  <div id="page-executive-brief"
       hx-get="/hx/executive-brief?offset=0&limit=15&status=unread&feed_id=12"
       hx-trigger="load"
       hx-swap="outerHTML"
       class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4 animate-pulse">
    <div class="flex items-center gap-2 mb-2">
      <i data-lucide="sparkles" class="w-4 h-4 text-brand/60"></i>
      <span class="text-xs font-bold uppercase text-muted-foreground font-sans">Generating Executive Brief...</span>
    </div>
    <div class="space-y-1.5">
      <div class="h-2.5 bg-muted rounded w-5/6"></div>
      <div class="h-2.5 bg-muted rounded w-4/6"></div>
    </div>
  </div>

  <!-- Lazy Response from /hx/executive-brief (Success) -->
  <div id="page-executive-brief" class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4">
    <div class="flex items-center gap-2 mb-1.5">
      <div class="p-1 rounded bg-muted text-foreground">
        <i data-lucide="sparkles" class="w-4 h-4 text-brand"></i>
      </div>
      <span class="text-xs font-bold uppercase tracking-wider text-brand font-sans">Executive Brief</span>
    </div>
    <p class="text-xs text-foreground/90 leading-relaxed font-sans">Overview synthesis of page...</p>
  </div>
  <span id="entry-title-101" hx-swap-oob="outerHTML" class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors" title="Original Headline 1">Punchy AI Title 1</span>
  <span id="entry-title-102" hx-swap-oob="outerHTML" class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors" title="Original Headline 2">Punchy AI Title 2</span>

  <!-- Lazy Response on Timeout/Error (Graceful Collapse) -->
  <div id="page-executive-brief" class="hidden"></div>
  ```
- **Pros**:
  - Instant navigation (<50ms initial paint).
  - Zero layout shift thanks to identically sized skeleton placeholder.
  - Updates all article row titles atomically in one network trip.
  - Clean error recovery: skeleton collapses on failure without pulsing forever or throwing client errors.
  - No client-side JavaScript polling or WebSocket complexity needed.
- **Cons**:
  - Titles update in place 1–2 seconds after initial render.

---

### Option 2: Blocking Server-Side Batch Synthesis

- **Explanation**: The server intercepts the list route, identifies uncached items, blocks the HTTP request while running the batch LLM CLI process, and returns the fully rendered HTML with AI titles and brief already populated.
- **Concrete Example**:
  ```rust
  // Inside render_entries_list
  if config.ai_enabled && is_eligible {
      let unbatched = get_unbatched_items(&entries);
      if !unbatched.is_empty() {
          ai_service.run_batch_and_wait(&unbatched).await;
      }
  }
  // Then render complete HTML
  ```
- **Pros**:
  - HTML arrives with all AI titles and brief already present.
  - No OOB swaps or client-side lazy triggers needed.
- **Cons**:
  - Slow initial response time (TTFB jumps from 30ms to 2500ms+ on cache misses).
  - Creates poor user experience during feed browsing.
  - Blocks server worker threads waiting on LLM CLI execution.

### Recommendation

**Option 1 is strongly recommended**. It guarantees sub-50ms page load times, satisfies the Annuaire design system with smooth skeleton transitions, and prevents jarring UI shifts.

---

## 3. Detailed Architecture & Technical Specifications

### A. Authentication & AI Decoupling

In `crates/server/src/config.rs`:
- Remove coupling: `ai_enabled` must not require `auth_enabled`.
- Parse `AI_ENABLED` directly from environment/configuration:
  ```rust
  let ai_enabled = map
      .get("AI_ENABLED")
      .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
      .unwrap_or(false);
  ```
- Update unit tests in `crates/server/src/config.rs` to verify matrix permutations where `auth_enabled == false` and `ai_enabled == true` successfully enables AI.

### B. Database Schema Evolution (`AiDb` v3)

In `crates/server/src/ai/db.rs`:
1. Bump `PRAGMA user_version = 3`.
2. Add table `page_executive_briefs_v2` and index on `created_at`:
   ```sql
   CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     page_key TEXT NOT NULL UNIQUE,
     brief TEXT NOT NULL,
     created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
   );
   CREATE INDEX IF NOT EXISTS idx_page_briefs_v2_created_at ON page_executive_briefs_v2(created_at);
   ```
   *(Note: The explicit secondary index on `page_key` is intentionally omitted because `page_key TEXT NOT NULL UNIQUE` creates an automatic unique index in SQLite).*

3. Methods added to `AiDb`:
   - `get_page_brief_by_key(&self, page_key: &str) -> Option<String>`: Fetches brief for the given `page_key`.
   - `save_page_brief_by_key(&self, page_key: &str, brief: &str) -> Result<(), AiDbError>`:
     Uses upsert semantics to prevent `SQLITE_CONSTRAINT_UNIQUE` errors during concurrent reloads:
     ```sql
     INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
     VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
     ON CONFLICT(page_key) DO UPDATE SET
       brief = excluded.brief,
       created_at = excluded.created_at;
     ```
   - `get_unprocessed_entry_ids(&self, entry_ids: &[i64]) -> Vec<i64>`:
     Performs a single batched query using `WHERE entry_id IN (...)` to identify entries lacking LLM processing (`model == 'extractive-fallback'` or missing). Eliminates N+1 pool acquisitions.
   - `save_batch_results(&self, page_key: Option<(&str, &str)>, items: &[ArticleSummaryInput]) -> Result<(), AiDbError>`:
     Executes article summary updates and the optional page brief upsert inside a single SQLite transaction (`conn.transaction()?`). Batches fsync disk operations down from N+1 writes to 1 atomic write (<5ms).
   - `prune_page_briefs(&self, retention_days: i64) -> Result<usize, AiDbError>`:
     Housekeeping helper that deletes brief records older than `retention_days` (default: 14 days) using `idx_page_briefs_v2_created_at`, keeping table size bounded.

### C. Deterministic Fingerprint Key Calculation

To ensure page brief cache integrity across filter and feed variations:
- Function: `compute_page_key(status: &str, entry_ids: &[i64]) -> String`
- Implementation:
  ```rust
  use sha2::{Digest, Sha256};

  pub fn compute_page_key(status: &str, entry_ids: &[i64]) -> String {
      let mut hasher = Sha256::new();
      for id in entry_ids {
          hasher.update(id.to_le_bytes());
      }
      let hash = format!("{:x}", hasher.finalize());
      format!("{}:{}", status, &hash[..16])
  }
  ```
- If new articles arrive or the active feed/category changes visible IDs, a distinct key is generated, triggering a fresh and accurate brief.

### D. Multilingual Prompt Enforcement

In `crates/server/src/ai/llm.rs`:
The batch prompt template strictly isolates language per article:
```text
CRITICAL MULTILINGUAL RULE:
For each article and for the overall brief, write your response in the EXACT SAME LANGUAGE as the source text.
If an article is in Vietnamese, its suggested title and brief MUST be in Vietnamese.
If an article is in English, its suggested title and brief MUST be in English.
Do NOT translate articles into a different language.
```

JSON output contract:
```json
{
  "top": "Page synthesis in the dominant language of the page...",
  "items": [
    { "i": 101, "t": "Tiêu đề tiếng Việt", "b": "Tóm tắt tiếng Việt." },
    { "i": 102, "t": "English Title", "b": "English summary." }
  ]
}
```

Robust CLI Parsing: `LlmCliRunner::summarize_batch` defensively strips markdown code blocks (````json ... ````) and ignores preamble or trailing conversational text.

### E. Lazy Endpoint Contract: `/hx/executive-brief`

Route in `crates/server/src/routes/entries.rs`:
- **Path**: `GET /hx/executive-brief`
- **Query parameters**: Reuses `EntriesFilterQuery` directly:
  `status: Option<String>`, `feed_id: Option<i64>`, `category_id: Option<i64>`, `search: Option<String>`, `offset: Option<i64>`, `limit: Option<i64>`
- **Response Headers**:
  - `Content-Type: text/html; charset=utf-8`
  - `Cache-Control: no-cache, no-store, must-revalidate`
- **Execution Logic**:
  1. **Guard**: Check `is_brief_eligible_filter(status, search)` and `config.ai_enabled`. If ineligible, return 200 OK with empty body.
  2. **Query Visible Entries**: Query Miniflux/GraphQL cache using the exact filter parameters (`feed_id`, `category_id`, `offset`, `limit`, `status`). Guarantees 1:1 synchronization between row items and executive brief.
  3. **Empty Entries Guard**: If `entries.is_empty()` (e.g. all caught up, no matching items, or offset exceeds total entries), return `200 OK` with an empty response immediately without computing hashes or invoking LLM.
  4. **Compute Key**: Compute `page_key = compute_page_key(status, &entry_ids)`.
  5. **Cache Hit Fast-Path**: If `AiDb::get_page_brief_by_key(&page_key)` exists:
     - Render brief card + any missing cached titles and return immediately.
  6. **Identify Uncached Entries**: Call `AiDb::get_unprocessed_entry_ids(&entry_ids)` in one batched SQL query.
  7. **Thundering Herd Deduplication**: Use an in-flight processing lock or atomic key set in `AiService` so concurrent requests for the same `page_key` await the first synthesis rather than spawning duplicate CLI commands.
  8. **Concurrent Extraction**: Extract content for uncached entries via `ArticleExtractor`.
  9. **Batch LLM Invocation & Failure Contract**:
     - Call `ai_service.llm.summarize_batch(&articles)` with timeout protection.
     - **Failure Contract**: On timeout, process crash, or malformed JSON, return `200 OK` with `<div id="page-executive-brief" class="hidden"></div>` and zero OOB spans. The skeleton collapses gracefully without leaving a pulsing card or breaking page layout.
  10. **Atomic Transaction Write**: Save all article summaries and the page brief in a single SQLite transaction via `AiDb::save_batch_results(Some((&page_key, &brief)), &items)`.
  11. **Render Output**:
      - Brief card HTML: `<div id="page-executive-brief" class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4">...</div>`.
      - OOB spans for updated titles with HTML escaping to prevent XSS:
        ```html
        <span id="entry-title-{id}"
              hx-swap-oob="outerHTML"
              class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors"
              title="{escaped_original_title}">
          {escaped_suggested_title}
        </span>
        ```

### F. Server-Side Prefetch on Page Threshold

In `crates/server/src/ai/prefetch.rs`:
- **Trigger**: Called during list rendering when `config.ai_enabled` is true.
- **Threshold check**: Only trigger batch LLM prefetch when there are enough un-summarized items (e.g. `uncached.len() >= 5` or a full page size).
- **Exclusion**: Items already present in `article_ai_summaries` (`model != 'extractive-fallback'`) are never re-queued for LLM calls.
- **Concurrency guard**: An in-flight lock or atomic set ensures multiple concurrent requests for the same batch do not run duplicate LLM CLI processes.

### G. Zero-LLM Article Detail View

In `crates/server/src/routes/entries.rs` (`get_entry_detail_htmx`):
- Never invoke `ai_service.get_or_create_summary(...)`.
- Fetch `AiDb::get_summary_and_title(id)`.
- If cached summary exists, render in `brief_card.html`.
- If uncached, use the entry's `summary` (teaser/extractive snippet) as the brief card content. Zero CLI spawns.

### H. Filter Scoping & Reconciliation

- **Filter Scope**:
  - **Allowed**: `status == "unread"` or `status == None` with no search query.
  - **Prohibited**: `today`, `read`, `starred`, `all`, and any view with active `search`. In these views, the skeleton card and executive brief are omitted entirely.
- **Code Reconciliation**:
  - `is_brief_eligible_filter` in `crates/server/src/routes/entries.rs` is updated to remove `Some("today")`, strictly adhering to the prohibition rule:
    ```rust
    pub fn is_brief_eligible_filter(status: Option<&str>, search: Option<&str>) -> bool {
        let has_search = search.is_some_and(|s| !s.trim().is_empty());
        match status {
            Some("unread") => true,
            None if !has_search => true,
            _ => false,
        }
    }
    ```

---

## 4. Annuaire UI & Template Components

1. **Skeleton Placeholder (`crates/ui-components/templates/partials/executive_brief_skeleton.html`)**:
   - Matches Annuaire design language: `rounded-lg p-3.5 bg-muted/40 border border-border mb-4 animate-pulse`.
   - In `crates/ui-components/templates/partials/entry_list.html`, wrap the skeleton card in `{% if !entries.is_empty() && is_brief_eligible %}` so empty feeds or folders do not render an unnecessary skeleton or trigger redundant network requests.
   - Forwards active query parameters:
     ```html
     <div id="page-executive-brief"
          hx-get="/hx/executive-brief?offset={{ query.offset|default(0) }}&limit={{ query.limit|default(15) }}&status={{ query.status|default("unread") }}{% if let Some(fid) = query.feed_id %}&feed_id={{ fid }}{% endif %}{% if let Some(cid) = query.category_id %}&category_id={{ cid }}{% endif %}"
          hx-trigger="load"
          hx-swap="outerHTML"
          class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4 animate-pulse">
     ```
   - Uses `data-lucide="sparkles"` with `text-brand/60`.
   - On error response (`<div id="page-executive-brief" class="hidden"></div>`), HTMX swaps the outerHTML, collapsing the placeholder instantly.
2. **Title Row Target (`crates/ui-components/templates/partials/entry_item.html`)**:
   - Wrap the article title in `<span id="entry-title-{{ entry.id }}" class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors" {% if let Some(suggested) = entry.suggested_title %}title="{{ entry.title }}"{% endif %}>{{ display_title }}</span>`.
   - Allows HTMX OOB swap to target the title without re-rendering the whole row, while preserving Annuaire typography (`font-sans`), line clamping (`line-clamp-2`), and tooltip hover for original headlines (`title="{escaped_original_title}"`).
3. **Completed Brief Card (`crates/ui-components/templates/partials/executive_brief_card.html`)**:
   - Matches skeleton placeholder padding exactly (`p-3.5`) to eliminate micro layout shifts: `rounded-lg p-3.5 bg-muted/40 border border-border mb-4`.
   - Clean Annuaire card styling with prominent title and badge.

---

## 5. Verifiability & Reference Sources

- **HTMX Out-Of-Band Swaps**: [https://htmx.org/attributes/hx-swap-oob/](https://htmx.org/attributes/hx-swap-oob/)
- **SQLite Upsert Clause**: [https://www.sqlite.org/lang_UPSERT.html](https://www.sqlite.org/lang_UPSERT.html)
- **SQLite user_version pragma**: [https://www.sqlite.org/pragma.html#pragma_user_version](https://www.sqlite.org/pragma.html#pragma_user_version)
- **Annuaire Design Tokens**: `crates/ui-components/templates/base.html`
- **Repo Config & DB Modules**: [crates/server/src/config.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/config.rs), [crates/server/src/ai/db.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/db.rs), [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)

---

## Result

Result: done
