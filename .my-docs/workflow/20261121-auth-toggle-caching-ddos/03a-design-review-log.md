Result: clean

# General Design Review: Authentication Toggle, Client-Side State, Caching, and DDoS Protection

- **Stage:** `design-review` (Mode full)
- **Status:** Paid (`clean`)
- **Review Targets:**
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/03-design.md)
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/04-tasks.md)
  - [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/03a-api-contract-review.md)
- **Review Result:** `clean` (Zero Critical / Zero Major blockers)

---

## 1. Key Takeaways & Executive Summary

- **Zero Critical / Major Blockers**: The architecture, technical specifications, and task breakdown meet all system requirements and security goals without breaking changes.
- **Upstream Miniflux Shielding**: In guest mode (`AUTH_ENABLED=false`), all mutation endpoints short-circuit on the server with valid HTML fragments, completely protecting upstream Miniflux from unauthorized writes.
- **Resilient Guest Experience**: Read status is tracked locally in browser IndexedDB with an in-memory `Set` cache for instantaneous DOM updates, while bookmarking uses native Web Share or a non-intrusive keyboard shortcut toast.
- **Traffic Protection**: In-process TTL caching (30s for lists/counters, 300s for entry details) prevents Miniflux request stampedes, while token bucket rate limiting (60 req/min, burst 20) and a 64KB body limit safeguard public endpoints.
- **Full Backward Compatibility**: When `AUTH_ENABLED=true` (the default), existing OIDC workflows, session validations, and upstream Miniflux mutations remain 100% unchanged.

---

## 2. Five-Axis Design Review

### 2.1 Architecture & Component Design
- **Configuration Flexibility**: `auth_enabled: bool` defaults to `true`. When disabled, safe dummy placeholders satisfy session and OIDC configuration without requiring an identity provider.
- **Clean Middleware Bypass**: `AuthenticatedUser::from_request_parts` short-circuits directly to a guest identity when auth is disabled, avoiding cookie lookups and redirects.
- **Zero New Dependencies**: Reuses existing Rust crates (`axum`, `tokio`, `tower`, `serde`, `chrono`) and native browser APIs (`IndexedDB`, `navigator.share`), preserving the project's zero-build frontend and lean footprint.

### 2.2 Security & Protection
- **No Unauthorized Upstream Mutations**: Guest users cannot alter Miniflux read statuses, bookmarks, or feeds.
- **DDoS Throttling**: Per-IP rate limiting returns `HTTP 429 Too Many Requests` with RFC 6585 `Retry-After: 60` headers.
- **Payload Guardrails**: Axum `DefaultBodyLimit::max(64 * 1024)` rejects oversized requests with `HTTP 413 Payload Too Large`.

### 2.3 User Experience & Client-Side State
- **Instantaneous UI Feedback**: Client-side state in IndexedDB and an in-memory `Set` eliminates network round-trips for read status toggles.
- **Visual Dimming & Consistency**: HTMX swap hooks (`htmx:afterSwap`) automatically apply read styles to rendered rows without UI flicker.
- **Clean Navigation Header**: The logout button is conditionally hidden in guest mode (`{% if auth_enabled %}`), avoiding dead UI actions.

### 2.4 Performance & Scalability
- **Miniflux Stampede Prevention**: Thread-safe `MinifluxCache` serves frequent reads directly from memory, slashing response times for public readers.
- **Low Memory Overhead**: In-memory cache entries store lightweight JSON structures with TTL timestamps.

### 2.5 Testability & TDD Rigor
- **Comprehensive Task Breakdown**: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/04-tasks.md) specifies 13 planned unit and integration tests across 6 tasks.
- **Strict Red-Green-Refactor Flow**: Each task begins with an explicit failing test name before production code is modified.

---

## 3. Cross-Cutting Observations & Non-Blocking Recommendations

1. **Cache Capacity & Eviction Guardrail**:
   - *Observation*: `MinifluxCache` uses TTL-based expiration on reads. Under extended uptime with high query variance, expired entries could accumulate in memory until overwritten.
   - *Recommendation*: While key count is naturally bounded in single-tenant/small deployments, a periodic sweep or a simple cap (e.g., max 1,000 keys) can be added during Task 3 implementation if desired.
2. **Reverse Proxy Header Inspection**:
   - *Observation*: Rate limiting per client IP must handle reverse proxies gracefully.
   - *Recommendation*: Prioritize `CF-Connecting-IP` and `X-Forwarded-For` (first IP) over the raw socket peer address in `crates/server/src/rate_limit.rs`.

---

## 4. Sign-Off & Next Steps

All design documents, interface contracts, and task plans are verified, consistent, and ready for implementation. The design stage is officially **clean** and ready for Gate B approval.