# Codebase Skim: LLM Error Handling & Toast Architecture

## Result: done

## Affected Components & Touchpoints

### 1. HTMX UI Components (`crates/ui-components`)
- **Template**: [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129) located at [`crates/ui-components/templates/partials/toast.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html).
- **Behavior**: Uses `hx-swap-oob="beforeend"` into `#toast-container`.
- **Styling**: Floating pill with `shadow-float`, `bg-foreground text-background`, Lucide `alert-circle` icon for error states, and a dismiss button (`onclick="dismissToast(this)"`).
- **Container**: Persistent `#toast-container` declared in [`base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L502).

### 2. HTTP Routes (`crates/server/src/routes/entries.rs`)
- **`get_entry_analysis_htmx`** ([Line 1517](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1517)):
  - Endpoint: `GET /hx/entries/{id}/analysis`
  - Current failure path: Returns `<div id="ai-analysis-box" class="hidden"></div>` silently.
  - Required change: Return the hidden div (to clean up skeleton) + append `ToastTemplate` with high-demand warning.
- **`get_entry_rewrite_htmx`** ([Line 1598](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1598)):
  - Endpoint: `GET /hx/entries/{id}/rewrite`
  - Current failure path: Renders fallback teaser or executive brief via `render_analysis_fragment("", "", &fallback)`.
  - Required change: Retain fallback content in `#ai-rewrite-box` + append `ToastTemplate` with high-demand warning.
- **`render_executive_brief_htmx`** ([Line 620](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L620)):
  - Endpoint: `GET /hx/entries/executive-brief`
  - Failure path: When batch synthesis produces empty top and items, returns `<div id="page-executive-brief" class="hidden"></div>`.
  - Change: Append `ToastTemplate` if batch fails.

### 3. Server Error Definitions (`crates/server/src/error.rs`)
- Holds `AppError` and user-facing messages.
- Centralize constant `LLM_HIGH_DEMAND_ERROR_MESSAGE`:
  `"Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later."`
- Provide a reusable helper `render_llm_error_toast()` to avoid duplication across routes.

### 4. Test Suite (`crates/server/src/routes/entries.rs`)
- Existing test [`test_get_entry_rewrite_htmx_fallback_on_llm_failure`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L3177).
- Needs assertions verifying that OOB `#toast-container` and user-friendly error message are delivered while fallback content remains intact.
- Add test for `get_entry_analysis_htmx` failure asserting OOB toast delivery.
