# Analysis

**Result:** done

## Context

This project hardens `crates/server`, `crates/miniflux`, and `crates/ui-components` for production deployment on **Raspberry Pi OS** behind **Cloudflare Tunnel (`cloudflared`)**, using **Pocket-ID** for OIDC authentication and retaining inline scripts in Askama templates (`00-run.md` provenance).

### Current Architecture & Verified Findings
1. **Authentication & Route Guards (`crates/server/src/routes/entries.rs`, `auth/`):**
   - `list_entries_htmx`, `get_entry_detail_htmx`, `get_entry_ai_summary_htmx`, `toggle_bookmark_htmx`, `mark_entry_read_htmx`, `mark_feed_read_htmx`, `mark_all_read_htmx`, `mark_page_read_htmx`, `index_page`, and `graphql_handler` extract `AuthenticatedUser`, which supports both `AUTH_ENABLED=true` (encrypted `__session` cookie) and `AUTH_ENABLED=false` (synthetic guest user).
   - `executive_brief_htmx`, `get_entry_analysis_htmx`, and `get_entry_rewrite_htmx` omit `_user: AuthenticatedUser`, leaving AI/LLM and extractor endpoints unauthenticated when `AUTH_ENABLED=true`.
2. **OIDC & Pocket-ID Integration (`crates/server/src/auth/oidc.rs`, `temp_cookie.rs`, `handlers.rs`, `auth/mod.rs`):**
   - `generate_auth_request` discards `_nonce`, and `OidcTempData` stores only `pkce_verifier` and `csrf_state`.
   - `exchange_token` allocates a fresh `reqwest::Client` per call, ignores `id_token` signatures/claims (`iss`, `aud`, `exp`, `nonce`) via unverified base64 decoding, falls back to `("usr_100", "pocket_user")` when claims fail, and bypasses failed network calls with a dummy session when `config.oidc_issuer_url.contains("example.com") || cfg!(test)`.
   - Primary Pocket-ID OIDC documentation confirms standard discovery at `/.well-known/openid-configuration`, JWKS at `/.well-known/jwks.json`, `RS256` default ID token signing, `/authorize` + `/api/oidc/token` endpoints, and `client_secret_post` (`AuthType::RequestBody` in `openidconnect`) as Pocket-ID's primary confidential-client token auth method.
   - `crates/server/src/auth/mod.rs` tests (`test_oidc_callback_clears_temp_cookie`, `test_secure_cookie_flag_when_https`) currently depend on the `example.com` dummy bypass and will need `wiremock` OIDC/JWKS test fixtures.
3. **Cookie Key Separation & Startup Precomputation (`crates/server/src/auth/crypto.rs`, `session.rs`, `temp_cookie.rs`):**
   - Both `__session` and `__oidc_temp` currently encrypt using `Sha256(session_secret)` with no domain separation, and `encrypt_json`/`decrypt_json` re-hash and re-initialize `Aes256Gcm` on every request.
   - `hkdf = "0.12.4"` is already present in `Cargo.lock` and supports HKDF-SHA256 (`Hkdf::<Sha256>::new(None, secret.as_bytes()).expand(info, &mut key)`) for domain-separated keys (`__session` vs `__oidc_temp`) and startup cipher precomputation.
4. **Rate Limiting Behind Cloudflare Tunnel (`crates/server/src/rate_limit.rs`, `app.rs`, `main.rs`):**
   - `rate_limit_middleware` reads `x-forwarded-for` / `x-real-ip` from any client without checking peer IP, ignores `CF-Connecting-IP`, and `main.rs` invokes `axum::serve(listener, app)` without `.into_make_service_with_connect_info::<SocketAddr>()`.
   - `RateLimiter` uses `tokio::sync::Mutex` around synchronous in-memory `HashMap` bucket math and only prunes when `len() > 100` without a hard upper cap.
   - Cloudflare Tunnel documentation confirms `cloudflared` injects a single bare client IP in `CF-Connecting-IP` (preferred over `X-Forwarded-For` to prevent spoofing).
5. **SSRF & Streaming Memory Bounds (`crates/server/src/ai/extractor.rs`):**
   - `ArticleExtractor::extract_content` checks only `url.starts_with("http")`, allows default reqwest redirects, performs no IP/DNS filtering, buffers the full `resp.text().await`, and traverses the full `<article>`/`<body>` DOM before truncating to 250 words.
   - `reqwest 0.12` supports `Response::chunk().await` without adding the `"stream"` Cargo feature, and `ClientBuilder::dns_resolver` + `redirect::Policy::none()` allow pre-connect DNS IP validation and safe manual redirect checks. Existing unit tests use `wiremock` on `127.0.0.1`, requiring an explicit test-only toggle for loopback in tests.
