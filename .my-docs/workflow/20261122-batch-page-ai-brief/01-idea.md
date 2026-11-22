# Ideation: Batch Page AI Brief & Multilingual Executive Briefs

## Executive Summary

This feature streamlines AI operations in My News by grouping all items on the current list page into a single batch LLM call, ensuring multilingual fidelity, showing responsive loading skeletons, and eliminating redundant AI requests on article detail views.

Key takeaways:
- **Language matching**: AI summaries and titles strictly match the language of the source article details (e.g., Vietnamese sources yield Vietnamese briefs; English sources yield English briefs).
- **Executive Brief skeleton**: Displays a clean, animated loading skeleton card while the batch AI request is processing, preventing layout shifts.
- **Unread filter only**: Executive Brief appears exclusively on the `unread` filter. It is hidden on `today`, `read`, `starred`, `all`, and search views.
- **Single batch call per page**: Exactly 1 LLM invocation processes all articles on the current page (e.g., page 2 of 3). It produces a 1–2 sentence page summary, 1–3 sentence briefs for each article, and compact suggested titles.
- **Zero-LLM detail views**: Opening an article detail view never spawns a new LLM call. It reuses the precomputed executive brief from the database, falling back instantly to the teaser if absent.

---

## Scope & Boundaries

### Goals
- **Multilingual prompts**: Enforce language consistency between source article text and generated titles/briefs.
- **Page-level batching**: Execute 1 prompt per page containing all visible items to minimize token overhead and process spawns.
- **Loading UX**: Render an Annuaire-compliant skeleton placeholder for the Executive Brief card while awaiting AI completion.
- **Filter scoping**: Show the page Executive Brief only when browsing the `unread` feed.
- **Database caching**: Store both item briefs and page-level briefs in SQLite (`AiDb`) mapped to the specific page batch.
- **No detail-triggered LLM**: Remove on-demand LLM calls from the article detail drawer route.

### Non-Goals
- Full-article translation into a single user-chosen language.
- Multi-turn conversational chat or Q&A on articles.
- Streaming token-by-token output to the browser (batch completes atomically).

### Anti-Goals
- Spawning separate LLM CLI processes when opening individual articles.
- Showing Executive Briefs on the `today` filter or search results.
- Blocking initial page HTML delivery while waiting for external LLM execution.

---

## Core Requirements & Behavior

### 1. Source Language Matching
- The LLM prompt must instruct the model to inspect each article's content and output both the suggested title and executive brief in the **exact same language** as the source article.
- For the page-level Executive Brief (synthesizing all items on the page):
  - If all or most articles are in Language X (e.g. Vietnamese), the overall brief is written in Language X.
  - If mixed, the model provides the synthesis in the predominant language or bilingual summary without translation loss.

### 2. Loading Skeleton for Executive Brief
- When a user navigates to the Unread list page (e.g., page 1, page 2):
  - If the page Executive Brief is already cached in `AiDb`, render the completed card immediately.
  - If the page Executive Brief is pending generation, render an Annuaire-styled skeleton card with `animate-pulse`, subtle placeholder bars, and a "Generating Executive Brief..." status indicator.
  - Once the batch LLM call finishes, swap the skeleton with the final brief content via HTMX.

### 3. Strict Filter Visibility
- **Unread filter**: Executive Brief is visible (rendered either as brief or skeleton while loading).
- **Today filter**: Executive Brief is completely hidden (no card, no skeleton, no batch trigger).
- **Read / Starred / All / Search**: Executive Brief is completely hidden.

### 4. Single Batch LLM Call per Page
- For any active list page with $N$ items (e.g., 15 items on page 2):
  - A single prompt contains all $N$ items (ID, Title, cleaned excerpt).
  - The LLM response returns a compact JSON structure:
    - `"top"`: 1–2 sentences synthesizing all items on the current page (<35 words).
    - `"items"`: Array of objects with:
      - `"i"`: item ID.
      - `"t"`: punchy suggested title ($\le$ 10 words, matching source language).
      - `"b"`: executive brief (1–3 sentences, <50 words, matching source language).
  - The results are saved to `AiDb` in one database transaction.

### 5. Detail View Zero-LLM Reuse
- In `routes/entries.rs` (`render_entry_detail`):
  - Look up the article in `AiDb`.
  - If cached, display the AI summary in the detail drawer's `brief_card.html`.
  - If not cached, do **not** invoke `ai_service.get_or_create_summary(...)`. Display the clean extractive summary/teaser without spawning any LLM process.

---

## Architectural Decisions & Trade-Offs

### Decision 1: Language Preservation in Batch Prompt

- **Option A (Recommended): In-Prompt Instruction for Per-Item Language Matching**
  - **Explanation**: Add explicit multilingual instructions directly in `format_batch_prompt`. The LLM naturally recognizes the language of each text block and outputs corresponding titles and briefs without external language detection libraries.
  - **Example Prompt**:
    ```text
    CRITICAL: For each item and for the overall brief, write your response in the SAME LANGUAGE as the source text. If an article is in Vietnamese, write its title and brief in Vietnamese. If in English, write in English.
    ```
  - **Pros**: Zero external dependencies; zero CPU overhead; handles mixed-language feeds seamlessly.
  - **Cons**: Relies on model adherence (standard LLMs follow this reliably).

- **Option B: Run a Rust Language Detection Crate Before Prompting**
  - **Explanation**: Add a crate like `whatlang` to detect ISO language codes for each item and pass `[Lang: vi]` or `[Lang: en]` tags in the prompt.
  - **Pros**: Explicit metadata provided to the prompt.
  - **Cons**: Adds a new Rust dependency, compile time, and potential misclassification on short news titles.

> **Recommendation**: **Option A**. Modern LLMs are adept at language matching when explicitly instructed.

