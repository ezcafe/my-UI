# Analysis: LLM Error Toast Handling

## Result: done

## Context & Problem Statement
Currently, when on-demand LLM operations (article rewrite generation via `get_entry_rewrite_htmx` or fallacy/impact analysis via `get_entry_analysis_htmx`) fail due to timeouts, rate limits, or process crashes:
1. `get_entry_rewrite_htmx` renders fallback teaser text inside `#ai-rewrite-box`.
2. `get_entry_analysis_htmx` renders an empty hidden div `<div id="ai-analysis-box" class="hidden"></div>`.
In both cases, no feedback is given to the reader, leaving them uncertain whether AI features are loading, disabled, or failed. Conversely, displaying raw backend errors (such as `OpenAI API error (503)`, `Process failed`, or CLI timeouts) degrades user experience and leaks internal implementation details.

## Core Requirements
1. **User-Friendly Error Message**:
   Exact string:
   `"Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later."`
2. **Non-Blocking Delivery**:
   Display as a floating toast notification using HTMX out-of-band swap (`hx-swap-oob="beforeend"`) into `#toast-container`.
3. **Graceful Fallback Preservation**:
   The primary reading UI must stay intact. For rewrites, the fallback teaser/summary is rendered. For analysis, the skeleton loading state is cleaned up by replacing `#ai-analysis-box`.
4. **No Technical Leaks**:
   Never expose HTTP 503, stack traces, CLI stderr, or JSON parse errors in user-facing views.

## Risk Assessment & Mitigations

### 1. HTTP Status Code in HTMX Swaps
- **Risk**: Returning HTTP 500 or 503 will cause HTMX to trigger `htmx:responseError` and abort DOM swaps, leaving loading skeletons frozen on screen and blocking OOB toast delivery.
- **Mitigation**: Return HTTP 200 OK containing both the graceful fallback HTML fragment for the primary target and the OOB `ToastTemplate` HTML.

### 2. Icon Rendering for Out-Of-Band Swaps
- **Risk**: `base.html` currently listens to `htmx:afterSwap` on `evt.detail.target`. For OOB swaps, HTMX fires `htmx:oobAfterSwap`. If `htmx:oobAfterSwap` is not handled, Lucide icons (`alert-circle` and `x`) inside the newly injected toast may not render as SVGs.
- **Mitigation**: Add an `htmx:oobAfterSwap` event listener in `base.html` to run `lucide.createIcons({ root: evt.detail.target })`.

### 3. Duplicate / Stacking Toasts
- **Risk**: Opening an uncached article can trigger both `/hx/entries/{id}/rewrite` and `/hx/entries/{id}/analysis` concurrently. If both fail, two identical high-demand toasts could appear simultaneously.
- **Mitigation**:
  - `ToastTemplate` uses `hx-swap-oob="beforeend"`, allowing them to stack without crashing DOM.
  - To prevent noisy duplicate warnings, either client-side deduplication in `#toast-container` or auto-dismiss timer ensures transient warnings don't clutter the screen.

### 4. Toast Dismissal & Auto-Cleanup
- **Risk**: Toasts without auto-dismissal accumulate in `#toast-container` if the reader does not manually click the close button.
- **Mitigation**: Add a 5-second auto-fade/remove timer on toast elements upon creation or swap, while keeping the manual `dismissToast(this)` button for instant dismissal.

## Open Questions for Grill
1. **Deduplication / Cooldown**: If both rewrite and analysis fail at the same second for an entry, should the client show two stacked toasts or deduplicate identical warning messages?
2. **Scope of Endpoints**: Should this toast warning apply to `get_entry_rewrite_htmx`, `get_entry_analysis_htmx`, and `render_executive_brief_htmx`, while keeping `get_entry_detail_htmx` purely cached?
