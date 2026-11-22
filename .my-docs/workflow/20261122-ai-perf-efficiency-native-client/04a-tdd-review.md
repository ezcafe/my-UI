Result: clean

# TDD Review: AI Performance, Efficiency & Native REST Client

- **Stage:** `tdd-review`
- **Reviewed Artifact:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/04-tasks.md)
- **Design Reference:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03-design.md)
- **Status:** Approved (`clean`)

---

## 1. Executive Summary

The planned test suite in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/04-tasks.md) fully covers every architectural component, optimization, and fallback path specified in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03-design.md).

All 11 test cases target explicit contracts across the extraction pipeline, CLI runner stdin piping, configuration parsing, native HTTP REST endpoints (OpenAI and Gemini), resilient fallback dispatching, background prefetch sensitivity, and cached page executive brief synthesis. The suite follows strict Test-Driven Development (TDD) discipline where each test verifies expected behavior and failure conditions before implementation.

---

## 2. Detailed Review of Planned Cases

### Case 1: `test_extractor_teaser_fast_path_skips_http`
- **Target:** `crates/server/src/ai/extractor.rs`
- **Intent:** Verifies that when an RSS teaser contains $\ge 50$ words, `extract_content` extracts clean text immediately without making an outbound HTTP GET call.
- **Assessment:** **Clean**. Eliminates unnecessary network overhead and verifies the fast path.

### Case 2: `test_extractor_word_truncation_100`
- **Target:** `crates/server/src/ai/extractor.rs`
- **Intent:** Verifies that scraped article bodies exceeding 100 words are truncated to 100 words.
- **Assessment:** **Clean**. Direct verification of the token-saving constraint.

### Case 3: `test_batch_prompt_no_duplicate_titles`
- **Target:** `crates/server/src/ai/llm.rs`
- **Intent:** Verifies that batch prompt formatting produces clean article listings without redundant nested `Title: ...` prefixes.
- **Assessment:** **Clean**. Validates clean prompt structure and prevents prompt token bloat.

### Case 4: `test_llm_cli_pipes_prompt_via_stdin`
- **Target:** `crates/server/src/ai/llm.rs`
- **Intent:** Verifies that `LlmCliRunner` passes prompt bytes via child process `stdin` instead of command-line arguments.
- **Assessment:** **Clean**. Guards against OS argument length limits (`ARG_MAX`) and avoids process argument exposure.

### Case 5: `test_llm_http_client_openai_request_and_response`
- **Target:** `crates/server/src/ai/llm.rs`
- **Intent:** Verifies JSON payload serialization (`/v1/chat/completions`) with bearer authorization and structured response extraction.
- **Assessment:** **Clean**. Validates OpenAI endpoint wire format and JSON extraction.

### Case 6: `test_llm_http_client_gemini_request_and_response`
- **Target:** `crates/server/src/ai/llm.rs`
- **Intent:** Verifies Gemini REST payload generation (`:generateContent`) with JSON response schema and parsing.
- **Assessment:** **Clean**. Ensures accurate Google Gemini REST protocol compliance.

### Case 7: `test_llm_dispatcher_fallback_to_cli_on_http_failure`
- **Target:** `crates/server/src/ai/llm.rs`
- **Intent:** Verifies transparent fallback from `LlmHttpClient` to `LlmCliRunner` upon network error, timeout, or 5xx server status.
- **Assessment:** **Clean**. Critical resilience test for Option 1 dynamic dispatch.

### Case 8: `test_prefetch_saves_brief_with_page_key`
- **Target:** `crates/server/src/ai/prefetch.rs`
- **Intent:** Verifies that background prefetch writes the generated page brief into `page_executive_briefs_v2` indexed by `batch_key`.
- **Assessment:** **Clean**. Corrects cache key indexing and prevents stale / missing prefetch cache hits.

### Case 9: `test_prefetch_triggers_with_single_uncached_item`
- **Target:** `crates/server/src/ai/prefetch.rs`
- **Intent:** Verifies that background prefetch triggers when uncached item count $\ge 1$ (lowered from 5).
- **Assessment:** **Clean**. Ensures prefetch sensitivity for timely cache warming.

### Case 10: `test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty`
- **Target:** `crates/server/src/routes/entries.rs`
- **Intent:** Verifies that `/hx/executive-brief` synthesizes a page executive brief from cached article summaries when all page items are already present in SQLite.
- **Assessment:** **Clean**. Guarantees brief availability on repeatedly visited or pre-cached feeds without redundant article scraping.

### Case 11: `test_config_llm_provider_and_credentials`
- **Target:** `crates/server/src/config.rs`
- **Intent:** Verifies parsing of `LLM_PROVIDER`, `LLM_API_KEY`, and `LLM_API_BASE_URL` with secure masking in debug output.
- **Assessment:** **Clean**. Prevents accidental secret leakage and validates startup configuration.

---

## 3. Coverage Matrix

| Design Requirement | Milestone in Tasks | Planned Test Case | Status |
|-------------------|-------------------|-------------------|--------|
| Teaser $\ge 50$ words bypasses HTTP | Milestone 1 | `test_extractor_teaser_fast_path_skips_http` | Covered |
| 100-word truncation | Milestone 1 | `test_extractor_word_truncation_100` | Covered |
| Clean prompt without duplicate titles | Milestone 2 | `test_batch_prompt_no_duplicate_titles` | Covered |
| Pipe prompt via child stdin | Milestone 2 | `test_llm_cli_pipes_prompt_via_stdin` | Covered |
| LLM configuration & secret hygiene | Milestone 3 | `test_config_llm_provider_and_credentials` | Covered |
| Native OpenAI REST client | Milestone 4 | `test_llm_http_client_openai_request_and_response` | Covered |
| Native Gemini REST client | Milestone 4 | `test_llm_http_client_gemini_request_and_response` | Covered |
| Dispatcher fallback to CLI on error | Milestone 4 | `test_llm_dispatcher_fallback_to_cli_on_http_failure` | Covered |
| Prefetch keyed page brief storage | Milestone 5 | `test_prefetch_saves_brief_with_page_key` | Covered |
| Prefetch sensitivity threshold $\ge 1$ | Milestone 5 | `test_prefetch_triggers_with_single_uncached_item` | Covered |
| Synthesize brief from cached summaries | Milestone 6 | `test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty` | Covered |

---

## 4. Conclusion

The planned tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/04-tasks.md) satisfy all verification needs of [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03-design.md).
TDD plan review is **clean**. Ready for Gate B.
