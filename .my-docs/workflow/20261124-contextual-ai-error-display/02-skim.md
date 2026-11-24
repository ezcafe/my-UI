# Codebase Skim: Contextual AI Error Display & Inline Retry

## Result: done

## Affected Components & Touchpoints

### 1. HTTP Routes (`crates/server/src/routes/entries.rs`)
- **`executive_brief_htmx`** ([Line 654](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L654)):
  - Endpoint: `GET /hx/executive-brief`
  - Current failure path ([Line 903-912](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L903-L912), [Line 969-971](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L969-L971)): Returns `<div id="page-executive-brief" class="hidden"></div>{toast}`.
  - Required change: Return contextual error card within `#page-executive-brief` featuring an HTMX "Retry" button targeting `#page-executive-brief` (`hx-swap="outerHTML"`), omitting any floating toast.
- **`get_entry_analysis_htmx`** ([Line 1603](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1603)):
  - Endpoint: `GET /hx/entries/{id}/analysis`
  - Current failure path ([Line 1640-1643](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1640-L1643)): Returns `<div id="ai-analysis-box" class="hidden"></div>{toast_html}`.
  - Required change: Return contextual error card within `#ai-analysis-box` featuring an HTMX "Retry" button targeting `#ai-analysis-box` (`hx-get="/hx/entries/{id}/analysis"` `hx-swap="outerHTML"`), without toast.
- **`get_entry_rewrite_htmx`** ([Line 1647](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1647)):
  - Endpoint: `GET /hx/entries/{id}/rewrite`
  - Current failure path ([Line 1735-1738](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1735-L1738)): Appends `toast_html` to `render_analysis_fragment("", "", &fallback)`.
  - Required change: Render contextual error card with "Retry" button (`hx-get="/hx/entries/{id}/rewrite"` `hx-target="#ai-rewrite-box"` `hx-swap="outerHTML"`) alongside or inside `#ai-rewrite-box`, omitting toast.

### 2. UI Templates & Client Sync (`crates/ui-components`)
- **Executive Brief Card & Skeleton**:
  - [`crates/ui-components/templates/partials/executive_brief_card.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/executive_brief_card.html)
  - [`crates/ui-components/templates/partials/executive_brief_skeleton.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/executive_brief_skeleton.html)
  - [`crates/ui-components/templates/partials/entry_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
- **Detail View Panels**:
  - [`crates/ui-components/templates/partials/entry_detail.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L97-L128): Declares `#ai-rewrite-box` and `#ai-analysis-box`.
- **Client Synchronization Script**:
  - [`crates/ui-components/templates/pages/index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L896-L921): The `htmx:afterSwap` event listener checks if `#page-executive-brief` swapped. When an error card is swapped in, it must update `#list-executive-brief` to remove the pulse animation and show the matching contextual error state with retry button.

### 3. Server Error Definitions (`crates/server/src/error.rs`)
- [`crates/server/src/error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L7-L17):
  - Holds `LLM_HIGH_DEMAND_ERROR_MESSAGE`.
  - Can provide reusable rendering helpers for the AI contextual error card to avoid HTML markup duplication across handlers.

### 4. Tests
- [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L3426):
  - Update `test_render_executive_brief_htmx_toast_on_failure` and detail failure tests to assert contextual error card markup (e.g. `hx-get`, `Retry`, alert icon) and confirm absence of `#toast-container` out-of-band fragment.
