# Technical Design: Executive Brief Duplication, Detail Caching, DB Reuse, and Prompt Rules

**Result:** done

---

## 1. Executive Summary

This design resolves four key issues:
1. **Executive Brief Duplication**: Eliminate duplicate markup in `/hx/executive-brief` and `executive_brief_card.html` so only a single card is returned over the wire.
2. **Instant Detail Title & Brief from Cache**: Fix client-side IndexedDB cache validation (removing obsolete string check) and render title and brief immediately from DOM/cache when opening an article.
3. **Database Reuse for Analysis**: Check and reuse `impact` and `fallacies` from the database in `get_entry_analysis_htmx` and `AiService` so the LLM is never called for already-analyzed articles.
4. **Rewrite Rules in Prompts**: Enforce inverted pyramid structure (<110 words, 30–60 sweet spot, 2–3 bullets, no conversational filler) across single and batch prompts.

---

## 2. Options and Trade-offs

### Area 1: Executive Brief Markup Duplication

#### Option 1 (Recommended): Single Brief Partial + Client Event Sync
- **Explanation**: In `crates/ui-components/templates/partials/executive_brief_card.html`, remove `<div id="list-executive-brief" hx-swap-oob="true">`. The partial returns only `<div id="page-executive-brief">...</div>`. In `crates/ui-components/templates/pages/index.html`, inside `document.addEventListener('htmx:afterSwap')`, when `#page-executive-brief` swaps, copy the brief text into `#list-executive-brief` if present.
- **Concrete Example**:
  ```html
  <!-- executive_brief_card.html: clean single card -->
  <div id="page-executive-brief" class="rounded-lg p-4 md:p-5 bg-muted/40 border border-border flex flex-col">
    <div class="flex items-center gap-2 pb-3 mb-3 border-b border-border/80">
      <div class="p-1 rounded bg-brand/10 text-brand">
        <i data-lucide="sparkles" class="w-4 h-4"></i>
      </div>
      <span class="text-xs md:text-sm font-bold tracking-wider uppercase text-foreground font-sans">Executive Brief</span>
    </div>
    <p class="text-xs md:text-sm text-foreground/90 leading-relaxed font-sans">{{ brief }}</p>
  </div>
  ```
  ```javascript
  // index.html afterSwap:
  if (isBriefSwap) {
    const pageBrief = document.querySelector('#page-executive-brief p');
    const listBrief = document.getElementById('list-executive-brief');
    if (pageBrief && listBrief) {
      listBrief.classList.remove('animate-pulse');
      listBrief.innerHTML = `
        <div class="flex items-center gap-2 mb-1.5">
          <div class="p-1 rounded bg-muted text-foreground">
            <i data-lucide="sparkles" class="w-4 h-4 text-brand"></i>
          </div>
          <span class="text-xs font-bold tracking-wide uppercase text-foreground font-sans">Executive Brief</span>
        </div>
        <p class="text-xs text-foreground/90 leading-relaxed font-sans">${escapeHtml(pageBrief.innerText)}</p>
      `;
    }
  }
  ```
- **Pros**: Zero payload bloat; single source of markup; list view and magazine view remain completely synchronized.
- **Cons**: Requires 5 lines of client JavaScript in `index.html`.

#### Option 2: View-Specific Query Parameter (`&view=list|magazine`)
- **Explanation**: The endpoint inspects `&view=` and returns either `#list-executive-brief` or `#page-executive-brief`.
- **Pros**: Pure server-side rendering.
- **Cons**: Doubles server route branches; prevents caching across view toggles; complicates query strings.

**Recommendation**: **Option 1** because it simplifies the HTML templates and avoids redundant DOM trees.

---

### Area 2: Instant Detail Title & Brief from Cache

#### Option 1 (Recommended): Fix IndexedDB Cache Validation + Immediate DOM Extraction on Open
- **Explanation**:
  1. Fix `index.html:630`: Change `cached.html.includes('Executive Brief')` to validate on `cached && cached.html`. The previous check was a bug that permanently disabled the 30-day IndexedDB cache.
  2. In `openEntryDetail(id)`: When showing the skeleton or waiting for network, extract the title from `#entry-title-${id}` and pass it to `openDetailWithSkeleton(id, title)`.
  3. Render the real title immediately inside the detail panel header so the user never sees an empty title bar or generic gray boxes.
