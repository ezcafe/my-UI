# Design: Performance, Loading, and Token Optimization

- **Workflow Slug:** `20261124-perf-loading-token-optimization`
- **Mode:** full
- **Result:** done

---

## 1. Architectural Options & Recommendation

### Option 1 (Recommended): Selective Eager Top-1 Rewrite, HTMX Lazy-Loading Drawer, Canonical Cache Keys, and Static CSS

- **Explanation**:
  This approach targets the primary performance bottlenecks identified in analysis and settles the three Grill decisions:
  1. **Selective Eager Rewrite (Q2 / 2B)**: The page batch LLM call eagerly generates an AI rewrite (`"r"`) for the top-1 featured story only, deferring rewrites for the remaining 14 articles. This cuts batch output tokens by ~80% (saving 1,200–1,500 tokens) and drops page batch latency from 25–30s to under 5s.
  2. **Instant Drawer with HTMX Lazy-Load (Q1 / 1A Variant)**: Opening an article drawer renders instantly (0ms delay) showing the 3-sentence executive brief (`brief_card.html`). If the rewrite is already cached (e.g. top-1 story), it displays immediately. If not cached, the drawer renders `#ai-rewrite-box` containing a skeleton loader with `hx-get="/hx/entries/{id}/rewrite" hx-trigger="load" hx-swap="outerHTML"`. Raw article content is never dumped below the skeleton.
  3. **Precompiled Static CSS (Q3 / 3A)**: Eliminate `cdn.tailwindcss.com` and its runtime JS evaluation. Serve `/static/annuaire-theme.css` directly from Axum with precompiled utility classes and Annuaire design tokens, removing the 150–300ms layout shift and FOUC.
  4. **Unified Canonical Cache Keys**: Unify the page key computation across background prefetching (`AiPrefetchService`) and the foreground brief route (`executive_brief_htmx`) using `(status, &page_entry_ids)` to eliminate orphaned prefetch results and redundant LLM executions.
  5. **Scraper & Token Guardrails**: Reduce teaser fast-path threshold from 50 to 25 words, shorten scraper HTTP timeout from 3.0s to 1.5s, truncate article input prompts from 300 to 130 words, and enforce strict source-grounding prompt constraints against hallucination.

- **Concrete Examples**:
  ```html
  <!-- Detail view: instant drawer with HTMX rewrite lazy-loader -->
  <div class="detail-body flex-1 overflow-y-auto pr-2 w-full max-w-[44rem] mx-auto">
    <h1 class="text-2xl font-bold font-sans text-foreground mb-6">{{ entry.title }}</h1>
    
    <!-- 3-Sentence Executive Brief Card -->
    {% include "brief_card.html" %}

    <!-- Rewrite container: direct rewrite if cached, or HTMX lazy-loader -->
    {% if is_rewrite %}
      <div id="ai-rewrite-box" class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
        {{ rewrite }}
      </div>
    {% else %}
      <div id="ai-rewrite-box" hx-get="/hx/entries/{{ entry.id }}/rewrite" hx-trigger="load" hx-swap="outerHTML">
        <div class="animate-pulse space-y-3 pt-2">
          <div class="h-3 bg-muted rounded w-3/4"></div>
          <div class="h-3 bg-muted rounded w-5/6"></div>
          <div class="h-3 bg-muted rounded w-2/3"></div>
        </div>
      </div>
    {% endif %}
  </div>
  ```

  ```rust
  // Shared canonical page key across prefetch and executive brief
  pub fn compute_page_key(status: &str, entry_ids: &[i64]) -> String {
      let mut hasher = Sha256::new();
      for id in entry_ids {
          hasher.update(id.to_le_bytes());
      }
      let hash = format!("{:x}", hasher.finalize());
      format!("{}:{}", status, &hash[..16])
  }
  ```

- **Pros**:
  - **Sub-50ms drawer opening**: Readers never wait for an LLM call when clicking an article; the executive brief and reading controls appear instantly.
  - **80% token savings**: Batch prompt only outputs one rewrite for the lead story instead of 15–20 rewrites, saving ~1,200–1,500 output tokens per page view.
  - **Zero FOUC & offline styling**: Precompiled CSS eliminates runtime CDN script evaluation, external dependencies, and layout shifts.
  - **Zero redundant LLM calls**: Canonical cache keys guarantee in-flight deduplication and prefetch hit rates.
  - **Fast-path scraper resilience**: 25-word teaser fast-path avoids HTTP requests for 70%+ of RSS feeds.

