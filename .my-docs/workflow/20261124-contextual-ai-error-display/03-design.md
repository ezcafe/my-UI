# Design: Contextual AI Error Display & Inline Recovery

## Result: done

## Executive Summary
This design replaces silent container collapses (`class="hidden"`) and detached floating toast notifications with **contextual inline error cards** equipped with HTMX-driven "Retry" buttons. When AI services encounter high demand, timeouts, or transient upstream errors, users receive immediate, localized feedback directly where the AI content belongs, alongside a one-click recovery mechanism that preserves their active navigation and filter context.

## Key Conclusions
- **Contextual Feedback**: Errors render directly within target containers (`#page-executive-brief`, `#ai-analysis-box`, `#ai-rewrite-box`), preventing layout jumps and eliminating toast noise.
- **Actionable Inline Recovery**: Each error card provides an HTMX-powered "Retry" button targeting its respective container (`hx-swap="outerHTML"`), with full query parameter preservation for paginated and filtered views.
- **Dual-View Synchronization**: Client-side `htmx:afterSwap` event handling synchronizes the error state between Magazine View (`#page-executive-brief`) and List View (`#list-executive-brief`) without persistent skeleton pulsing.
- **Reading Continuity**: In article detail view, rewrite failures display the inline error card above the fallback article summary/teaser text, ensuring readers can continue reading without interruption.
- **Zero Technical Leakage**: Messages use user-friendly text (`LLM_HIGH_DEMAND_ERROR_MESSAGE`) without exposing raw status codes, CLI exit codes, or stack traces.

---

## Architectural Comparison

### Option 1 (Recommended): Dedicated Askama `AiErrorCardTemplate` with Configurable HTMX Targets

- **Explanation**:
  Create a reusable Askama template struct `AiErrorCardTemplate` in `crates/ui-components/src/templates.rs` paired with `crates/ui-components/templates/partials/ai_error_card.html`. The template encapsulates layout, alert styling, typography, and HTMX attributes (`hx-get`, `hx-target`, `hx-swap`, `hx-disabled-elt`). Route handlers in `crates/server/src/routes/entries.rs` instantiate this template with endpoint-specific parameters and return standard HTTP 200 responses.

- **Concrete Examples**:
  ```rust
  // In crates/ui-components/src/templates.rs
  #[derive(Template)]
  #[template(path = "partials/ai_error_card.html")]
  pub struct AiErrorCardTemplate<'a> {
      pub title: &'a str,
      pub message: &'a str,
      pub retry_url: &'a str,
      pub target: &'a str,
      pub swap: &'a str,
      pub container_id: &'a str,
  }
  ```

  ```html
  <!-- crates/ui-components/templates/partials/ai_error_card.html -->
  <div {% if !container_id.is_empty() %}id="{{ container_id }}"{% endif %} class="rounded-lg p-5 bg-muted/40 border border-border flex flex-col items-start gap-3 is-error">
    <div class="flex items-center gap-2 text-muted-foreground">
      <div class="p-1 rounded bg-destructive/10 text-destructive">
        <i data-lucide="alert-circle" class="w-4 h-4"></i>
      </div>
      <span class="text-xs font-bold uppercase tracking-wider text-foreground font-sans">{{ title }}</span>
    </div>
    <p class="text-xs text-muted-foreground leading-relaxed font-sans">
      {{ message }}
    </p>
    <button
      hx-get="{{ retry_url }}"
      hx-target="{{ target }}"
      hx-swap="{{ swap }}"
      hx-disabled-elt="this"
      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-secondary text-secondary-foreground text-xs font-medium font-sans hover:bg-secondary/80 transition-colors cursor-pointer">
      <i data-lucide="refresh-cw" class="w-3.5 h-3.5"></i>
      <span>Retry</span>
    </button>
  </div>
  ```

- **Pros**:
  - **Type-safe & DRY**: Single source of truth for AI error card styling and markup across all three endpoints.
  - **Compile-time validation**: Askama checks HTML templates at compile time.
  - **Flexible targeting**: Can render either as a standalone container element (with `container_id`) or as a nested card inside composite containers.
  - **Clean HTMX semantics**: Standard hypermedia attributes without ad-hoc string formatting in route logic.

