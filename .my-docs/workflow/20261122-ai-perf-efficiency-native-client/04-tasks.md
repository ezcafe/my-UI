# Tasks: AI Performance, Efficiency & Native REST Client

## Planned Tests

- [x] `test_extractor_teaser_fast_path_skips_http`: Verifies that an article teaser with $\ge 50$ words returns extracted text without triggering an external HTTP GET request.
- [x] `test_extractor_word_truncation_100`: Verifies that long article contents are truncated to strictly 100 words (down from 250).
- [x] `test_batch_prompt_no_duplicate_titles`: Verifies that `format_batch_prompt` outputs clean article content without redundant nested `Title: ...` prefixes.
- [x] `test_llm_cli_pipes_prompt_via_stdin`: Verifies that `LlmCliRunner` writes prompt bytes directly to child process `stdin` and does not pass the prompt string in CLI arguments.
- [x] `test_llm_http_client_openai_request_and_response`: Verifies OpenAI-compatible JSON request construction (`/v1/chat/completions`) and structured JSON response parsing.
- [x] `test_llm_http_client_gemini_request_and_response`: Verifies Gemini REST API JSON request construction (`:generateContent`) and structured JSON response parsing.
- [x] `test_llm_dispatcher_fallback_to_cli_on_http_failure`: Verifies that when the HTTP client encounters a network error, 5xx status, or timeout, the unified dispatcher seamlessly falls back to `LlmCliRunner`.
- [x] `test_prefetch_saves_brief_with_page_key`: Verifies that `run_prefetch` stores the synthesized page executive brief in `page_executive_briefs_v2` with the computed `batch_key`.
- [x] `test_prefetch_triggers_with_single_uncached_item`: Verifies that background prefetch activates when the number of uncached entries is $\ge 1$ (lowered threshold from 5 to 1).
- [x] `test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty`: Verifies that `/hx/executive-brief` loads cached summaries and synthesizes the top page brief when all visible entries are already cached in SQLite.
- [x] `test_config_llm_provider_and_credentials`: Verifies environment variable parsing for `LLM_PROVIDER`, `LLM_API_KEY`, and `LLM_API_BASE_URL` with secure defaults.

---

## Milestone 1: Content Extractor Fast-Path & Truncation

- [x] **Task 1.1: TDD tests for teaser fast-path and 100-word truncation**
  - Path: `crates/server/src/ai/extractor.rs`
  - Action: Add tests `test_extractor_teaser_fast_path_skips_http` and `test_extractor_word_truncation_100`. Verify failure before implementation.
- [x] **Task 1.2: Implement teaser fast-path and 100-word truncation**
  - Path: `crates/server/src/ai/extractor.rs`
  - Action:
    - In `extract_content`, check if `miniflux_client::summary::extract_summary(teaser, 2)` or raw `teaser` has $\ge 50$ words. If so, return directly without network fetch.
    - Change word truncation threshold from 250 words down to 100 words in `parse_html` and fallbacks.
    - Ensure clean text return without duplicate `Title: ...` formatting.
  - Verification: `cargo test --package server ai::extractor` passes.

---

## Milestone 2: CLI Stdin Piping & Batch Prompt Cleanliness

- [x] **Task 2.1: TDD tests for stdin piping and prompt de-duplication**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Add tests `test_llm_cli_pipes_prompt_via_stdin` and `test_batch_prompt_no_duplicate_titles`.
- [x] **Task 2.2: Implement stdin piping in LlmCliRunner and clean format_batch_prompt**
  - Path: `crates/server/src/ai/llm.rs`
  - Action:
    - Update `build_args` to omit the raw prompt string.
    - Configure `.stdin(Stdio::piped())` and write prompt asynchronously via `tokio::io::AsyncWriteExt::write_all`.
    - Adjust `format_batch_prompt` to avoid redundant title prefixes.
  - Verification: `cargo test --package server ai::llm` passes.

---

## Milestone 3: Configuration & Credential Management

- [x] **Task 3.1: TDD tests for LLM provider and credentials**
  - Path: `crates/server/src/config.rs`
  - Action: Add `test_config_llm_provider_and_credentials` validating `LLM_PROVIDER`, `LLM_API_KEY`, and `LLM_API_BASE_URL`.
- [x] **Task 3.2: Implement Config fields**
  - Path: `crates/server/src/config.rs`
  - Action: Add `llm_provider: String`, `llm_api_key: Option<String>`, `llm_api_base_url: Option<String>` to `Config`. Ensure `llm_api_key` is masked in debug output.
  - Verification: `cargo test --package server config` passes.

---

## Milestone 4: Native HTTP REST Client & Unified Dispatcher

- [x] **Task 4.1: TDD tests for OpenAI REST client, Gemini REST client, and fallback**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Add tests for JSON payload serialization, response deserialization, regex fallback for malformed responses, and automatic fallback to CLI runner.
- [x] **Task 4.2: Implement LlmHttpClient and LlmBackend**
  - Path: `crates/server/src/ai/llm.rs`
  - Action:
    - Implement `LlmHttpClient` with `reqwest::Client` (30s timeout, connection pooling).
    - Support OpenAI `/v1/chat/completions` and Gemini `:generateContent`.
    - Implement `LlmBackend` wrapping `LlmHttpClient` and `LlmCliRunner` with transparent fallback on error/timeout.
  - Verification: `cargo test --package server ai::llm` passes.

---

## Milestone 5: Background Prefetch Keyed Storage & Lower Threshold

- [x] **Task 5.1: TDD tests for prefetch threshold and keyed storage**
  - Path: `crates/server/src/ai/prefetch.rs`
  - Action: Add tests asserting prefetch runs for 1 item and saves page brief with computed `batch_key`.
- [x] **Task 5.2: Update prefetch implementation**
  - Path: `crates/server/src/ai/prefetch.rs`
  - Action:
    - Change `PREFETCH_THRESHOLD` from 5 to 1.
    - Save top brief with `ai.db.save_page_brief_with_key(&batch_key, &batch_res.top).await`.
  - Verification: `cargo test --package server ai::prefetch` passes.

---

## Milestone 6: Cached Page Executive Brief Synthesis

- [x] **Task 6.1: TDD test for brief synthesis on cached feeds**
  - Path: `crates/server/src/routes/entries.rs`
  - Action: Add test verifying `/hx/executive-brief` generates top brief from cached summaries when `uncached_ids` is empty.
- [x] **Task 6.2: Implement cached synthesis in executive_brief_htmx**
  - Path: `crates/server/src/routes/entries.rs`
  - Action:
    - When `uncached_set.is_empty()` and page brief is absent in `AiDb`:
      - Fetch summaries via `ai_service.db.get_batch_summaries(&entry_ids).await`.
      - Build synthesis prompt and call `ai_service.llm.synthesize_page_brief(&cached_summaries).await`.
      - Store brief in `page_executive_briefs_v2` with `page_key`.
      - Return rendered card and OOB title spans.
  - Verification: `cargo test --package server` passes.
