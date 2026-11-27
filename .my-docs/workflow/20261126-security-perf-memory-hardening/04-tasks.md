# Tasks (TDD-ready)

Each task: behavior, failing test name first, files allowed, done evidence.

## Planned tests

| Case | Failing test name | Covers |
|------|-------------------|--------|
| 1 | `test_ai_routes_reject_unauthenticated_when_auth_enabled` | `executive_brief_htmx`, `get_entry_analysis_htmx`, `get_entry_rewrite_htmx` require `AuthenticatedUser` (`AUTH_ENABLED=true` returns `HX-Redirect: /login` / `401`) |
| 2 | `test_oidc_rejects_missing_or_invalid_id_token_and_nonce_mismatch` | OIDC verification rejects bad signature, mismatched `nonce`, wrong `aud`/`iss`, expired `exp`, missing `id_token`, and removed `example.com` bypass |
| 3 | `test_oidc_valid_id_token_and_jwks_key_rotation_refresh` | OIDC callback succeeds with real `wiremock` RS256 ID token + `AuthType::RequestBody`, and refreshes JWKS on unknown `kid` rotation |
| 4 | `test_hkdf_domain_separation_rejects_cross_cookie_swap` | HKDF-SHA256 domain separation (`__session` vs `__oidc_temp`) rejects cross-cookie decryption |
| 5 | `test_trusted_proxies_cidr_and_spoof_prevention` | `TRUSTED_PROXIES` default `127.0.0.1,::1` + CIDR env config trusts `CF-Connecting-IP` only from trusted `ConnectInfo` peers and blocks header spoofing from untrusted/missing peers |
| 6 | `test_rate_limiter_hard_capacity_cap_evicts_oldest` | `RateLimiter` enforces `MAX_RATE_LIMIT_BUCKETS` cap using `std::sync::Mutex` |
| 7 | `test_extractor_blocks_ssrf_private_loopback_and_redirects` | `ArticleExtractor` blocks private/loopback/link-local/CGNAT/ULA IPs and unsafe redirects |
| 8 | `test_extractor_streams_byte_cap_and_early_word_exit` | `ArticleExtractor` caps streamed bytes at 512 KiB and stops DOM traversal at 250 words |
| 9 | `test_security_headers_csp_sri_and_no_hx_vals_eval` | `app.rs` emits CSP (`img-src 'self' https: data:`, no `'unsafe-eval'`), `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`; `base.html` has pinned SRI for `htmx@1.9.12` + `lucide@0.576.0`; `index.html` drops `hx-vals="js:..."` |
| 10 | `test_template_safe_urls_and_no_feed_title_js_breakout` | `entry.url` rejects `javascript:`/`data:`, extracted images enforce `https:`/`data:image/*` (blocking `http:` & SVG), and `entry_list.html` safely passes `feed_title` with single quotes (`O'Reilly's; alert(1)//`) |
| 11 | `test_sanitizer_lazylock_sets_target_blank_without_regex` | `sanitize_html` uses `LazyLock<Builder>` `set_tag_attribute_value` without regex post-processing and keeps `target="_blank" rel="noopener noreferrer"` |
| 12 | `test_graphql_enforces_depth_complexity_and_limit_caps` | `async-graphql` schema rejects depth > 8, complexity > 100, and `EntryFilterInput.limit > 100`, while exposing additive `EntryGql.imageUrl` |
| 13 | `test_executive_brief_notify_wakes_waiter_without_polling` | `InFlightRegistry` (`std::sync::Mutex` + `Notify` + capacity cap) removes keys synchronously on `Drop` and wakes concurrent `executive_brief_htmx` waiters without polling |
| 14 | `test_cache_stores_slim_entries_without_raw_content_html` | Authenticated list + detail routes omit raw `content` HTML from `MinifluxCache` while preserving `image_url`, `hero_image`, `body_images`, and guest `sanitized_content` |

---

## Task Breakdown (TDD Order)

