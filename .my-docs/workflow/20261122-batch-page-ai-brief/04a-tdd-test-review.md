Result: clean

# TDD Test Review: Batch Page AI Brief & Multilingual Executive Briefs

## 1. Executive Summary & Review Verdict

The planned test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-batch-page-ai-brief/04-tasks.md) provide complete, high-fidelity test coverage for all architecture and functional requirements defined in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-batch-page-ai-brief/03-design.md).

- **Verdict**: **clean**.
- **Coverage**: 100% of required functional areas, edge cases, failure modes, and performance optimizations have dedicated TDD test tasks.
- **TDD Sequencing**: Every implementation task is preceded by an explicit failing test task.

---

## 2. Requirement Coverage Matrix

| Requirement Area | Design Specification ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-batch-page-ai-brief/03-design.md)) | Planned Task ([04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-batch-page-ai-brief/04-tasks.md)) | Assessment |
|---|---|---|---|
| **Config & Auth Decoupling** | `AI_ENABLED=true` independent of `AUTH_ENABLED` | Task 1.1 (`crates/server/src/config.rs`) | **Full** |
| **AiDb v3 Schema & Migration** | Bump `user_version = 3`, table `page_executive_briefs_v2`, single index on `created_at` | Task 2.1 (`crates/server/src/ai/db.rs`) | **Full** |
| **AiDb Upsert Idempotency** | Upsert on `page_key` without `SQLITE_CONSTRAINT_UNIQUE` errors | Task 2.1 (`test_page_brief_by_key_upsert`) | **Full** |
| **Batched Uncached Query** | Single SQL query using `WHERE entry_id IN (...)` | Task 2.1 (`test_get_unprocessed_entry_ids`) | **Full** |
| **Atomic Transactional Write** | Save summaries + page brief in 1 `conn.transaction()?` | Task 2.1 (`test_save_batch_results_transaction`) | **Full** |
| **Brief Retention Pruning** | Housekeeping helper deleting records older than retention threshold | Task 2.1 (`test_prune_page_briefs`) | **Full** |
| **Deterministic Page Key** | SHA256 hashing of status + visible entry IDs | Task 2.3 (`crates/server/src/ai/llm.rs`) | **Full** |
| **Multilingual Prompt Rule** | Prompt strictly forces response in article source language | Task 3.1 (`crates/server/src/ai/llm.rs`) | **Full** |
| **Robust Batch JSON Parsing** | Strip markdown fences (` ```json `), preambles, and postambles | Task 3.1 (`crates/server/src/ai/llm.rs`) | **Full** |
| **HTMX Filter Reconciliation** | Strict rejection of `today`, `read`, `starred`, `all`, search | Task 4.2 (`is_brief_eligible_filter` tests) | **Full** |
| **Empty Entries Guard** | Immediate 200 OK empty response when visible entries are empty | Task 4.2 (`crates/server/tests/`) | **Full** |
| **OOB Title Swap & Styling** | Swap `span#entry-title-{id}` with Annuaire typography and tooltip | Task 4.2 (`crates/server/tests/`) | **Full** |
| **Graceful Collapse on Error** | 200 OK with `<div id="page-executive-brief" class="hidden"></div>` on LLM failure/timeout | Task 4.2 (`crates/server/tests/`) | **Full** |
| **XSS Prevention on Titles** | HTML escaping on all injected OOB title swap spans | Task 4.2 (`crates/server/tests/`) | **Full** |
| **Cache Hit Fast-Path** | Immediate response on cache hit without spawning CLI process | Task 4.2 (`crates/server/tests/`) | **Full** |
| **Server Prefetch Guard** | Prefetch triggers on batch threshold, ignores processed IDs | Task 5.1 (`crates/server/src/ai/prefetch.rs`) | **Full** |
| **Zero-LLM Detail Drawer** | Drawer reads from cache or extractive teaser, 0 CLI spawns | Task 5.3 (`crates/server/src/routes/entries.rs`) | **Full** |

---

## 3. Detailed Review by Functional Area

### A. Authentication & AI Decoupling (Milestone 1)
- **Design Requirement**: Setting `AI_ENABLED=true` while `AUTH_ENABLED=false` must result in `config.ai_enabled == true`.
- **Planned Tests**: Task 1.1 covers unit tests and permutation matrices testing `(auth, ai) = (false, true)`, `(false, false)`, `(true, false)`, and `(true, true)`.
- **Verdict**: Complete. Tests verify behavior before modifying configuration parser.

### B. Database Schema & Page Fingerprint Caching (Milestone 2)
- **Design Requirement**: SQLite v3 migration, table `page_executive_briefs_v2`, index `idx_page_briefs_v2_created_at`, upsert semantics, batch queries, atomic transactions, and retention pruning.
- **Planned Tests**: Task 2.1 details five distinct unit test cases covering every database operation. Task 2.3 adds deterministic unit tests for SHA256 page key calculation across ID order permutations.
- **Verdict**: Complete. Directly addresses SQLite constraint hazards and disk I/O reduction.

### C. Multilingual Prompt Enforcement & CLI Execution (Milestone 3)
- **Design Requirement**: Strict language isolation per article (source language preservation) and resilient JSON output parsing against LLM markdown artifacts.
- **Planned Tests**: Task 3.1 verifies prompt string formatting contains the multilingual instruction block and verifies parser behavior with raw JSON, markdown-wrapped JSON, and conversational text wrappers.
- **Verdict**: Complete.

### D. HTMX Lazy Skeleton & OOB Title Swaps (Milestone 4)
- **Design Requirement**: Sub-50ms initial render with skeleton placeholder, lazy load via `/hx/executive-brief`, query parameter forwarding, strict eligibility filter rejection (`today` rejection), empty list guard, Annuaire styling (`p-3.5` padding, `text-sm font-medium font-sans leading-snug line-clamp-2`), original title tooltip (`title="{escaped_original_title}"`), XSS escaping, cache fast-path, and graceful collapse on LLM timeout or error.
- **Planned Tests**: Task 4.2 defines comprehensive integration and route test assertions for all response branches (success, cached, empty entries, ineligible filter, error/timeout).
- **Verdict**: Complete. Protects both UI layout stability and security.

### E. Server Prefetch & Zero-LLM Article Detail View (Milestone 5)
- **Design Requirement**: Background batch prefetch threshold logic, duplicate avoidance, and guaranteed zero CLI invocations when opening article details in the HTMX drawer.
- **Planned Tests**: Task 5.1 tests prefetch threshold counts and cached ID exclusion. Task 5.3 verifies that `get_entry_detail_htmx` reads from cache and falls back to extractive summaries without spawning CLI processes.
- **Verdict**: Complete.

---

## 4. Conclusion & Gate B Readiness

The planned test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261122-batch-page-ai-brief/04-tasks.md) satisfy all technical criteria. No missing tests or gaps were identified.

The plan is clean and ready to advance to **Gate B**.
