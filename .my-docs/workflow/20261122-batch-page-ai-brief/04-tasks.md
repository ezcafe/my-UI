# Tasks: Batch Page AI Brief & Multilingual Executive Briefs

## Milestone 1: Authentication & AI Decoupling

- [x] **Task 1.1: TDD tests for Config AI decoupling**
  - Path: `crates/server/src/config.rs`
  - Action: Update existing unit tests and matrix permutation tests to assert `config.ai_enabled == true` when `AUTH_ENABLED=false` and `AI_ENABLED=true`. Verify tests fail before implementation.
- [x] **Task 1.2: Decouple AI config implementation**
  - Path: `crates/server/src/config.rs`
  - Action: Modify `ai_enabled` parsing so it reads `AI_ENABLED` directly without requiring `auth_enabled`.
  - Verification: Run `cargo test --package server config` to verify all config tests pass.

---

## Milestone 2: Database Schema & Page Fingerprint Caching

- [x] **Task 2.1: TDD tests for AiDb v3 migration & page key queries**
  - Path: `crates/server/src/ai/db.rs`
  - Action: Add unit tests:
    - `test_migration_v2_to_v3`: Upgrades SQLite database to version 3, validates `page_executive_briefs_v2` schema, verifies `idx_page_briefs_v2_created_at` index, and asserts absence of redundant `idx_page_briefs_v2_key`.
    - `test_page_brief_by_key_upsert`: Verifies saving, retrieving, and re-saving brief for the same `page_key` without `SQLITE_CONSTRAINT_UNIQUE` failure.
    - `test_get_unprocessed_entry_ids`: Verifies single batched SQL query identifying uncached entry IDs.
    - `test_save_batch_results_transaction`: Verifies transactional atomic write of summaries and page brief using `conn.transaction()?`.
    - `test_prune_page_briefs`: Verifies retention cleanup deleting records older than retention threshold.
- [x] **Task 2.2: Implement AiDb v3 schema and methods**
  - Path: `crates/server/src/ai/db.rs`
  - Action:
    - Upgrade `init_schema` with `PRAGMA user_version = 3` migration block adding `page_executive_briefs_v2` and `idx_page_briefs_v2_created_at`.
    - Implement `get_page_brief_by_key` and upsert `save_page_brief_by_key` (`ON CONFLICT(page_key) DO UPDATE`).
    - Implement batched `get_unprocessed_entry_ids(&self, entry_ids: &[i64]) -> Vec<i64>`.
    - Implement atomic `save_batch_results(&self, page_key: Option<(&str, &str)>, items: &[ArticleSummaryInput]) -> Result<(), AiDbError>` in `conn.transaction()?`.
    - Implement `prune_page_briefs(&self, retention_days: i64) -> Result<usize, AiDbError>`.
  - Verification: Run `cargo test --package server ai::db` to verify all DB tests pass.
- [x] **Task 2.3: Implement deterministic page key calculation**
  - Path: `crates/server/src/ai/llm.rs` or `crates/server/src/ai/mod.rs`
  - Action: Create `compute_page_key(status: &str, entry_ids: &[i64]) -> String` using SHA256 of entry IDs.
  - Verification: Unit test verifying identical ID list yields identical key, while altered order or IDs yields distinct key.

---

## Milestone 3: Multilingual Prompt Enforcement & CLI Execution

- [x] **Task 3.1: Unit test for multilingual batch prompt formatting & output parsing**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Test prompt construction ensuring the multilingual preservation rule is included, and test robust JSON parsing of batch responses including markdown fence stripping (````json ... ````).
- [x] **Task 3.2: Update prompt & parser in LlmCliRunner**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Insert strict multilingual instruction into `summarize_batch` system instructions and harden JSON parsing against markdown fences and conversational preambles.
  - Verification: Run `cargo test --package server ai::llm`.

---

## Milestone 4: HTMX Lazy Skeleton & OOB Title Swaps

