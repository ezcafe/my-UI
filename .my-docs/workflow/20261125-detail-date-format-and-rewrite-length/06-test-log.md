# Test Log: Details Date Format (`DD/MM/YYYY`), Longer AI Rewrite, Guest Client Details & Tab View Rules

- **Run slug:** `20261125-detail-date-format-and-rewrite-length`
- **Updated:** 2026-11-25T07:50:20+07:00
- **Test path:** lite
- **Result:** success
- **Smoke source:** Build-verify

---

## 1. Test Suite Verification Summary

| Target / Crate | Command | Passed | Failed | Ignored | Status |
|----------------|---------|--------|--------|---------|--------|
| `server` (`tests/e2e_tests.rs`) | `cargo test --test e2e_tests` | 8 | 0 | 0 | PASSED |
| `miniflux-client` (`src/lib.rs`) | `cargo test --workspace` | 24 | 0 | 0 | PASSED |
| `server` (`src/lib.rs`) | `cargo test --workspace` | 140 | 0 | 0 | PASSED |
| `server` (`tests/e2e_tests.rs`) | `cargo test --workspace` | 8 | 0 | 0 | PASSED |
| `ui-components` (`src/lib.rs`) | `cargo test --workspace` | 76 | 0 | 0 | PASSED |
| **Total Workspace** | `cargo test --workspace` | **248** | **0** | **0** | **ALL PASSED** |

Note: Initial sandboxed run of `cargo test --test e2e_tests` blocked local TCP loopback connections (`127.0.0.1`) used by `wiremock::MockServer`; re-run with local loopback access enabled passed all 8 E2E tests and all 248 workspace tests.

---

## 2. Static Analysis & Formatting

- `cargo clippy --all-targets -- -D warnings`: exit code 0 (0 warnings)
- `cargo fmt --check`: exit code 0 (clean)

---

## 3. Feature & Targeted E2E Coverage Verified

### Round 1 Baseline (Date Format & AI Rewrite Length)
- `templates::tests::test_format_published_date_ddmmyyyy_parses_rfc3339_and_falls_back` — ok
- `templates::tests::test_entry_detail_renders_ddmmyyyy_time_element_in_both_headers` — ok
- `templates::tests::test_index_template_formats_detail_dates_on_swap_and_cache_restore` — ok
- `routes::entries::tests::test_get_entry_detail_formats_published_at_as_ddmmyyyy` — ok
- `ai::llm::tests::test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid` — ok
- `ai::llm::tests::test_format_batch_prompt_requires_strictly_3_to_5_sentences` — ok
- `ai::llm::tests::test_format_prompt_requires_strictly_3_to_5_sentences` — ok
- `ai::extractor::tests::test_extract_article_fallback_on_http_error_or_block` — ok

### Round 2 (Guest Client Details & Tab View Rules)
- `templates::tests::test_entry_list_guest_json_embeds_sanitized_content_and_escapes_script_tags` — ok
- `templates::tests::test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` — ok
- `templates::tests::test_index_template_tab_view_rules_and_toggle_visibility` — ok
- `templates::tests::test_index_template_renders_ai_toggle_button_and_scripts` — ok
- `routes::entries::tests::test_list_routes_embed_sanitized_guest_data_only_when_auth_disabled` — ok
- `routes::entries::tests::test_get_entry_detail_guest_mode_keeps_route_active_and_omits_author_and_reading_time` — ok

### Targeted E2E Suite (`crates/server/tests/e2e_tests.rs`)
- `test_app_starts_and_responds_to_health_check` — ok
- `test_mutation_endpoints_reject_with_403_when_auth_disabled` — ok
- `test_executive_brief_ineligible_views_and_empty_guard` — ok
- `test_executive_brief_cache_hit_and_oob_swaps` — ok
- `test_e2e_detail_drawer_analysis_flow` — ok
- `test_e2e_authenticated_session_flow` — ok
- `test_e2e_guest_mode_flow` — ok
- `test_e2e_page_load_and_detail_drawer_rewrite_flow` — ok