- **Cons**:
  - Non-lead stories (items 2–15) take 1.5–2s to lazy-load their detailed rewrite after the drawer opens.

---

### Option 2: 100% Deferral with Synchronous Detail Route and Runtime CDN

- **Explanation**:
  Generate zero rewrites in the batch prompt. When a user opens an article, block the detail route synchronously until the LLM generates the rewrite. Retain `cdn.tailwindcss.com` with inline height guards.
- **Concrete Example**:
  ```rust
  // Blocking synchronous call inside detail route
  let rewrite = if let Some(r) = record.rewrite {
      r
  } else {
      ai_service.generate_single_rewrite(id, &title, &content).await?
  };
  ```
- **Pros**:
  - Batch call generates zero rewrite tokens.
  - No client-side HTMX swap after drawer is opened.
- **Cons**:
  - **Sluggish UX**: Clicking any article causes a 2–4s blocking delay before the drawer opens.
  - **Layout shifts**: Runtime Tailwind CDN script continues to cause 150–300ms FOUC on page load.
  - **Fragility**: Upstream LLM timeouts break drawer navigation completely.

---

### Recommendation & Rationale

**Option 1 is strongly recommended**. It implements the exact settled decisions from `02b-grill.md` (1A variant, 2B, 3A). It combines immediate visual feedback with asynchronous background completion, cuts LLM output tokens by 80%, and eliminates external CDN dependencies and layout shifts.

---

## 2. Core Contracts & Specifications

### A. Canonical Page Cache Key & Deduplication

- **Problem Solved**: `AiPrefetchService` previously computed keys with prefix `"prefetch"` using only uncached entry IDs, while `executive_brief_htmx` computed keys with the query status (e.g. `"unread"`) and all entry IDs on the page. The keys diverged, causing duplicate LLM calls and orphaned prefetch entries.
- **Contract**:
  Both prefetch and foreground brief use the exact same signature and canonical status:
  ```rust
  pub fn compute_page_key(status: &str, entry_ids: &[i64]) -> String
  ```
  - When triggering prefetch from `render_index` or `list_entries_htmx`, pass `status` (normalized, defaulting to `"unread"`) and the slice of all page `entry_ids`.
  - Both prefetch and foreground acquire the mutex on `ai_service.in_flight` using the identical `page_key`.

---

### B. Batch Prompt & Token Budget Optimization

- **Target File**: [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs)
- **Word Truncation Cap**: Truncate input content for each article to at most **130 words** (down from 300 words).
- **Eager Top-1 Rewrite Rules**:
  - In `format_batch_prompt_structured`, instruct the LLM:
    1. `"top"`: Executive synthesis across all page stories.
    2. `"items"`:
       - `"t"`: One-sentence punchy title for all new articles.
       - `"b"`: Strict 3-sentence brief for all new articles.
       - `"r"`: Inverted-pyramid rewrite (**ONLY** for the first new article `[ID <id>]`; omit or set null for all other articles).
  - Schema: `{"top":"...","items":[{"i":101,"t":"...","b":"...","r":"..."},{"i":102,"t":"...","b":"..."}]}`
- **Grounding Constraint**:
  Every prompt (batch and single rewrite) must include:
  ```text
  GROUNDING CONSTRAINT: Rely strictly and solely on the provided article text. Do not extrapolate, speculate, or introduce external claims or background facts not directly stated in the source text.
  ```

---

### C. Detail Drawer Render & HTMX Lazy-Loading Route

- **Drawer Initial Render (`crates/server/src/routes/entries.rs: get_entry_detail_htmx`)**:
  - Reads `AiDb` for `entry_id`.
  - If `record.rewrite` is present and non-empty: sets `is_rewrite = true`, passes `rewrite` to template.
  - If `record.rewrite` is absent or empty: sets `is_rewrite = false`, passes `summary` (or teaser fallback) as `summary`.
- **Drawer Template (`crates/ui-components/templates/partials/entry_detail.html`)**:
  - Always shows the 3-sentence executive brief via `brief_card.html`.
  - When `is_rewrite` is true: renders `rewrite` directly in `#ai-rewrite-box`.
  - When `is_rewrite` is false: renders skeleton loader inside `#ai-rewrite-box` with HTMX attributes:
    ```html
    <div id="ai-rewrite-box" hx-get="/hx/entries/{{ entry.id }}/rewrite" hx-trigger="load" hx-swap="outerHTML">
      <div class="animate-pulse space-y-3 pt-2">
        <div class="h-3 bg-muted rounded w-3/4"></div>
        <div class="h-3 bg-muted rounded w-5/6"></div>
        <div class="h-3 bg-muted rounded w-2/3"></div>
      </div>
    </div>
    ```
  - **No raw content dump**: Do not display raw article text or duplicate teaser below the skeleton.
