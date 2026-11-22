# Codebase Skim: AI Performance, Efficiency & Native REST Client

## Core Modules & Touchpoints

1. **Content Extraction (`crates/server/src/ai/extractor.rs`)**:
   - `extract_content`:
     - Inspect teaser before initiating an outbound HTTP GET. If extractive teaser words $\ge 50$ (or raw teaser words $\ge 50$), return teaser content directly, avoiding external network latency and scraper timeouts.
     - Lower word truncation limit from 250 words down to 100 words in `parse_html` and extraction fallbacks.
     - Clean up content formatting to avoid prepending redundant `Title: ...` prefixes that duplicate prompt templates.

2. **LLM Engine & Execution (`crates/server/src/ai/llm.rs`)**:
   - `LlmCliRunner`:
     - Switch process execution from command-line arguments to `tokio::process::Command::stdin(Stdio::piped())`.
     - Write prompt bytes asynchronously into child process `stdin` to prevent `E2BIG` / command line argument length limits.
   - `LlmHttpClient` & Unified Dispatcher:
     - Implement native REST client using `reqwest::Client` with connection pooling and 30-second timeout.
     - Support OpenAI-compatible format: `POST {base_url}/chat/completions` with `Bearer {api_key}` and `response_format: {"type": "json_object"}`.
     - Support Gemini REST format: `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}` with `responseMimeType: "application/json"`.
     - Implement fallback dispatcher: dispatch to HTTP client if `LLM_API_KEY` is present; if request fails or returns an error status/timeout, log warning and automatically fall back to `LlmCliRunner`.
   - `format_batch_prompt`:
     - Optimize prompt instructions and token layout.
     - Ensure clean item formatting without repeated title headers.

3. **Background Prefetch (`crates/server/src/ai/prefetch.rs`)**:
   - Change `PREFETCH_THRESHOLD` from 5 to 1 so background prefetching triggers immediately when new articles appear.
   - When saving top brief in `run_prefetch`, invoke `ai.db.save_page_brief_with_key(&batch_key, &batch_res.top).await` to ensure background-precomputed page briefs are discoverable by `executive_brief_htmx`.

4. **HTMX Executive Brief Endpoint (`crates/server/src/routes/entries.rs`)**:
   - In `executive_brief_htmx`:
     - Detect when `uncached_ids` is empty but the page brief for `page_key` is not yet saved in SQLite.
     - Instead of returning `<div id="page-executive-brief" class="hidden"></div>`, load existing item summaries via `ai_service.db.get_batch_summaries(&entry_ids)` and synthesize a concise page-level brief from the cached summaries.
     - Save the synthesized page brief into `page_executive_briefs_v2` with `page_key`.

5. **Configuration Subsystem (`crates/server/src/config.rs`)**:
   - Add configuration fields: `llm_provider`, `llm_api_key`, `llm_api_base_url`.
   - Support environment variables: `LLM_PROVIDER` (defaulting to "openai"), `LLM_API_KEY`, `LLM_API_BASE_URL`.
   - Mask `llm_api_key` in debug/display implementations to prevent credential leakage.

6. **AI Service Wiring (`crates/server/src/ai/mod.rs`)**:
   - Initialize unified `LlmBackend` or `LlmDispatcher` in `AiService::new(config)`.
   - Expose consistent `summarize_batch` and `summarize_single` interfaces.

## Test Infrastructure

- Extractor tests in `crates/server/src/ai/extractor.rs`.
- LLM tests in `crates/server/src/ai/llm.rs` (mocking HTTP endpoints via wiremock or custom client handlers).
- Prefetch tests in `crates/server/src/ai/prefetch.rs`.
- Route tests in `crates/server/src/routes/entries.rs`.

## Result

Result: done
