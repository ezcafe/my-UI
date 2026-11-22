# System Analysis: Batch Page AI Brief & Multilingual Executive Briefs

## 1. Context & Objectives

This feature streamlines AI operations in My News:
- **Multilingual Fidelity**: The batch prompt strictly instructs the LLM to output suggested titles and briefs in the source article's language (e.g. Vietnamese articles stay in Vietnamese; English articles stay in English).
- **Single Batch Call per Page**: A single prompt evaluates all visible articles on the current list page (e.g. 15 items on page 2), returning a page synthesis (`"top"`) and per-article suggested titles (`"t"`) and briefs (`"b"`).
- **Loading Skeleton UX**: While the batch is generating, render an Annuaire-styled skeleton card with animated pulsing. Initial page load remains sub-50ms.
- **Dynamic OOB Swap**: When the batch returns, the skeleton is swapped with the Executive Brief, and the visible list row titles are instantly updated via HTMX Out-Of-Band swaps (`hx-swap-oob="outerHTML"`).
- **Unread Filter Scoping**: The Executive Brief card and skeleton appear strictly on the `unread` feed. On `today`, `read`, `starred`, `all`, and search, the card is omitted entirely.
- **Zero-LLM Detail View**: Opening the detail drawer reads directly from `AiDb`. If uncached, it falls back to the extractive summary/teaser without spawning an external LLM process.

---

## 2. Technical & Architectural Breakdown

### A. Multilingual Prompt Design (`crates/server/src/ai/llm.rs`)

To ensure zero language drift:
```text
CRITICAL MULTILINGUAL RULE:
For each article and for the overall brief, write your response in the EXACT SAME LANGUAGE as the source text.
If an article is in Vietnamese, its suggested title and brief MUST be in Vietnamese.
If an article is in English, its suggested title and brief MUST be in English.
Do NOT translate articles into a different language.
```

The output JSON structure:
```json
{
  "top": "Page-level synthesis strictly in the dominant language of the page...",
  "items": [
    { "i": 101, "t": "Tiêu đề súc tích tiếng Việt", "b": "Tóm tắt ngắn gọn tiếng Việt." },
    { "i": 102, "t": "Punchy English Title", "b": "Concise English executive brief." }
  ]
}
```

### B. Database Schema Evolution (`AiDb` v3)

Add `page_executive_briefs_v2` table:
```sql
CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  page_key TEXT NOT NULL UNIQUE,
  brief TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_page_briefs_v2_key ON page_executive_briefs_v2(page_key);
PRAGMA user_version = 3;
```
- Upgrade path: On initialization, if `user_version < 3`, create `page_executive_briefs_v2` and set `user_version = 3`.
- `page_key`: Constructed from query parameters and offset (e.g., `unread:offset_0:limit_15` or fingerprint of entry IDs).

### C. HTMX Lazy-Loading & Out-Of-Band Swaps

1. **Initial List Rendering (`render_entries_list`)**:
   - If filter is `unread` and AI is enabled:
     - Check `AiDb` for cached page brief with `page_key`.
     - If cached: render the completed `#page-executive-brief` card directly.
     - If not cached: render the `#page-executive-brief` skeleton card:
       ```html
       <div id="page-executive-brief"
            hx-get="/hx/executive-brief?offset=0&limit=15"
            hx-trigger="load"
            hx-swap="outerHTML"
            class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4 animate-pulse">
         ...
       </div>
       ```
2. **Lazy Endpoint (`/hx/executive-brief`)**:
   - Fetches the entries on that page.
   - Runs the batch LLM call (or reads cached if generated concurrently).
   - Returns the rendered `#page-executive-brief` card HTML, plus trailing OOB elements:
     ```html
     <div id="page-executive-brief" class="...">
       ... brief content ...
     </div>
     <span id="entry-title-101" hx-swap-oob="outerHTML" class="...">New AI Title 1</span>
     <span id="entry-title-102" hx-swap-oob="outerHTML" class="...">New AI Title 2</span>
     ```
   - HTMX swaps `#page-executive-brief` into place and automatically updates the title elements on each article row.

### D. Zero-LLM Article Detail View

In `render_entry_detail`:
- Query `AiDb` for the entry's cached brief.
- If found: pass to `brief_card.html`.
- If not found: pass the article's extractive summary/teaser to `brief_card.html` without invoking `ai_service.get_or_create_summary(...)`. Zero CLI spawns.

### E. Filter Scoping

Update `is_brief_eligible_filter`:
```rust
pub fn is_brief_eligible_filter(status: Option<&str>, search: Option<&str>) -> bool {
    let has_search = search.is_some_and(|s| !s.trim().is_empty());
    if has_search {
        return false;
    }
    match status {
        Some("unread") => true,
        None => true, // default feed is unread
        _ => false,   // today, read, starred, all -> false
    }
}
```

---

## 3. Risks & Mitigations

1. **Duplicate In-Flight Requests**: If a user refreshes or multiple tabs request the same page brief simultaneously, both might trigger LLM calls.
   - *Mitigation*: Ensure batch worker or semaphore guards against duplicate runs, or use an in-flight job registry.
2. **LLM Output Deviations**: Model might output malformed JSON or markdown codeblocks.
   - *Mitigation*: Existing regex parser (`TOP_REGEX` and `ITEM_REGEX`) extracts fields even if surrounded by markdown or explanatory text.
3. **Empty / Disabled AI**:
   - *Mitigation*: If AI is disabled (`AI_ENABLED=false`) or fails, return empty response (removes skeleton cleanly) and leaves original titles intact.

---

## 4. Frontier Questions for Grill

1. **Page Key Identification & Cache Invalidation**:
   - Should `page_key` be based on `unread:offset:limit` (with a time-to-live e.g. 30 minutes), or a hash/fingerprint of the entry IDs currently on the page?
2. **Skeleton Shimmer & Visual Indicator**:
   - Should the loading skeleton card show a subtle pulsing placeholder with an explicit "Generating Executive Brief..." indicator, matching the Annuaire muted palette?

## Result

Result: done
