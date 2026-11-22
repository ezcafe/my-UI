# Test Log: AI Performance, Efficiency & Native REST Client

- **Run slug:** `20261122-ai-perf-efficiency-native-client`
- **Updated:** 2026-11-22T21:04:40+07:00
- **Test path:** full
- **Result:** success
- **Smoke source:** Smoke-Task

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 94 | 94 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 48 | 48 | 0 | PASSED |
| **Total** | **171** | **171** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Milestone Test Details

### Milestone 1: Content Extractor Fast-Path & Truncation
- Fast-path verification: when clean teaser word count $\ge 50$, extractor returns teaser text directly without issuing any HTTP requests.
- Word truncation lowered from 250 to strictly 100 words in `parse_html` and fallbacks.
- Extractor returns clean raw article text without duplicate `Title: ...` prefix.
- Tests: `ai::extractor::tests::test_extractor_teaser_fast_path_skips_http`, `ai::extractor::tests::test_extractor_word_truncation_100`.

### Milestone 2: CLI Stdin Piping & Batch Prompt Cleanliness
- Replaced CLI prompt argument passing with streaming prompt bytes directly via child `stdin` (`Stdio::piped()`) followed by EOF drop.
- Updated `build_args` to omit raw prompt string, eliminating `ARG_MAX` buffer limits.
- Cleaned `format_batch_prompt` to detect and strip redundant `Title: ...` prefixes from article content.
- Tests: `ai::llm::tests::test_llm_cli_pipes_prompt_via_stdin`, `ai::llm::tests::test_batch_prompt_no_duplicate_titles`.

### Milestone 3: Configuration & Credential Management
- Added `llm_provider: String`, `llm_api_key: Option<String>`, `llm_api_base_url: Option<String>` to `Config` with secure defaults.
- Implemented custom `std::fmt::Debug` for `Config` to strictly mask `llm_api_key`, `session_secret`, and `miniflux_api_key` as `[REDACTED]`.
- Updated `.env.example` with documented environment variable keys and usage comments.
- Tests: `config::tests::test_config_llm_provider_and_credentials`.

### Milestone 4: Native HTTP REST Client & Unified Dispatcher
- Implemented `LlmHttpClient` with `reqwest::Client` keep-alive connection pooling and 30-second timeout.
- Supported OpenAI-compatible `/chat/completions` JSON schema with Bearer authentication and `response_format: { type: "json_object" }`.
- Supported Google Gemini REST API `:generateContent` JSON schema with `responseMimeType: "application/json"`.
- Implemented `LlmBackend` wrapping `LlmHttpClient` and `LlmCliRunner` with transparent fallback on error/timeout.
- Tests: `ai::llm::tests::test_llm_http_client_openai_request_and_response`, `ai::llm::tests::test_llm_http_client_gemini_request_and_response`, `ai::llm::tests::test_llm_dispatcher_fallback_to_cli_on_http_failure`.

### Milestone 5: Background Prefetch Keyed Storage & Lower Threshold
- Lowered `PREFETCH_THRESHOLD` from 5 to 1, activating background prefetch for any fresh uncached entries.
- Stored synthesized page executive brief in `page_executive_briefs_v2` with `batch_key` via `save_page_brief_with_key`.
- Tests: `ai::prefetch::tests::test_prefetch_saves_brief_with_page_key`, `ai::prefetch::tests::test_prefetch_triggers_with_single_uncached_item`, `ai::prefetch::tests::test_prefetch_threshold_and_duplicate_avoidance`.

### Milestone 6: Cached Page Executive Brief Synthesis
- Implemented synthesis path in `/hx/executive-brief` when `uncached_set.is_empty()` and page brief is absent from SQLite.
- Gathers cached summaries for visible entries, synthesizes top brief via `synthesize_page_brief`, stores into `page_executive_briefs_v2`, and renders the card + OOB title swaps.
- Tests: `routes::entries::tests::test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty`.

---

## 4. Execution Details & Environment

- **Full test command:** `cargo test --workspace`
  - miniflux-client: 23 passed, 0 failed
  - server (unit): 94 passed, 0 failed
  - server (integration / e2e): 6 passed, 0 failed
  - ui-components: 48 passed, 0 failed
  - Total: 171 passed, 0 failed
- **Lints command:** `cargo clippy --workspace --all-targets -- -D warnings`
  - 0 warnings, exit code 0
- **Final Result:** `verify-pass`
