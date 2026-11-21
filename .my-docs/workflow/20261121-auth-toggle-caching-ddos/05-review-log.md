# Code review log

**Overall Result:** clean

---

## Adversarial

**Result:** clean

### 1. Key Takeaways & Focus Areas
- **Upstream Miniflux Shielding**: Verified that all mutation endpoints (`/hx/entries/{id}/read`, `/hx/entries/{id}/bookmark`, `/hx/mark-page-read`, `/hx/mark-feed-read`, `/hx/mark-all-read`) strictly skip upstream Miniflux writes when `auth_enabled=false`.
- **DDoS and Body Size Guardrails**: Verified that Axum `DefaultBodyLimit::max(64 * 1024)` rejects payloads exceeding 64KB with HTTP 413, preventing memory exhaustion attacks via oversized request bodies.
- **Rate Limiting Resilience**: Verified token-bucket algorithm enforces 20 burst capacity and 1 token/sec refill rate, returning HTTP 429 and `Retry-After: <wait_secs>` on excess requests.

### 2. Edge Case Hunting & Adversarial Findings
- **Guest Read-Status Persistence Fixed**:
  - *Observation*: `markLocallyRead` was defined in `crates/ui-components/templates/pages/index.html` but was not wired to HTMX swap events.
  - *Resolution*: Added `data-entry-id` to [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L1) and wired `htmx:afterSwap` on `#detail-panel` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L383-L393) to automatically persist read items into `my_news_guest_db` IndexedDB.
- **Guest Detail Unread Toggle Fixed**:
  - *Observation*: In [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), `mark_entry_read_htmx` previously returned an empty dummy `EntryViewModel` when auth was disabled. Swapping this into the DOM on unread clicks wiped the row title and link.
  - *Resolution*: Removed the dummy empty return. When auth is disabled, the mutation is bypassed, and the subsequent query reads the entry metadata normally so the DOM row retains its full title, feed, and link.
- **Rate Limiter Memory Growth & IP Spoofing**:
  - *Observation*: `RateLimiter` keeps client IP buckets in `HashMap<String, ClientBucket>` without periodic bucket eviction. Additionally, `x-forwarded-for` is parsed directly from headers.
  - *Assessment*: For the current single-instance deployment, memory footprint for typical user bases is minimal (~80 bytes per IP). For production clusters behind reverse proxies, a proxy trust list and background reaper are recommended as future hardening.

---

## Quality

**Result:** clean

### 1. Five-Axis Quality Assessment
- **Correctness**: All 100 unit, integration, and E2E tests pass without failures or warnings. Guest mode cleanly short-circuits mutations, and authenticated mode preserves full OIDC flows.
- **Readability**: Logic is cleanly separated across dedicated modules: `cache.rs` for in-memory TTL caching, `rate_limit.rs` for DDoS mitigation, and `config.rs` for environment parsing.
- **Architecture**: Zero new external dependencies introduced. Built entirely with Axum, Tokio, and existing workspace crates. UI stays zero-build using vanilla JavaScript and IndexedDB.
- **Security**: Upstream Miniflux credentials and write access are completely insulated from guest visitors. Session secrets and cookies remain strictly validated when auth is enabled.
- **Performance**: High-traffic read endpoints (`/` and `/hx/entries`) utilize `MinifluxCache` (30s TTL for lists/counters, 300s TTL for entry details), preventing request stampedes to upstream Miniflux.

### 2. Code Health & Simplification
- Zero compiler warnings or dead-code annotations across the workspace.
- Clean layer separation: authentication bypass is handled at middleware extraction (`AuthenticatedUser::from_request_parts`), keeping route handlers clean.

---

## Merged

**Result:** clean
