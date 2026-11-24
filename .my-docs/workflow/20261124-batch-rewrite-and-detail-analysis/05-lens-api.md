# Lens: API Contract Review

- **Workflow Slug:** `20261124-batch-rewrite-and-detail-analysis`
- **Lens:** api
- **Result:** clean

## Summary

The API additions and modifications for inverted-pyramid rewrite text and asynchronous detail analysis adhere to best practices:
1. `GET /hx/entries/{id}` immediately provides the cached `rewrite` (0ms delay) and removes the Executive Brief card header.
2. `GET /hx/entries/{id}/analysis` serves the decoupled analysis fragment (`#ai-analysis-box`) asynchronously via HTMX, serving from cache or triggering on-demand LLM analysis with robust fallbacks.
3. Batch prompt schema requests inverted-pyramid rewrite `r` for new items, bidirectionally parsing and supporting both `b` and `r` for maximum backward compatibility.

---

## API Checklist

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Typed input and output** | Pass | Strong Rust types: `BatchAiItem` (`b`, `r`), `AiSummaryRecord` (`impact`, `fallacies`, `rewrite`), `EntryDetailTemplate` (`has_analysis`), `AnalysisCardsTemplate`. |
| **Additive changes only** | Pass | `GET /hx/entries/{id}/analysis` is an additive endpoint. Batch parser supports both `"b"` and `"r"`. |
| **Stable error shapes** | Pass | If LLM analysis fails or times out, the endpoint falls back gracefully to an empty or error fragment without crashing or leaking details. |
| **Edge validation** | Pass | Path parameter `id: i64` validated at the Axum router edge. |
| **Lists bounded** | Pass | Truncation of article content bounded to 130 words per article; batch size strictly limited to page entries. |
| **Mutating endpoints idempotent** | Pass | `save_analysis` executes an idempotent parameterized UPDATE on SQLite. |
| **Design parity** | Pass | Implementation matches all contracts and sequence diagrams in `03-design.md`. |
| **Leak-free responses** | Pass | All dynamic strings in HTML templates are sanitized with HTML entity escaping (`escape_html` and Askama auto-escaping). Zero internal errors or paths exposed. |

---

## Findings

- **No Critical or Major issues found.**
- **Contracts Verified**:
  - `GET /hx/entries/{id}/analysis`: returns `<div id="ai-analysis-box">...</div>` with Impact Analysis and Logical Fallacy cards.
  - `GET /hx/entries/{id}`: renders rewrite on initial load, omitting Executive Brief heading.
  - Batch prompt & response: inverted-pyramid `r` requested and bidirectionally synchronized into `brief` and `rewrite`.
- **Verification**: All 215 workspace tests pass cleanly (`cargo test`).
