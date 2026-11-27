# Skim

**Result:** done

## Findings

### Workspace Layout & Dependencies
- Workspace has 3 crates in `Cargo.toml` (`edition = "2021"`): `crates/server`, `crates/miniflux`, `crates/ui-components`.
- `Cargo.lock` resolves `axum = 0.8.9`, `tower-http = 0.6.11`, `async-graphql = 7.2.1`, `openidconnect = 3.5.0`, `ammonia = 4.2.5`, `reqwest = 0.12.28` (plus `0.11.27` pulled transitively by `openidconnect`), `aes-gcm = 0.10`, `sha2 = 0.10.9`, `hkdf = 0.12.4` (already in `Cargo.lock`), `scraper = 0.22`, `sqlx = 0.8`.
- Deployment files (`Dockerfile`, `docker-compose.yml`, `.env.example`, `README.md`) target Docker Compose + PostgreSQL (`AiDb` in `crates/server/src/ai/db.rs`) or native binary on Raspberry Pi OS, behind Cloudflare Tunnel (`cloudflared`), with Pocket-ID OIDC.

### Security Hotspots
1. **Unauthenticated AI HTMX routes (`crates/server/src/routes/entries.rs`):** `executive_brief_htmx`, `get_entry_analysis_htmx`, and `get_entry_rewrite_htmx` omit `_user: AuthenticatedUser`, exposing LLM calls and article fetching when `AUTH_ENABLED=true`. (Direct unit tests in `entries.rs` also invoke `executive_brief_htmx` directly.)
2. **OIDC verification & auth bypass (`crates/server/src/auth/oidc.rs`, `temp_cookie.rs`, `handlers.rs`, `auth/mod.rs`):**
   - `generate_auth_request` discards `_nonce`; `OidcTempData` stores only `pkce_verifier` and `csrf_state`.
   - `exchange_token` parses JWT payload base64 without signature, `iss`, `aud`, `exp`, or `nonce` verification, creates a fresh `reqwest::Client` per call, falls back to `("usr_100", "pocket_user")` if claims fail, and mints a fake session whenever `oidc_issuer_url.contains("example.com") || cfg!(test)` on network error.
   - `auth/mod.rs` tests (`test_oidc_callback_clears_temp_cookie`, `test_secure_cookie_flag_when_https`) currently depend on that `example.com` fallback.
3. **SSRF & unbounded reads (`crates/server/src/ai/extractor.rs`):** `ArticleExtractor::extract_content` checks only `url.starts_with("http")`, follows redirects by default, performs no IP/DNS validation (loopback, private, link-local, unspecified, ULA/CGNAT), reads full `resp.text().await` into memory, and parses the full DOM before truncating to 250 words. Existing unit tests use `wiremock` on `127.0.0.1`.
4. **Rate limiter spoofing & unbounded buckets (`crates/server/src/rate_limit.rs`, `main.rs`):** `rate_limit_middleware` trusts `X-Forwarded-For` / `X-Real-IP` unconditionally from any sender (ignores `CF-Connecting-IP` and peer `SocketAddr`), `main.rs` uses `axum::serve(listener, app)` without `into_make_service_with_connect_info::<SocketAddr>()`, uses `tokio::sync::Mutex` for pure in-memory math, and has no hard bucket cap.
5. **Missing browser headers & SRI (`crates/server/src/app.rs`, `crates/ui-components/templates/base.html`, `index.html`):** No CSP, `X-Content-Type-Options`, `X-Frame-Options`, or `Referrer-Policy` headers on responses. `base.html` loads unpinned `lucide@latest` and `htmx.org@1.9.12` without `integrity`/`crossorigin`, plus `https://cdn.tailwindcss.com` (Play CDN) and inline scripts/handlers (`hx-vals="js:..."` on `index.html:51`).
6. **Unsafe URL & attribute rendering (`crates/server/src/routes/entries.rs`, `crates/ui-components/templates/partials/`):** Rust view-model builders pass raw `entry.url` and extracted `<img src>` URLs into `href="{{ entry.url }}"` / `src="..."` without scheme validation (`http://`/`https://` only). Additionally, `entry_list.html:124,155` interpolates `{{ entry.feed_title }}` directly inside single-quoted JS in `onclick="openEntryFromMagazine(..., '{{ entry.feed_title }}')"`, allowing quote-breakout JS injection if a feed title contains `'`.
7. **Unbounded GraphQL (`crates/miniflux/src/schema/mod.rs`, `query.rs`):** `build_schema` sets no `limit_depth` or `limit_complexity`, and `EntryFilterInput.limit` has no upper bound.
8. **Cookie encryption without domain separation (`crates/server/src/auth/crypto.rs`):** `derive_key` hashes `session_secret` with raw SHA-256 for both `__session` and `__oidc_temp`, and re-derives the key + `Aes256Gcm` cipher on every request.
9. **Sanitizer regex post-processing (`crates/ui-components/src/sanitizer.rs`):** Rebuilds `ammonia::Builder` + `HashSet` on every call and rewrites `<a ...>` with `RE_ANCHOR`/`RE_TARGET` regexes.

### Performance & Memory Hotspots
- **100ms DB polling + `tokio::spawn` in `Drop` (`routes/entries.rs`, `ai/prefetch.rs`, `ai/mod.rs`):** `executive_brief_htmx` polls `ai_service.db.get_page_brief_by_key` up to 40x100ms when `in_flight` contains `page_key`; both `InFlightGuard::drop` impls call `tokio::spawn` to remove the key from `tokio::sync::Mutex<HashSet<String>>`, and `in_flight` has no capacity cap.
- **Hot-path `println!`/`eprintln!` (`ai/llm.rs`, `error.rs`, `routes/entries.rs`, `app.rs`, `main.rs`):** 12 synchronous `println!`/`eprintln!` calls in `llm.rs` (including full prompt previews & raw LLM outputs) plus `eprintln!` in `error.rs` and `entries.rs`.
- **Full `content` HTML in GraphQL & `MinifluxCache` (`routes/pages.rs`, `routes/entries.rs`, `cache.rs`):** Authenticated list queries fetch full `content` HTML only to regex-match the first image URL, then store full `serde_json::Value` response trees (up to 1000 entries) in `MinifluxCache`; detail cache (`entry:{id}`) also stores raw `content` HTML even though `get_entry_detail_htmx` only extracts image URLs and never renders raw `content`.
