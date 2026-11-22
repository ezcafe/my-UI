# Test Log: Batch Page AI Brief & Multilingual Executive Briefs

- **Run slug:** `20261122-batch-page-ai-brief`
- **Updated:** 2026-11-22T17:27:00+07:00
- **Test path:** full
- **Result:** success

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 84 | 84 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 48 | 48 | 0 | PASSED |
| **Total** | **161** | **161** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Milestone Test Details

### Milestone 1: Authentication & AI Decoupling
- Verified AI config operates when `AUTH_ENABLED=false` and `AI_ENABLED=true`.
- Verified permutation test matrix for all 4 states (`auth_val`, `ai_val`).
- Tests: `config::tests::test_config_ai_enabled_when_auth_disabled`, `config::tests::test_config_ai_matrix_permutations`.

### Milestone 2: Database Schema & Page Fingerprint Caching
- Upgraded `AiDb` to `PRAGMA user_version = 3`.
- Added `page_executive_briefs_v2` with unique index on `page_key` and created_at index.
- Implemented upsert for `save_page_brief_by_key` avoiding SQLite unique constraint errors.
- Implemented batched `get_unprocessed_entry_ids` query.
- Implemented atomic `save_batch_results` in SQLite transaction.
- Implemented `prune_page_briefs` for retention cleanup.
- Implemented deterministic `compute_page_key(status, entry_ids)`.
- Tests: `ai::db::tests::test_migration_v2_to_v3`, `ai::db::tests::test_page_brief_by_key_upsert`, `ai::db::tests::test_get_unprocessed_entry_ids`, `ai::db::tests::test_save_batch_results_transaction`, `ai::db::tests::test_prune_page_briefs`, `ai::tests::test_compute_page_key`.

### Milestone 3: Multilingual Prompt Enforcement & CLI Execution
- Added strict multilingual instructions in `LlmCliRunner::format_batch_prompt` enforcing articles and briefs to remain in source text language without cross-translation.
- Verified robust JSON parsing with markdown code fences (````json ... ````) and conversational preambles/trailers.
- Tests: `ai::llm::tests::test_multilingual_batch_prompt_and_robust_parsing`, `ai::llm::tests::test_parse_batch_response`.

### Milestone 4: HTMX Lazy Skeleton & OOB Title Swaps
- Added Annuaire skeleton card template (`partials/executive_brief_skeleton.html`) with `p-3.5` padding, forwarding active query parameters via `hx-get="/hx/executive-brief?..."`, `hx-trigger="load"`, `hx-swap="outerHTML"`.
- Guarded skeleton in `partials/entry_list.html` with `{% if !entries.is_empty() && is_brief_eligible %}`.
- Wrapped article titles in `entry_item.html` and `entry_list.html` with `<span id="entry-title-{{ entry.id }}">...</span>` retaining full typography classes and original title tooltip.
- Reconciled `is_brief_eligible_filter` to strictly exclude `today` view, active search, read, starred, and all views.
- Implemented `/hx/executive-brief` route with empty entries guard, cache hit fast-path, in-flight deduplication, and graceful collapse (`<div id="page-executive-brief" class="hidden"></div>`) on failure or timeout.
- Tests: `routes::entries::tests::test_filter_specific_brief`, `routes::entries::tests::test_render_title_oob_span_typography_and_escaping`, `routes::entries::tests::test_escape_html_entities`, `routes::entries::tests::test_executive_brief_failure_contract_hidden_div`, `e2e_tests::test_executive_brief_ineligible_views_and_empty_guard`, `e2e_tests::test_executive_brief_cache_hit_and_oob_swaps`.

### Milestone 5: Server Prefetch & Zero-LLM Article Detail View
- Added prefetch threshold check (`uncached.len() >= 5`) and batched unprocessed check in `AiPrefetchService`.
- Eliminated CLI synthesis on article detail open in `get_entry_detail_htmx`, reading strictly from cache or falling back to extractive summary teaser.
- Tests: `ai::prefetch::tests::test_prefetch_threshold_and_duplicate_avoidance`, `routes::tests::test_get_entry_detail_htmx_fetches_single_entry`.

---

## 4. Execution Details & Environment

- **Full test command:** `cargo test --workspace`
  - miniflux-client: 23 passed, 0 failed
  - server (unit): 84 passed, 0 failed
  - server (integration / e2e): 6 passed, 0 failed
  - ui-components: 48 passed, 0 failed
  - Total: 161 passed, 0 failed
- **Lints command:** `cargo clippy --workspace --all-targets -- -D warnings`
  - 0 warnings, exit code 0
- **Final Result:** `success`
