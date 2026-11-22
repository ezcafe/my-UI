Result: clean

# General Design Review: AI Performance, Efficiency & Native REST Client

**Date:** 2026-11-22  
**Workflow Slug:** `20261122-ai-perf-efficiency-native-client`  
**Review Stage:** `design-review`  
**Target Artifacts:**  
- [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03-design.md)
- [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/04-tasks.md)
- [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03a-api-contract-review.md)
- [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03a-db-design-review.md)

---

## 1. Executive Summary

This cross-cutting design review evaluates the architecture, end-to-end task decomposition, and resilience mechanisms designed in `03-design.md` and `04-tasks.md`.

- **Overall Verdict**: **`Result: clean`** (Zero Critical, Zero Major defects).
- **Core Strengths**:
  - **Comprehensive Resiliency**: Transparent fallback from native HTTP REST client to piped `stdin` CLI runner guarantees uninterrupted AI summaries during network hiccups or upstream API failures.
  - **Significant Latency & Cost Reduction**: Teaser word-count fast-path ($\ge 50$ words) removes unnecessary external HTTP scraping; reducing truncation to 100 words cuts prompt token overhead by over 60%.
  - **Seamless Task Alignment**: All 6 milestones in `04-tasks.md` map directly to component designs in `03-design.md` with explicit TDD test cases and verification commands.
  - **Isolated Reviews Verified**: Both Isolated API review (`03a-api-contract-review.md`) and Database review (`03a-db-design-review.md`) are verified and recorded as `clean`.

---

## 2. Review Checklist & Findings

| Category | Assessment | Status | Notes |
| :--- | :--- | :---: | :--- |
| **Isolated Reviews** | API & DB reviews completed and clean | **PASS** | `03a-api-contract-review.md` and `03a-db-design-review.md` both clean |
| **System Architecture** | Component boundaries & data flow clarity | **PASS** | Clear separation between Extractor, Dispatcher, HTTP client, and DB |
| **Resilience & Fallbacks** | Network outage, timeout, and process error handling | **PASS** | `LlmBackend` catches HTTP errors and immediately cascades to CLI runner |
| **Task & TDD Completeness** | Granular milestones with test-first requirements | **PASS** | 11 planned tests covering unit and integration behavior |
| **Buffer & Scale Limits** | Process piping vs OS argument limits | **PASS** | Stdin streaming replaces CLI arguments, removing `ARG_MAX` limits |
| **Security & Secrets** | Safe credential handling & injection prevention | **PASS** | `LLM_API_KEY` masked, no shell expansion, credentials not logged |
| **Database & Concurrency** | SQLite contention, locking, and uniqueness | **PASS** | `page_executive_briefs_v2` unique index and WAL mode handle concurrency |

---

## 3. Cross-Cutting Architectural Analysis

### 3.1 Content Extraction & Token Efficiency
- **Fast-path Scraper Avoidance**: Checking teaser word count ($\ge 50$ words) before dispatching an HTTP GET prevents unnecessary outbound network roundtrips and protects against scraper timeouts.
- **De-duplication & Truncation**: Dropping nested `Title: ...` prefixes and reducing body truncation from 250 words to 100 words directly lowers token consumption while preserving the context required for high-quality summaries.

### 3.2 Dual-Backend Dispatcher (`LlmBackend`)
- **Connection Reuse**: Utilizing `reqwest::Client` with internal connection pooling and keep-alive headers minimizes TLS handshake overhead.
- **Zero-Downtime Fallback**: If an HTTP call fails (e.g. 429 Too Many Requests, 502/503/504 Bad Gateway, or 30s timeout), `LlmBackend` logs a warning and transparently delegates the batch to `LlmCliRunner`.
- **Stdin Stream Safety**: Writing prompt bytes directly to child process `stdin` avoids buffer overflow and environment variable leaking.

### 3.3 Cache Sensitivity & Synthesis Guarantees
- **Sensitivity Threshold**: Lowering `PREFETCH_THRESHOLD` from 5 to 1 ensures prefetching activates whenever new entries arrive.
- **Cache Hit Synthesis**: When all visible entries are already cached in SQLite, the system triggers a lightweight synthesis prompt using existing summaries instead of returning an empty card.

---

## 4. Non-Blocking Recommendations for Implementation

1. **Gemini Auth Header**: In `LlmHttpClient::gemini`, prefer passing the API key via the HTTP header `x-goog-api-key: {api_key}` rather than query parameter `?key={api_key}` to keep access logs clean.
2. **Database Method Naming**: `03a-db-design-review.md` notes that `AiDb` defines `save_page_brief_by_key`. Implementers can either use `save_page_brief_by_key` directly or add a 1-line forwarding alias `save_page_brief_with_key`.
3. **Atomic Batch Prefetching**: During Milestone 5, update `prefetch.rs` to persist page briefs atomically alongside batch summaries in a single SQLite transaction.

---

## 5. Conclusion

The design and tasks are complete, robust, secure, and ready for development.

- **Gate B Readiness**: All design prerequisites and reviews are fully satisfied.
