# Tasks: Batch Rewrite & Decoupled Detail Analysis

- **Workflow Slug:** `20261124-batch-rewrite-and-detail-analysis`
- **Result:** done

---

## Task List (TDD Steps)

### Task 1: LLM Engine Contracts & Prompt Formatting (TDD)
- **Files**: `crates/server/src/ai/llm.rs`
- **Tests First**:
  - `test_format_batch_prompt_requires_rewrite_r`: Verify batch prompt instructs LLM to produce `"r"` (inverted pyramid rewrite) instead of `"b"` (brief).
  - `test_parse_batch_response_parses_rewrite_field`: Verify `BatchAiItem` correctly deserializes `"r"` and maps it to `rewrite`.
  - `test_parse_batch_response_fallback_regex_captures_rewrite`: Verify regex parser captures `"r"` field.
  - `test_format_impact_and_fallacies_prompt_and_parser`: Verify dedicated prompt and parser for impact & fallacies.
- **Implementation**:
  - Update `BatchAiItem` struct with `#[serde(rename = "r", alias = "b")] pub rewrite: String`.
  - Update `format_batch_prompt_structured` to request `"r"` instead of `"b"`.
  - Update `ITEM_REGEX` to match `"r"`.
  - Implement `format_impact_and_fallacies_prompt` and `parse_impact_and_fallacies_response`.

### Task 2: Database Persistence & AiService Analysis (TDD)
- **Files**: `crates/server/src/ai/db.rs`, `crates/server/src/ai/mod.rs`
- **Tests First**:
  - `test_ai_db_save_and_update_analysis`: Verify saving batch items persists `rewrite` and updating analysis saves `impact` and `fallacies`.
- **Implementation**:
  - Implement `AiDb::save_analysis(entry_id, impact, fallacies)`.
  - Implement `AiService::analyze_impact_and_fallacies(entry_id, title, content)`.

### Task 3: UI Templates - Detail View & Analysis Partial (TDD)
- **Files**: `crates/ui-components/src/templates.rs`, `crates/ui-components/templates/partials/entry_detail.html`, `crates/ui-components/templates/partials/analysis_cards.html`
- **Tests First**:
  - `test_entry_detail_renders_rewrite_on_load_without_brief_card`: Verify rendered HTML contains `rewrite`, does NOT contain "Executive Brief", and includes the analysis trigger/cards.
  - `test_analysis_cards_partial_renders_impact_and_fallacies`: Verify analysis cards template output.
- **Implementation**:
  - Create `crates/ui-components/templates/partials/analysis_cards.html`.
  - Update `crates/ui-components/templates/partials/entry_detail.html` to remove `brief_card.html`, render `rewrite` in `#ai-rewrite-box`, and render either cached analysis cards or the lazy-load container.
  - Update `EntryDetailTemplate` struct in `templates.rs` with `has_analysis: bool`.

### Task 4: Server Route Handlers & Integration (TDD)
- **Files**: `crates/server/src/routes/entries.rs`, `crates/server/src/app.rs`
- **Tests First**:
  - `test_get_entry_detail_htmx_renders_rewrite_on_load_without_brief`: Integration test for `GET /hx/entries/{id}`.
  - `test_get_entry_analysis_htmx_serves_impact_and_fallacies`: Integration test for `GET /hx/entries/{id}/analysis`.
- **Implementation**:
  - In `get_page_executive_brief_htmx`, populate `ArticleSummaryInput` with `item.rewrite` for both `summary` and `rewrite`.
  - In `get_entry_detail_htmx`, set `rewrite` and `has_analysis`, render `EntryDetailTemplate`.
  - Implement handler `get_entry_analysis_htmx` at `/hx/entries/{id}/analysis` (and keep `/hx/entries/{id}/rewrite` route compatible).
  - Register route in `crates/server/src/app.rs`.
