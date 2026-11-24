# Grill: Design Tree & Frontier

## Result: frontier-empty

## Decision Frontier & Settled Positions

---

### ❓ **Q1** - **Endpoint Coverage for LLM Error Toast**
Which endpoints should emit the out-of-band user-friendly LLM error toast when an upstream failure occurs?
- Option A: Only `get_entry_rewrite_htmx` and `get_entry_analysis_htmx`.
- Option B (Recommended): `get_entry_rewrite_htmx`, `get_entry_analysis_htmx`, and `render_executive_brief_htmx`.

➡️ **Settled Position**: Option B. All on-demand HTMX endpoints that execute live LLM calls (`/hx/entries/{id}/rewrite`, `/hx/entries/{id}/analysis`, and `/hx/entries/executive-brief`) will append the OOB toast on failure. Pure cache-reading endpoints (such as `/hx/entries/{id}` detail view) and background workers (`AiPrefetchService`) do not emit toasts.

---

### ❓ **Q2** - **Error Message String & Centralization**
How should the user-friendly message be stored and formatted?
- Option A: Inline string literals inside route handler functions.
- Option B (Recommended): Centralized constant in `crates/server/src/error.rs` (or `crates/server/src/ai/mod.rs`), rendered via `ToastTemplate { message: LLM_HIGH_DEMAND_ERROR_MESSAGE.to_string(), is_error: true }`.

➡️ **Settled Position**: Option B. Define `pub const LLM_HIGH_DEMAND_ERROR_MESSAGE: &str = "Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.";` and a helper function `pub fn render_llm_error_toast() -> String` in `crates/server/src/error.rs`.

---

### ❓ **Q3** - **Client-Side Toast Lifecycle & Icon Initialization**
How should out-of-band toasts behave once injected into `#toast-container`?
- Option A: Require manual user click on `x` button to dismiss; do not auto-dismiss.
- Option B (Recommended): Auto-dismiss after 5000ms with fade-out animation while retaining the manual `dismissToast` button. Also listen to `htmx:oobAfterSwap` in `base.html` so Lucide icons (`alert-circle`, `x`) are always rendered.

➡️ **Settled Position**: Option B. Enhance `base.html` with an `htmx:oobAfterSwap` handler that initializes Lucide icons on swapped toast elements and sets a 5-second auto-dismiss timeout.

---

## Frontier Status
All frontier decisions are settled based on the user's explicit prompt requirements and established codebase conventions.
Frontier is empty. Ready to proceed to Design.