### Task 1: Config `TRUSTED_PROXIES` + HKDF Domain-Separated Cipher KeyRing
- **Behavior:** Parse `TRUSTED_PROXIES` (default `"127.0.0.1,::1"`, IP/CIDR via `ipnet::IpNet`, empty string supported) in `Config`, document in `.env.example`, and derive domain-separated `Aes256Gcm` ciphers (`my-ui:__session:v1` vs `my-ui:__oidc_temp:v1`) via `hkdf::Hkdf<Sha256>` precomputed in `AuthKeyRing` on `AppState`.
- **Failing tests first:** `test_config_trusted_proxies_default_and_cidr_parsing`, `test_hkdf_domain_separation_rejects_cross_cookie_swap` (Case 4).
- **Files allowed:** `crates/server/Cargo.toml`, `crates/server/src/config.rs`, `crates/server/src/auth/crypto.rs`, `crates/server/src/auth/session.rs`, `crates/server/src/auth/temp_cookie.rs`, `crates/server/src/auth/middleware.rs`, `crates/server/src/auth/handlers.rs`, `crates/server/src/app.rs`, `.env.example`.
- **Done evidence:** `cargo test -p server config::tests auth::session::tests auth::temp_cookie::tests` passes; cross-cookie decryption fails.

### Task 2: Route Authentication Guards on AI HTMX Endpoints
- **Behavior:** Require `_user: AuthenticatedUser` in `executive_brief_htmx`, `get_entry_analysis_htmx`, and `get_entry_rewrite_htmx`; update direct handler test calls in `crates/server/src/routes/entries.rs`.
- **Failing tests first:** `test_ai_routes_reject_unauthenticated_when_auth_enabled` (Case 1).
- **Files allowed:** `crates/server/src/routes/entries.rs`.
- **Done evidence:** Unauthenticated requests with `AUTH_ENABLED=true` return `HX-Redirect: /login` (HTMX) or `303` / `401`; guest mode (`AUTH_ENABLED=false`) tests continue passing.

### Task 3: Full Pocket-ID OIDC Verification, Nonce, Shared HTTP Client & `wiremock` Tests
- **Behavior:** Store `nonce` in `OidcTempData`, share one `reqwest::Client` (no redirects) in `OidcService` on `AppState`, cache discovery + JWKS with single-refresh on unknown `kid`, use `AuthType::RequestBody`, verify signature/`iss`/`aud`/`exp`/`nonce` via `id_token.claims(&client.id_token_verifier(), &nonce)`, remove `example.com || cfg!(test)` and `usr_100` fallbacks, and update `auth/mod.rs` tests to `wiremock` + `CoreRsaPrivateSigningKey`.
- **Failing tests first:** `test_oidc_rejects_missing_or_invalid_id_token_and_nonce_mismatch` (Case 2), `test_oidc_valid_id_token_and_jwks_key_rotation_refresh` (Case 3).
- **Files allowed:** `crates/server/src/auth/oidc.rs`, `crates/server/src/auth/temp_cookie.rs`, `crates/server/src/auth/handlers.rs`, `crates/server/src/auth/mod.rs`, `crates/server/src/app.rs`.
- **Done evidence:** `cargo test -p server auth::` passes cleanly without any `example.com` bypass.

### Task 4: Trusted-Proxy Rate Limiting, `ConnectInfo`, `std::sync::Mutex` & Hard Cap
- **Behavior:** Wire `into_make_service_with_connect_info::<SocketAddr>()` in `main.rs`, resolve client IP from `CF-Connecting-IP` -> `X-Forwarded-For` -> `X-Real-IP` only when TCP peer `ConnectInfo` matches `config.trusted_proxies` (fail closed when untrusted or `ConnectInfo` absent), switch `RateLimiter` to `std::sync::Mutex`, and enforce `MAX_RATE_LIMIT_BUCKETS = 10_000`.
- **Failing tests first:** `test_trusted_proxies_cidr_and_spoof_prevention` (Case 5), `test_rate_limiter_hard_capacity_cap_evicts_oldest` (Case 6).
- **Files allowed:** `crates/server/src/rate_limit.rs`, `crates/server/src/app.rs`, `crates/server/src/main.rs`.
- **Done evidence:** `cargo test -p server rate_limit::tests` passes; spoofed forwarding headers from untrusted peers are ignored.

