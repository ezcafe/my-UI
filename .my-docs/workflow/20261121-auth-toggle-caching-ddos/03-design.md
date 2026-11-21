# Design: Authentication Toggle, Client-Side State, Caching, and DDoS Protection

**Result:** done

## Key Takeaways
- **Authentication Toggle**: Add `AUTH_ENABLED: bool` (defaulting to `true`). When set to `false`, OIDC credentials and session secrets become optional, and the request extractor treats all incoming traffic as an authenticated guest user (`user_id: "guest"`).
- **UI Adaptation**: Navigation header suppresses the logout button when auth is disabled. Auth routes redirect gracefully to `/`.
- **Client-Side Read & Bookmark State**: In guest mode, read status is managed locally in browser IndexedDB with an in-memory `Set` cache for instant DOM updates without UI lag. Bookmarks trigger the Web Share API (`navigator.share`) or show a non-intrusive keyboard shortcut toast ("Press ⌘+D / Ctrl+D to bookmark").
- **Upstream Miniflux Shielding**: Mutation endpoints short-circuit with successful UI responses without calling Miniflux. Upstream Miniflux queries are cached in an in-memory TTL store (30s for entry lists/counters, 300s for entry details).
- **DDoS & Traffic Protection**: Axum/Tower rate-limiting middleware applies token-bucket throttling per IP address (60 req/min, burst 20) with a 64KB body size limit to safeguard public endpoints.

---

## Architecture Options

### Option A: In-Memory Protection & Native Browser State (Recommended)
- **Explanation**: 
  - Add `auth_enabled: bool` to `Config` in [crates/server/src/config.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/config.rs). When `auth_enabled` is false, fallback values are used for OIDC settings so the application runs without identity provider dependencies.
  - Implement `AuthenticatedUser` bypass in [crates/server/src/auth/middleware.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/auth/middleware.rs) that immediately provides a guest identity when `AUTH_ENABLED=false`.
  - Add a server-side in-memory cache `MinifluxCache` in `crates/server/src/cache.rs` that caches GraphQL query responses with a TTL to prevent upstream stampedes.
  - Add a per-IP rate-limiting layer `RateLimitLayer` in `crates/server/src/rate_limit.rs` using an in-memory token bucket.
  - Client-side read tracking in [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) initializes an IndexedDB database `my_news_guest_db` and synchronizes read IDs into an in-memory `Set`, synchronously toggling `.annuaire-row` indicators (3px red bar and unread dot) on initial load and HTMX swaps.
  - Client-side bookmarking invokes `navigator.share` or displays a clean toast without issuing server mutation requests.
- **Concrete Example**:
  ```rust
  // crates/server/src/cache.rs
  use std::collections::HashMap;
  use std::sync::Arc;
  use std::time::{Duration, Instant};
  use tokio::sync::RwLock;

  pub struct MinifluxCache {
      store: Arc<RwLock<HashMap<String, (Instant, serde_json::Value)>>>,
      ttl: Duration,
  }

  impl MinifluxCache {
      pub fn new(ttl: Duration) -> Self {
          Self {
              store: Arc::new(RwLock::new(HashMap::new())),
              ttl,
          }
      }

      pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
          let store = self.store.read().await;
          if let Some((inserted_at, val)) = store.get(key) {
              if inserted_at.elapsed() < self.ttl {
                  return Some(val.clone());
              }
          }
          None
      }

      pub async fn insert(&self, key: String, val: serde_json::Value) {
          let mut store = self.store.write().await;
          store.insert(key, (Instant::now(), val));
      }
  }
  ```
- **Pros**:
  - Self-contained in Rust and native browser APIs with zero external service dependencies.
  - Retains zero-build frontend architecture: pure vanilla JS + IndexedDB without npm.
  - Full backward compatibility for existing OIDC authenticated deployments.
  - High performance: instantaneous client-side UI updates and fast in-memory cache hits.
- **Cons**:
  - In-memory cache and rate limits are local to the single server process (suitable for typical deployments).

### Option B: Distributed Redis Cache & Service Worker Layer
- **Explanation**: 
  - Deploy Redis for response caching and distributed token bucket rate limiting.
  - Install a Service Worker on the frontend with the Cache API and Background Sync to intercept mutation calls and store read state.
- **Concrete Example**:
  ```rust
  pub struct RedisCache {
      conn: redis::aio::ConnectionManager,
  }
  ```
- **Pros**:
  - Coordinates cache and rate limits across horizontal server clusters.
  - Service Worker offers offline read support.
