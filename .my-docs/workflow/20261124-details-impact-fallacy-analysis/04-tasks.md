# Tasks (TDD-ready): Article Detail AI Rewrite with Impact & Fallacy Analysis

- **Stage:** design
- **Status:** completed
- **Result:** `done`

---

## 1. Planned Tests Table

| Case | Failing Test Name | Covers |
|:-----|:------------------|:-------|
| 1. Database Analysis Persistence | `test_ai_db_save_and_retrieve_analysis` | Verifies `save_summary_and_title_with_analysis` and `get_summary_and_title` persist and retrieve `impact` and `fallacies`. |
| 2. LLM Analysis Deserialization & Reassurance | `test_llm_rewrite_and_analyze_json_parsing_and_reassurance` | Verifies JSON parsing (raw and fenced), extraction of rewrite/impact/fallacies, and normalization of reassuring fallacy text when objective. |
| 3. UI Template Omission of Raw Content & Highlight Cards | `test_entry_detail_template_renders_analysis_cards_without_sanitized_content` | Verifies `EntryDetailTemplate` renders the Executive Brief, Impact Card, Fallacy Card, and Rewrite, and does NOT render raw `sanitized_content`. |
| 4. Server Route On-Demand & Cache Integration | `test_get_entry_rewrite_htmx_serves_impact_and_fallacies` | Verifies `/hx/entries/:id/rewrite` returns HTML containing Impact Card, Fallacy Card, and Rewrite Box, caching them into `AiDb`. |
| 5. E2E Drawer Lazy Loading | `test_e2e_detail_drawer_analysis_flow` | Verifies full E2E flow opening the detail drawer, swapping skeleton with analysis cards, and viewing cached content. |

---

## 2. Implementation Tasks

### Task 1: Database Schema and `AiDb` Analysis Support
- **Failing test name:** `test_ai_db_save_and_retrieve_analysis`
- **Behavior:**
  - Update `INIT_SCHEMA_SQL` in `crates/server/src/ai/db.rs` with `ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;` and `ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;`.
  - Extend `AiSummaryRecord` and `ArticleSummaryRow` with `impact: Option<String>` and `fallacies: Option<String>`.
  - Implement `save_summary_and_title_with_analysis` in `AiDb` (supporting both PostgreSQL pool and in-memory mock).
  - Ensure backward compatibility with `save_summary_and_title_with_rewrite` (defaults impact & fallacies to None or preserves existing).
- **Files allowed:**
  - `crates/server/src/ai/db.rs`
- **Done evidence:**
  - `cargo test -p server test_ai_db_save_and_retrieve_analysis` passes.

---

### Task 2: LLM Structured Output and Normalization
- **Failing test name:** `test_llm_rewrite_and_analyze_json_parsing_and_reassurance`
- **Behavior:**
  - Define `DetailAnalysisResult` struct with `rewrite`, `impact`, and `fallacies`.
  - Implement `format_rewrite_and_analyze_prompt` enforcing strict JSON schema and multilingual rules.
  - Implement `parse_rewrite_and_analyze_response` supporting raw JSON, fenced code blocks, and fallback recovery.
  - Add reassurance logic for `fallacies` when output indicates objective / no fallacies found.
  - Implement `rewrite_and_analyze` on `LlmCliRunner`, `LlmHttpClient`, and `LlmClient`.
  - Update `AiService::generate_rewrite_and_analysis` in `crates/server/src/ai/mod.rs`.
- **Files allowed:**
  - `crates/server/src/ai/llm.rs`
  - `crates/server/src/ai/mod.rs`
- **Done evidence:**
  - `cargo test -p server test_llm_rewrite_and_analyze_json_parsing_and_reassurance` passes.

---

### Task 3: UI Template Refactoring & Annuaire Highlight Cards
- **Failing test name:** `test_entry_detail_template_renders_analysis_cards_without_sanitized_content`
- **Behavior:**
  - Update `EntryDetailTemplate` in `crates/ui-components/src/templates.rs`: add `impact: String`, `fallacies: String`, and remove `sanitized_content: String`.
  - Update `crates/ui-components/templates/partials/entry_detail.html`:
    - Remove `{{ sanitized_content|safe }}` entirely.
    - Render Impact Card (`trending-up` icon, styled like `brief_card.html`).
    - Render Fallacy Card (`shield-alert` icon, styled like `brief_card.html`).
    - Render Rewrite Box (`#ai-rewrite-box`).
    - Update skeleton loading state when `!is_rewrite` inside `#ai-detail-analysis-box`.
  - Update all existing test instantiations of `EntryDetailTemplate` in `ui-components`.
- **Files allowed:**
  - `crates/ui-components/src/templates.rs`
  - `crates/ui-components/templates/partials/entry_detail.html`
- **Done evidence:**
  - `cargo test -p ui-components` passes.

---

### Task 4: Server Route & HTMX Handler Integration
- **Failing test name:** `test_get_entry_rewrite_htmx_serves_impact_and_fallacies`
- **Behavior:**
  - Update `get_entry_detail_htmx` in `crates/server/src/routes/entries.rs` to read `impact` and `fallacies` from `AiSummaryRecord` and pass them to `EntryDetailTemplate`.
  - Update `get_entry_rewrite_htmx` in `crates/server/src/routes/entries.rs`:
    - Check cache for `rewrite`, `impact`, and `fallacies`.
    - If uncached, call `ai_service.generate_rewrite_and_analysis`.
    - Render HTML snippet with `#ai-detail-analysis-box` containing Impact Card, Fallacy Card, and Rewrite Box.
  - Update existing tests in `crates/server/src/routes/entries.rs`.
- **Files allowed:**
  - `crates/server/src/routes/entries.rs`
- **Done evidence:**
  - `cargo test -p server test_get_entry_rewrite_htmx_serves_impact_and_fallacies` passes.

---

### Task 5: End-to-End Flow Verification & Clippy Lints
- **Failing test name:** `test_e2e_detail_drawer_analysis_flow`
- **Behavior:**
  - Update `crates/server/tests/e2e_tests.rs` to verify that opening the drawer displays the analysis cards without any original HTML content.
  - Run full test suite (`cargo test`) and linter (`cargo clippy --all-targets -- -D warnings`).
- **Files allowed:**
  - `crates/server/tests/e2e_tests.rs`
- **Done evidence:**
  - All tests passing, 0 clippy warnings.
