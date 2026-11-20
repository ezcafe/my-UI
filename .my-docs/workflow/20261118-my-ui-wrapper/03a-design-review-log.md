Result: clean

# General Design Review: my-ui wrapper

**Review Date:** 2026-11-19  
**Review Profile:** full  
**Status:** Clean — Zero Critical or Major issues  
**Reviewed Documents & References:**
- Design Document: [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md)
- Tasks Document: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md)
- API Contract Review: [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03a-api-contract-review.md)
- Reference UI Prototypes: [ar-list.html](file:///Users/ptquang86/ws/my-ui/ar-list.html) and [ar-detail.html](file:///Users/ptquang86/ws/my-ui/ar-detail.html)

---

## Executive Summary

The architectural design and TDD task breakdown for the **my-ui wrapper** have been thoroughly reviewed across five primary dimensions:
1. Architectural completeness and future extensibility
2. Fidelity to reference AR spatial glassmorphism UI styles
3. Security architecture, sessions, and authentication
4. Error handling and resilience against upstream failures
5. TDD task breakdown and test planning completeness

**Conclusion:** The design is comprehensive, clean, and robust. Zero critical and zero major architectural defects were identified. All previous API contract corrections have been properly reflected in both the design specification and the implementation task list. The project is ready to proceed to Gate B.

---

## Detailed Evaluation by Dimension

### 1. Architectural Completeness & Future Extensibility
- **Evaluation:** **Pass (Clean)**.
- **Modularity:** The Cargo workspace cleanly separates concerns into three focused crates: `crates/server` (orchestration, routing, auth, sessions), `crates/miniflux` (upstream client, domain models, summary, GraphQL schema), and `crates/ui-components` (Askama templates, styling, sanitization).
- **Decoupling:** `crates/miniflux` has zero dependencies on `crates/ui-components`, ensuring domain boundaries remain strictly unpolluted.
- **Future Extensibility:** Wrapping additional self-hosted services in the future follows the exact same pattern: add a new service crate (e.g., `crates/paperless`), integrate its GraphQL queries into the schema, and provide matching Askama UI templates without modifying core server plumbing.

### 2. Fidelity to Reference AR Spatial Glassmorphism UI
- **Evaluation:** **Pass (Clean)**.
- **Glassmorphism Tokens:** The design directly inherits the visual styling defined in [ar-list.html](file:///Users/ptquang86/ws/my-ui/ar-list.html) and [ar-detail.html](file:///Users/ptquang86/ws/my-ui/ar-detail.html), including dark glass panels (`rgba(15, 23, 42, 0.72)` with `backdrop-filter: blur(32px) saturate(190%)`), subtle border highlights (`rgba(255, 255, 255, 0.14)`), and gaze-friendly hover states.
- **Modern Responsive Layouts:** The templates adhere strictly to modern CSS container queries (`container-type: inline-size`, `@container (min-width: 720px)`) and fluid grid properties (`repeat(auto-fit, minmax(280px, 1fr))`), completely avoiding rigid hardcoded viewport breakpoints.
- **Interaction Fidelity:** The spatial grab handle, custom minimal scrollbars, out-of-band `#unread-pill` counter synchronization, and AI Executive Brief cards are fully accounted for in the template breakdown.

### 3. Security Architecture & Session Management
- **Evaluation:** **Pass (Clean)**.
- **Stateless PKCE & CSRF Storage:** The ephemeral `__oidc_temp` cookie safely stores `{ pkce_verifier, csrf_state }` across the Pocket-ID redirect loop without needing a local database or server-side memory. It is scoped to `Path = /auth/callback` with `Max-Age = 300` and cleared immediately upon callback completion.
- **Cookie Budget & Encryption:** `__session` uses AES-256-GCM encryption via `tower-cookies` private jars with `HttpOnly`, `SameSite = Lax`, and conditional `Secure` flags. The session payload (`SessionData`) strictly discards bulky JWT `id_token` payloads, easily remaining within the 4KB browser cookie limit.
- **Cross-Site Protection:** `SameSite = Lax` cookies combined with custom HTMX headers (`HX-Request`) protect mutating endpoints against cross-site request forgery.
- **Content Sanitization:** Feed HTML is strictly sanitized using `ammonia`, stripping script tags, iframes, and inline event handlers, while enforcing `rel="noopener noreferrer"` and `target="_blank"` on links.
- **Secret Isolation:** Miniflux credentials (`MINIFLUX_BASE_URL` and `MINIFLUX_API_KEY`) remain strictly on the server and are never exposed to browser clients.

### 4. Error Handling & Resilience
- **Evaluation:** **Pass (Clean)**.
- **Upstream Miniflux Outages:** HTTP client calls to Miniflux enforce a 5-second timeout. Full page loads gracefully fall back to an AR glass error card with a retry button.
- **DOM Protection on HTMX Errors:** HTMX request errors return HTTP 200 with `HX-Reswap: none` and an out-of-band toast notification (`#toast-container`), preventing destructive replacement of existing UI rows or action buttons with raw error messages.
- **OIDC Expiration & Re-authentication:** Expired tokens trigger silent refresh via `refresh_token`. If refresh fails, HTMX responses receive `HX-Redirect: /auth/login` to redirect the user without broken partial renders.

### 5. TDD Task Breakdown & Test Planning
- **Evaluation:** **Pass (Clean)**.
- **TDD Enforcement:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md) enumerates 7 sequential tasks with 28 explicit unit, integration, WireMock, and end-to-end tests planned to fail before implementation.
- **Verification of API Review Fixes:** All 5 previous API contract findings (PKCE cookie storage, `get_me` user ID caching, OOB unread pills, concurrent counter fetching, and extractive summary crate placement) are directly assigned to concrete tasks and test cases.

---

## Non-Blocking Suggestions & Observations

1. **Search Debounce Delay**: When rendering the search input in `index.html` / `entry_list.html`, ensure the HTMX trigger uses `hx-trigger="keyup changed delay:300ms, search"` so keystrokes do not flood upstream Miniflux with transient queries.
2. **Reader Previous/Next Navigation**: For `entry_detail.html`, passing `prev_entry_id: Option<i64>` and `next_entry_id: Option<i64>` in the view model will allow the chevron buttons to fetch the adjacent stories directly via `hx-get="/hx/entries/{prev_id}"` without client-side state.
3. **Local Dev Secure Cookie Flag**: Ensure cookie `Secure` flag logic checks whether the environment is development/localhost (`http://`) so cookies are not silently rejected by browsers during local testing.

---

## Review Finding Summary Table

| Finding ID | Severity | Category | Description | Status |
|---|---|---|---|---|
| None | None | Architecture / Security / UI | All design specifications and task plans verified clean | **Clean** |

---

## Next Steps

With both Isolated API Contract Review (`03a-api-contract-review.md`) and General Design Review (`03a-design-review-log.md`) verified as **clean**, the design phase is fully satisfied. The workflow may now advance to **Gate B** for user confirmation and approval before initiating the TDD build.
