Result: clean

# Design Review: Details Page AI Rewrite with Batch Pre-request

- **Workflow Slug:** `20261123-details-ai-rewrite-batch-pre-request`
- **Stage ID:** `design-review`
- **Result:** clean
- **Review Date:** 2026-11-23

---

## 1. Executive Summary & Verdict

This review evaluates the overall system architecture, cross-cutting concerns, UI integration, security, and task planning for the **Details Page AI Rewrite with Batch Pre-request** feature defined in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/04-tasks.md).

- **Isolated Reviews Prerequisite:**
  - [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03a-api-contract-review.md) is paid and `clean`.
  - [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03a-db-design-review.md) is paid and `clean`.
- **Overall Verdict:** `Result: clean`.
- **Key Takeaway:** Zero Critical or Major issues. The architectural synthesis between the background prefetch pipeline, SQLite caching, Axum route handlers, and Askama HTML templates is cohesive, performant (< 5ms response time), secure, and fully aligned with Annuaire design principles and user specifications.

---

## 2. Evaluation of Cross-Cutting Concerns

### 2.1 Overall Architecture & Cohesion

- **Unified Pipeline:** Combining titles (`t`), briefs (`b`), rewrites (`r`), and page overviews (`top`) into a single batch LLM request avoids redundant roundtrips and amortizes system prompt overhead (~300 tokens once vs. 20 separate times).
- **Decoupled Asynchrony:** Background prefetching (`AiPrefetchService`) operates asynchronously on feed page load without blocking Axum HTTP workers or client responses.
- **Cache-First Reading:** The detail view reads directly from local SQLite (`article_ai_summaries`), ensuring an immediate (< 5ms) 60fps view transition with zero network latency.

### 2.2 UI Integration & Annuaire Design System

- **Visual Hierarchy & Typography:**
  - Maintains Annuaire styling: Work Sans typography, tabular figures for metadata/dates, `leading-relaxed`, and high contrast `text-foreground/90`.
  - Inverted Pyramid cognitive model delivers the core takeaway and impact in the first two sentences, followed by 1–2 bullet points (`whitespace-pre-line`).
- **Responsive Layout & Fluid Design:**
  - Container width is cleanly constrained using `max-w-[44rem] mx-auto` with fluid margins, avoiding hardcoded pixel breakpoints.
  - Action bar buttons use compact, circular, and pill-shaped touch targets (`h-8`, `rounded-full`) that gracefully adapt to narrow mobile screens and wider desktop panels.
- **Graceful Fallback (Settled Option 1A):**
  - If a rewrite is not yet generated or is pending, `EntryDetailTemplate` renders the 3-sentence executive brief (`summary`) or teaser via `brief_card.html` (`is_rewrite: false`).
  - Raw HTML (`sanitized_content`) is completely omitted from the DOM as intended, while the header retains the "Original" button directing to `entry.url`.

### 2.3 Security & Hardening

- **XSS Protection:** AI rewrite content is rendered with `{{ rewrite }}` (auto-escaped by Askama) instead of raw unescaped HTML (`|safe`), preventing malicious RSS payloads from injecting script tags into the detail view.
- **Injection Safety:**
  - All database queries use parameterized SQL bindings (`?1, ?2, ...`).
  - LLM CLI execution invokes `std::process::Command` with direct arguments, eliminating shell interpolation vulnerabilities.
- **Token & Resource Safety:**
  - 300-word input truncation safely caps batch prompt size at ~6,000–7,000 tokens for 20 entries, well below CLI buffer limits (`ARG_MAX`) and model context boundaries.
  - Concurrency guard via `ai.in_flight` locks on `batch_key` to avoid thundering-herd duplicate LLM invocations.

### 2.4 Performance & Reliability

- **SQLite WAL Concurrency:** Detail route reads are non-blocking alongside background prefetch writes; 5000ms busy timeout prevents lock contention.
- **Upsert Safeguards:** `COALESCE` logic in `save_batch_results` and `save_summary_and_title` ensures partial updates or fallback entries do not nullify existing cached rewrites.

---

## 3. Review of Task Breakdown (`04-tasks.md`)

- **TDD Compliance:** The plan establishes 10 concrete, test-first milestones across 5 sequential phases:
  1. Database Evolution (`AiDb` Migration v4, CRUD, COALESCE logic).
  2. Prompt Formatting & 300-Word Truncation (whitespace-aware).
  3. Response Parsing & Regex Fallback (group capture for `r`).
  4. Prefetch Service Persistence.
  5. Detail View UI & Route Integration.
- **Scope & Independence:** Each task has clearly defined assertions and isolated file scopes, preventing regressions in existing feed or summary functionality.

---

## 4. Minor Implementation Notes (Non-blocking)

1. **Askama Template Struct Fields:** Ensure `EntryDetailTemplate` retains `pub summary: String` so that `brief_card.html` compiles cleanly when `is_rewrite: false`.
2. **Whitespace-Aware Word Counter:** Ensure `truncate_words` tracks whitespace transitions (`char::is_whitespace`) to avoid multi-space counting skew.
3. **Empty String Protection:** Treat whitespace-only rewrites as `None` in `routes/entries.rs` to guarantee proper triggering of fallback rendering.

---

## 5. Review Findings

| Severity | Count | Summary |
| :--- | :--- | :--- |
| **Critical** | 0 | None |
| **Major** | 0 | None |
| **Minor** | 0 | 3 implementation notes incorporated above |

---

## 6. Conclusion & Recommendation

The overall design is comprehensive, robust, and clean. All prerequisite isolated reviews are satisfied, and the task breakdown provides an actionable roadmap for Test-Driven Development.

**Result: clean**
