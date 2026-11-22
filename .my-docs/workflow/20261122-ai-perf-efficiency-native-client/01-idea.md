# Ideation: AI Performance, Efficiency & Native REST Client

## Executive Summary

This initiative optimizes AI pipeline throughput, removes subprocess and scraping bottlenecks, and introduces a native HTTP REST client for LLM providers (OpenAI-compatible and Gemini REST API) with seamless CLI fallback.

Key takeaways:
- **Zero subprocess overhead**: Native `reqwest` HTTP client communicates directly with OpenAI-compatible and Gemini REST APIs, eliminating process spawn latency while retaining the CLI runner as an automatic fallback.
- **Fast-path network scraping avoidance**: If an article's RSS teaser is already substantial ($\ge 50$ words), the pipeline skips external HTTP GET scraping, saving network roundtrips and preventing scraper timeouts.
- **Compact token footprint**: Extracted content is truncated to 100 words (down from 250), and duplicate title headers in prompts are eliminated, drastically cutting prompt tokens and LLM latency.
- **Robust CLI stdin piping**: For CLI fallback invocations, prompts are piped via standard input (`stdin`) rather than shell arguments, bypassing command-line length limits.
- **Correct prefetch page caching**: Background prefetch saves page-level executive briefs with the computed `page_key` so HTMX fast-path cache lookups succeed immediately.
- **Reliable brief generation on cached feeds**: If all articles on an unread page are already cached in SQLite, the executive brief synthesizes directly from the cached summaries instead of returning an empty placeholder.
- **Responsive prefetch trigger**: The prefetch activation threshold is reduced from 5 down to 1 item, ensuring fresh feed items are processed proactively.

---

## Scope & Boundaries

### Goals
- **Option 1 Quick-Wins**:
  1. Truncate extracted content words to 100 (down from 250).
  2. Eliminate duplicate "Title: ..." strings inside prompt formatting.
  3. Skip external web page scraping when the RSS teaser has $\ge 50$ words.
  4. Pipe prompt text via standard input (`stdin`) in `LlmCliRunner`.
  5. Ensure background prefetch saves page briefs with `page_key` into `page_executive_briefs_v2`.
  6. Synthesize page-level executive briefs from cached item summaries when all page items are already cached.
  7. Lower the prefetch threshold (e.g. from 5 to 1) so single or few new items trigger background processing.
- **Option 2 Native HTTP LLM Client**:
  - Implement a direct `reqwest` HTTP client supporting OpenAI-compatible (`/v1/chat/completions`) and Gemini REST API endpoints with structured JSON responses.
  - Automatically fallback to `LlmCliRunner` if HTTP configuration is absent or if a network request fails.
  - Support configuration via environment variables (e.g., `LLM_API_KEY`, `LLM_API_BASE_URL`, `LLM_PROVIDER`).

### Non-Goals
- Building custom fine-tuned models or self-hosting local weight servers inside this crate.
- Changing the Miniflux upstream synchronization protocol.
- Altering the visual layout of Annuaire theme cards or the reader UI.

### Anti-Goals
- Breaking existing CLI workflows (users without API keys must continue working via `llm` CLI seamlessly).
- Blocking HTTP responses while scraping paywalled or slow third-party websites.
- Emitting unvalidated JSON or bypassing database transactions.

---

## Core Requirements & Behavior

### 1. Extractor & Prompt Optimizations
- **Teaser Fast-Path**: In `ArticleExtractor::extract_content`, if `miniflux_client::summary::extract_summary(teaser, 2)` or raw teaser contains $\ge 50$ words, use the teaser directly without firing an HTTP GET to the external URL.
- **Word Truncation**: Lower word truncation limit from 250 words down to 100 words.
- **De-duplication of Titles**: Ensure `extractor.rs` and `llm.rs` do not produce nested `Title: ...` headers in batch prompts.

### 2. Prefetch & Brief Caching Fixes
- **Page Key Mapping in Prefetch**: When `AiPrefetchService::run_prefetch` generates a top brief, save it to `page_executive_briefs_v2` with the computed `batch_key` (matching `status` and entry IDs).
- **Cached Page Brief Generation**: In `executive_brief_htmx`, if `uncached_ids` is empty but the page brief for `page_key` is not yet saved, format a synthesis prompt from the cached article summaries and generate the page brief, rather than returning a hidden `div`.
- **Lower Prefetch Threshold**: Reduce the minimum uncached entries threshold from 5 to 1.

