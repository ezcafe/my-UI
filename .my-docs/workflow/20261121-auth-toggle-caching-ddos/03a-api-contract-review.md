Result: clean

# API Contract Review: Authentication Toggle, Client-Side State, Caching, and DDoS Protection

- **Stage:** `api-contract-review` (Mode full / isolated review)
- **Status:** Paid (`clean`)
- **Review Target:** Axum Middleware, OIDC Handlers, HTMX Entry Endpoints, Cache Layer, and Rate Limiting

---

## 1. Executive Summary

This API contract review evaluates the technical interfaces and HTTP contracts defined in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/04-tasks.md).

- **Zero Critical / Major Blockers**: All API routes, middleware extractors, headers, and payload structures adhere to RESTful best practices and project architecture.
- **Backwards Compatibility**: When `AUTH_ENABLED=true` (the default), existing OIDC workflows, session validations, and upstream Miniflux mutations remain 100% unchanged.
- **Upstream Protection & Graceful Degradation**: In guest mode (`AUTH_ENABLED=false`), upstream write mutations short-circuit on the server with valid HTML fragments, shielding Miniflux from unauthorized changes while preserving HTMX DOM swap contracts.
- **Traffic Safety & DDoS Mitigation**: Axum middleware enforces per-IP token bucket rate limiting (60 req/min, burst 20) with RFC 6585 standard headers (`429 Too Many Requests`, `Retry-After`) and a 64KB request body cap (`413 Payload Too Large`).

---

## 2. Interface Contract Checklist

### 2.1 Configuration & Environment Contract ([crates/server/src/config.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/config.rs))

- [x] **`auth_enabled: bool` Field**:
  - Environment variable: `AUTH_ENABLED` (case-insensitive boolean: `"true"`, `"1"`, `"false"`, `"0"`). Defaults to `true` if omitted.
- [x] **Conditional OIDC Validation**:
  - When `auth_enabled == true`: `SESSION_SECRET`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, and `OIDC_REDIRECT_URL` are strictly validated as required.
  - When `auth_enabled == false`: Missing OIDC variables and `SESSION_SECRET` fallback to internal dummy placeholders, allowing the server to boot smoothly without external IdP configuration.

### 2.2 Authentication Middleware & Handlers ([crates/server/src/auth/](file:///Users/ptquang86/ws/my-ui/crates/server/src/auth/))

- [x] **`AuthenticatedUser` Extractor**:
  - When `auth_enabled == false`: Instantly resolves to:
    ```rust
    AuthenticatedUser {
        user_id: "guest".to_string(),
        username: "Guest".to_string(),
        access_token: String::new(),
    }
    ```
    Bypasses session cookie parsing and expiration checks. Does not redirect or return HTTP 401.
- [x] **Auth Routes Contract (`/login`, `/auth/login`, `/auth/callback`, `/auth/logout`)**:
  - When `auth_enabled == false`:
    - `GET /login`: Redirects directly to `/` (HTTP 303 See Other / 302 Found).
    - `GET /auth/login`: Redirects directly to `/`.
    - `GET /auth/callback`: Redirects directly to `/`.
    - `GET /auth/logout`: Redirects directly to `/`.
  - When `auth_enabled == true`: Normal OIDC redirect and callback flow executes unchanged.

### 2.3 HTMX Endpoints & Upstream Shielding ([crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs))

| Endpoint | Method | Guest Mode (`AUTH_ENABLED=false`) | Authenticated Mode (`AUTH_ENABLED=true`) |
|----------|--------|-----------------------------------|------------------------------------------|
| `/hx/entries` | `GET` | Serves entries from in-memory cache (TTL 30s) or Miniflux; read state updated client-side. | Serves entries from cache / Miniflux with server status. |
| `/hx/entries/{id}` | `GET` | Skips `updateEntriesStatus` mutation against Miniflux; serves article detail HTML with 200 OK. | Marks read in Miniflux; serves article detail HTML + OOB counters. |
| `/hx/entries/{id}/bookmark` | `POST` | Skips `toggleBookmark` mutation against Miniflux; returns bookmark button HTML with 200 OK. | Toggles bookmark in Miniflux; returns updated button HTML. |
| `/hx/entries/{id}/read` | `POST` | Skips `updateEntriesStatus` mutation against Miniflux; returns empty 200 OK. | Marks entry read in Miniflux; returns 200 OK. |
| `/hx/mark-page-read` | `POST` | Skips batch `updateEntriesStatus` mutation against Miniflux; returns rendered entries list with 200 OK. | Marks entries read in Miniflux; returns refreshed list. |
| `/hx/mark-all-read` | `POST` | Skips `markAllAsRead` mutation; returns 200 OK / list. | Marks all read in Miniflux; returns 200 OK. |
| `/hx/feeds/{id}/mark-all-read` | `POST` | Skips `markFeedAsRead` mutation; returns 200 OK / list. | Marks feed read in Miniflux; returns 200 OK. |

- [x] **HTMX Swap Safety**: In guest mode, all mutation endpoints maintain consistent HTML return types matching `hx-target` expectations (`outerHTML` or `innerHTML`), ensuring the UI does not flicker or break.

### 2.4 Caching Layer Contract ([crates/server/src/cache.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/cache.rs))

- [x] **In-Memory TTL Store**:
  - Keys: `entries:<hash>` (TTL 30s), `entry:<id>` (TTL 300s), `counters` (TTL 30s).
  - Thread-safe access via `Arc<RwLock<HashMap<String, (Instant, serde_json::Value)>>>`.
  - Non-blocking reads: Stale or expired entries trigger upstream fetch and background cache write.

### 2.5 DDoS Mitigation & Rate Limiting Contract ([crates/server/src/rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs))

- [x] **Token Bucket Limiter**:
  - Capacity: 20 tokens (burst limit).
  - Refill rate: 1 token/sec (60 requests/minute).
  - Key: Client IP address (with `X-Forwarded-For` inspection fallback to socket peer address).
- [x] **Response Headers**:
  - `X-RateLimit-Limit`: `60`
  - `X-RateLimit-Remaining`: Current token balance.
- [x] **Rate Exceeded Response**:
  - Status: `HTTP 429 Too Many Requests`
  - Header: `Retry-After: 60`
  - Body: Plain text `"Too Many Requests\n"` (or JSON if `Accept: application/json`).
- [x] **Request Body Limit**:
  - Enforced via `axum::extract::DefaultBodyLimit::max(64 * 1024)`.
  - Payloads exceeding 64KB are rejected with `HTTP 413 Payload Too Large`.

---

## 3. Findings & Recommendations

### Critical / Major Issues
- **None**. Zero Critical or Major issues identified.

### Minor Recommendations (Design Polish)
1. **Client IP Resolution Behind Proxies**:
   - In [crates/server/src/rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs), parse the first IP from `X-Forwarded-For` or `CF-Connecting-IP` when present, falling back to Axum's `ConnectInfo<SocketAddr>`, to avoid grouping all clients under a single reverse proxy IP.
2. **Authenticated Mutation Cache Invalidation**:
   - In authenticated mode (`auth_enabled == true`), when mutations succeed (e.g., `toggle_bookmark_htmx` or `mark_page_read_htmx`), invalidate the affected `entry:<id>` and `entries:*` cache keys so subsequent reads do not serve stale data before TTL expiry.

---

## 4. Contract Sign-Off

The API contracts defined for the authentication toggle, client-side guest state, in-memory caching, and DDoS rate limiting meet all design specifications, backward compatibility requirements, and security constraints. The interface contracts are approved for implementation.
