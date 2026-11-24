Result: clean

# API Contract Review: User-Friendly Non-Blocking LLM Error Notification

**Workflow Slug:** `20261124-user-friendly-llm-error-message`  
**Review Stage:** `api-contract-review`  
**Target Design:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/03-design.md)  
**Target Tasks:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/04-tasks.md)  

---

## 1. Executive Summary

This review assesses the API and interface contracts in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/03-design.md) for non-blocking LLM error toast notifications. The review verifies endpoint compatibility, payload structures, HTTP status codes, HTMX out-of-band (OOB) swap semantics, and error message sanitization.

- **Verdict:** `Result: clean` (Zero Critical, Zero Major, One Minor clarification).
- **Core Conclusion:** The proposed API contracts preserve existing HTMX DOM swap targets while appending out-of-band toast elements. Returning HTTP 200 OK with graceful fallback content aligns with canonical HTMX patterns, preventing UI stalls or unhandled network errors.

---

## 2. Endpoint Contract Evaluations

### 2.1 On-Demand Entry Rewrite (`GET /hx/entries/{id}/rewrite`)
- **Path:** `/hx/entries/{id}/rewrite`
- **Method:** `GET`
- **Parameters:** `id: i64` (entry identifier)
- **Success Contract:** Returns HTTP 200 with `<div id="ai-rewrite-box" ...>` containing AI rewrite and impact/fallacy content.
- **LLM Error Contract:**
  - Status: HTTP 200 OK.
  - Body: Concatenation of the primary fallback fragment (`render_analysis_fragment("", "", &fallback)`) and the OOB toast fragment (`crate::error::render_llm_error_toast()`).
  - HTMX Behavior: HTMX swaps `#ai-rewrite-box` with fallback content and simultaneously appends the toast into `#toast-container` via `hx-swap-oob="beforeend"`.
- **Verdict:** Sound and fully backward compatible.

### 2.2 Entry Analysis Detail (`GET /hx/entries/{id}/analysis`)
- **Path:** `/hx/entries/{id}/analysis`
- **Method:** `GET`
- **Parameters:** `id: i64` (entry identifier)
- **Success Contract:** Returns HTTP 200 with `<div id="ai-analysis-box" class="grid ...">...</div>`.
- **LLM Error Contract:**
  - Status: HTTP 200 OK.
  - Body: Concatenation of `<div id="ai-analysis-box" class="hidden"></div>` and the OOB toast fragment.
  - HTMX Behavior: Replaces the pulse skeleton loader with a hidden container (preventing stuck loading states) and appends the user-friendly toast to `#toast-container`.
- **Verdict:** Clean and prevents broken skeleton states.

### 2.3 Page Executive Brief (`GET /hx/executive-brief`)
- **Path:** `/hx/executive-brief` (defined in [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L109))
- **Method:** `GET`
- **Query Parameters:** `EntriesFilterQuery` (`status`, `search`, `limit`, `offset`)
- **Success Contract:** Returns HTTP 200 with `<div id="page-executive-brief" ...>` and optional title replacement OOB spans.
- **LLM Error Contract:**
  - Status: HTTP 200 OK.
  - Body: `<div id="page-executive-brief" class="hidden"></div>` appended with `crate::error::render_llm_error_toast()`.
  - HTMX Behavior: Hides the brief container on the page and alerts the user of high demand via non-blocking toast.
- **Verdict:** Sound.

---

## 3. Payload & HTMX Out-Of-Band Mechanics

### 3.1 Toast Template Structure
The toast fragment is rendered using the existing [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129) via [toast.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html):
```html
<div id="toast-container" hx-swap-oob="beforeend">
  <div class="pointer-events-auto shadow-float rounded-full bg-foreground text-background pl-4 pr-2.5 py-2 flex items-center gap-2.5 text-xs font-bold font-sans transition-opacity duration-300">
    <i data-lucide="alert-circle" class="w-4 h-4 text-destructive shrink-0"></i>
    <span>Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.</span>
    <button type="button" onclick="dismissToast(this)" class="p-1 rounded-full text-background/60 hover:text-background hover:bg-background/20 transition-colors shrink-0" title="Close notification" aria-label="Close notification">
      <i data-lucide="x" class="w-3.5 h-3.5"></i>
    </button>
  </div>
</div>
```

### 3.2 Status Code Rationale (HTTP 200 vs 4xx/5xx)
- HTMX by default halts DOM processing on non-2xx responses (`htmx:responseError`).
- Returning HTTP 200 OK ensures HTMX completes both the primary target swap (e.g. restoring reading text or hiding skeletons) and the OOB toast append without requiring custom JavaScript interceptors (`htmx:beforeOnLoad`).
- This guarantees graceful degradation across all supported browsers.

---

## 4. Security & Sanitization

- **No Information Leakage:** The constant error string contains zero stack traces, HTTP codes, CLI exit statuses, or IP addresses.
- **Auto-Escaped Template:** The message is rendered through Askama's HTML escaping engine, avoiding any XSS risk.
- **Stateless & Safe:** No new cookies, headers, or state mutations are introduced.

---

## 5. Findings Summary

- **Critical:** 0
- **Major:** 0
- **Minor:** 1
  - **Minor 1 (Route Path Clarification):** In [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/04-tasks.md), Planned Test 4 references `GET /hx/entries/executive-brief`. The server route mounted in `app.rs` is `/hx/executive-brief`. The implementation and integration tests must invoke `GET /hx/executive-brief`.

---

## 6. Stage Verdict

- **Result:** `clean`
- **Next Stage:** `design-review`