### 3. Native HTTP REST Client with CLI Fallback
- **Unified LLM Dispatcher**: Introduce an abstraction (e.g., `LlmService` or `LlmBackend`) that decides whether to dispatch via HTTP or CLI.
- **HTTP Client**:
  - Uses `reqwest::Client` with timeout (e.g., 30s) and keep-alive connection pooling.
  - For OpenAI-compatible endpoints: `POST /v1/chat/completions` with `response_format: { "type": "json_object" }`.
  - For Gemini endpoints: `POST /v1beta/models/{model}:generateContent` with `responseMimeType: "application/json"`.
- **CLI Fallback**:
  - If `LLM_API_KEY` is not set, dispatch immediately to `LlmCliRunner`.
  - If HTTP returns a connection error, 5xx, or timeout, log a warning and fall back to `LlmCliRunner`.
  - `LlmCliRunner` writes prompt into child process `stdin` instead of command-line arguments.

---

## Architectural Decisions & Trade-Offs

### Decision 1: LLM Execution Engine Architecture

- **Option A (Recommended): Dual-Backend Engine (Native HTTP REST + CLI Fallback)**
  - **Explanation**: Create a unified backend abstraction where HTTP is attempted first when configured (`LLM_API_KEY`), and CLI (`llm` or custom command) is used when HTTP is unconfigured or encounters an unrecoverable network failure.
  - **Concrete Configuration**:
    ```bash
    # .env.example additions
    LLM_PROVIDER=openai # or gemini, ollama, cli
    LLM_API_KEY=your-api-key-here
    LLM_API_BASE_URL=https://api.openai.com/v1 # optional custom base URL
    ```
  - **Pros**: Zero subprocess latency in production (saving ~300–800ms per call); robust backward compatibility with existing CLI setups; zero disruption for users running local `llm` CLI.
  - **Cons**: Requires maintaining two execution paths and schema payload mappers (OpenAI and Gemini REST formats).

- **Option B: Replace CLI Entirely with HTTP Client Only**
  - **Explanation**: Remove `LlmCliRunner` and require all deployments to provide an HTTP API endpoint (or local Ollama HTTP endpoint).
  - **Pros**: Single code path to maintain.
  - **Cons**: Breaks setups that rely on local CLI plugins or existing `llm` CLI authentication.

> **Recommendation**: **Option A**. Providing native HTTP with CLI fallback offers maximum performance without breaking existing environments.

---

### Decision 2: Synthesis of Page Brief when All Items are Cached

- **Option A (Recommended): Synthesize from Cached Summaries**
  - **Explanation**: If a user lands on a page where all 15 articles already have individual summaries cached in SQLite, but the composite `page_key` brief does not exist yet: pass the 15 cached summaries (short 1–2 sentence texts) to the LLM to generate just the top executive brief.
  - **Example Prompt**:
    ```text
    Synthesize a 1-2 sentence executive brief (<35 words) from these article summaries:
    [ID 1] Title: ... Summary: ...
    ```
  - **Pros**: Instant token savings (sending short summaries instead of full article texts); ensures the executive brief is never blank on cached pages.
  - **Cons**: Requires one lightweight LLM call to synthesize the top brief if prefetch did not compute it with the exact page key.

- **Option B: Do Nothing and Leave Brief Blank**
  - **Explanation**: Retain current behavior where empty uncached items result in a hidden container.
  - **Pros**: Zero additional LLM calls.
  - **Cons**: Degrades user experience; users see no Executive Brief simply because articles were browsed or prefetched earlier.

> **Recommendation**: **Option A**. Synthesizing from cached summaries is extremely fast and low-token, guaranteeing 100% executive brief availability.

---

## Has UI / API / DB / auth/PII/schema Assessment

- **Has UI**: **yes** (Executive brief rendering, skeleton states, and HTMX swap behavior).
- **Has API**: **yes** (New direct HTTP client making external REST requests to OpenAI/Gemini; HTMX executive brief endpoint adjustments).
- **Has DB**: **yes** (SQLite queries for `page_executive_briefs_v2`, prefetch key persistence, and batch summary lookup).
- **Has auth/PII/schema**: **yes** (`LLM_API_KEY` handling, environment config expansion, avoiding secret logging).

---

## Framing & Open Questions for User

To confirm shared understanding before moving to Gate A and detailed technical design:

> **Option to bypass**: If you agree with the goals, scope, and recommended options above, you can reply **"just start"** or **"confirmed"** to proceed immediately to Gate A and Design.

### Core Question for User:
1. **Provider Scope**: For the native HTTP REST client in Option 2, should we prioritize **OpenAI-compatible REST format** (`/v1/chat/completions` which also covers Ollama, Groq, OpenRouter, vLLM, DeepSeek) and **Google Gemini REST format**, or is there any additional provider API you would like supported out of the box?
