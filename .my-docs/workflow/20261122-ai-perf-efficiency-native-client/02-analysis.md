# Analysis: AI Performance, Efficiency & Native REST Client

## 1. Context & Objectives

The application currently relies on external processes (`tokio::process::Command` executing the `llm` CLI) and synchronous scraping over HTTP for uncached RSS entries. While this design is robust for local prototyping, it introduces observable latency in production:
- Process spawning introduces ~300–800ms of overhead per batch request.
- Scraping third-party article URLs over HTTP can incur 1–3 second timeouts or 403 Forbidden responses (Cloudflare/paywalls), even when Miniflux already provides a rich summary or teaser in the feed payload.
- Prompts contain duplicate title fields and 250-word truncations, resulting in bloated token usage (~5000+ prompt tokens per 15-item batch).
- CLI prompts are passed as command-line arguments, which risk hitting OS-level buffer limitations (`ARG_MAX`).
- Background prefetching does not store page-level briefs with `page_key`, resulting in cache misses during HTMX lazy-loads.
- When all 15 articles on an unread page are already cached, the HTMX brief endpoint fails to generate an executive brief because no "uncached" items exist.

## 2. Technical Nuances & Failure Modes

### Fast-Path Scraper Avoidance
- **Condition**: If an article's teaser already contains $\ge 50$ words, it typically contains the core lead paragraph or comprehensive news synopsis. Firing an outbound HTTP GET to an external domain is wasteful.
- **Risk**: Some teasers may contain raw HTML or navigation snippets.
- **Mitigation**: Sanitize with `miniflux_client::summary::extract_summary(teaser, 2)`. Count words accurately on the clean text. If $\ge 50$ words, use it immediately.

### Prompt Token & Stdin Optimization
- **Token Reduction**: Truncating words to 100 per article keeps token count around ~130 tokens per item, lowering total batch prompt tokens from >5000 to <2000.
- **Title De-duplication**: `format_batch_prompt` prints `Title: ...` for each article. `extractor.rs` also returns `Title: ...\n\n...`. Eliminating this redundancy ensures prompt cleanliness.
- **CLI Stdin Piping**: When executing `Command::new(&self.cli_cmd)`, configuring `.stdin(Stdio::piped())` and writing prompt bytes via `AsyncWriteExt::write_all` avoids passing megabytes through OS process argument lists.

### Prefetch & Page Brief Cache Coherence
- When prefetch processes 15 entries, it knows the batch targets. Saving the page brief with `compute_page_key("prefetch", &target_ids)` ensures that when `executive_brief_htmx` queries for matching status and entry IDs, it gets a direct hit.
- Lowering prefetch threshold from 5 to 1 ensures background jobs execute promptly when fresh unread items arrive.

### Cached-Item Synthesis
- When `uncached_ids` is empty, all articles have summaries in `AiDb`.
- Instead of returning a hidden element, `executive_brief_htmx` queries `db.get_batch_summaries(&entry_ids)` and formats a lightweight synthesis prompt:
  "Synthesize a 1-2 sentence executive brief (<35 words) from these article summaries: ..."
- This requires an ultra-fast LLM call (~150 tokens) and produces the top-level brief instantly.

### Native REST Client Architecture & Resilience
- **OpenAI Endpoint**: `POST {base_url}/chat/completions` with JSON payload:
  `{"model": "...", "messages": [{"role": "user", "content": "..."}], "response_format": {"type": "json_object"}}`.
  Parses `choices[0].message.content`.
- **Gemini Endpoint**: `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}` with JSON payload:
  `{"contents": [{"parts": [{"text": "..."}]}], "generationConfig": {"responseMimeType": "application/json"}}`.
  Parses `candidates[0].content.parts[0].text`.
- **Fallback Guarantee**: If HTTP execution fails (network drop, 5xx server error, 429 rate limit, or timeout), log a warning and fall back to `LlmCliRunner`.
- **Security**: `LLM_API_KEY` must never be logged in cleartext or returned in error messages.

## 3. Risks & Open Questions

- **Risk**: LLM returning invalid JSON formatting despite schema headers.
  - *Mitigation*: Reuse the battle-tested regex parsing fallbacks (`TOP_REGEX` and `ITEM_REGEX`) in `llm.rs`.
- **Risk**: Provider differences between OpenAI base URLs (e.g. Ollama or OpenRouter vs official OpenAI).
  - *Mitigation*: Allow `LLM_API_BASE_URL` override with default `https://api.openai.com/v1`.
- **Decision Status**: All architectural options were confirmed during Ideation by user:
  "Both: All quick-wins (scraping fast-path, token reduction, cache fixes) + Native HTTP REST client (OpenAI & Gemini) with CLI fallback".

## Result

Result: done