- **Cons**:
  - Adds one new template file to `crates/ui-components`.

---

### Option 2: Endpoint-Specific Hardcoded HTML Strings

- **Explanation**:
  Format HTML error snippets manually inside `crates/server/src/routes/entries.rs` using `format!(r#"..."#)` for each route handler individually, avoiding new Askama template definitions.

- **Concrete Example**:
  ```rust
  // Inside executive_brief_htmx in crates/server/src/routes/entries.rs
  let retry_url = format!("/hx/executive-brief?offset={}&limit={}&status={}", ...);
  let error_html = format!(
      r#"<div id="page-executive-brief" class="rounded-lg p-5 bg-muted/40 border border-border flex flex-col gap-3">
          <span class="text-xs font-bold uppercase text-foreground">Executive Brief Unavailable</span>
          <p class="text-xs text-muted-foreground">{}</p>
          <button hx-get="{}" hx-target="#page-executive-brief" hx-swap="outerHTML">Retry</button>
      </div>"#,
      crate::error::LLM_HIGH_DEMAND_ERROR_MESSAGE,
      retry_url
  );
  ```

- **Pros**:
  - Avoids adding a new Askama template file in `crates/ui-components`.

- **Cons**:
  - **Markup Duplication**: Violates DRY; three separate routes duplicate HTML styling, icon classes, and button markup.
  - **No Compile-time Verification**: Syntax errors or broken closing tags in raw string literals are only caught at runtime.
  - **Styling Inconsistencies**: Risk of divergence in margins, colors, and button padding between executive brief, analysis, and rewrite views.

---

### Recommendation & Rationale
**Option 1** is recommended. Centralizing AI error presentation into Askama's `AiErrorCardTemplate` guarantees visual consistency across all views, ensures compile-time template safety, and keeps route handlers focused purely on business logic.

---

## Detailed Specifications by Component

### 1. Template Definition (`crates/ui-components`)
- **File**: `crates/ui-components/src/templates.rs`
  ```rust
  #[derive(Template)]
  #[template(path = "partials/ai_error_card.html")]
  pub struct AiErrorCardTemplate<'a> {
      pub title: &'a str,
      pub message: &'a str,
      pub retry_url: &'a str,
      pub target: &'a str,
      pub swap: &'a str,
      pub container_id: &'a str,
  }
  ```
- **Partial**: `crates/ui-components/templates/partials/ai_error_card.html`
  - Container styling: `rounded-lg p-5 bg-muted/40 border border-border flex flex-col items-start gap-3 is-error`
  - Alert header: Lucide `alert-circle` icon inside `p-1 rounded bg-destructive/10 text-destructive`, bold uppercase title.
  - Error message: `text-xs text-muted-foreground leading-relaxed font-sans`.
  - Action button: `bg-secondary text-secondary-foreground text-xs font-medium hover:bg-secondary/80`, with `refresh-cw` icon and "Retry" label.

### 2. Route Handlers (`crates/server/src/routes/entries.rs`)

#### A. Executive Brief (`executive_brief_htmx`)
- **Trigger**: When batch synthesis returns empty results (`batch_res.top.is_empty() && batch_res.items.is_empty()`) or brief text is empty on failure.
- **Retry URL Construction**:
  ```rust
  let mut retry_url = format!(
      "/hx/executive-brief?offset={}&limit={}&status={}",
      offset_val,
      limit_val,
      status_str
  );
  if let Some(fid) = query.feed_id {
      retry_url.push_str(&format!("&feed_id={}", fid));
  }
  if let Some(cid) = query.category_id {
      retry_url.push_str(&format!("&category_id={}", cid));
  }
  ```
- **Response**:
  ```rust
  let card_html = AiErrorCardTemplate {
      title: "Executive Brief Unavailable",
      message: crate::error::LLM_HIGH_DEMAND_ERROR_MESSAGE,
      retry_url: &retry_url,
      target: "#page-executive-brief",
      swap: "outerHTML",
      container_id: "page-executive-brief",
  }
  .render()
  .unwrap_or_default();

  return (StatusCode::OK, headers, Html(card_html)).into_response();
  ```
- **Toast Removal**: OOB toast string is completely eliminated.

