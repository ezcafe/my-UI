# Grill log

**Result:** frontier-empty

## Design Tree & Decisions

```
Performance & Token Optimization
├── 1. Detail Rewrite Triggering
│   └── [Selected: Option 1A Variant] Immediate drawer showing 3-sentence brief + loading skeleton (not raw article text), HTMX lazy-loads rewrite
├── 2. Batch Prompt Scope
│   └── [Selected: Option 2B] Eager rewrite generated for top-1 story only, remaining 14 deferred
└── 3. Frontend Stylesheet Delivery
    └── [Selected: Option 3A] Precompiled static CSS via Axum, eliminate runtime CDN
```

---

## Round log

### Round 1 — Settled

- **Q1**: Detail View Rewrite Triggering
  - **Decision**: **1A Variant** — Open drawer instantly showing the 3-sentence executive brief and a loading skeleton for the rewrite (do NOT dump unformatted raw article text). Lazy-load the detailed inverted-pyramid rewrite via HTMX.
- **Q2**: Batch Prompt Generation Scope
  - **Decision**: **2B** — Generate eager rewrite `"r"` for the top-1 featured story in the batch call so the main lead opens instantly; defer rewrites for the other 14 items to on-demand requests.
- **Q3**: Frontend Stylesheet & Layout Shift
  - **Decision**: **3A** — Replace `cdn.tailwindcss.com` with precompiled static CSS served from Axum to eliminate layout shifts and external CDN dependencies.


❓ **Q1** - **Detail View Rewrite Triggering**: Should we load the article detail drawer immediately with the executive brief and asynchronously lazy-load the detailed rewrite via HTMX?

- **Option A (Recommended): Immediate Drawer with HTMX Lazy-Load Rewrite**
  - **Explanation**: Opening an article drawer responds instantly (0ms delay) showing the article content and the 3-sentence executive brief. An HTMX container (`hx-get="/hx/entries/{id}/rewrite" hx-trigger="load" hx-swap="outerHTML"`) fetches or generates the 30–60 word inverted-pyramid rewrite in the background and replaces the brief card with the full rewrite when completed.
  - **Concrete Example**:
    ```html
    <div id="ai-rewrite-box" hx-get="/hx/entries/123/rewrite" hx-trigger="load" hx-swap="outerHTML">
      {% include "brief_card.html" %}
      <div class="htmx-indicator text-xs text-muted-foreground mt-2">Loading detailed rewrite...</div>
    </div>
    ```
  - **Pros**: Zero blocking latency when clicking an article; reader sees immediate text; server generates rewrite only when drawer is opened.
  - **Cons**: UI swaps content in place after ~1.5–2s when the rewrite LLM call completes.

- **Option B: Synchronous Generation in Detail Route**
  - **Explanation**: In `get_entry_detail_htmx`, if `record.rewrite` is absent, invoke the LLM before returning the HTML response.
  - **Concrete Example**:
    ```rust
    let rewrite = if let Some(r) = record.rewrite {
        r
    } else {
        ai_service.generate_article_rewrite(id, &title, &content).await
    };
    ```
  - **Pros**: No client-side HTMX swap; drawer opens with final rewrite already rendered.
  - **Cons**: Adds 2–4s delay to opening the article drawer.

➡️ **Recommendation**: Accept Option A (yes). It keeps drawer navigation instantaneous and guarantees smooth reading flow.

---

❓ **Q2** - **Batch Prompt Generation Scope**: Should the batch prompt defer 100% of individual article rewrites, outputting only the page executive synthesis (`top`), punchy titles (`t`), and 3-sentence briefs (`b`)?

- **Option A (Recommended): 100% Deferral of Rewrites in Batch Call**
  - **Explanation**: Batch LLM call generates the page-level executive brief (`top`), punchy titles (`t`), and 3-sentence card briefs (`b`) for all new articles on the page. Inverted-pyramid rewrites (`r`) are excluded from batch generation and produced only on-demand when an article is opened.
  - **Concrete Example**:
    ```json
    {"top": "Tech giants announce...", "items": [{"i": 101, "t": "Apple Unveils New Chip", "b": "Apple introduced M4... It boosts throughput... Shipments begin Friday."}]}
    ```
  - **Pros**: Maximum token efficiency. Saves 1,200–1,500 output tokens per page view (~80% reduction), reducing batch latency from 30s to under 5s.
  - **Cons**: Detail view must trigger rewrite on-demand if the user clicks an article.

- **Option B: Eager Rewrite for Top Featured Article Only**
  - **Explanation**: Batch call generates `"r"` for the single most recent or featured article on the page, while deferring the remaining 14 articles.
  - **Concrete Example**:
    ```json
    {"top": "...", "items": [{"i": 101, "t": "...", "b": "...", "r": "Leading story rewrite..."}, {"i": 102, "t": "...", "b": "..."}]}
    ```
  - **Pros**: Top article drawer opens with rewrite pre-cached.
  - **Cons**: Adds ~100 output tokens to batch call; wasteful if user clicks a different article first.

➡️ **Recommendation**: Accept Option A (yes). 100% deferral delivers the cleanest token savings and fastest page loads.

---

❓ **Q3** - **Frontend Stylesheet & Layout Shift**: Should we replace the runtime `cdn.tailwindcss.com` script with a local precompiled CSS stylesheet served directly by the server?

- **Option A (Recommended): Static Precompiled CSS via Axum Route**
  - **Explanation**: Bundle the required Tailwind utilities and custom Annuaire theme rules into a static CSS file (or serve `/static/annuaire-theme.css` compiled ahead of time). Remove `<script src="https://cdn.tailwindcss.com"></script>`.
  - **Concrete Example**:
    ```html
    <!-- In base.html -->
    <link rel="stylesheet" href="/static/annuaire-theme.css">
    <!-- Removed: <script src="https://cdn.tailwindcss.com"></script> -->
    ```
  - **Pros**: Completely eliminates the 150–300ms layout shift and FOUC; removes external network dependency; loads instantly from local server cache.
  - **Cons**: Any new Tailwind utility class must be included in the precompiled stylesheet.

- **Option B: Keep CDN with Preload & Inline Skeleton Height Guards**
  - **Explanation**: Retain `cdn.tailwindcss.com` but add `<link rel="preload">` and fixed height styles to prevent reflow.
  - **Concrete Example**:
    ```html
    <link rel="preload" href="https://cdn.tailwindcss.com" as="script">
    ```
  - **Pros**: Retains on-the-fly Tailwind class experimentation without compiling CSS.
  - **Cons**: Still causes layout shifts, relies on external internet access, and introduces third-party CDN risk.

➡️ **Recommendation**: Accept Option A (yes). Eliminates layout shifts and external CDN dependencies completely.
