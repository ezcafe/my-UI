# Idea / framing

## Intent
Optimize end-to-end performance, page loading speed, and AI token consumption across backend and frontend, while ensuring strict factual correctness of news summaries and keeping the reader application stable and responsive.

## Goals
1. **Unify Page Cache Keys**: Synchronize cache key calculation between background prefetching (`AiPrefetchService`) and foreground HTMX calls (`executive_brief_htmx`), completely eliminating duplicate LLM dispatches and lock contention.
2. **On-Demand Detailed Rewrites**: Restrict batch LLM calls to generating page-level syntheses, punchy titles, and 3-sentence card briefs. Defer the 30–60 word inverted-pyramid rewrite to on-demand article detail views (or top featured items), saving 1,200–1,500 output tokens per page view and slashing latency by 20–30s.
3. **Trim Batch Input Token Cap**: Reduce per-article input truncation from 300 words down to 120–150 words. News leads provide all necessary context, cutting input tokens by ~50% (~3,000 tokens saved per batch call).
4. **Strict Factual Grounding (Anti-Hallucination)**: Introduce explicit negative constraints in the prompt requiring the LLM to strictly adhere to provided text without extrapolation or speculation, guaranteeing news truthfulness even on brief teasers.
5. **Scraper Fast-Path & Timeout Protection**: Lower the teaser fast-path threshold to 25 words and shorten HTTP scraper timeouts to 1.5s, preventing hung requests when external news sites are slow or blocked.
6. **Frontend Static Asset & Layout Shift Optimization**: Replace runtime client-side Tailwind CDN compilation with prebuilt/preloaded assets or optimized styling to eliminate 150–300ms layout shifts and render blocking.

## Non-goals
- Rewriting Miniflux upstream synchronization logic or database schemas outside of AI summaries and cache keys.
- Changing the Annuaire editorial design system or core visual identity.
- Removing AI features or disabling magazine/list views.

## Anti-goals
- Sacrificing factual accuracy or depth for brevity.
- Introducing fragile third-party dependencies or external JS CDNs that break in offline/restricted environments.
- Breaking existing tests or violating backward compatibility for stored summaries in SQLite.

## Constraints
- Rust Axum backend with SQLite (WAL mode, r2d2 connection pooling).
- Multi-source multilingual parity (English and Vietnamese parity strictly preserved).
- Visx / Lightweight Charts compliance for charts; modern container queries / CSS grid for responsive UI (no hardcoded device breakpoints).
- Zero secrets in repo or environment files.
- Gate C strictly blocking before commit/merge.

## Has UI (tentative)
- Yes (modifies template asset loading, detail rewrite triggers, and brief rendering).

## Shared understanding
- [x] User confirmed (brainstorm hard-gate)
- User confirmed Option 2 and explicit architectural choice: "load on-demand" for detailed article rewrites (no wasted batch output tokens, instant page views).
