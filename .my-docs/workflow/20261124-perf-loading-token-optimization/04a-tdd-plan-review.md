Result: clean

# TDD Plan Review: Performance, Loading, and Token Optimization

- **Workflow Slug:** `20261124-perf-loading-token-optimization`
- **Reviewed Documents:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/04-tasks.md) against [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03-design.md)
- **Reviewer:** Subagent (my-dev-flow-v3-code)
- **Review Status:** Clean (`04a ok`)

---

## 1. Executive Summary

A comprehensive review of the Test-Driven Development (TDD) plan in `04-tasks.md` was conducted against the architectural specifications and contracts established in `03-design.md`.

The planned test suite includes 15 named test cases spanning 6 phases. Every task adheres strictly to the TDD iron law ("No production code without a failing test first"), specifies concrete test identifiers, file locations, expected assertions, failure modes, and implementation targets. Edge cases, performance constraints, and fallback paths identified during Grill and Design phases are fully accounted for.

---

## 2. Review Checklist & Evaluation Criteria

| Evaluation Dimension | Status | Notes |
| :--- | :--- | :--- |
| **Completeness vs Design** | **PASS** | Covers all architectural pillars: 25-word teaser fast-path, 1.5s scraper timeout, canonical cache key unification, 130-word prompt truncation, top-1 eager rewrite with grounding, HTMX detail rewrite endpoint with skeleton loading, static CSS route, and CDN elimination. |
| **Test Case Isolation** | **PASS** | Tests are modular and isolated across `crates/server/src/ai/`, `crates/server/src/routes/`, and `crates/ui-components/src/templates.rs`. Network dependencies (HTTP scraper, LLM CLI runner) use mock delays or synthetic payloads without external flakiness. |
| **Edge Case & Resilience Coverage** | **PASS** | Explicit tests for: teaser threshold boundaries (20 vs 26 words), scraper timeouts (1500ms cap), empty ID slices in key hashing, cached vs uncached drawer states, and graceful fallback to executive brief when LLM generation fails. |
| **Adherence to TDD Iron Law** | **PASS** | Every task begins with a Red stage defining the exact failing test prior to code modification. Steps specify exact assertions and file paths. |

---

## 3. Phase-by-Phase Assessment

### Phase 1: Scraper Latency & Fast-Path Optimization
- **Coverage**:
  - `test_extractor_teaser_fast_path_25_words`: Verifies bypass of HTTP network stack when teaser length &ge; 25 words.
  - `test_extractor_timeout_1500_millis`: Verifies 1.5s timeout ceiling and fallback to available teaser under network delays.
- **Verdict**: Compliant. Tests verify both latency optimization and resilience.

### Phase 2: Canonical Page Cache Key & Deduplication
- **Coverage**:
  - `test_canonical_page_key_consistency`: Tests deterministic hashing across callers and edge cases (empty entry list).
  - `test_prefetch_and_executive_brief_share_page_key`: Verifies background prefetch and foreground brief use the exact same cache key and shared in-flight mutex.
- **Verdict**: Compliant. Solves key divergence and eliminates orphaned prefetch jobs.

### Phase 3: Batch Prompt Formatting, Truncation, Top-1 Rewrite & Grounding
- **Coverage**:
  - `test_batch_prompt_input_truncation_130_words`: Verifies strict 130-word input cap per article.
  - `test_format_batch_prompt_eager_rewrite_top_1_only`: Verifies `"r"` generation only for lead item and deferral of remaining items, adhering to inverted pyramid format.
  - `test_format_batch_prompt_strict_grounding_constraint`: Verifies inclusion of anti-hallucination source-grounding instructions.
- **Verdict**: Compliant. Directly enforces 80% token reduction and output quality guardrails.

### Phase 4: HTMX Lazy-Loading Detail Rewrite Route
- **Coverage**:
  - `test_entry_detail_renders_skeleton_when_rewrite_missing`: Validates initial drawer rendering with `#ai-rewrite-box` skeleton and omission of raw content dump.
  - `test_entry_detail_renders_rewrite_directly_when_cached`: Validates instantaneous display of cached rewrites.
  - `test_get_entry_rewrite_htmx_serves_existing_cache`: Tests cache-hit latency (< 5ms) and omission of redundant LLM calls.
  - `test_get_entry_rewrite_htmx_generates_and_caches_rewrite`: Tests on-demand generation and SQLite persistence.
  - `test_get_entry_rewrite_htmx_fallback_on_llm_failure`: Tests graceful fallback to executive brief on LLM timeout or error.
  - `test_generate_single_rewrite`: Tests `AiService` single-item rewrite method.
- **Verdict**: Compliant. Complete UI and route coverage with full error handling.

### Phase 5: Static CSS Delivery & CDN Elimination
- **Coverage**:
  - `test_serve_static_annuaire_theme_css_route`: Verifies `/static/annuaire-theme.css` route, HTTP headers, and 200 OK.
  - `test_base_template_excludes_tailwind_cdn_script`: Verifies `base.html` includes stylesheet link and excludes CDN runtime script and config.
- **Verdict**: Compliant. Guarantees removal of FOUC and runtime JS overhead.

### Phase 6: Integration Verification
- **Coverage**:
  - `test_e2e_page_load_and_detail_drawer_rewrite_flow`: End-to-end integration test validating the entire pipeline from page load to drawer lazy-loading.
- **Verdict**: Compliant.

---

## 4. Conclusion & Next Step

The TDD plan in `04-tasks.md` meets all quality, isolation, and coverage requirements. `04a-tdd-plan-review.md` is approved with **`Result: clean`**.

Per workflow protocol, execution hard-stops here to return control to the parent orchestrator for Gate B review and approval.
