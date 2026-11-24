# Tasks: Performance, Loading, and Token Optimization

## Task Overview

This document defines the ordered Test-Driven Development (TDD) phases for implementing performance, loading, and token optimizations. Every task begins with a failing test before implementation.

---

## Planned Tests Index

| Phase | Test Identifier | Scope & Assertions |
| :--- | :--- | :--- |
| **1. Scraper Optimization** | `test_extractor_teaser_fast_path_25_words` | Verifies `ArticleExtractor::extract_content` returns teaser immediately without HTTP fetch if word count &ge; 25. |
| **1. Scraper Optimization** | `test_extractor_timeout_1500_millis` | Verifies `ArticleExtractor` client timeout is set to 1500ms and gracefully returns teaser on delay. |
| **2. Cache Key Unification** | `test_canonical_page_key_consistency` | Verifies `compute_page_key` produces deterministic keys across different callers with matching status and IDs. |
| **2. Cache Key Unification** | `test_prefetch_and_executive_brief_share_page_key` | Verifies `AiPrefetchService` and `executive_brief_htmx` compute identical keys for the same page view. |
| **3. Prompt & Token Budget** | `test_batch_prompt_input_truncation_130_words` | Verifies `format_batch_prompt_structured` truncates each article's input content to at most 130 words. |
| **3. Prompt & Token Budget** | `test_format_batch_prompt_eager_rewrite_top_1_only` | Verifies batch prompt requests `"r"` only for the first new article ID and defers all subsequent items. |
| **3. Prompt & Token Budget** | `test_format_batch_prompt_strict_grounding_constraint` | Verifies batch prompt contains anti-hallucination grounding constraints. |
| **4. HTMX Detail Rewrite** | `test_entry_detail_renders_skeleton_when_rewrite_missing` | Verifies `EntryDetailTemplate` renders `#ai-rewrite-box` with HTMX lazy-load skeleton and excludes raw content dump. |
| **4. HTMX Detail Rewrite** | `test_entry_detail_renders_rewrite_directly_when_cached` | Verifies `EntryDetailTemplate` renders cached rewrite directly in `#ai-rewrite-box` without skeleton. |
| **4. HTMX Detail Rewrite** | `test_get_entry_rewrite_htmx_serves_existing_cache` | Verifies `GET /hx/entries/{id}/rewrite` serves existing rewrite from `AiDb` in < 5ms. |
| **4. HTMX Detail Rewrite** | `test_get_entry_rewrite_htmx_generates_and_caches_rewrite` | Verifies `GET /hx/entries/{id}/rewrite` generates on-demand rewrite when missing and persists to `AiDb`. |
| **4. HTMX Detail Rewrite** | `test_get_entry_rewrite_htmx_fallback_on_llm_failure` | Verifies `GET /hx/entries/{id}/rewrite` gracefully renders executive brief in `#ai-rewrite-box` when LLM fails. |
| **5. Static CSS Delivery** | `test_serve_static_annuaire_theme_css_route` | Verifies `GET /static/annuaire-theme.css` returns 200 OK with `Content-Type: text/css`. |
| **5. Static CSS Delivery** | `test_base_template_excludes_tailwind_cdn_script` | Verifies `base.html` links to `/static/annuaire-theme.css` and omits `cdn.tailwindcss.com`. |
| **6. Integration Verification** | `test_e2e_page_load_and_detail_drawer_rewrite_flow` | End-to-end integration test validating page load, top-1 eager rewrite, and on-demand rewrite lazy-load. |

---

## Phase 1: Scraper Latency & Fast-Path Optimization

### Task 1.1: 25-Word Teaser Fast-Path
- **Test**: `crates/server/src/ai/extractor.rs` &rarr; `tests::test_extractor_teaser_fast_path_25_words`
  - Pass a teaser with 26 words and a dummy/failing URL.
  - Assert that `extract_content` returns immediately without attempting HTTP connection.
  - Pass a teaser with 20 words.
  - Assert that `extract_content` proceeds past the fast-path check.
- **Implementation**:
  - In [crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs), update `extract_content`:
    Change threshold from `word_count >= 50` to `word_count >= 25`.

