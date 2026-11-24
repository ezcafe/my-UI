Result: clean

# General Design Review: User-Friendly Non-Blocking LLM Error Notification

**Workflow Slug:** `20261124-user-friendly-llm-error-message`  
**Review Stage:** `design-review`  
**Target Design:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/03-design.md)  
**Target Tasks:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/04-tasks.md)  
**Supporting Reviews:**
- [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/03a-api-contract-review.md) (`Result: clean`)
- Isolated DB Review: `skipped` (`Has DB = no`)

---

## 1. Executive Summary

This review assesses the full system design and task specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/04-tasks.md). The design provides user-friendly feedback during LLM high-demand periods without disrupting user reading workflows.

- **Overall Result:** `Result: clean` (Zero Critical, Zero Major).
- **Core Verdict:** The design is cohesive, technically sound, and integrates seamlessly with the existing Askama templates and HTMX architecture. Skeletons are safely hidden on error, fallback content is preserved, and toast notifications auto-dismiss cleanly.

---

## 2. Evaluation Across Engineering Dimensions

### 2.1 Architecture & Architectural Consistency
- **Option 1 (OOB Fragment Appending):** Directly aligns with existing codebase patterns in [routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1159) and [error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L41).
- **Template Reuse:** Reuses the verified [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129) and [toast.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html), eliminating duplicate HTML generation in client-side scripts.
- **Clean Boundaries:** The error constant `LLM_HIGH_DEMAND_ERROR_MESSAGE` and helper `render_llm_error_toast()` are centralized in `crates/server/src/error.rs`, keeping route handlers DRY.

### 2.2 User Experience (UX) & Reading Flow
- **Non-Blocking Delivery:** The notification floats at the bottom center of the viewport with `pointer-events-none` on the container, allowing users to scroll and interact with article content uninterrupted.
- **Graceful Reading Fallbacks:**
  - `get_entry_rewrite_htmx`: Falls back to existing executive brief or article teaser text.
  - `get_entry_analysis_htmx`: Swaps pulse loading skeleton with `<div id="ai-analysis-box" class="hidden"></div>`, avoiding infinite skeleton animations.
  - `executive_brief_htmx`: Replaces the section with a hidden container.
- **Explicit & Automatic Dismissal:** Toasts feature both a manual close button (`dismissToast(this)`) and a 5000ms auto-dismiss fadeout.

### 2.3 Accessibility (WCAG Compliance)
- **Status Messages (WCAG 4.1.3):** The persistent `#toast-container` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L502) should have `role="status"` and `aria-live="polite"`. This informs screen reader users of the transient error without jarringly stealing focus.
- **Timing & Controls (WCAG 2.2.1):** The notification includes an accessible close button with `aria-label="Close notification"`. The 5000ms duration is generous for the 18-word message, and pausing the timer on hover/focus is recommended.

### 2.4 Performance & Security
- **Zero Overhead:** Rendering the Askama toast template takes negligible CPU time (<1ms) and requires zero extra database or external network queries.
- **Safe Sanitization:** The error message is static, safe, and completely free of stack traces, internal URLs, or system exit codes.

### 2.5 Task Structure & TDD Verification
- **TDD Workflow:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/04-tasks.md) specifies a strict Red-Green-Refactor sequence across unit tests, route integration tests, server implementations, and client-side lifecycle hooks.
- **High Test Coverage:** Covers the helper function, rewrite route fallback, analysis route fallback, and executive brief route fallback.

---

## 3. Recommendations & Minor Hardening

1. **Accessibility Attributes on Container:**
   In [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L502), add `role="status"` and `aria-live="polite"` to `#toast-container`.
2. **Hover Pause on Auto-Dismiss:**
   In the `htmx:oobAfterSwap` script listener, attach `mouseenter` and `mouseleave` handlers to pause the 5000ms timer when the user hovers over the toast.
3. **Route Name Consistency:**
   Ensure tests target the mounted route `GET /hx/executive-brief` and function `executive_brief_htmx`.

---

## 4. Stage Verdict

- **Overall Result:** `clean`
- **Zero Critical / Zero Major Blockers**
- **Design Status:** Verified and ready for Gate B / Build arming.
