# Idea Framing — AI News Batch Summary

## Intent
Provide an executive, AI-driven overview of news articles. When AI is enabled, users can view a curated batch of 15 news items grouped by feed source, with concise (under 35 words) AI-generated summaries that load smoothly as items scroll into view. The system scrapes and cleans full article content when feeds provide only teasers, falls back gracefully on scrape blocks, invokes a lightweight LLM CLI with zero reasoning budget, and permanently caches summaries so no article is ever re-summarized.

## Goals

1. **AI Filter Button**:
   - Add an `AI` filter pill before the `Unread` button in the top filter bar when `AI_ENABLED=true` (or configured via environment).
   - When AI is disabled, the button is hidden and all AI-related endpoints remain inactive.
   - Clicking `AI` requests the AI batch summary view (`/hx/ai-summary` or `status=ai`).

2. **Grouped Batch Summary (15 Items)**:
   - Query the latest 15 articles across active feeds.
   - Group the 15 items by feed source (e.g., "Ars Technica", "Rust Blog", "Hacker News") with feed headers.
   - Each item displays title, feed source, relative timestamp, and an AI summary block.

3. **Viewport / Scroll Lazy Summarization**:
   - Summaries are triggered dynamically via HTMX when items enter the viewport (`hx-trigger="revealed"` or IntersectionObserver), preventing bulk LLM latency and API bottlenecking on initial page load.
   - While summarizing, display an AR glass skeleton/pulsing badge; once ready, swap in the clean 1–2 sentence summary.

4. **Robust Content Extraction & Fallback Pipeline**:
   - Check if full article text is present in the feed `<content:encoded>` or `<content>`.
   - If only a teaser exists in `<description>` / `<summary>`, make an asynchronous HTTP request to fetch the source article URL.
   - Parse `<article>` tag content (extract body within `<article>`, strip inner tags/scripts/nav, falling back to body/teaser if `<article>` tag is absent).
   - Truncate cleaned text to Title + first 400–600 words for the LLM prompt.
   - Fallback guarantee: if scraper receives HTTP 403 Forbidden, Cloudflare challenge, or paywall, immediately fall back to the existing RSS title + excerpt. Never leave an item blank and never block or stall the batch.

5. **LLM CLI Summarization**:
   - Execute a fast, lightweight model via CLI command with thinking/reasoning budget set to 0.
   - Prompt constraint: "Extract what changed or why it matters in 1–2 sentences (under 35 words)."
   - Strict timeout and process isolation.

6. **Persistent Cross-User Caching**:
   - Persist generated summaries in local storage (e.g., SQLite table `ai_summaries(entry_id, summary, model, created_at)`).
   - Before scraping or calling the LLM, check the cache. Never re-summarize an article across different users or page reloads.

## Non-goals
- Full browser automation or headless scraping (Puppeteer/Playwright).
- Solving or bypassing CAPTCHAs, Cloudflare Turnstile, or paywalls (fallback is mandatory).
- Infinite scroll of hundreds of AI summaries at once (strict batch of 15 items).

## Anti-goals
- Never stall the UI or batch while waiting for an individual article to scrape or summarize.
- Never leave an article summary empty or broken.
- Never invoke LLM CLI repeatedly for already summarized articles.

## Constraints
- **Design system**: Dark spatial glassmorphism matching `ar-list.html` and `ar-detail.html`.
- **Responsive layout**: Use modern CSS (`grid`, `minmax`, container queries); no hardcoded screen breakpoints.
- **Environment**: Use `.env.example` placeholders; never inspect `.env`.
- **TDD verification**: Unit tests for HTML boilerplate stripping, word truncation, fallback logic, CLI execution, and template rendering before implementation.

---

## Suggested UI Options for Grouped News Summary

As requested, here are the viable UI options for presenting the batch of 15 news items grouped by feed source with viewport-based summarization:

### Option A: Feed-Grouped Glass Cards with HTMX Viewport Revealing (Recommended)
- **Explanation**: The main content window organizes the 15 items under distinct feed source headers (with feed icon and count). Each article is presented as a sleek glass card containing the title, source metadata, and a dedicated AI summary section. As each card scrolls into the viewport, an HTMX trigger (`hx-get="/hx/ai-summary/entry/{id}" hx-trigger="revealed" hx-swap="outerHTML"`) fetches and swaps in the cached or newly generated AI summary.
- **Concrete Example**:
  ```html
  <!-- Feed Group Section -->
  <div class="feed-group mb-6">
    <div class="flex items-center gap-2 mb-3 px-2">
      <span class="text-xs font-semibold uppercase tracking-wider text-indigo-400">Ars Technica</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full bg-white/5 text-slate-400 border border-white/10">3 items</span>
    </div>
    <div class="grid gap-3">
      <!-- Article Card with lazy summary -->
      <article class="ar-glass-panel p-4 rounded-xl border border-white/10 hover:border-indigo-500/30 transition-all">
        <a href="..." class="text-sm font-medium text-slate-100 hover:text-indigo-300">SpaceX launches new lunar payload</a>
        <div class="text-xs text-slate-400 mt-1">2 hours ago</div>
        <div class="ai-summary-slot mt-3 pt-3 border-t border-white/5"
             hx-get="/hx/ai-summary/entry/101"
             hx-trigger="revealed"
             hx-swap="outerHTML">
          <div class="skeleton-pulse text-xs text-indigo-300/60 flex items-center gap-2">
            <i data-lucide="sparkles" class="w-3.5 h-3.5 animate-spin"></i> Generating brief...
          </div>
        </div>
      </article>
    </div>
  </div>
  ```
- **Pros**:
  - Clean, hierarchical reading eye flow matching the AR glass panel language.
  - Native HTMX `revealed` trigger works seamlessly without complex client-side JS state.
  - Highly scannable: users see sources at a glance and read concise bullet summaries without visual clutter.
- **Cons**:
  - Slightly longer vertical scrolling than a dense table format.

### Option B: Collapsible Accordion by Feed Source
- **Explanation**: Each feed source is rendered as a collapsible accordion panel. Expanding a feed reveals its news cards with AI summaries.
- **Concrete Example**:
  ```html
  <details class="ar-glass-panel rounded-xl border border-white/10 mb-3" open>
    <summary class="p-4 cursor-pointer font-medium flex justify-between items-center text-slate-200">
      <span>Rust Blog (2)</span>
      <i data-lucide="chevron-down" class="w-4 h-4"></i>
    </summary>
    <div class="p-4 pt-0 space-y-3">
      <!-- Article summaries -->
    </div>
  </details>
  ```
- **Pros**:
  - Very compact if users want to focus on only one feed at a time.
- **Cons**:
  - Viewport-based lazy loading (`hx-trigger="revealed"`) inside closed accordion panels requires extra logic to avoid triggering or to trigger properly on disclosure.
  - Extra user clicks required to browse all 15 articles.

### Option C: Two-Column Feed Rail + Summary Feed
- **Explanation**: A left sub-rail lists the active feed sources in the batch, while the right pane shows the feed's articles.
- **Concrete Example**:
  ```html
  <div class="grid grid-cols-[200px_1fr] gap-4">
    <aside class="feed-rail ar-glass-panel p-3">...</aside>
    <section class="summary-list ar-glass-panel p-4">...</section>
  </div>
  ```
- **Pros**:
  - Desktop-centric rapid jumping between sources.
- **Cons**:
  - Breaks the current streamlined 720px spatial glass window layout on medium/narrow viewports.
  - Disproportionately complex layout for a fixed batch of 15 items.

**Recommendation**: **Option A (Feed-Grouped Glass Cards with HTMX Viewport Revealing)** provides the best user experience, conforms strictly to the AR spatial glass aesthetic, and works out-of-the-box with HTMX viewport event triggers.

---

## Open Decisions & Technical Details

1. **AI Button Placement**:
   - Filter bar order when AI is enabled: `[AI]`, `[Unread]`, `[Today]`, `[Starred]`, `[All Titles]`.
   - Active styling for `AI`: Glowing indigo/violet pill with `sparkles` icon.

2. **Persistence Store**:
   - SQLite table in `crates/server` (e.g. `summaries.db` or embedded inside server storage):
     `CREATE TABLE IF NOT EXISTS article_ai_summaries (entry_id INTEGER PRIMARY KEY, summary TEXT NOT NULL, created_at TEXT NOT NULL);`

3. **LLM CLI Configuration**:
   - Environment variables in `.env.example`:
     - `AI_ENABLED=true`
     - `LLM_CLI_CMD=llm` (or custom executable path, e.g., `/usr/local/bin/llm`)
     - `LLM_MODEL=gpt-4o-mini` (or local fast model)
     - `LLM_EXTRA_ARGS=--thinking-budget 0`
   - If LLM CLI invocation fails or is unavailable, fallback cleanly to the extractive 2-sentence summary already present in `crates/miniflux/src/summary.rs`.

---

## Shared Understanding & Confirmation (HITL)

- [x] User confirmed (brainstorm hard-gate)
- User instruction: `Update "Strip HTML boilerplate (tags, nav, ads, footers)" to "Parse <artical> tag content"`
- UI selected: Option A (Feed-Grouped Glass Cards with HTMX revealed trigger)
- Placement: `[AI]` button before `[Unread]`