### Task 5: SSRF Guard, DNS Resolver, Safe Redirects & Streaming Bounds in `ArticleExtractor`
- **Behavior:** Block loopback/private/link-local/unspecified/ULA/CGNAT/multicast/IPv4-mapped IPs in URL validation and custom `reqwest::dns::Resolve` (`SsrSafeDnsResolver`), disable automatic redirects and manually validate up to 3 hops, stream chunks with `MAX_HTML_BYTES = 512 * 1024`, stop DOM traversal at 250 words, and add test-only `new_allow_loopback_for_tests()` for `wiremock` HTTP tests.
- **Failing tests first:** `test_extractor_blocks_ssrf_private_loopback_and_redirects` (Case 7), `test_extractor_streams_byte_cap_and_early_word_exit` (Case 8).
- **Files allowed:** `crates/server/Cargo.toml`, `crates/server/src/ai/extractor.rs`.
- **Done evidence:** `cargo test -p server ai::extractor::tests` passes.

### Task 6: Browser Security Headers, SRI, `eval` Removal, URL/Template Hardening & `ammonia` Refactor
- **Behavior:** Add CSP (`img-src 'self' https: data:`, no `'unsafe-eval'`) + `X-Content-Type-Options` + `X-Frame-Options` + `Referrer-Policy` middleware in `app.rs`; pin `htmx.org@1.9.12` and `lucide@0.576.0` with SHA-384 SRI and set `htmx.config.allowEval = false` in `base.html`; move `#mark-page-read-btn` `entry_ids` from `hx-vals="js:..."` to `htmx:configRequest` in `index.html`; fix `feed_title` JS quote-breakout in `entry_list.html` + `index.html`; validate `http`/`https` link URLs and `https`/`data:image/*` (non-SVG) image URLs in `entries.rs`; replace per-call `ammonia::Builder` + regex passes with `LazyLock<Builder>` in `sanitizer.rs`.
- **Failing tests first:** `test_security_headers_csp_sri_and_no_hx_vals_eval` (Case 9), `test_template_safe_urls_and_no_feed_title_js_breakout` (Case 10), `test_sanitizer_lazylock_sets_target_blank_without_regex` (Case 11).
- **Files allowed:** `crates/server/src/app.rs`, `crates/server/src/routes/entries.rs`, `crates/ui-components/src/sanitizer.rs`, `crates/ui-components/src/templates.rs`, `crates/ui-components/templates/base.html`, `crates/ui-components/templates/pages/index.html`, `crates/ui-components/templates/partials/entry_list.html`.
- **Done evidence:** `cargo test -p ui-components` and `cargo test -p server app::tests` pass.

### Task 7: GraphQL Depth, Complexity, Limit Bounds & `EntryGql.imageUrl`
- **Behavior:** Enforce `.limit_depth(8)` and `.limit_complexity(100)` in `build_schema`, validate `EntryFilterInput.limit <= 100` in `build_domain_filter`, and add additive `image_url: Option<String>` (`imageUrl`) on `EntryGql` (sanitized HTTPS/data image URL).
- **Failing tests first:** `test_graphql_enforces_depth_complexity_and_limit_caps` (Case 12).
- **Files allowed:** `crates/miniflux/src/schema/mod.rs`, `crates/miniflux/src/schema/query.rs`, `crates/miniflux/src/schema/types.rs`.
- **Done evidence:** `cargo test -p miniflux-client` passes.

### Task 8: Event-Driven Brief Wait (`InFlightRegistry`), Lean Cache Payloads & `tracing` Migration
- **Behavior:** Replace `AiService::in_flight` + `tokio::spawn` in `Drop` + `40 x 100ms` DB polling with bounded `InFlightRegistry` (`std::sync::Mutex` + `Notify`, cap 512); switch authenticated list & detail queries/caches (`pages.rs`, `entries.rs`, `cache.rs`) to slim typed payloads without raw HTML `content`; replace all 20 `println!`/`eprintln!` sites across `llm.rs`, `error.rs`, `entries.rs`, `app.rs`, and `main.rs` with `tracing` (`debug!` for prompt/raw LLM previews).
- **Failing tests first:** `test_executive_brief_notify_wakes_waiter_without_polling` (Case 13), `test_cache_stores_slim_entries_without_raw_content_html` (Case 14).
- **Files allowed:** `crates/server/src/ai/mod.rs`, `crates/server/src/ai/prefetch.rs`, `crates/server/src/ai/llm.rs`, `crates/server/src/cache.rs`, `crates/server/src/error.rs`, `crates/server/src/routes/pages.rs`, `crates/server/src/routes/entries.rs`, `crates/server/src/app.rs`, `crates/server/src/main.rs`.
- **Done evidence:** Full workspace `cargo test` passes with zero `println!`/`eprintln!` in non-test production code.