### Task 1.2: 1.5s Client Timeout & Truncation Cap
- **Test**: `crates/server/src/ai/extractor.rs` &rarr; `tests::test_extractor_timeout_1500_millis`
  - Start a mock server with a 2.0s delay.
  - Call `extract_content` with a short teaser (< 25 words).
  - Assert call completes in ~1.5s and returns the fallback teaser without error.
- **Implementation**:
  - In [crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs), set `reqwest::Client` timeout to `Duration::from_millis(1500)`.
  - Update `Self::truncate_words(&extracted_text, 130)`.

---

## Phase 2: Canonical Page Cache Key & Deduplication

### Task 2.1: Canonical Page Key Consistency
- **Test**: `crates/server/src/ai/mod.rs` &rarr; `tests::test_canonical_page_key_consistency`
  - Assert `compute_page_key("unread", &[1, 2, 3]) == compute_page_key("unread", &[1, 2, 3])`.
  - Assert empty slice returns a valid deterministic key with prefix.
- **Implementation**:
  - Ensure `compute_page_key` in [crates/server/src/ai/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs) takes `(status: &str, entry_ids: &[i64])`.

### Task 2.2: Shared In-Flight Mutex across Prefetch & Executive Brief
- **Test**: `crates/server/src/ai/prefetch.rs` &rarr; `tests::test_prefetch_and_executive_brief_share_page_key`
  - Verify that `AiPrefetchService::spawn_prefetch_job` calculates `page_key` using the full list of page entry IDs and query status.
  - Verify that when `page_key` is inserted in `ai_service.in_flight`, `executive_brief_htmx` detects it and avoids redundant generation.
- **Implementation**:
  - Update `AiPrefetchService::spawn_prefetch_job` to accept `(status: String, page_entry_ids: Vec<i64>, candidate_entries: Vec<T>)`.
  - Compute `batch_key = crate::ai::compute_page_key(&status, &page_entry_ids);` in `AiPrefetchService::run_prefetch`.
  - Update caller in `crates/server/src/routes/entries.rs` to pass `(status_str.to_string(), entry_ids.clone(), candidates)`.

---

## Phase 3: Batch Prompt Formatting, 130-Word Truncation, Top-1 Rewrite & Grounding

### Task 3.1: 130-Word Input Truncation
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_batch_prompt_input_truncation_130_words`
  - Provide an article input with 350 words.
  - Call `format_batch_prompt_structured`.
  - Assert the article content in the generated prompt contains at most 130 words.
- **Implementation**:
  - In [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs), change `truncate_words(clean_content, 300)` to `truncate_words(clean_content, 130)`.

### Task 3.2: Eager Top-1 Rewrite & Remaining Deferral
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_format_batch_prompt_eager_rewrite_top_1_only`
  - Pass 3 new articles to `format_batch_prompt_structured`.
  - Assert prompt explicitly states that `"r"` must be generated **only** for the first article ID, and omitted for the rest.
  - Assert prompt specifies inverted-pyramid style and < 110 words limit (30–60 words sweet spot).
- **Implementation**:
  - Update prompt template in [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) to instruct eager rewrite generation strictly for the lead item.

### Task 3.3: Strict Grounding Constraint
- **Test**: `crates/server/src/ai/llm.rs` &rarr; `tests::test_format_batch_prompt_strict_grounding_constraint`
  - Verify that the formatted prompt includes the exact `GROUNDING CONSTRAINT` anti-hallucination directive.
- **Implementation**:
  - Add grounding directive to `format_batch_prompt_structured` and `format_prompt` in `crates/server/src/ai/llm.rs`.

---

## Phase 4: HTMX Lazy-Loading Detail Rewrite Route

### Task 4.1: Template Rendering with `#ai-rewrite-box` Skeleton
- **Test**: `crates/server/src/routes/entries.rs` &rarr; `tests::test_entry_detail_renders_skeleton_when_rewrite_missing`
  - Render `EntryDetailTemplate` with `is_rewrite = false`.
  - Assert `#ai-rewrite-box` contains `hx-get="/hx/entries/{id}/rewrite"` and `hx-trigger="load"`.
  - Assert `#ai-rewrite-box` contains skeleton loader markup (`animate-pulse`).
  - Assert template does NOT dump raw article HTML or duplicate summary below skeleton.
