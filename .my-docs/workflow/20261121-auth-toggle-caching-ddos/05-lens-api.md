# Lens

**Lens:** api  
**Result:** clean  

## Findings

### 1. Key Takeaways
- **Public API Contract Compliance**: All public HTMX and page endpoints match the contracts specified in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/03-design.md) and [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/03a-api-contract-review.md).
- **Graceful Guest Degradation**: In guest mode (`AUTH_ENABLED=false`), mutating endpoints return valid HTML fragments without triggering 401/403 or corrupting client-side state.
- **Zero Critical / Major Defects**: No silent breaking changes, schema regressions, or internal error leaks identified.

---

### 2. API Checklist Table

| Item | Requirement | Status | Evidence / Notes |
|------|-------------|--------|------------------|
| **1. Typed Input & Output** | All handlers parse typed inputs (`Query`, `Path`, `Form`) and return typed responses (`Html<String>`, `Response`). | Pass | [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L20-L38) structs `EntriesFilterQuery`, `MarkPageReadForm`, `ReadStatusQuery`. |
| **2. Stable Error Shapes** | Unified error handling returns OOB error toast or RFC-compliant HTTP status codes. | Pass | `handle_htmx_error` used across all HTMX endpoints; standard HTTP 413, 429, 303 for middleware. |
| **3. Edge Validation** | Validation occurs at system boundaries (Axum extractors, `DefaultBodyLimit`), not redundant internal checks. | Pass | 64KB body limit configured at root router level in [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L106). |
| **4. Pagination & Filtering** | List endpoints support `limit`, `offset`, and query filters without stateful leaks. | Pass | `render_entries_list` parses query params into GraphQL filter strings with proper defaults in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L39-L84). |
| **5. Additive Fields** | Changes are non-breaking and backward-compatible. | Pass | `auth_enabled` defaults to `true`. Existing auth sessions and OIDC flows remain 100% compatible. |
| **6. Consistent Naming** | Endpoint URLs and query keys align with repository conventions (`/hx/*`, snake_case params). | Pass | Routes in [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L94-L105) follow `/hx/entries/*` pattern. |
| **7. Mutating Endpoints** | Mutations are idempotent or shielded from upstream side effects when unauthenticated. | Pass | When auth is disabled, upstream mutations are skipped across all `/hx/*` write endpoints. |
| **8. Design Contract Match** | Routes, status codes, and rate limit headers match `03-design.md`. | Pass | Rate limiting headers `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `Retry-After: <wait_secs>` verified in tests. |
| **9. No Internal Leakage** | Internal database or Miniflux errors are sanitized before presentation. | Pass | Handled through `AppError` and generic toast notifications without stack trace leakage. |

---

### 3. Verification Summary
- **Unit and Integration Tests**: 100 tests pass covering rate limiting, DDoS payload limits, cache TTL expiration, guest mode route redirects, and mutation shielding.
- **Contract Integrity**: Verified clean with zero open Critical or Major defects.