- **On-Demand Rewrite Endpoint (`GET /hx/entries/{id}/rewrite`)**:
  - Checks `AiDb` first; if rewrite exists, returns rendered `#ai-rewrite-box` immediately.
  - If missing:
    - Extracts article text via `ArticleExtractor` (teaser fast-path / 1.5s timeout).
    - Calls `AiService::generate_rewrite(id, &title, &content)`.
    - Persists rewrite into `AiDb` via `save_summary_and_title_with_rewrite`.
    - Returns rendered `#ai-rewrite-box` HTML fragment.
  - Fallback on error: Renders the existing 3-sentence brief in `#ai-rewrite-box` without breaking the drawer layout.

---

### D. Scraper Latency Optimization

- **Target File**: [crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs)
- **25-Word Teaser Fast-Path**: If `teaser.split_whitespace().count() >= 25`, return teaser immediately without making any HTTP request.
- **1.5s HTTP Timeout**: Reduce `reqwest::Client` timeout from `Duration::from_secs(3)` to `Duration::from_millis(1500)`.
- **Fallback**: Any network error, HTTP status != 200, or timeout returns teaser text truncated to 130 words.

---

### E. Static CSS Delivery & CDN Elimination

- **Target Files**:
  - [crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)
  - [crates/server/src/app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs)
  - [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs)
- **Static Route**:
  Add `GET /static/annuaire-theme.css` to Axum router:
  - Serves static precompiled CSS bundle with `Content-Type: text/css; charset=utf-8` and `Cache-Control: public, max-age=86400`.
- **Template Updates**:
  - Remove `<script src="https://cdn.tailwindcss.com"></script>`.
  - Remove inline `tailwind.config = { ... }` script block.
  - Add `<link rel="stylesheet" href="/static/annuaire-theme.css">` in `<head>`.
- **CSS Bundle Composition**:
  - All Tailwind reset and utility classes used by templates (`flex`, `hidden`, `grid`, `text-xs`, `rounded-full`, `animate-pulse`, etc.).
  - Annuaire OKLCH color variables and design tokens for light and dark modes.
  - Container queries (`@container`), typography, and reduced motion overrides.

---

## 3. Data Structures & Schema

### Database (`AiDb`) Upsert Semantics

Existing SQLite schema (`user_version = 4`) already contains `rewrite TEXT` on `article_ai_summaries`:
```sql
CREATE TABLE IF NOT EXISTS article_ai_summaries (
    entry_id INTEGER PRIMARY KEY,
    summary TEXT NOT NULL,
    suggested_title TEXT,
    rewrite TEXT,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

When saving on-demand rewrites:
```sql
INSERT INTO article_ai_summaries (entry_id, summary, suggested_title, rewrite, model, created_at)
VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
ON CONFLICT(entry_id) DO UPDATE SET
  summary = excluded.summary,
  suggested_title = COALESCE(excluded.suggested_title, article_ai_summaries.suggested_title),
  rewrite = COALESCE(excluded.rewrite, article_ai_summaries.rewrite),
  model = excluded.model,
  created_at = excluded.created_at;
```

---

## 4. API Endpoints

| Endpoint | Method | Purpose | Response Format |
| :--- | :--- | :--- | :--- |
| `/hx/entries/{id}/rewrite` | `GET` | Lazy-loads AI rewrite for article detail drawer | HTML fragment (`#ai-rewrite-box`) |
| `/static/annuaire-theme.css` | `GET` | Serves precompiled CSS stylesheet | `text/css; charset=utf-8` |
| `/hx/executive-brief` | `GET` | Returns executive synthesis & updates titles | HTML fragment + OOB spans |

---

## 5. Error Handling & Resilience Matrix

| Component | Failure Scenario | Fallback Action |
| :--- | :--- | :--- |
| **Scraper** | HTTP timeout (>1.5s) or 403/429 | Returns available teaser text truncated to 130 words. |
| **Prefetch** | Background LLM batch error | Fails silently; leaves unread items for subsequent requests. |
| **HTMX Rewrite** | LLM timeout or process crash | Renders the 3-sentence executive brief in `#ai-rewrite-box`. |
| **Static CSS** | Route requested with unknown file | Returns 404 Not Found for non-matching static assets. |
| **Cache Key** | Empty entry list | Returns empty response without calling LLM or acquiring mutex. |
