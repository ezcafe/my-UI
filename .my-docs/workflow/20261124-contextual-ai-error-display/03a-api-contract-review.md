# API Contract Review: Contextual AI Error Display

## Result: clean

## Executive Summary
This review evaluates the hypermedia API contracts and route handler specifications for contextual AI error display across `/hx/executive-brief`, `/hx/entries/{id}/analysis`, and `/hx/entries/{id}/rewrite`. The contract adheres to idiomatic HTMX patterns, ensures type safety via Askama templates, prevents duplicate submissions with `hx-disabled-elt`, and maintains complete query/path parameter preservation for retries. Zero Critical or Major issues were identified.

## Key Conclusions
- **Idiomatic Hypermedia Semantics**: All endpoints return HTTP 200 with inline HTML fragments designed for `outerHTML` swapping, which is standard in HTMX to enable seamless recovery without requiring complex client-side error interceptors.
- **Strict Parameter Preservation**: The `/hx/executive-brief` retry URL dynamically carries forward `offset`, `limit`, `status`, `feed_id`, and `category_id`, guaranteeing navigation state continuity.
- **Double-Click Protection**: The retry action uses `hx-disabled-elt="this"`, preventing duplicate concurrent requests while an active retry is in-flight.
- **Information Leakage Prevention**: Technical error details, CLI exit codes, and model stack traces are suppressed; all failure paths render the standardized user-safe message (`LLM_HIGH_DEMAND_ERROR_MESSAGE`).

---

## Detailed Endpoint Evaluation

### 1. Executive Brief Endpoint
- **Route**: `GET /hx/executive-brief`
- **Query Parameters**:
  - `status`: String (`"unread"` | `"all"` | `"starred"`)
  - `limit`: Integer (`1..100`, default `15`)
  - `offset`: Integer (`>= 0`, default `0`)
  - `feed_id`: Optional Integer
  - `category_id`: Optional Integer
- **Failure Contract**:
  - **Status Code**: `200 OK`
  - **Headers**: `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-cache, no-store, must-revalidate`
  - **Payload Structure**:
    ```html
    <div id="page-executive-brief" class="rounded-lg p-5 bg-muted/40 border border-border flex flex-col items-start gap-3 is-error">
      <div class="flex items-center gap-2 text-muted-foreground">
        <div class="p-1 rounded bg-destructive/10 text-destructive">
          <i data-lucide="alert-circle" class="w-4 h-4"></i>
        </div>
        <span class="text-xs font-bold uppercase tracking-wider text-foreground font-sans">Executive Brief Unavailable</span>
      </div>
      <p class="text-xs text-muted-foreground leading-relaxed font-sans">
        This model is currently experiencing high demand. Please try again in a few moments.
      </p>
      <button
        hx-get="/hx/executive-brief?offset=0&limit=15&status=unread"
        hx-target="#page-executive-brief"
        hx-swap="outerHTML"
        hx-disabled-elt="this"
        class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-secondary text-secondary-foreground text-xs font-medium font-sans hover:bg-secondary/80 transition-colors cursor-pointer">
        <i data-lucide="refresh-cw" class="w-3.5 h-3.5"></i>
        <span>Retry</span>
      </button>
    </div>
    ```
- **Assessment**: Pass. Completely eliminates floating toast noise while providing deterministic container replacement.

---

### 2. Article AI Analysis Endpoint
- **Route**: `GET /hx/entries/{id}/analysis`
- **Path Parameters**:
  - `id`: Integer (`i64`)
- **Failure Contract**:
  - **Status Code**: `200 OK`
  - **Headers**: `Content-Type: text/html; charset=utf-8`
  - **Payload Structure**:
    - Standalone container with `id="ai-analysis-box"`.
    - Retry button targeting `#ai-analysis-box` with `hx-swap="outerHTML"`.
- **Assessment**: Pass. Matches the exact DOM footprint of the target element and preserves the article identifier in the retry path.

---

### 3. Article AI Rewrite Endpoint
- **Route**: `GET /hx/entries/{id}/rewrite`
- **Path Parameters**:
  - `id`: Integer (`i64`)
- **Failure Contract**:
  - **Status Code**: `200 OK`
  - **Headers**: `Content-Type: text/html; charset=utf-8`
  - **Payload Structure**:
    - Outer wrapper `<div id="ai-rewrite-box" class="space-y-6">`.
    - Inner error card (without redundant `container_id` to avoid ID collision).
    - Fallback prose block with sanitized article summary/teaser.
    - Retry button targeting `#ai-rewrite-box` with `hx-swap="outerHTML"`.
- **Assessment**: Pass. Maintains reading continuity by coupling the recovery card with fallback summary text.

---

## Contract Safety & Compliance Checklist

| Check | Status | Details |
|---|---|---|
| **Idempotency** | **Pass** | All endpoints use `GET` methods without side effects on read operations. |
| **XSS Prevention** | **Pass** | Askama templates automatically HTML-escape all struct field bindings by default. |
| **HTMX Attribute Correctness** | **Pass** | `hx-get`, `hx-target`, `hx-swap="outerHTML"`, and `hx-disabled-elt="this"` conform to htmx specifications. |
| **Error Message Clarity** | **Pass** | Clean plain English copy without internal stack traces, tokens, or raw HTTP status codes. |
| **State Synchronization** | **Pass** | `is-error` class marker enables the magazine/list sync script to cleanly update dual views. |

---

## Verifiability & Reference Sources
- **HTMX Swapping & Attributes**: [htmx.org/docs/#swapping](https://htmx.org/docs/#swapping) · [htmx.org/attributes/hx-disabled-elt/](https://htmx.org/attributes/hx-disabled-elt/)
- **Askama Compile-Time Escaping**: [docs.rs/askama](https://docs.rs/askama)
- **Target Source Code**:
  - [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L654-L965)
  - [`crates/ui-components/src/templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L93-L105)
