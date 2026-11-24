# Analysis: Contextual AI Error Display & Inline Recovery

## Result: done

## Context & Problem Statement
When background AI services fail (due to API rate limits, transient network timeouts, or high model demand), the application currently hides target containers (`class="hidden"`) and emits a detached floating toast notification (`render_llm_error_toast()`).

This causes three user experience issues:
1. **Disorienting Layout Shifts**: Target containers (such as `#page-executive-brief` or `#ai-analysis-box`) vanish or collapse into empty zero-height elements.
2. **Disconnected Feedback**: Floating toasts appear away from the user's reading focal point, making it difficult to understand which specific block of content failed to load.
3. **No Direct Recovery**: The user cannot easily re-trigger the failed AI generation without refreshing the entire page or re-selecting filters.

The proposed solution replaces floating toasts with **contextual inline error cards** positioned directly within the target element, complete with an actionable HTMX "Retry" button.

---

## Core Requirements & Specifications

### 1. Executive Brief Contextual Error (`#page-executive-brief`)
- When batch AI brief synthesis fails or returns empty items/top:
  - Return an inline card with `id="page-executive-brief"`.
  - Include an alert icon (`alert-circle` or `alert-triangle`), an explicit status label (e.g., "Executive Brief Unavailable"), and the friendly message (`LLM_HIGH_DEMAND_ERROR_MESSAGE`).
  - Provide an inline `<button>` with `hx-get="/hx/executive-brief?...query_params..."`, `hx-target="#page-executive-brief"`, and `hx-swap="outerHTML"`.
  - Preserve all active filter parameters (`status`, `offset`, `limit`, `feed_id`, `category_id`) in the retry URL so retrying respects the user's current view.

### 2. Dual-View Synchronization (`#list-executive-brief`)
- In List View, `#list-executive-brief` currently pulses until `#page-executive-brief` finishes loading.
- When `#page-executive-brief` swaps an error card, the `htmx:afterSwap` listener in `index.html` must detect the error state, clear the pulse animation (`animate-pulse`), and render the matching error card with a retry button inside `#list-executive-brief`.
- Clicking "Retry" in either view triggers the HTMX request and updates both views upon response.

### 3. Article Detail Analysis Box (`#ai-analysis-box`)
- When `/hx/entries/{id}/analysis` fails:
  - Instead of `<div id="ai-analysis-box" class="hidden"></div>`, return `<div id="ai-analysis-box">` containing the contextual error card.
  - Include a "Retry Analysis" button with `hx-get="/hx/entries/{id}/analysis"`, `hx-target="#ai-analysis-box"`, and `hx-swap="outerHTML"`.

### 4. Article Detail Rewrite Box (`#ai-rewrite-box`)
- When `/hx/entries/{id}/rewrite` fails:
  - Render `<div id="ai-rewrite-box">` containing the contextual error card with a "Retry Rewrite" button (`hx-get="/hx/entries/{id}/rewrite"`, `hx-target="#ai-rewrite-box"`, `hx-swap="outerHTML"`).
  - Also display the fallback teaser or existing executive summary below the error card so the reader can still read the article uninterrupted.

### 5. Floating Toast Removal
- Remove `render_llm_error_toast()` from the AI failure responses in `crates/server/src/routes/entries.rs`.
- All feedback for these operations becomes strictly localized and contextual.

---

## Risk Assessment & Mitigations

### 1. Query Parameter Reconstruction for Executive Brief Retry
- **Risk**: If the server fails to reconstruct the original query parameters (`offset`, `limit`, `status`, `feed_id`, `category_id`), the retry button might request the default unread feed rather than the user's current filtered view.
- **Mitigation**: Construct the retry URL directly from the parsed `EntriesFilterQuery` in `executive_brief_htmx`.

### 2. Double-Click / Rapid Retries
- **Risk**: Rapid clicks on "Retry" could dispatch multiple concurrent LLM calls.
- **Mitigation**: Use standard HTMX button states (`hx-disabled-elt="this"`) and visual feedback (`this.disabled = true; this.classList.add('opacity-50')`) during in-flight requests.

### 3. Lucide Icon Initialization on Dynamic Re-render
- **Risk**: Newly injected error cards contain `<i data-lucide="...">` icons that need client-side SVG rendering.
- **Mitigation**: Ensure `lucide.createIcons()` is triggered on `htmx:afterSwap` for the swapped element (which `base.html` already executes).

### 4. Test Suite Parity
- **Risk**: Existing tests in `crates/server` asserted `<div id="page-executive-brief" class="hidden"></div>` or the presence of `#toast-container`.
- **Mitigation**: Update assertions to test the new contextual error card markup, verifying the absence of `#toast-container` and presence of the retry button.
