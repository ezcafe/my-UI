# Design — Security, Performance, and Memory Hardening

**Result:** done

**Design present** only when Result **`done`** (file alone / `pending` never pays) — [../derive-next.md](../derive-next.md).

## Key Takeaways

We recommend **Option B: Precomputed Security Primitives, Typed Cache & `openidconnect` Verification**. It upgrades all security, speed, and memory targets across `crates/server`, `crates/miniflux`, and `crates/ui-components` using the existing locked dependencies (`openidconnect 3.5`, `hkdf 0.12.4`, `ipnet 2.12.2`, `ammonia 4.2.5`, `async-graphql 7.2.1`, `reqwest 0.12.28`, `tokio::sync::Notify`) without changing database schemas or dropping inline scripts.

---

## Option A — Minimal In-Place Patches (Per-Call Helpers & Unmodified Cache Shapes)

### Explanation
Keep the current structural patterns while patching individual vulnerabilities locally:
- Add `AuthenticatedUser` to the three unprotected AI routes, check `nonce` and `sub` with manual JSON/RSA validation in `oidc.rs`, and keep creating per-request HTTP clients and AES-GCM ciphers.
- Validate `X-Forwarded-For` / `CF-Connecting-IP` using simple `IpAddr` equality against `127.0.0.1`/`::1` string splits (no CIDR subnet support).
- Keep `serde_json::Value` in `MinifluxCache` (simply stripping `"content"` keys in JSON maps before caching), keep `tokio::sync::Mutex` in `RateLimiter` / `AiService`, and only shorten the polling loop or switch `println!` to `tracing`.

### Sketch
```rust
// Option A example: strip JSON fields before caching raw Value
if let Some(entries) = data.pointer_mut("/entriesResult/entries").and_then(|v| v.as_array_mut()) {
    for item in entries {
        if let Some(obj) = item.as_object_mut() {
            obj.remove("content");
        }
    }
}
cache.insert_with_ttl(cache_key, data.clone(), Duration::from_secs(30)).await;
```

### Pros
- Smallest diff in `AppState`, `cache.rs`, and `oidc.rs`.
- Fewer helper structs to introduce.

### Cons
- **Fails CIDR proxy requirement:** String `IpAddr` checks cannot match Docker bridge subnets (e.g., `172.17.0.0/16`) configured in `TRUSTED_PROXIES`.
- **Re-derives crypto & HTTP pools on hot paths:** Per-request SHA-256/AES-GCM setup and per-login `reqwest::Client` creation waste CPU and TLS handshakes on Raspberry Pi.
- **Brittle OIDC validation:** Manual JWT validation risks missing OIDC edge cases (`iss`, `aud`, `exp`, `kid` rotation) already handled by `openidconnect::CoreIdTokenVerifier`.
- **Retains memory/lock overhead:** `serde_json::Value` cloning still allocates dynamic JSON trees on every cache hit, and `tokio::spawn` in `InFlightGuard::drop` still schedules detached tasks.

---

## Option B — Precomputed Security Primitives, Typed Cache & `openidconnect` Verification (Recommended)

### Explanation
Build small, startup-precomputed, type-safe primitives inside `AppState` and `ui-components` while keeping existing public behavior, Askama templates, and PostgreSQL schema intact.

### Concrete Architecture & Contracts

#### 1. Route Auth Guards (`crates/server/src/routes/entries.rs`)
- Add `_user: AuthenticatedUser` extractor to:
  - `executive_brief_htmx`
  - `get_entry_analysis_htmx`
  - `get_entry_rewrite_htmx`
- When `AUTH_ENABLED=true`, unauthenticated HTMX callers receive `200 OK` + `HX-Redirect: /login` (matching `list_entries_htmx` and `get_entry_detail_htmx`); JSON callers receive `401 Unauthorized`; when `AUTH_ENABLED=false`, guest mode continues unchanged.
- Update the 4 direct unit test invocations of `executive_brief_htmx` in `crates/server/src/routes/entries.rs` to supply `AuthenticatedUser` (or test via `Router::oneshot`), plus add integration tests asserting unauthenticated `AUTH_ENABLED=true` rejections on all three routes.

