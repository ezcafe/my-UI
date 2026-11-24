# Analysis

**Result:** done

## Context
The application is a high-performance news reader built with Rust Axum, Miniflux GraphQL client, and an SQLite-backed AI caching layer (`AiDb`). It serves executive syntheses, punchy titles, and article briefs using local LLM dispatch (`LlmCliRunner`).

Current profiling reveals three major performance bottlenecks:
1. **Cache Key Desynchronization**: Background prefetching (`AiPrefetchService`) computes keys using `"prefetch"` and only uncached entry IDs, while the foreground HTMX call (`executive_brief_htmx`) computes keys using the active query filter (e.g. `"unread"`) and all entry IDs on the page. Because the keys diverge, the in-flight deduplication mutex fails to prevent duplicate work, and HTMX requests execute redundant LLM batch dispatches while prefetch results are orphaned.
2. **Excessive Batch Token Generation**: The batch prompt requires generating an inverted-pyramid rewrite (30–60 words) for every article on the page (up to 15–20 articles). This generates 1,200–1,500 output tokens per page view, creating a 20–30 second latency penalty for content that the user may never click.
3. **Scraper Latency & Frontend Layout Shifts**: Article content extraction times out after 3.0s, and teasers under 50 words always trigger external HTTP requests. Furthermore, the frontend relies on `cdn.tailwindcss.com` for client-side script compilation, incurring a 150–300ms layout shift.

## Risks
1. **Cache Key Fragility**: If the key generation algorithm between `render_index` / `list_entries_htmx` (prefetch trigger) and `executive_brief_htmx` differs by parameter ordering or status defaults, cache misses will persist. The computation must be centralized in a single helper taking `(status, &[entry_ids])`.
2. **Context Truncation vs Factual Accuracy**: Shortening article inputs from 300 to 120–150 words could risk omitting critical details if lead paragraphs contain boilerplate. This requires combining truncation with strict prompt instructions preventing hallucination and speculative extrapolation.
3. **Detail View Latency**: If full article rewrites are deferred, opening an article detail view must remain instant. The detail view must render the existing 3-sentence executive brief immediately as a primary brief, with on-demand rewrite loading smoothly without blocking page interaction.
4. **Offline / Test Resilience**: Scraper timeouts must be resilient in environments without public internet access, failing gracefully to the available teaser text within 1.5s.

## Open questions
1. **Detail Rewrite Trigger Mechanism**: Should the on-demand article rewrite be triggered automatically via HTMX when the reader opens the article drawer, or generated synchronously if missing?
2. **Batch Generation Scope**: Should the top 1 featured article retain an upfront rewrite in the batch prompt, or should the batch prompt be strictly restricted to `top`, `t` (title), and `b` (brief)?
3. **Static CSS Strategy**: Should the Tailwind runtime CDN script be replaced with a static pre-compiled stylesheet or combined with the existing `annuaire-theme.css`?
