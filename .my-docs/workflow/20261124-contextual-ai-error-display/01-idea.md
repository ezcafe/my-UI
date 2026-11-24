# Idea & Framing: Contextual AI Error Display

## Summary
When an AI request fails (for instance, due to high demand or network timeout), display the error message directly within the target UI container where the content was expected (such as the Executive Brief card or the Article Analysis box), rather than collapsing the container to `class="hidden"` and relying solely on a floating toast notification.

## Current vs Proposed State
- **Current State**: On LLM failure, endpoints return hidden divs (e.g., `<div id="page-executive-brief" class="hidden"></div>` or `<div id="ai-analysis-box" class="hidden"></div>`) accompanied by an out-of-band floating toast (`render_llm_error_toast()`). In list view, `#list-executive-brief` remains pulsing or disappears abruptly.
- **Proposed State**: On LLM failure, endpoints return a contextual error card inside the target container (`#page-executive-brief`, `#ai-analysis-box`), informing the reader what happened with an inline retry button, while syncing cleanly to `#list-executive-brief` in list view.

## Goals
- **Contextual Feedback**: Show clear, readable error state right where the user is looking.
- **Actionable Recovery**: Include an inline "Retry" button using HTMX to let the user re-trigger the AI generation directly without refreshing the page.
- **Layout Stability**: Prevent jarring layout shifts where cards suddenly vanish from the page.
- **Dual-View Support**: Ensure both Magazine View (`#page-executive-brief`) and List View (`#list-executive-brief`) display the error state gracefully.

## Non-Goals
- Changing the backend LLM models or prompt engineering.
- Modifying non-AI error handling (such as Miniflux upstream GraphQL failures).

## Anti-Goals
- Leaving infinite skeleton pulsing loaders when an error occurs.
- Hiding containers completely so the user has no clue why expected AI insights are missing.

## Tentative Scope & Architecture
- **Has UI (tentative)**: yes
- **Has API (tentative)**: yes
- **Has DB (tentative)**: no
- **Has auth/PII/schema (tentative)**: no

## Decision & Options for Error Presentation

### Option 1 (Recommended): Dedicated Contextual Error Card with Inline Retry (Replace Toast)
- **Explanation**: Render a clean inline alert component directly inside `#page-executive-brief` and `#ai-analysis-box`. The card displays a friendly notice (e.g. "AI generation temporarily unavailable due to high demand") along with a compact "Retry" button that re-triggers the HTMX request. Remove the floating toast so feedback is 100% contextual.
- **Concrete Example**:
  ```html
  <div id="page-executive-brief" class="rounded-lg p-5 bg-muted/40 border border-border flex flex-col items-start gap-3">
    <div class="flex items-center gap-2 text-muted-foreground">
      <i data-lucide="alert-circle" class="w-4 h-4 text-destructive"></i>
      <span class="text-xs font-bold uppercase tracking-wider">Executive Brief Unavailable</span>
    </div>
    <p class="text-xs text-muted-foreground leading-relaxed">
      This model is currently experiencing high demand.
    </p>
    <button hx-get="/hx/executive-brief?..." hx-target="#page-executive-brief" hx-swap="outerHTML"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-secondary text-secondary-foreground text-xs font-medium hover:bg-secondary/80 transition-colors">
      <i data-lucide="refresh-cw" class="w-3 h-3"></i>
      <span>Retry</span>
    </button>
  </div>
  ```
- **Pros**:
  - Direct, clear user feedback where the user's focus is.
  - One-click recovery via HTMX without page reload.
  - Eliminates redundant floating toast noise.
- **Cons**:
  - Replaces the global toast behavior for AI failures.

### Option 2: Hybrid Contextual Card + Floating Toast Notification
- **Explanation**: Render the inline contextual card inside the section AND keep the out-of-band toast notification (`#toast-container`).
- **Concrete Example**: Server returns both the inline error card fragment and the `hx-swap-oob="beforeend"` toast container fragment.
- **Pros**: Ensures user notices the failure even if scrolled away from the brief/analysis section.
- **Cons**: Redundant and cluttered; users see duplicate error messages simultaneously.

## Open Question for User

Do you prefer **Option 1 (Recommended: pure inline contextual card with Retry button, removing the floating toast)**, or would you like **Option 2 (both inline card and toast)**? (You can also reply **"just start"** with Option 1 if you are ready to proceed).