#### 2. Pocket-ID OIDC Verification & Shared HTTP Client (`crates/server/src/auth/oidc.rs`, `temp_cookie.rs`, `handlers.rs`, `app.rs`, `auth/mod.rs`)
- **Nonce storage:** Add `pub nonce: String` to `OidcAuthRequest` and `OidcTempData` (`__oidc_temp` encrypted cookie).
- **Shared HTTP client & discovery/JWKS cache:** Store `OidcService` in `AppState` holding:
  - One shared `reqwest::Client` (0.12) configured with `redirect::Policy::none()` and a 10s timeout.
  - A tiny `AsyncHttpClient` adapter converting `openidconnect::HttpRequest` ↔ `reqwest::Request` and returning `openidconnect::HttpResponse` (since `openidconnect 3.5` exposes a functional closure implementation of `AsyncHttpClient`).
  - `RwLock<Option<CachedOidcProvider>>` caching `(CoreProviderMetadata, Instant)` so discovery + JWKS (`/.well-known/openid-configuration` + `jwks_uri`) is reused across logins (e.g., 15-minute TTL) and force-refreshed once when ID token signature verification fails (handling Pocket-ID key rotation).
- **Token exchange & claim verification:**
  - Instantiate `CoreClient::from_provider_metadata(metadata, client_id, Some(client_secret)).set_auth_type(AuthType::RequestBody).set_redirect_uri(redirect_url)` (matching Pocket-ID's `client_secret_post` requirement).
  - Exchange code via `client.exchange_code(AuthorizationCode::new(code)).set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier)).request_async(&http_adapter).await`.
  - Require `id_token` on token response (`OidcError::Token` if missing) and verify signature, `iss`, `aud`, `exp`, and `nonce` via `id_token.claims(&client.id_token_verifier(), &expected_nonce)`. On signature mismatch, force-refresh metadata/JWKS once and re-verify before failing closed.
  - Extract `sub` (required) and `username` (`preferred_username` -> `name` -> `email` -> `sub`).
  - **Delete** the `config.oidc_issuer_url.contains("example.com") || cfg!(test)` dummy bypass and the `("usr_100", "pocket_user")` fallback.
  - Update `crates/server/src/auth/mod.rs` tests (`test_oidc_callback_clears_temp_cookie`, `test_secure_cookie_flag_when_https`) to mount a `wiremock` OIDC provider using `openidconnect::core::CoreRsaPrivateSigningKey` to sign valid ID tokens, and add negative tests for invalid signature, wrong `nonce`, wrong `aud`/`iss`, and expired `exp`.

#### 3. HKDF Key Separation & Startup Cipher Precomputation (`crates/server/src/auth/crypto.rs`, `session.rs`, `temp_cookie.rs`, `middleware.rs`, `handlers.rs`, `app.rs`)
- Add `hkdf = "0.12"` (already locked at `0.12.4` in `Cargo.lock`) to `crates/server/Cargo.toml`.
- Add `AuthKeyRing` (or `CryptoCiphers`) precomputed at startup in `AppState`:
  - `session_cipher: Aes256Gcm` derived with `Hkdf::<Sha256>::new(None, secret.as_bytes()).expand(b"my-ui:__session:v1", &mut key)`.
  - `oidc_temp_cipher: Aes256Gcm` derived with `Hkdf::<Sha256>::new(None, secret.as_bytes()).expand(b"my-ui:__oidc_temp:v1", &mut key)`.
- Handlers (`login_handler`, `callback_handler`) and `AuthenticatedUser` extractor use the precomputed `AuthKeyRing` from `AppState` (zero per-request SHA-256 / AES key-schedule allocation).
- Existing secret-taking helpers (`create_session_cookie_value`, `extract_session_cookie_value`, `create_temp_cookie_value`, `extract_temp_cookie_value`) also delegate to domain-separated HKDF helpers so cross-cookie token substitution (`__oidc_temp` pasted as `__session` or vice versa) always fails decryption.

#### 4. Trusted-Proxy Rate Limiting & Hard Cap (`crates/server/src/rate_limit.rs`, `config.rs`, `main.rs`, `.env.example`)
- Add `ipnet = "2.12"` (already locked at `2.12.2` in `Cargo.lock`) to `crates/server/Cargo.toml`.
- Add `pub trusted_proxies: Vec<ipnet::IpNet>` to `Config`:
  - Parsed from `TRUSTED_PROXIES` env var (default `"127.0.0.1,::1"`).
  - Each comma-separated entry parses as either `IpNet` (`172.17.0.0/16`) or bare `IpAddr` (`127.0.0.1` -> `/32`, `::1` -> `/128`). Invalid entries return `ConfigError::InvalidValue`. Empty `TRUSTED_PROXIES=""` disables proxy header trust cleanly.
- Update `crates/server/src/main.rs`:
  - `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;`
- Update `rate_limit_middleware`:
  - Extract `peer: Option<Extension<ConnectInfo<SocketAddr>>>` (or `req.extensions().get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip())`).
  - Only when `peer` is `Some(peer_ip)` AND `config.trusted_proxies.iter().any(|net| net.contains(&peer_ip))`:
    - Read `CF-Connecting-IP` first (single valid IP), falling back to first valid IP in `X-Forwarded-For`, then `X-Real-IP`, then `peer_ip`.
  - Otherwise (untrusted peer OR missing `ConnectInfo`), ignore all forwarding headers and rate-limit by `peer_ip` (or `"unknown"` if `ConnectInfo` is absent, failing closed against spoofing).
- Update `RateLimiter`:
  - Switch `buckets` lock from `tokio::sync::Mutex` to `std::sync::Mutex` (fast synchronous critical section, non-blocking across `.await`).
  - Enforce `MAX_RATE_LIMIT_BUCKETS: usize = 10_000`: when `buckets.len() >= MAX_RATE_LIMIT_BUCKETS`, first prune stale buckets (`>= 600s` idle); if still at cap and inserting a new key, evict the bucket with the oldest `last_refill`.
- Document `TRUSTED_PROXIES` (loopback default + Docker bridge CIDR note) in `.env.example`.

#### 5. SSRF Protection & Streaming Memory Bounds (`crates/server/src/ai/extractor.rs`)
- Add `url = "2.5"` (already locked in `Cargo.lock`) to `crates/server/Cargo.toml`.
- Implement `is_disallowed_ip(ip: IpAddr, allow_loopback: bool) -> bool` blocking:
  - Loopback (`127.0.0.0/8`, `::1`) unless `allow_loopback` is true (test-only constructor `ArticleExtractor::new_allow_loopback_for_tests()`).
  - Private (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), ULA (`fc00::/7`).
  - Link-local (`169.254.0.0/16`, `fe80::/10`).
  - Unspecified (`0.0.0.0/8`, `::`), multicast, broadcast, documentation/benchmark/reserved, and CGNAT (`100.64.0.0/10`), plus IPv4-mapped IPv6 (`::ffff:0:0/96`, inspecting the embedded IPv4 address).
- Install a custom `SsrSafeDnsResolver` implementing `reqwest::dns::Resolve` on the `reqwest::Client` (`redirect::Policy::none()`, 3s timeout): resolves host via `tokio::net::lookup_host`, rejects if any returned IP is disallowed (`io::Error::new(PermissionDenied, ...)`), preventing DNS rebinding and internal hostnames before connecting.
- Validate initial URL (`http`/`https` only, non-empty host, literal IP check) and follow up to `MAX_REDIRECTS = 3` manually by inspecting `3xx` `Location` headers, resolving relative redirects against current `Url`, and re-validating each target before sending the next `GET`.
- Stream response bytes via `resp.chunk().await` up to `MAX_HTML_BYTES = 512 * 1024` (512 KiB) — aborting read once reached (also rejecting upfront if `Content-Length > MAX_HTML_BYTES` or passing capped buffer to `parse_html`).
- Update `ArticleExtractor::traverse_element` to stop traversing DOM nodes once `word_count >= MAX_EXTRACTED_WORDS` (250).

#### 6. Security Headers, SRI, Template Injection & URL Validation (`app.rs`, `base.html`, `index.html`, `entry_list.html`, `entries.rs`, `sanitizer.rs`, `templates.rs`)
- **HTTP Response Security Headers (`app.rs`):** Add `security_headers_middleware` layer setting:
  - `Content-Security-Policy`:
    `default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://unpkg.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' https: data:; connect-src 'self' https://cdn.tailwindcss.com; frame-ancestors 'none'; object-src 'none'; base-uri 'self'; form-action 'self'`
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`
- **CDN Pinning, SRI & `eval` Removal (`base.html`, `index.html`):**
  - Keep `https://cdn.tailwindcss.com` (no SRI, per Q2; Play CDN v3 does not require `'unsafe-eval'`).
  - Pin HTMX with official SHA-384 SRI:
    `<script src="https://unpkg.com/htmx.org@1.9.12" integrity="sha384-ujb1lZYygJmzgSwoxRggbCHcjc0rB2XoQrxeTUQyRjrOnlCoYta87iKBWq3EsdM2" crossorigin="anonymous"></script>`
  - Pin Lucide with verified SHA-384 SRI:
    `<script src="https://unpkg.com/lucide@0.576.0/dist/umd/lucide.min.js" integrity="sha384-b05ba3pt6xaC7F4r130arhf8cF18GH/gKu9JDz/NMf+BhLlBVwIWUdAZSpf1IWRZ" crossorigin="anonymous"></script>`
  - Set `if (window.htmx) { htmx.config.allowEval = false; }` on startup in `base.html`.
  - Remove `hx-vals="js:{entry_ids: getVisibleEntryIds()}"` from `#mark-page-read-btn` in `index.html` and inject `evt.detail.parameters['entry_ids'] = getVisibleEntryIds();` inside the `htmx:configRequest` listener when `trigger && trigger.id === 'mark-page-read-btn'`.
- **Template Quote-Breakout XSS Fix (`entry_list.html`, `index.html`, `templates.rs`):**
  - Replace `onclick="openEntryFromMagazine({{ entry.id }}, '{{ entry.feed_title }}')"` (at `entry_list.html:124,155,183`) with `onclick="openEntryFromMagazine({{ entry.id }}, this)"` and update `openEntryFromMagazine(id, feedOrEl)` in `index.html` to read `feedOrEl.closest('.magazine-card')?.getAttribute('data-feed-title')` when passed a DOM element (preserving string compatibility if called with a string). Update the matching assertion in `crates/ui-components/src/templates.rs:2814` and add a quote-breakout test (`O'Reilly's; alert(1)//`).
- **Safe URL Validation (`entries.rs` / `ui-components`):**
  - Validate `entry.url` with `sanitize_http_url(raw)` allowing only `http://` and `https://` schemes (case-insensitive, trimmed), falling back to `"#"`.
  - Validate extracted image URLs in `extract_first_image_url` and `extract_article_images` with `sanitize_image_url(raw)` allowing `https://` and non-SVG raster `data:image/(png|jpeg|jpg|gif|webp);base64,` (blocking `http://`, `javascript:`, `data:image/svg+xml`, protocol-relative `//`), matching CSP `img-src 'self' https: data:`.
- **HTML Sanitizer Optimization (`crates/ui-components/src/sanitizer.rs`):**
  - Store `static AMMONIA_BUILDER: LazyLock<Builder<'static>>` initialized once with `.tags(...)`, `.add_tag_attributes("a", &["href"])`, `.set_tag_attribute_value("a", "target", "_blank")`, `.link_rel(Some("noopener noreferrer"))`, `.url_schemes(HashSet::from(["http", "https"]))`, `.add_tag_attributes("img", &["src", "alt", "title", "loading"])`.
  - Remove `RE_ANCHOR`, `RE_TARGET`, and regex post-processing completely. (Update `test_sanitizer_multiple_links_all_receive_target_blank` if ammonia outputs `href`/`target`/`rel` in a different attribute order, asserting semantic attribute presence on each `<a>` tag.)

#### 7. GraphQL Bounds (`crates/miniflux/src/schema/mod.rs`, `query.rs`, `types.rs`)
- Add `.limit_depth(8)` and `.limit_complexity(100)` to `Schema::build(Query, Mutation, EmptySubscription)` in `build_schema`.
- Enforce upper bound `MAX_GQL_LIMIT: i32 = 100` in `build_domain_filter` (`query.rs`), returning `Err(Error::new("Limit cannot exceed 100"))` when `lim > 100`.
- Add `pub image_url: Option<String>` (`imageUrl` field) to `EntryGql` (`types.rs`), populated in `map_entry` and `Query::entry` from `content` using first safe image extraction so list queries never need to select full `content` HTML when `auth_enabled = true`.

#### 8. Event-Driven Brief Wait, Lean Cache & Structured Logging (`ai/mod.rs`, `ai/prefetch.rs`, `cache.rs`, `pages.rs`, `entries.rs`, `llm.rs`, `error.rs`, `main.rs`, `app.rs`)
- **`InFlightRegistry` (`ai/mod.rs`, `ai/prefetch.rs`, `entries.rs`):**
  - Replace `Arc<tokio::sync::Mutex<HashSet<String>>>` with `InFlightRegistry` backed by `std::sync::Mutex<HashMap<String, Arc<tokio::sync::Notify>>>` and `MAX_IN_FLIGHT_KEYS: usize = 512`.
  - `try_acquire(key) -> AcquireOutcome` returns either `Leader(InFlightGuard)` (or `CapacityFull` if at cap) or `Follower(Arc<tokio::sync::Notify>)`.
  - `InFlightGuard::drop` synchronously removes `key` under `std::sync::Mutex` and calls `notify.notify_waiters()` — zero `tokio::spawn` in `Drop`.
  - `executive_brief_htmx` awaits `let notified = notify.notified();` (created before re-checking `ai_service.db.get_page_brief_by_key(&page_key)`) with a 10s `tokio::time::timeout`, eliminating the `40 x 100ms` DB polling loop.
- **Lean `MinifluxCache` (`cache.rs`, `pages.rs`, `entries.rs`):**
  - Add typed slim cache payloads (`CachedEntriesPage`, `CachedEntrySummaryItem`, `CachedEntryDetail`) alongside or inside `MinifluxCache` (keeping backward-compatible `get`/`insert` methods or typed cache wrapper methods) so:
    - Authenticated `index_page` and `render_entries_list_html` omit `content` from the GraphQL query (using `imageUrl` instead) and cache slim `CachedEntriesPage` structs with zero raw HTML `content` strings.
    - Guest `index_page` and `render_entries_list_html` sanitize `content` once on cache miss and store slimmed `EntryViewModel`s (never raw unsanitized `content` JSON trees).
    - `get_entry_detail_htmx` extracts `(hero_image, body_images)` and `teaser` once on cache miss and stores `CachedEntryDetail` — discarding raw `content` immediately.
- **Structured Logging (`llm.rs`, `error.rs`, `entries.rs`, `app.rs`, `main.rs`):**
  - Replace all 20 `println!` / `eprintln!` calls with `tracing::info!`, `tracing::warn!`, `tracing::error!`, and `tracing::debug!` (specifically putting `dispatch_msg` prompt previews and `return_msg` raw LLM outputs in `llm.rs` at `tracing::debug!` so production `INFO` logs stay concise and avoid SD-card churn).

### Pros
- Addresses every security, performance, and memory target cleanly with production-grade primitives (`openidconnect::CoreIdTokenVerifier`, `hkdf`, `ipnet`, `SsrSafeDnsResolver`, `tokio::sync::Notify`, `ammonia::Builder::set_tag_attribute_value`).
- Requires **zero new external crate downloads** (`hkdf`, `ipnet`, `url` are already locked in `Cargo.lock`).
- Preserves existing PostgreSQL schema (`AiDb`), guest IndexedDB mode, HTMX UX, and inline template structure.

### Cons
- Existing `auth/mod.rs` tests relying on the `example.com` bypass must be migrated to `wiremock` + `CoreRsaPrivateSigningKey` fixtures.
- Existing `entries.rs` direct handler tests for `executive_brief_htmx` must pass `AuthenticatedUser`.
- Deploying HKDF domain separation invalidates pre-existing `__session` and `__oidc_temp` cookies once (users sign in again cleanly).

---

## Recommendation

We recommend **Option B**. It resolves every security vulnerability cleanly (especially Pocket-ID OIDC `id_token` verification, CIDR-capable `TRUSTED_PROXIES`, DNS-rebinding-safe SSRF protection, quote-breakout XSS prevention, and domain-separated HKDF cookie keys) while delivering the full memory and latency gains on Raspberry Pi OS with zero unverified dependencies.

---

## Proposed Has (parent applies to card)

- **Has UI:** `yes`
- **Has API:** `yes`
- **Has DB:** `no`
- **Has auth/PII/schema:** `yes`

*(No Has flag changes from the current run card.)*

## Has API / DB (proposed)

- **Has API = `yes`:** Adds `AuthenticatedUser` auth enforcement on `/hx/executive-brief`, `/hx/entries/{id}/analysis`, `/hx/entries/{id}/rewrite`; updates `/auth/login` & `/auth/callback` OIDC state/nonce/ID-token contract; adds HTTP response security headers + CSP; adds `TRUSTED_PROXIES` header resolution on all routes; adds GraphQL `limit_depth(8)` / `limit_complexity(100)` / `EntryFilterInput.limit <= 100` and additive `EntryGql.imageUrl`.
- **Has DB = `no`:** Zero PostgreSQL schema, migration, or SQL query changes (`AiDb` unchanged).

---

## Verifiable References

- `openidconnect` 3.5 `CoreIdTokenVerifier`, `AuthType::RequestBody`, `CoreProviderMetadata::discover_async`, `CoreRsaPrivateSigningKey`: <https://docs.rs/openidconnect/3.5.0/openidconnect/>
- `reqwest` 0.12 `dns::Resolve` & `Response::chunk`: <https://docs.rs/reqwest/0.12/reqwest/>
- `ammonia` 4.2.5 `Builder::set_tag_attribute_value`, `link_rel`, `url_schemes`: <https://docs.rs/ammonia/4.2.5/ammonia/struct.Builder.html>
- `async-graphql` 7.2.1 `SchemaBuilder::limit_depth` & `limit_complexity`: <https://docs.rs/async-graphql/7.2.1/async_graphql/struct.SchemaBuilder.html>
- `axum` 0.8 `ConnectInfo`: <https://docs.rs/axum/0.8/axum/extract/struct.ConnectInfo.html>
- `htmx` 1.9.12 SRI & `allowEval` / `htmx:configRequest` CSP docs: <https://htmx.org/docs/#installing> · <https://htmx.org/docs/#config>
- `lucide` `0.576.0` UMD build & `hkdf` `0.12.4` / `ipnet` `2.12.2` locked versions: `Cargo.lock`
- Existing workspace implementation files:
  - `crates/server/src/auth/{oidc.rs,crypto.rs,session.rs,temp_cookie.rs,handlers.rs,middleware.rs,mod.rs}`
  - `crates/server/src/{app.rs,main.rs,config.rs,rate_limit.rs,cache.rs,error.rs}`
  - `crates/server/src/ai/{mod.rs,extractor.rs,prefetch.rs,llm.rs}`
  - `crates/server/src/routes/{pages.rs,entries.rs}`
  - `crates/miniflux/src/schema/{mod.rs,query.rs,types.rs}`
  - `crates/ui-components/src/{sanitizer.rs,templates.rs}` and `crates/ui-components/templates/{base.html,pages/index.html,partials/entry_list.html}`
