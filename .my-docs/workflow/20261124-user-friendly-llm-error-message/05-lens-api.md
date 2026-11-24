# Lens: API Contract Review

- **Workflow Slug:** `20261124-user-friendly-llm-error-message`
- **Lens:** api
- **Result:** clean

## Summary

The API modifications deliver non-blocking toast notifications during LLM outages and timeouts across `get_entry_rewrite_htmx`, `get_entry_analysis_htmx`, and `executive_brief_htmx`. The endpoints return valid HTTP 200 responses containing fallback content for the requested container and an out-of-band swap element (`#toast-container` with `hx-swap-oob="beforeend"`), strictly adhering to the architectural contract in `03-design.md`.

---

## API Checklist

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Typed input and output** | Pass | Strong Rust types: `ToastTemplate`, `render_llm_error_toast()` returning standard HTML fragment, Axum `Html(format!("{primary}{toast}")).into_response()`. |
| **Additive changes only** | Pass | Extends existing HTMX endpoints with additive OOB toast fragments on failure arms; no breaking alterations to request contracts or parameters. |
| **Stable error shapes** | Pass | Returns HTTP 200 with predictable fallback markup and standardized toast payload rather than returning inconsistent raw HTTP 500 status codes. |
| **Edge validation** | Pass | Path and query parameters (`id: i64`, `query: EntriesFilterQuery`) validated at the Axum router edge. |
| **Lists bounded** | Pass | Executive brief synthesis batch bounded to 15 items per page; toast element bounded to single message per event. |
| **Mutating endpoints idempotent** | Pass | Error notification generation is read-only; no unintended side effects or database mutations occur during toast generation. |
| **Design parity** | Pass | Exact parity with `03-design.md` Option 1 specifications: uses `LLM_HIGH_DEMAND_ERROR_MESSAGE`, `render_llm_error_toast()`, and `base.html` lifecycle hooks. |
| **Leak-free responses** | Pass | Internal error representations and stack traces are suppressed. Constant user-friendly message prevents technical exposure (CWE-209). |

---

## Findings

- **No Critical or Major issues found.**
- **Contracts Verified**:
  - `GET /hx/entries/{id}/rewrite`: On error, returns fallback summary/teaser in `#ai-rewrite-box` + `#toast-container` OOB toast.
  - `GET /hx/entries/{id}/analysis`: On error, returns hidden `#ai-analysis-box` + `#toast-container` OOB toast.
  - `GET /hx/executive-brief`: On empty synthesis or LLM error, returns hidden `#page-executive-brief` + `#toast-container` OOB toast.
- **Verification**: All 225 workspace tests pass (`cargo test --all`), and all 8 end-to-end tests pass (`cargo test --test e2e_tests`).
