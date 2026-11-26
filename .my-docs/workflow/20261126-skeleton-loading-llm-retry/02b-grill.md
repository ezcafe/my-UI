# Grill log — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

**Result:** frontier-empty

**Grill open** = ¬Notes `grill-skip` ∧ (no `02b` ∨ Result ≠ `frontier-empty`). Closed here via Result `frontier-empty`.

## Design Tree & Settled Decisions

All design branches were settled by the user's confirmation in [`01-idea.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/01-idea.md) and [`00-run.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/00-run.md) (`ideation-confirmed`):

1. **Loading Skeleton Presentation & Triggers:**
   - **Choice settled:** View-aware `animate-pulse` skeleton inside `#entries-loading-overlay` (3-column `.magazine-3col-grid` skeleton when `currentViewMode === 'ai'`, stacked Annuaire row skeletons when `currentViewMode === 'list'`) triggered on Next, Previous, and Mark Done, with `motion-reduce:animate-none`, `aria-busy`, `aria-hidden="true"` on shapes, and `sr-only` loading label.
2. **LLM Auto-Retry Timing & Scope:**
   - **Choice settled:** Wait 1 second (`tokio::time::sleep(Duration::from_secs(1))`) and automatically retry once (2 total attempts) across `LlmCliRunner` methods (`summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, `analyze_impact_and_fallacies`) and `LlmHttpClient::request_chat_completion`, plus coordinate `executive_brief_htmx` `in_flight` wait so it does not time out at 4 seconds prematurely.
3. **Failure Feedback After Retry Exhaustion:**
   - **Choice settled (Option 1):** Keep inline `AiErrorCardTemplate` with manual Retry buttons (`#page-executive-brief`, `#list-executive-brief`, `#ai-analysis-box`, `#ai-rewrite-box`), extend `AiErrorCardTemplate` to `/hx/entries/{id}/ai-summary`, AND append `crate::error::render_llm_error_toast()` (`#toast-container` OOB) across failed AI endpoints, with client-side toast deduplication and `.is-error` IndexedDB cache exclusion.

## Round log

### Round 1

No open frontier questions remain; all product decisions are settled.
