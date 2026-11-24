# Test Log: Details Page AI Rewrite with Batch Pre-request

- **Run slug:** `20261123-details-ai-rewrite-batch-pre-request`
- **Updated:** 2026-11-24T06:13:00+07:00
- **Test path:** full
- **Result:** success
- **Smoke source:** Smoke-Task

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 117 | 117 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 61 | 61 | 0 | PASSED |
| **Total** | **207** | **207** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. TDD Phase Verification Details

### Phase 1: Database Evolution (`AiDb` Migration v4)
- Upgraded `AiDb` schema to `PRAGMA user_version = 4`.
- Added column `rewrite TEXT` to `article_ai_summaries` table via non-destructive migration.
- Extended `AiSummaryRecord` and `ArticleSummaryInput` with `pub rewrite: Option<String>`.
- Updated upsert logic to protect against null overwrites via `COALESCE(excluded.rewrite, article_ai_summaries.rewrite)`.
- Verified single and batch retrieval via `get_summary_and_title` and `get_batch_summaries`.
- **Tests**:
  - `ai::db::tests::test_migration_v3_to_v4_adds_rewrite_column`
  - `ai::db::tests::test_save_and_get_summary_with_rewrite`

### Phase 2: Batch Prompt Formatting & 300-Word Truncation
- Updated `LlmCliRunner::format_batch_prompt_structured` to include rewrite instructions:
  - Total length cap `< 110 words`, target sweet spot `30–60 words` in 2–3 bullet points or short paragraphs.
  - Inverted Pyramid model: conclusion and primary impact in the first two sentences.
  - Strict language parity (Vietnamese sources produce Vietnamese rewrite, English sources produce English rewrite).
  - Compact schema includes `"r"`: `{"top":"...","items":[{"i":101,"t":"...","b":"...","r":"..."}]}`.
- Implemented whitespace-aware `truncate_words(s: &str, max_words: usize) -> &str` capping new article content at 300 words.
- **Tests**:
  - `ai::llm::tests::test_format_batch_prompt_with_rewrite_schema_and_rules`
  - `ai::llm::tests::test_batch_prompt_input_truncation_300_words`

### Phase 3: Response Parsing & Regex Fallback
- Extended `BatchAiItem` with `#[serde(default, rename = "r")] pub rewrite: Option<String>`.
- Updated `ITEM_REGEX` to optionally capture the `"r"` field.
- Implemented unescaping for newlines and quotes in fallback regex extraction.
- Verified parsing of direct JSON, markdown code fences, and fallback extraction when `"r"` is omitted or JSON is malformed.
- **Tests**:
  - `ai::llm::tests::test_parse_batch_response_with_rewrite_json`
  - `ai::llm::tests::test_parse_batch_response_with_rewrite_regex_fallback`

### Phase 4: Server Background Prefetching (`AiPrefetchService`)
- Updated `AiPrefetchService::run_prefetch` to map `item.rewrite` into `AiDb::save_summary_and_title_with_rewrite`.
- Verified background worker persists rewrites into SQLite alongside titles and briefs.
- **Tests**:
  - `ai::prefetch::tests::test_prefetch_service_persists_rewrites`

### Phase 5: Detail View Template & Route Integration
- Extended `EntryDetailTemplate` with `pub rewrite: String` and `pub is_rewrite: bool`.
- Updated `entry_detail.html` partial to render `rewrite` in prose container and completely omit raw article HTML (`sanitized_content`).
- Supported Option 1A graceful fallback: when `is_rewrite` is false (rewrite missing or pending), renders the executive brief card.
- Updated `get_entry_detail_htmx` route in `crates/server/src/routes/entries.rs` to load `record.rewrite` from `AiDb` and serve sub-5ms responses.
- **Tests**:
  - `templates::tests::test_entry_detail_template_renders_rewrite`
  - `templates::tests::test_entry_detail_template_fallback_when_rewrite_missing`
  - `routes::entries::tests::test_get_entry_detail_route_loads_rewrite_from_ai_db`

---

## 4. Smoke Verification

- **Smoke source:** Smoke-Task
- **Result:** smoke-pass
- **Summary:** Independent smoke verification executed `cargo test --workspace` (207 passed, 0 failed) and `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings). Verified core flows: schema migration v4, prompt formatting with 300-word truncation and inverted pyramid rewrite, parsing with JSON & regex fallback, background prefetch persistence, and HTMX detail view rendering with fallback.

---

## 5. Full Test Verification

- **Test path:** full
- **Result:** success
- **Summary:** Executed full workspace test suite `cargo test` across all crates (207 passed, 0 failed) and strict static analysis `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings). All unit, integration, and UI template tests passed cleanly.

