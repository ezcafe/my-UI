# Skim

**Result:** done

## Findings

### 1. Page Cache Key Misalignment (`AiPrefetchService` vs `executive_brief_htmx`)
- **Prefetch key generation** (`crates/server/src/ai/prefetch.rs:72`): `crate::ai::compute_page_key("prefetch", &target_ids)` where `target_ids` are only uncached entries (up to 20).
- **HTMX endpoint key generation** (`crates/server/src/routes/entries.rs:626`): `compute_page_key(status_str, &entry_ids)` where `status_str` is `"unread"` (or query status) and `entry_ids` are all entries on the current page.
- **Impact**: Prefetching and foreground HTMX dispatches never share the same cache key or the same in-flight mutex entry (`ai.in_flight`), causing duplicate LLM dispatches, race conditions, and complete cache misses for the executive brief.

### 2. Batch LLM Token Consumption & Truncation Cap
- **Current Prompt** (`crates/server/src/ai/llm.rs:98-160`): Truncates article inputs at 300 words (`LlmCliRunner::truncate_words(clean_content, 300)`).
- **Mandatory Rewrite in Batch**: Prompt forces generation of `"r"` (inverted pyramid rewrite, 30–60 words) for *every* new article in the batch, producing 1,200–1,500 unnecessary output tokens and adding 20–30s of LLM generation latency on page views.
- **Hallucination / Grounding**: Prompt currently lacks explicit negative constraints against speculative extrapolation.

### 3. Scraper Fast-Path & Timeouts (`ArticleExtractor`)
- **HTTP Timeout** (`crates/server/src/ai/extractor.rs:20`): Set to `Duration::from_secs(3)`, risking blocking worker threads on slow external news domains.
- **Teaser Fast-Path** (`crates/server/src/ai/extractor.rs:55`): Currently requires `clean_teaser` word count >= 50 words to bypass network fetch.

### 4. Frontend Asset & Layout Shift
- **Tailwind CDN** (`crates/ui-components/templates/base.html:14`): Loads `https://cdn.tailwindcss.com` at runtime with an inline script block, causing runtime DOM scanning, render blocking, and 150–300ms layout shift.