- **Test**: `crates/server/src/routes/entries.rs` &rarr; `tests::test_entry_detail_renders_rewrite_directly_when_cached`
  - Render `EntryDetailTemplate` with `is_rewrite = true` and `rewrite = "Leading rewrite text"`.
  - Assert `#ai-rewrite-box` contains "Leading rewrite text" directly without `hx-get` attribute.
- **Implementation**:
  - Update [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html) to render `#ai-rewrite-box` conditionally based on `is_rewrite`.

### Task 4.2: Route Handler `GET /hx/entries/{id}/rewrite`
- **Test**: `crates/server/src/routes/entries.rs` &rarr; `tests::test_get_entry_rewrite_htmx_serves_existing_cache`
  - Seed `AiDb` with an existing rewrite for entry 501.
  - Send `GET /hx/entries/501/rewrite`.
  - Assert response is 200 OK, returns `#ai-rewrite-box` with cached rewrite, and runs in < 5ms without LLM invocation.
- **Test**: `crates/server/src/routes/entries.rs` &rarr; `tests::test_get_entry_rewrite_htmx_generates_and_caches_rewrite`
  - For entry 502 without cached rewrite, send `GET /hx/entries/502/rewrite`.
  - Assert handler generates rewrite via `AiService`, saves it to `AiDb`, and returns `#ai-rewrite-box`.
- **Test**: `crates/server/src/routes/entries.rs` &rarr; `tests::test_get_entry_rewrite_htmx_fallback_on_llm_failure`
  - When `AiService` returns error / timeout for entry 503, send `GET /hx/entries/503/rewrite`.
  - Assert handler does not fail with 500, but renders existing executive brief inside `#ai-rewrite-box` gracefully.
- **Implementation**:
  - Add `get_entry_rewrite_htmx` in [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs).
  - Register route in [crates/server/src/app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs).

### Task 4.3: On-Demand Single Rewrite in `AiService`
- **Test**: `crates/server/src/ai/mod.rs` &rarr; `tests::test_generate_single_rewrite`
  - Call `AiService::generate_rewrite(entry_id, title, content)`.
  - Assert returned rewrite is formatted and saved in `AiDb`.
- **Implementation**:
  - Implement `generate_rewrite` in `AiService` and `LlmBackend`.

---

## Phase 5: Static CSS Delivery & CDN Elimination

### Task 5.1: Static Route in Axum
- **Test**: `crates/server/src/app.rs` &rarr; `tests::test_serve_static_annuaire_theme_css_route`
  - Send `GET /static/annuaire-theme.css`.
  - Assert status 200 OK.
  - Assert `Content-Type` header is `text/css; charset=utf-8`.
  - Assert body contains required styles and design tokens.
- **Implementation**:
  - Add handler `serve_annuaire_theme_css` in `crates/server/src/routes/pages.rs`.
  - Register `.route("/static/annuaire-theme.css", get(serve_annuaire_theme_css))` in [crates/server/src/app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs).

### Task 5.2: Template Migration in `base.html`
- **Test**: `crates/ui-components/src/templates.rs` &rarr; `tests::test_base_template_excludes_tailwind_cdn_script`
  - Render `base.html`.
  - Assert output includes `<link rel="stylesheet" href="/static/annuaire-theme.css">`.
  - Assert output does NOT include `https://cdn.tailwindcss.com`.
  - Assert output does NOT include inline `tailwind.config = {`.
- **Implementation**:
  - Edit [crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) to link static stylesheet and remove runtime CDN script.

---

## Phase 6: Integration Verification

### Task 6.1: Full Pipeline Verification
- **Test**: `crates/server/tests/e2e_tests.rs` &rarr; `test_e2e_page_load_and_detail_drawer_rewrite_flow`
  - Run full test suite (`cargo test --workspace`).
  - Verify all unit and integration tests pass cleanly.
