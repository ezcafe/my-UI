# Test log

**Result:** success  
**Test path last run:** lite  
**Smoke source:** Build-verify  
**Updated:** 2026-11-27T07:30:10+07:00

## Smoke

- **Build-verify (`2026-11-27 07:07 +07`):**
  - `cargo test --workspace` — **PASS** (`269 passed, 0 failed`: `miniflux-client` 24 passed, `server` unit 147 passed, `server` e2e 8 passed, `ui-components` 90 passed).
  - `cargo clippy --workspace --all-targets -- -D warnings` — **PASS** (`0 warnings`, `0 errors`).
  - `rustfmt --check --edition 2021` on all modified Rust files (`crates/ui-components/src/sanitizer.rs`, `crates/ui-components/src/templates.rs`, `crates/server/src/routes/entries.rs`, `crates/server/src/routes/pages.rs`, `crates/server/tests/e2e_tests.rs`) — **PASS**.

## Suite / e2e

- **Test-lite (`2026-11-27T07:30:10+07:00`):**
  - `cargo test --workspace` (outside sandbox for loopback `wiremock` TCP) — **PASS** (`269 passed, 0 failed, 0 ignored`: `miniflux-client` lib 24 passed, `server` lib 147 passed, `server` e2e `tests/e2e_tests.rs` 8 passed, `ui-components` lib 90 passed; doc-tests 0).
  - Covers all planned feature tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md): `sanitizer::tests::test_extract_plain_summary_strips_html_and_truncates_content`, `templates::tests` metadata omission / section order / Guest JSON / `AnalysisCardsTemplate` OOB / `index.html` client sync & cache guards, `routes::entries::tests` image scheme parity / `/hx/entries/{id}` / `/hx/entries/{id}/rewrite` / `/hx/executive-brief` / guest list embedding, and `e2e_tests::test_e2e_page_load_and_detail_drawer_rewrite_flow` + `test_e2e_detail_drawer_analysis_flow`.
