# Test Log: AI Source Grouping, 3-Sentence Brief & UI Polish

- **Run slug:** `20261122-ai-source-grouping-3-sentence-brief`
- **Updated:** 2026-11-23T07:34:00+07:00
- **Test path:** full
- **Result:** success
- **Smoke source:** Smoke-Task

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 100 | 100 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 50 | 50 | 0 | PASSED |
| **Total** | **179** | **179** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Milestone Test Details

### Milestone 1: LLM Prompt Directives & 3-Sentence Length
- Prompt directive enforcement: `format_prompt` specifies strictly 3 concise sentences.
- Batch prompt item directive enforcement: `format_batch_prompt` item brief instruction specifies strictly 3 sentences.
- Extractive fallback updated to strictly 3 sentences across single-article and batch fallback flows.
- Tests:
  - `ai::llm::tests::test_format_prompt_strictly_3_sentences`
  - `ai::llm::tests::test_format_batch_prompt_strictly_3_sentences`
  - `ai::llm::tests::test_extractive_fallback_3_sentences`

### Milestone 2: Source Grouping & Date Sorting in Batch Prompt
- Introduced `BatchArticleInput` capturing article id, title, source name, published_at timestamp, content/brief, and is_cached flag.
- Prompt groups articles under source headers (`### Source: <Feed Title>`).
- Articles within each source group are ordered by `published_at` descending (latest first).
- Cached articles labeled as `[ID x] (Existing Brief) Title: ...\nBrief: ...` while uncached are `[ID y] (New Article) Title: ...\nContent: ...`.
- Backward-compatible adapter provided for existing call sites.
- Tests:
  - `ai::llm::tests::test_format_batch_prompt_source_grouping_and_date_sorting`
  - `ai::llm::tests::test_format_batch_prompt_handles_cached_briefs_and_new_articles`

### Milestone 3: Route Orchestration & Cache Reuse in `/hx/executive-brief` and Prefetch
- GraphQL query expanded to retrieve `publishedAt` and `feed { title }`.
- Partitioned entries into cached (via `AiDb::get_batch_summaries`) and uncached.
- Full-page context passed to `summarize_batch_structured` so `"top"` accurately synthesizes all page sources and items.
- Item persistence restricted to new/uncached articles, preventing redundant database overwrites.
- Preserved existing cached OOB title swaps alongside newly generated titles.
- Prefetch background worker aligned with `PrefetchCandidate` carrying source and published date.
- Tests:
  - `routes::entries::tests::test_executive_brief_reuses_db_summaries_and_synthesizes_top`
  - `routes::entries::tests::test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty`
  - `ai::prefetch::tests::test_prefetch_saves_brief_with_page_key`
  - `ai::prefetch::tests::test_page_batch_processing`

### Milestone 4: UI Row Click Target Expansion & Pagination Border Polish
- Relocated `onclick="openEntryDetail(id)"` to outer `.annuaire-row` element across `entry_item.html`, `entry_list.html`, and `index.html`.
- Removed nested inner click handler to prevent redundant double dispatches.
- Preserved `event.stopPropagation()` on external link buttons.
- Removed `border-t border-border` on `<nav aria-label="Pagination">` in `entry_list.html`.
- Tests:
  - `templates::tests::test_annuaire_row_click_target`
  - `templates::tests::test_pagination_nav_no_top_border`

### Milestone 5: Full Workspace Verification
- Ran `cargo test --workspace` (179 passing tests across all crates).
- Ran `cargo clippy --workspace --all-targets` (0 warnings).

---

## 4. Execution Details & Environment

- **Full test command:** `cargo test --workspace` (179 passed, 0 failed)
- **Lints command:** `cargo clippy --workspace --all-targets` (0 warnings)
- **Result:** `success`

---

## 5. Independent Smoke Verification (Smoke-Task)

- **Execution Time:** 2026-11-23T07:28:35+07:00
- **Surface Coverage:**
  - AI LLM prompt directives & strictly 3 sentences (`test_format_prompt_strictly_3_sentences`, `test_format_batch_prompt_strictly_3_sentences`, `test_extractive_fallback_3_sentences`)
  - AI batch formatting source grouping & publication date sorting (`test_format_batch_prompt_source_grouping_and_date_sorting`, `test_format_batch_prompt_handles_cached_briefs_and_new_articles`)
  - Route orchestration & executive brief cache reuse (`test_executive_brief_reuses_db_summaries_and_synthesizes_top`, `test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty`, `test_executive_brief_cache_hit_and_oob_swaps`)
  - Prefetch worker candidate sourcing & DB persistence (`test_prefetch_saves_brief_with_page_key`, `test_page_batch_processing`)
  - UI template row click target & pagination border polish (`test_annuaire_row_click_target`, `test_pagination_nav_no_top_border`)
- **Workspace Test Suite:** `cargo test --workspace` (179 passed, 0 failed)
- **Workspace Clippy:** `cargo clippy --workspace --all-targets` (0 warnings)
- **Smoke source:** `Smoke-Task`
- **Result:** `smoke-pass`

---

## 6. Full Test Suite Verification (test-full Stage)

- **Execution Time:** 2026-11-23T07:34:00+07:00
- **Test path:** `full`
- **Workspace Test Suite:** `cargo test --workspace` (179 passed, 0 failed)
  - `miniflux-client`: 23 passed, 0 failed
  - `server`: 100 unit tests passed, 6 integration/e2e tests passed, 0 failed
  - `ui-components`: 50 passed, 0 failed
- **Workspace Clippy:** `cargo clippy --workspace --all-targets` (0 warnings, clean)
- **Result:** `success`
