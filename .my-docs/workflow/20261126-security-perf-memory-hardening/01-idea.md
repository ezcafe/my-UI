# Idea & Framing — Security, Performance, and Memory Hardening

- **Size:** Project with written design (Mode `full`, Review profile `full`)
- **Has UI (tentative):** `yes` (also tentative: `Has API: yes`, `Has DB: no`, `Has auth/PII/schema: yes`)
- **Status:** Open — awaiting user confirmation or `"just start"` skip

## Key Takeaways

We will harden `crates/server`, `crates/miniflux`, and `crates/ui-components` across security, speed, and memory without adding new user features or changing database schemas. Every fix stays inside the existing Axum, Askama, `async-graphql`, `openidconnect`, and `ammonia` stack and will be backed by automated tests.

**One-line approach:** Harden authentication, network boundaries, HTTP headers, HTML sanitization, caching, and async concurrency primitives in place across the three Rust workspace crates.

## Goals

1. **Security hardening**
   - **Route authentication:** Require `AuthenticatedUser` on `executive_brief_htmx`, `get_entry_analysis_htmx`, and `get_entry_rewrite_htmx` in `crates/server/src/routes/entries.rs`.
   - **OIDC verification:** Verify ID token cryptographic signatures, standard claims (`iss`, `aud`, `exp`), and `nonce` in `crates/server/src/auth/oidc.rs` (stop discarding `_nonce` and remove the `example.com` dummy session fallback outside tests).
   - **SSRF & read bounds:** Protect `ArticleExtractor::extract_content` in `crates/server/src/ai/extractor.rs` by blocking private, loopback, link-local, and unspecified IPs, blocking or validating redirects safely, and capping downloaded response bytes.
   - **Rate-limiter integrity:** Prevent `X-Forwarded-For` / `X-Real-IP` header spoofing in `rate_limit_middleware` (`crates/server/src/rate_limit.rs`), using socket peer `ConnectInfo` unless an explicit trusted proxy is configured.
   - **Browser & transport headers:** Add security response headers (CSP, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`) in `crates/server/src/app.rs` and Subresource Integrity (SRI) hashes on pinned CDN scripts in `crates/ui-components/templates/base.html`.
   - **Safe URL rendering:** Validate `entry.url` schemes (`http://` and `https://` only) in `entry_view_model_from_json` / templates (`entry_detail.html`, `entry_list.html`, `entry_item.html`) so `javascript:` or `data:` links cannot render.
   - **GraphQL bounds:** Enforce `limit_depth` and `limit_complexity` in `crates/miniflux/src/schema/mod.rs`.
   - **Key separation:** Use HKDF-SHA256 with distinct `info` domain labels (`__session` vs `__oidc_temp`) in `crates/server/src/auth/crypto.rs`.
   - **HTML sanitizer safety:** Replace post-sanitization regex HTML rewriting (`RE_ANCHOR` / `RE_TARGET`) in `crates/ui-components/src/sanitizer.rs` with an `ammonia::Builder` `attribute_filter`.

2. **Performance improvements**
   - **Shared HTTP client:** Reuse a shared `reqwest::Client` for OIDC requests via `AppState` instead of creating clients per call in `crates/server/src/auth/oidc.rs`.
   - **Startup pre-computation:** Pre-derive domain-separated AES-GCM ciphers/keys at startup (`crypto.rs`) and store the configured `ammonia::Builder` in a `LazyLock` (`sanitizer.rs`).
   - **Event-driven brief wait:** Replace the 100ms DB polling loop in `executive_brief_htmx` (`entries.rs`) with `tokio::sync::Notify` (or channel notification) for in-flight page brief completion.
   - **Lock & task cleanup:** Remove `tokio::spawn` from `InFlightGuard::drop` (`entries.rs`, `prefetch.rs`) and replace `tokio::sync::Mutex` in `RateLimiter` (`rate_limit.rs`) with faster non-blocking synchronization (`std::sync::Mutex` or sharded/concurrent map).
   - **Structured logging:** Replace synchronous `println!` / `eprintln!` calls on request and LLM hot paths (`llm.rs`, `error.rs`, `entries.rs`) with `tracing`.

3. **Memory improvements**
   - **Streaming & early exit:** Stream-cap remote HTML downloads and stop DOM traversal once 250 words are collected in `ArticleExtractor` (`extractor.rs`).
   - **Lean GraphQL & cache payloads:** Skip fetching and caching full `content` HTML in authenticated GraphQL list queries (`pages.rs`, `entries.rs`) and store slimmed structs/payloads in `MinifluxCache` (`cache.rs`) instead of cloning raw `serde_json::Value` trees.
   - **Hard capacity caps:** Enforce hard maximum entry counts on `RateLimiter` buckets (`rate_limit.rs`) and `AiService::in_flight` (`ai/mod.rs`).

## Non-Goals

- Adding new user-facing features or redesigning page layouts.
- Changing PostgreSQL database schemas or upstream Miniflux endpoints.
- Building a custom OIDC provider or external Redis rate-limit store.

## Anti-Goals

- Do not silently authenticate unverified users via `example.com` or default user fallbacks in production.
- Do not break `AUTH_ENABLED=false` guest mode, HTMX swaps, or existing AI summary caching.

## Constraints & Marked Guesses

- **Existing cookie invalidation (guess):** Adding HKDF domain separation in `crypto.rs` will invalidate existing `__session` and `__oidc_temp` cookies once upon deployment, requiring active users to sign in again.
- **Reverse proxy & CSP interaction (guess):** `base.html` currently loads `https://cdn.tailwindcss.com`, `unpkg.com/lucide@latest`, and inline `<script>` / `onclick` handlers; `rate_limit.rs` trusts `X-Forwarded-For`. How strict CSP can be and how client IPs should be read depend on your real production setup.
- **OIDC provider metadata (guess):** Production OIDC (Pocket-ID) serves standard JWKS / `.well-known/openid-configuration` endpoints, while automated tests can use local `wiremock` / test keys without needing the `example.com` bypass.

## Open Question for User

You can reply **"just start"** to skip ideation and proceed straight to analysis and design with safe production defaults.

Otherwise, **one question before we lock the scope:**

> **Can you walk me through how you deploy and run this app in production**—specifically whether traffic arrives through a reverse proxy (and which header it sets), which OIDC provider issues your ID tokens, and whether you want to keep Tailwind Play CDN/inline scripts (softer CSP) or refactor/self-host them for a strict CSP?
