# Design: Non-Blocking LLM Error Toast Notification

## Result: done

## Executive Summary
This design introduces user-friendly, non-blocking toast notifications when LLM operations experience high demand, timeouts, or transient errors. It replaces silent failure modes with clear feedback while keeping the user's reading experience intact.

## Key Conclusions
- **User-Friendly Error**: The exact notification string `"Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later."` is delivered without exposing technical details (HTTP status codes, process exit codes, or stack traces).
- **Out-Of-Band (OOB) Delivery**: Delivers toasts via HTMX `hx-swap-oob="beforeend"` into the persistent `#toast-container`.
- **Preserved Reading View**: Responses return HTTP 200 containing graceful fallback reading content alongside the OOB toast, ensuring HTMX swaps both elements without breaking or freezing UI state.
- **Auto-Dismiss & Icon Initialization**: `base.html` hooks into `htmx:oobAfterSwap` to initialize Lucide SVG icons and automatically dismiss toasts after 5 seconds.

---

## Architectural Comparison

### Option 1: Server-Side OOB Toast Appending (Recommended)

- **Explanation**:
  The server centralizes the LLM high-demand message in `crates/server/src/error.rs` and renders Askama's [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129). When an LLM error occurs in on-demand endpoints (`get_entry_rewrite_htmx`, `get_entry_analysis_htmx`, `render_executive_brief_htmx`), the server renders the fallback content for the primary target and appends the rendered OOB toast HTML into the same HTTP 200 response.

- **Concrete Example**:
  ```rust
  // In crates/server/src/error.rs
  use ui_components::templates::ToastTemplate;
  use askama::Template;

  pub const LLM_HIGH_DEMAND_ERROR_MESSAGE: &str =
      "Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.";

  pub fn render_llm_error_toast() -> String {
      ToastTemplate {
          message: LLM_HIGH_DEMAND_ERROR_MESSAGE.to_string(),
          is_error: true,
      }
      .render()
      .unwrap_or_default()
  }

  // In crates/server/src/routes/entries.rs (get_entry_rewrite_htmx failure arm)
  Err(_) => {
      let fallback = if let Some(record) = ai_service.db.get_summary_and_title(id).await {
          if !record.summary.is_empty() {
              record.summary
          } else {
              teaser
          }
      } else {
          teaser
      };
      let primary_html = render_analysis_fragment("", "", &fallback);
      let toast_html = crate::error::render_llm_error_toast();
      Html(format!("{primary_html}{toast_html}")).into_response()
  }
  ```

- **Pros**:
  - Reuses existing Askama [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129) and styling without code duplication.
  - Aligns with existing out-of-band patterns in [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1159).
  - Works out of the box with standard HTMX without requiring custom client-side event listeners or JavaScript libraries.
  - Fully verifiable via unit and integration tests.

- **Cons**:
  - The HTTP response body combines two HTML fragments (the primary target fragment and the OOB toast container).

---

### Option 2: HTMX `HX-Trigger` Header with Client-Side DOM Injection

- **Explanation**:
  The server returns only the primary HTML fallback, but sets an `HX-Trigger: {"showToast": {"message": "...", "is_error": true}}` response header. A custom JavaScript event listener on `document.body` catches the event, creates a toast DOM element, and appends it to `#toast-container`.

- **Concrete Example**:
  ```rust
  // Server-side
  let mut resp = Html(primary_html).into_response();
  resp.headers_mut().insert(
      "HX-Trigger",
      axum::http::HeaderValue::from_static(
          r#"{"showToast":{"message":"Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.","is_error":true}}"#,
      ),
  );
  resp
  ```
  ```javascript
  // Client-side in base.html
  document.body.addEventListener('showToast', (evt) => {
    showToast(evt.detail.message);
  });
  ```

- **Pros**:
  - Keeps response body limited to the single target container HTML.
  - Decouples notification trigger from markup structure.

- **Cons**:
  - Duplicates UI markup logic between Askama (`toast.html`) and client-side JavaScript (`index.html`).
  - Requires maintaining custom JavaScript event handling and DOM construction.
  - Difficult to test server-side integration without headless browser testing.

---

### Recommendation & Rationale
**Option 1** is recommended. It conforms directly to the project's existing HTMX server-rendered architecture, reuses the tested Askama `ToastTemplate`, and avoids duplicating toast styling across Rust and JavaScript.

---

## Detailed Component Specifications

### 1. Error Message Definition (`crates/server/src/error.rs`)
- **Constant**:
  ```rust
  pub const LLM_HIGH_DEMAND_ERROR_MESSAGE: &str =
      "Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.";
  ```
- **Helper Function**:
  ```rust
  pub fn render_llm_error_toast() -> String {
      ToastTemplate {
          message: LLM_HIGH_DEMAND_ERROR_MESSAGE.to_string(),
          is_error: true,
      }
      .render()
      .unwrap_or_default()
  }
  ```

### 2. Route Endpoint Updates (`crates/server/src/routes/entries.rs`)
- **`get_entry_rewrite_htmx`**:
  On `ai_service.generate_rewrite_and_analysis` error:
  1. Determine fallback summary/teaser text.
  2. Generate primary HTML via `render_analysis_fragment("", "", &fallback)`.
  3. Render toast HTML via `crate::error::render_llm_error_toast()`.
  4. Return `Html(format!("{primary_html}{toast_html}")).into_response()`.

- **`get_entry_analysis_htmx`**:
  On `ai_service.analyze_entry_impact_and_fallacies` error:
  1. Generate primary HTML: `<div id="ai-analysis-box" class="hidden"></div>` (cleans up loading skeleton).
  2. Render toast HTML via `crate::error::render_llm_error_toast()`.
  3. Return `Html(format!("{hidden_box}{toast_html}")).into_response()`.

- **`render_executive_brief_htmx`**:
  On batch synthesis producing empty results (`batch_res.top.is_empty() && batch_res.items.is_empty()`):
  1. Generate primary HTML: `<div id="page-executive-brief" class="hidden"></div>`.
  2. Append toast HTML via `crate::error::render_llm_error_toast()`.
  3. Return `(StatusCode::OK, headers, Html(full_html)).into_response()`.

### 3. Client-Side Lifecycle & Scripting (`crates/ui-components/templates/base.html`)
- In `base.html`, handle `htmx:oobAfterSwap`:
  - When an element is swapped into `#toast-container`, initialize Lucide icons (`alert-circle` and `x`).
  - Set a 5000ms timeout to smoothly fade out (`opacity = '0'`) and remove the toast element, while preserving the existing manual `dismissToast(this)` button.

---

## Verifiability & Reference Sources
- **HTMX Out-Of-Band Swaps**: [htmx.org/attributes/hx-swap-oob](https://htmx.org/attributes/hx-swap-oob/)
- **Askama Templates**: [crates.io/crates/askama](https://crates.io/crates/askama)
- **Lucide Icons**: [lucide.dev/guide/packages/lucide](https://lucide.dev/guide/packages/lucide)
- **Existing Toast Template**: [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129)
- **Existing Base Layout**: [`base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L501-L511)
