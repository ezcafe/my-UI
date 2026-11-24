# Grill: Details Page AI Rewrite with Batch Pre-request

## 1. Settled Decisions & Architectural Alignment

### Detail View Content Replacement (Settled: Option B)
- **User Choice**: Option B (Complete Replacement).
- **Behavior**: The article detail page will exclusively display the concise, high-impact **AI Rewrite** as the body content. Raw unformatted article HTML (`sanitized_content`) is omitted from the details view DOM.
- **Reference**: The top-bar external "Original" link serves as the definitive gateway to the original publisher's website.

### Comparative Evaluation (User Provenance Query)
The user requested: *"Compare the batch llm request and request details on details page view, what is better for token usage, and loading time?"*

- **Token Usage**: **Batch pre-request is significantly better**.
  - System instructions (~300 tokens) are amortized once across 20 items (~15 tokens/item), rather than repeated on every article click.
  - Generates titles, list-card briefs, and detail rewrites together in one roundtrip (~5,500 total tokens for 20 articles vs 11,000–22,000 tokens for on-demand calls).
- **Loading Time**: **Batch pre-request is >300&times; faster**.
  - Local SQLite retrieval takes **< 5ms** on article click, providing instant 60fps opening with zero loading spinners or network stalls.
  - On-demand requests would force the user to wait 1.5s–4.5s with a loading spinner every time an article is opened.

---

## 2. Settled Frontier Decisions (Round 1)

All questions resolved with user approval:

- **Q1: Fallback Display**: **Option 1A** approved. If an article detail is opened before the rewrite is ready (or on upstream failure), gracefully display the 3-sentence Executive Brief / teaser.
- **Q2: Input Content Cap**: **Option 2A (Adjusted to 300 words)** approved. Cap each article's input text at the first **300 words** (or teaser if $\ge 50$ words) to give the LLM richer context for inverted pyramid generation while bounding the batch prompt size.
- **Q3: Prefetch Trigger Timing**: **Option 3A** approved. Trigger background batch prefetching asynchronously on active page load for uncached entries via `AiPrefetchService`.

---

## Result

Result: frontier-empty