- **Concrete Example**:
  ```javascript
  // openEntryDetail(id)
  const titleEl = document.getElementById('entry-title-' + numId);
  const title = titleEl ? titleEl.textContent.trim() : '';
  openDetailWithSkeleton(title);
  ```
  ```javascript
  // openDetailWithSkeleton(title)
  // Instead of gray placeholder:
  <h1 class="text-2xl font-bold font-sans text-foreground mb-6 leading-snug">${escapeHtml(title)}</h1>
  ```
- **Pros**: Instant UI responsiveness; eliminates layout shifts and empty title flicker; restores IndexedDB offline caching.
- **Cons**: Title displayed during fetch is the title from the list, which will be updated if the server returns a suggested title.

#### Option 2: Server-Side Pre-Rendering Only
- **Explanation**: Keep client simple and rely solely on server response speed.
- **Pros**: No changes to client skeleton script.
- **Cons**: User sees a blank/pulsing skeleton on every click, degrading perceived speed.

**Recommendation**: **Option 1** because instant feedback is essential for perceived performance.

---

### Area 3: Database Reuse for Analysis

#### Option 1 (Recommended): Database Check in Route and AiService
- **Explanation**:
  1. In `get_entry_analysis_htmx` (`crates/server/src/routes/entries.rs`), check `ai_service.db.get_summary_and_title(id).await`. If `impact` or `fallacies` is present and non-empty, render and return `render_analysis_cards_fragment(imp, fal)` immediately.
  2. In `AiService::analyze_entry_impact_and_fallacies` (`crates/server/src/ai/mod.rs`), check `self.db.get_summary_and_title(entry_id).await`. If either `impact` or `fallacies` exists, return early without invoking `self.llm.analyze_impact_and_fallacies`.
- **Concrete Example**:
  ```rust
  if let Some(record) = ai_service.db.get_summary_and_title(id).await {
      let imp = record.impact.as_deref().unwrap_or("").trim();
      let fal = record.fallacies.as_deref().unwrap_or("").trim();
      if !imp.is_empty() || !fal.is_empty() {
          let html = render_analysis_cards_fragment(imp, fal);
          return Html(html).into_response();
      }
  }
  ```
- **Pros**: Completely prevents duplicate LLM invocations; zero latency for subsequent views; robust against single or dual analysis fields.
- **Cons**: None.

#### Option 2: In-Memory Cache in Route Handler Only
- **Explanation**: Store analysis in an in-memory TTL cache inside Axum state.
- **Pros**: Fast in-memory lookup.
- **Cons**: Volatile across server restarts; ignores persistent data already saved in PostgreSQL/SQLite.

**Recommendation**: **Option 1** because it reuses existing persistent database tables.

---

### Area 4: Rewrite Rules in Prompts

#### Option 1 (Recommended): Inverted Pyramid, Strict Word Limits, and Fluff Removal
- **Explanation**: Update prompt templates in `crates/server/src/ai/llm.rs`:
  - Length: Under 110 words total (sweet spot 30–60 words).
  - Structure: Lead with outcome and key impacts in the first 2 sentences, followed by 2–3 concise bullet points with essential context.
  - Tone & Style: Plain and simple words; no introductory filler ("Here is a summary", "In recent news") and no concluding boilerplate.
  - Multilingual constraint: Output strictly in the same language as the source text.
- **Concrete Example**:
  ```text
  For "r" (rewrite):
  - Under 110 words total (sweet spot 30-60 words).
  - Inverted pyramid structure: lead with outcome and key impacts in the first 2 sentences.
  - Follow with 2-3 concise bullet points containing essential details.
  - Use plain, simple language. Strictly omit introductory filler or closing commentary.
  ```
- **Pros**: Produces concise, scannable summaries matching user reading flow; reduces token costs and generation time.
- **Cons**: None.

---

## 3. Verifiability and Source References

- `crates/ui-components/templates/partials/executive_brief_card.html`
- `crates/ui-components/templates/pages/index.html`
- `crates/server/src/routes/entries.rs`
- `crates/server/src/ai/llm.rs`
- `crates/server/src/ai/mod.rs`
- HTMX Out-of-band Swap Documentation: <https://htmx.org/attributes/hx-swap-oob/>
