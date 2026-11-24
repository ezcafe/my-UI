# Gate A Review: Article Detail AI Rewrite with Impact & Fallacy Analysis

- **Stage:** gate-a
- **Status:** completed
- **Result:** `ok`

---

## 1. Scope and Clarity Evaluation

- **Clarity:** Excellent. The user provided explicit confirmation and selected decisions:
  - **Area 1:** Option 1A (Strict JSON schema for LLM: `{"rewrite": "...", "impact": "...", "fallacies": "..."}`).
  - **Area 2:** Option 2A (Dedicated `impact TEXT` and `fallacies TEXT` columns in PostgreSQL `article_ai_summaries`).
  - **Area 3:** Option 3B (Two separate card boxes styled like executive brief: one for Impact Analysis, one for Fallacy & Objectivity Check).
  - **Reassuring Indicator:** If no fallacy is detected, explicitly render a reassuring message (e.g. *"Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"* or English equivalent matching article language).
  - **Details Page Scope:** Remove raw original article content (`sanitized_content`) completely from the details drawer. Only show Brief, Impact, Fallacy, and Rewrite.

- **Scope Boundaries:**
  - **In scope:**
    - `crates/ui-components/templates/partials/entry_detail.html`: remove `sanitized_content`, render two highlight cards for impact and fallacies (handling skeleton/lazy load states).
    - `crates/server/src/ai/llm.rs`: update prompt to request inverted pyramid rewrite + impact + fallacies in strict JSON format; parse structured response; support reassurance fallback.
    - `crates/server/src/ai/db.rs`: update schema (`ADD COLUMN IF NOT EXISTS impact TEXT;`, `ADD COLUMN IF NOT EXISTS fallacies TEXT;`), update `AiSummaryRecord`, `save_summary_and_title_with_rewrite` (or extend to save impact and fallacies), update mock DB.
    - `crates/server/src/ai/mod.rs` & `crates/server/src/routes/entries.rs`: update `generate_rewrite` / rewrite HTMX route to pass and return the new fields.
    - Unit tests, integration tests, and E2E tests verifying the rewrite flow, no raw content, and highlight cards.
  - **Out of scope:**
    - Modifying feed list view or executive summary batch cards.
    - Changing external publisher links.

- **Risks & Feasibility:**
  - Fully feasible within existing Rust `sqlx` + PostgreSQL architecture.
  - Token budget is well within limits (~300 extra output tokens per single article detail view).
  - Backward compatibility: existing cached summaries without impact/fallacies will either show empty or trigger on-demand rewrite/analysis when requested.

---

## 2. Verdict

- **Gate A Result:** `ok`
- **Proceed to:** `skim`