- [x] **Task 4.1: Create Annuaire skeleton template, guard empty states, and wrap article title IDs**
  - Paths:
    - `crates/ui-components/templates/partials/executive_brief_skeleton.html`
    - `crates/ui-components/templates/partials/entry_item.html`
    - `crates/ui-components/templates/partials/entry_list.html`
  - Action:
    - Create pulsing skeleton card matching Annuaire styling with `p-3.5` padding, forwarding active `EntriesFilterQuery` parameters (`offset`, `limit`, `status`, `feed_id`, `category_id`) via `hx-get="/hx/executive-brief?..."`, `hx-trigger="load"`, `hx-swap="outerHTML"`.
    - In `entry_list.html`, guard the skeleton card inclusion with `{% if !entries.is_empty() && is_brief_eligible %}` so empty feeds/folders do not render a skeleton or trigger redundant requests.
    - Ensure entry titles in `entry_item.html` have `<span id="entry-title-{{ entry.id }}">...</span>` for OOB targeting while preserving full Annuaire typography classes (`article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors`) and `title="{{ entry.title }}"` tooltip for original headlines.
- [x] **Task 4.2: TDD tests for `/hx/executive-brief` endpoint, empty entries guard & filter reconciliation**
  - Path: `crates/server/tests/` or `crates/server/src/routes/entries.rs`
  - Action: Write tests for `/hx/executive-brief`:
    - Reconciles `is_brief_eligible_filter`: asserts `status=today` is ineligible and returns 200 OK empty response.
    - Empty entries guard: returns 200 OK empty response when `entries.is_empty()` (e.g. empty feed or offset exceeds total entries).
    - Forwards `feed_id` and `category_id` in query parameters without desynchronizing briefs.
    - Returns skeleton swap HTML with completed brief card (`p-3.5` padding) and OOB title updates with Annuaire classes (`text-sm font-medium font-sans leading-snug line-clamp-2`), tooltip `title="{escaped_original_title}"`, and `Cache-Control: no-cache` header.
    - Failure contract: LLM timeout, process error, or empty synthesis returns 200 OK with `<div id="page-executive-brief" class="hidden"></div>` and 0 OOB title spans.
    - Verifies HTML escaping on OOB title swaps to prevent XSS.
    - Returns cached brief immediately on cache hit without invoking LLM CLI.
- [x] **Task 4.3: Implement `/hx/executive-brief` endpoint, empty entries guard, and Annuaire OOB swaps**
  - Path: `crates/server/src/routes/entries.rs`, `crates/server/src/routes/mod.rs`
  - Action:
    - Update `is_brief_eligible_filter` to strictly exclude `Some("today")`.
    - Implement endpoint accepting `Query(query): Query<EntriesFilterQuery>`.
    - Query entries honoring `feed_id` and `category_id`.
    - Add early return with 200 OK empty response if `entries.is_empty()`.
    - Deduplicate concurrent requests per `page_key` in `AiService` to avoid thundering herds.
    - Use batched `get_unprocessed_entry_ids` and transactional `save_batch_results`.
    - Implement graceful failure contract (returns `<div id="page-executive-brief" class="hidden"></div>` on LLM error/timeout).
    - Render completed brief card with `p-3.5` padding and HTML-escaped OOB title spans with Annuaire classes and original title tooltip (`title="{escaped_original_title}"`), including `Cache-Control: no-cache, no-store, must-revalidate` headers.

---

## Milestone 5: Server Prefetch & Zero-LLM Article Detail View

- [x] **Task 5.1: TDD tests for server prefetch threshold & duplicate avoidance**
  - Path: `crates/server/src/ai/prefetch.rs`
  - Action: Verify prefetch triggers only when uncached count meets threshold and ignores already processed IDs.
- [x] **Task 5.2: Update AiPrefetchService**
  - Path: `crates/server/src/ai/prefetch.rs`
  - Action: Implement threshold check and skip already processed items.
- [x] **Task 5.3: Ensure zero-LLM invocation in article detail drawer**
  - Path: `crates/server/src/routes/entries.rs` (`get_entry_detail_htmx`)
  - Action: Replace any CLI summary generation with cache read falling back to extractive summary.
  - Verification: Run integration tests verifying no CLI commands spawned during drawer open.

---

## Milestone 6: Verification & End-to-End Testing

- [x] **Task 6.1: Full test suite verification**
  - Action: Run `cargo test` across all crates to ensure zero regressions.
- [x] **Task 6.2: Manual verification of HTMX skeleton and OOB title updates**
  - Action: Start server, load unread feed, observe pulsing skeleton, verify clean OOB title updates, test feed-filtered view, test simulated LLM timeout skeleton collapse, and verify sub-50ms initial page load.