#### B. Article AI Analysis (`get_entry_analysis_htmx`)
- **Trigger**: When `ai_service.analyze_entry_impact_and_fallacies` returns `Err(_)`.
- **Response**:
  ```rust
  let retry_url = format!("/hx/entries/{id}/analysis");
  let card_html = AiErrorCardTemplate {
      title: "AI Analysis Unavailable",
      message: crate::error::LLM_HIGH_DEMAND_ERROR_MESSAGE,
      retry_url: &retry_url,
      target: "#ai-analysis-box",
      swap: "outerHTML",
      container_id: "ai-analysis-box",
  }
  .render()
  .unwrap_or_default();

  Html(card_html).into_response()
  ```

#### C. Article AI Rewrite (`get_entry_rewrite_htmx`)
- **Trigger**: When `ai_service.generate_rewrite_and_analysis` returns `Err(_)`.
- **Response**:
  ```rust
  let retry_url = format!("/hx/entries/{id}/rewrite");
  let error_card = AiErrorCardTemplate {
      title: "AI Rewrite Unavailable",
      message: crate::error::LLM_HIGH_DEMAND_ERROR_MESSAGE,
      retry_url: &retry_url,
      target: "#ai-rewrite-box",
      swap: "outerHTML",
      container_id: "",
  }
  .render()
  .unwrap_or_default();

  let fallback_html = format!(
      r#"<div class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">{}</div>"#,
      escape_html(&fallback)
  );

  let full_html = format!(
      r#"<div id="ai-rewrite-box" class="space-y-6">{error_card}{fallback_html}</div>"#
  );
  Html(full_html).into_response()
  ```

### 3. Client Synchronization Script (`crates/ui-components/templates/pages/index.html`)
In the `htmx:afterSwap` event listener:
```javascript
if (isBriefSwap) {
  const pageBrief = document.querySelector('#page-executive-brief p');
  const listBrief = document.getElementById('list-executive-brief');
  const isErrorCard = document.querySelector('#page-executive-brief.is-error, #page-executive-brief .is-error');

  if (isErrorCard && listBrief) {
    const retryBtn = isErrorCard.querySelector('button');
    const retryUrl = retryBtn ? retryBtn.getAttribute('hx-get') : '/hx/executive-brief?status=unread';
    listBrief.classList.remove('animate-pulse');
    listBrief.innerHTML = `
      <div class="flex items-center gap-2 mb-1.5 text-muted-foreground">
        <div class="p-1 rounded bg-destructive/10 text-destructive">
          <i data-lucide="alert-circle" class="w-4 h-4"></i>
        </div>
        <span class="text-xs font-bold uppercase tracking-wider text-foreground font-sans">Executive Brief Unavailable</span>
      </div>
      <p class="text-xs text-muted-foreground leading-relaxed font-sans mb-2">${escapeHtml(pageBrief?.innerText || 'This model is currently experiencing high demand.')}</p>
      <button
        hx-get="${retryUrl}"
        hx-target="#page-executive-brief"
        hx-swap="outerHTML"
        hx-disabled-elt="this"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-secondary text-secondary-foreground text-xs font-medium font-sans hover:bg-secondary/80 transition-colors cursor-pointer">
        <i data-lucide="refresh-cw" class="w-3 h-3"></i>
        <span>Retry</span>
      </button>
    `;
    if (window.htmx) {
      htmx.process(listBrief);
    }
    if (window.lucide && typeof lucide.createIcons === 'function') {
      lucide.createIcons({ root: listBrief });
    }
  } else if (pageBrief && listBrief) {
    // Normal successful brief render sync...
  }
}
```

---

## Verifiability & Reference Sources
- **HTMX Swapping and Response Targets**: [htmx.org/docs/#swapping](https://htmx.org/docs/#swapping)
- **HTMX Client API (`htmx.process`)**: [htmx.org/api/#process](https://htmx.org/api/#process)
- **Askama Documentation**: [docs.rs/askama](https://docs.rs/askama)
- **Lucide Icons**: [lucide.dev/icons/alert-circle](https://lucide.dev/icons/alert-circle)
- **Codebase Touchpoints**:
  - [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - [`crates/ui-components/src/templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - [`crates/ui-components/templates/pages/index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