---

### Decision 2: Skeleton Rendering and HTMX Swap Mechanism

- **Option A (Recommended): HTMX Lazy-Load Endpoint with Skeleton Fallback**
  - **Explanation**: When rendering the list view on the unread filter, if the page brief is not yet in SQLite, render a `#page-executive-brief` skeleton card that triggers `hx-get="/hx/executive-brief?page=X&offset=Y&limit=Z" hx-trigger="load" hx-swap="outerHTML"`. The backend executes the batch job (or joins an in-flight job) and returns the completed card HTML.
  - **Example**:
    ```html
    <!-- Skeleton rendered during list load -->
    <div id="page-executive-brief"
         hx-get="/hx/executive-brief?offset=15&limit=15"
         hx-trigger="load"
         hx-swap="outerHTML"
         class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4 animate-pulse">
      <div class="flex items-center gap-2 mb-2">
        <i data-lucide="sparkles" class="w-4 h-4 text-brand/60"></i>
        <span class="text-xs font-bold uppercase text-muted-foreground font-sans">Executive Brief</span>
      </div>
      <div class="space-y-1.5">
        <div class="h-2.5 bg-muted rounded w-5/6"></div>
        <div class="h-2.5 bg-muted rounded w-4/6"></div>
      </div>
    </div>
    ```
  - **Pros**: Instant list page response; zero lag on initial page load; standard HTMX idiom; clean fallback if AI is disabled or fails.
  - **Cons**: Requires a dedicated lightweight HTMX route `/hx/executive-brief`.

- **Option B: Synchronous Server-Side Blocking on List Page Load**
  - **Explanation**: Wait for the LLM batch call to complete before returning the list HTML.
  - **Pros**: No client-side HTMX swap required.
  - **Cons**: Degrades list page navigation speed by 1–3 seconds, causing perceptible UI hesitation.

- **Option C: Background Prefetch + Client Polling**
  - **Explanation**: Keep prefetch in the background and poll `/hx/executive-brief` every 2 seconds until available.
  - **Pros**: Decouples prefetch entirely.
  - **Cons**: Unnecessary polling requests and timer management.

> **Recommendation**: **Option A**. It delivers an instantaneous page load with an immediate visual skeleton, cleanly swapped when the batch response arrives.

---

### Decision 3: Associating Page Executive Briefs with Pages in Database

- **Option A (Recommended): Cache Page Brief by Entry ID Fingerprint / Page Offset Key**
  - **Explanation**: Store page briefs in `page_executive_briefs` with a `page_key` (e.g. `unread:offset_15:limit_15` or a hash of the entry IDs on that page). When browsing page 2, fetch the brief for page 2 rather than a single global brief.
  - **Example Schema**:
    ```sql
    CREATE TABLE IF NOT EXISTS page_executive_briefs (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      page_key TEXT NOT NULL UNIQUE,
      brief TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    );
    CREATE INDEX IF NOT EXISTS idx_page_briefs_key ON page_executive_briefs(page_key);
    ```
  - **Pros**: Allows each page (page 1, page 2, page 3) to maintain its own accurate brief without clobbering other pages.
  - **Cons**: Small schema evolution in `AiDb`.

- **Option B: Single Global Latest Page Brief**
  - **Explanation**: Keep current schema with only the latest single brief.
  - **Pros**: No schema changes.
  - **Cons**: Browsing back to page 1 displays page 2's summary until regenerated; confuses users.

> **Recommendation**: **Option A**. Ensures pagination is reliable and summaries accurately reflect the page being viewed.

---

### Decision 4: Detail View Fallback when Article Unsummarized

- **Option A (Recommended): Silent Fallback to Extractive Teaser (Strict No-LLM)**
  - **Explanation**: If an item in the detail drawer has no cached AI brief (e.g. accessed directly or before batch completed), display the existing article summary/teaser in the brief card without triggering an LLM CLI process.
  - **Pros**: 100% compliant with user constraint; zero latency; eliminates duplicate AI costs.
  - **Cons**: Does not provide AI-generated text for uncached items until batch runs.

- **Option B: Display a "Brief not generated" Notice**
  - **Explanation**: Show an explicit notice saying the brief is unavailable.
  - **Pros**: Clear indication.
  - **Cons**: Visually noisy compared to a clean excerpt.

> **Recommendation**: **Option A**. Transparent, elegant fallback consistent with Annuaire design principles.

---

## Database Schema Evolution

Update `AiDb` to version 3 to support page-specific executive briefs:

```sql
-- Migration to user_version 3
CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  page_key TEXT NOT NULL UNIQUE,
  brief TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_page_briefs_v2_key ON page_executive_briefs_v2(page_key);
PRAGMA user_version = 3;
```

---

## Tentative Lens & Card Flags

- **Has UI**: yes (Executive Brief skeleton component, unread-only visibility, detail view brief card)
- **Has API**: yes (New `/hx/executive-brief` endpoint, updated `/hx/entries` filter logic)
- **Has DB**: yes (`AiDb` schema evolution to version 3 for keyed page briefs)
- **Has auth/PII/schema**: yes (SQLite migration for page briefs)

---

## Open Questions for User Confirmation

1. **Skeleton and Title Updates**: When the single batch LLM call finishes for the page, should the response also update the list item titles dynamically on the page (via HTMX Out-Of-Band swaps), or is updating the Executive Brief card alone sufficient for the initial view?
2. **Page Brief Cache Invalidation**: How long should a page-level Executive Brief remain cached before regenerating? (Recommended: 30 minutes, or until new unread articles arrive on that page).
3. **Skeleton Style**: Should the skeleton include a pulsing shimmer animation with "Generating Executive Brief..." label, matching the Annuaire theme palette?