- **Cons**:
  - Requires running a separate Redis service, adding operational friction.
  - Violates project anti-goals ("Complex distributed caching infrastructure (like Redis) when in-process memory caching satisfies requirements cleanly").
  - Service Workers introduce cache-invalidation bugs and lifecycle complexity.

---

## Technical Specifications

### 1. Configuration & Auth Toggle
- **File**: `crates/server/src/config.rs`
- **Fields**:
  - `pub auth_enabled: bool` (parses `AUTH_ENABLED`, defaults to `true`).
- **Validation logic**:
  - When `auth_enabled == true`, `SESSION_SECRET`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, and `OIDC_REDIRECT_URL` are strictly validated as today.
  - When `auth_enabled == false`, missing OIDC variables and `SESSION_SECRET` use safe internal placeholders (e.g., `guest-mode-session-secret-at-least-32-chars-long!`) instead of returning `ConfigError::MissingVar`.

### 2. Authentication Bypass & Routes
- **File**: `crates/server/src/auth/middleware.rs`
  - In `AuthenticatedUser::from_request_parts`:
    ```rust
    if !config.auth_enabled {
        return Ok(AuthenticatedUser {
            user_id: "guest".to_string(),
            username: "Guest".to_string(),
            access_token: String::new(),
        });
    }
    ```
- **File**: `crates/server/src/auth/handlers.rs`
  - When `!config.auth_enabled`, `login_handler`, `callback_handler`, and `logout_handler` redirect immediately to `/`.

### 3. Upstream Shielding & In-Memory Caching
- **File**: `crates/server/src/cache.rs`
  - Thread-safe `MinifluxCache` wrapped in `AppState`.
  - Cache keys:
    - `entries:<filter_hash>` (TTL 30s)
    - `entry:<id>` (TTL 300s)
    - `counters` (TTL 30s)
- **File**: `crates/server/src/routes/entries.rs`
  - In `get_entry_detail_htmx`: If `!config.auth_enabled`, skip executing `updateEntriesStatus` mutation against Miniflux.
  - In `mark_entry_read_htmx`, `mark_page_read_htmx`, `mark_feed_read_htmx`, `mark_all_read_htmx`, `toggle_bookmark_htmx`: If `!config.auth_enabled`, return an appropriate successful HTML fragment or empty 200 OK without mutating Miniflux.

### 4. DDoS Protection & Rate Limiting
- **File**: `crates/server/src/rate_limit.rs`
  - Per-IP token-bucket rate limiter:
    - 60 tokens per minute, refill rate 1 token/sec, burst size 20.
    - Headers added: `X-RateLimit-Limit`, `X-RateLimit-Remaining`.
    - On limit exceeded: HTTP 429 Too Many Requests with header `Retry-After: 60`.
- **File**: `crates/server/src/app.rs`
  - Register rate limiter middleware and `DefaultBodyLimit::max(64 * 1024)`.

### 5. UI Templates & Client-Side State
- **File**: `crates/ui-components/src/templates.rs`
  - Add `pub auth_enabled: bool` to `IndexTemplate`.
- **File**: `crates/ui-components/templates/pages/index.html`
  - Render logout icon button only `{% if auth_enabled %}`.
  - Client-side script for guest mode:
    ```javascript
    const DB_NAME = 'my_news_guest_db';
    const STORE_NAME = 'read_entries';
    let readSet = new Set();

    function initGuestDb() {
      const req = indexedDB.open(DB_NAME, 1);
      req.onupgradeneeded = (e) => {
        e.target.result.createObjectStore(STORE_NAME, { keyPath: 'id' });
      };
      req.onsuccess = (e) => {
        const db = e.target.result;
        const tx = db.transaction(STORE_NAME, 'readonly');
        const store = tx.objectStore(STORE_NAME);
        store.getAllKeys().onsuccess = (ev) => {
          readSet = new Set(ev.target.result);
          syncDomReadStates();
        };
      };
    }

    function markLocallyRead(id) {
      readSet.add(Number(id));
      syncDomReadStates();
      const req = indexedDB.open(DB_NAME, 1);
      req.onsuccess = (e) => {
        const tx = e.target.result.transaction(STORE_NAME, 'readwrite');
        tx.objectStore(STORE_NAME).put({ id: Number(id), read_at: Date.now() });
      };
    }
    ```
  - Hook into `htmx:afterSwap` and `openDetailWithSkeleton` to update indicators and record read state.
  - Bookmark click handler in guest mode checks for `navigator.share` or shows a toast notification.
