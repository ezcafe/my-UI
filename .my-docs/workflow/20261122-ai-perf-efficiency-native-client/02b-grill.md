# Grill: AI Performance, Efficiency & Native REST Client

## Settled Decisions & Architectural Alignment

### 1. Unified LLM Execution Engine (Settled)
- **Decision**: Dual-Backend Engine (Native HTTP REST + CLI Fallback).
- **Behavior**:
  - If `LLM_API_KEY` is provided, the engine attempts native HTTP REST requests (`reqwest`) directly to OpenAI-compatible endpoints or Google Gemini REST API.
  - If `LLM_API_KEY` is omitted, or if an HTTP call encounters a network failure, 5xx server error, or timeout, the engine falls back seamlessly to `LlmCliRunner`.
- **Configuration**:
  - `LLM_PROVIDER`: `"openai"` (default) or `"gemini"`.
  - `LLM_API_KEY`: API key for direct HTTP authentication.
  - `LLM_API_BASE_URL`: Optional custom base URL (defaulting to `https://api.openai.com/v1`).
- **User Confirmation**: Confirmed in Ideation ("Both: All quick-wins ... + Native HTTP REST client (OpenAI & Gemini) with CLI fallback").

### 2. Fast-Path Extractor & Scraping Avoidance (Settled)
- **Decision**: Skip external HTTP GET scraping when the RSS teaser already contains $\ge 50$ words.
- **Behavior**: Evaluates word count of the sanitized teaser. If $\ge 50$ words, bypasses external HTTP GET and returns teaser text directly, avoiding 1–3s timeouts and paywall/Cloudflare blocks.

### 3. Prompt Footprint & CLI Stdin Piping (Settled)
- **Decision**: Lower truncation limit from 250 words down to 100 words; eliminate redundant `Title: ...` headers in prompt formatting; pipe prompts to CLI via `stdin`.
- **Behavior**: Truncation cuts token count by >60%; stdin piping prevents OS `ARG_MAX` buffer overflow.

### 4. Background Prefetch Fixes (Settled)
- **Decision**: Lower threshold to 1; store top brief in `page_executive_briefs_v2` with `batch_key`.
- **Behavior**: Ensures background prefetching triggers for new articles proactively and page briefs are discoverable on page load.

### 5. Synthesize Brief on Cached Feeds (Settled)
- **Decision**: Synthesize page-level executive brief from cached article summaries when all items on an unread page are already cached in SQLite.
- **Behavior**: When `uncached_ids` is empty, `/hx/executive-brief` loads cached summaries and calls the LLM with a brief synthesis prompt, guaranteeing brief availability without blank cards.

---

## Frontier Status

No open frontier decisions remain. All design options and provider scopes have been confirmed.

## Result

Result: frontier-empty