6. **Security Headers, SRI & Template Injection Surfaces (`crates/server/src/app.rs`, `crates/ui-components/templates/`, `routes/entries.rs`):**
   - `app.rs` sets no security response headers (`Content-Security-Policy`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`).
   - `base.html` loads `https://cdn.tailwindcss.com` (Play CDN), `https://unpkg.com/htmx.org@1.9.12` (no SRI), `https://unpkg.com/lucide@latest` (unpinned, no SRI), and inline scripts/handlers (`index.html:51` also uses `hx-vals="js:{entry_ids: getVisibleEntryIds()}"`, which causes HTMX to call `new Function()`).
   - Rust templates render `entry.url` into `href="{{ entry.url }}"` and `data-url` without scheme checks, and `entry_list.html:124,155` embeds `{{ entry.feed_title }}` inside single-quoted JS (`onclick="openEntryFromMagazine({{ entry.id }}, '{{ entry.feed_title }}')"`), which is vulnerable to quote-breakout XSS if a feed title contains `'`.
7. **HTML Sanitizer (`crates/ui-components/src/sanitizer.rs`):**
   - `sanitize_html` builds a fresh `ammonia::Builder` and `HashSet` per call and runs `RE_ANCHOR`/`RE_TARGET` regexes over the output HTML. `ammonia 4.2.5` provides `Builder::set_tag_attribute_value("a", "target", "_blank")` (while preserving default `link_rel(Some("noopener noreferrer"))`) and `LazyLock<Builder<'static>>` reuse.
8. **GraphQL Bounds (`crates/miniflux/src/schema/mod.rs`, `query.rs`):**
   - `build_schema` lacks `.limit_depth(...)` and `.limit_complexity(...)`, and `EntryFilterInput.limit` accepts unbounded positive integers.
9. **Concurrency, Caching & Logging (`routes/entries.rs`, `pages.rs`, `cache.rs`, `ai/mod.rs`, `ai/prefetch.rs`, `ai/llm.rs`, `error.rs`):**
   - `executive_brief_htmx` polls `ai_service.db.get_page_brief_by_key` up to 40x100ms while `in_flight` contains `page_key`; both `InFlightGuard::drop` implementations spawn detached Tokio tasks to remove keys from `tokio::sync::Mutex<HashSet<String>>`.
   - `index_page` and `render_entries_list_html` query full `content` HTML over GraphQL even when `auth_enabled = true` (where only `image_url` is needed) and cache full `serde_json::Value` trees (up to 1000 items) in `MinifluxCache`.
   - `llm.rs` (12 sites), `error.rs`, `entries.rs`, `app.rs`, and `main.rs` use synchronous `println!`/`eprintln!` on request and LLM hot paths.

## Risks

- **Cloudflare Tunnel proxy IP mismatch:** If `my-ui` runs in Docker Compose (`docker-compose.yml`) while `cloudflared` runs on the Pi host or another container, peer `ConnectInfo<SocketAddr>` is a Docker bridge IP (`172.x.x.x`), not `127.0.0.1`. Hardcoding only `127.0.0.1` without configurable `TRUSTED_PROXIES` would collapse all internet users into one rate-limit bucket; trusting headers from untrusted peers allows rate-limit spoofing.
- **Tailwind Play CDN vs SRI / CSP `'unsafe-eval'`:** Primary Tailwind/CSP documentation confirms `https://cdn.tailwindcss.com` is unpinned/dynamic and **cannot** use SRI (`integrity=...`), and HTMX `hx-vals="js:..."` / Play CDN dynamic styling interact with `'unsafe-eval'`/`'unsafe-inline'`. Claiming SRI on all CDN scripts while keeping Play CDN would silently skip Tailwind or break Tailwind if a hash were added.
- **Pocket-ID OIDC client authentication & key rotation:** `openidconnect::CoreClient` defaults to HTTP Basic auth (`client_secret_basic`), whereas Pocket-ID primarily expects `client_secret_post` (`AuthType::RequestBody`). Additionally, caching JWKS without refreshing on an unknown `kid` would break logins after Pocket-ID key rotation.
- **Existing test suite breakage:** Removing the `example.com` bypass in `oidc.rs` will fail `test_oidc_callback_clears_temp_cookie` and `test_secure_cookie_flag_when_https` unless converted to `wiremock` OIDC mocks; adding `AuthenticatedUser` to `executive_brief_htmx` changes direct handler test signatures in `entries.rs`; adding HKDF domain separation invalidates pre-existing cookies once on deploy.
- **Sandbox network constraint:** Because the test/build environment has no external network access, any pinned CDN SRI hash (e.g., for `lucide`) must either be verified against an authoritative source or vendored locally rather than guessed.

## Open questions

1. **Trusted proxy topology & defaults (`rate_limit.rs` / `config.rs`):** Does `cloudflared` connect to `my-ui` on host loopback (`127.0.0.1`/`::1`) or across a Docker bridge network, and should `TRUSTED_PROXIES` default to loopback (`127.0.0.1,::1`) while preferring `CF-Connecting-IP`?
2. **Tailwind Play CDN vs SRI & `'unsafe-eval'` (`base.html` / `app.rs`):** Since `cdn.tailwindcss.com` cannot support SRI (and `hx-vals="js:..."` triggers `eval`), should we keep Tailwind Play CDN without SRI, refactor `hx-vals="js:..."` to avoid `eval`, and add SRI hashes only to pinned `htmx.org@1.9.12` + `lucide` (or vendor/prebuild CSS/JS)?
3. **Mixed-content images in CSP (`app.rs`):** Should CSP `img-src` restrict RSS article thumbnails to `'self' https: data:` (blocking cleartext `http:` image loads) or allow `'self' https: http: data:` for older HTTP-only feeds?
