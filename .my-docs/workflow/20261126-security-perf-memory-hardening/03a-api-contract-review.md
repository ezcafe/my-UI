# API contract review

**Result:** needs update

## Key Takeaways

The design (`03-design.md`) and task plan (`04-tasks.md`) cannot move forward to `design-review` or `build` yet because **5 Major API and cross-crate contract gaps** would break compilation, fail type-checking against locked crate versions, or leave runtime races/errors open:

1. **`Config` & `AuthKeyRing` breaking changes vs. Task 1 scope:** Adding `trusted_proxies` to `Config` breaks 34 existing `Config { ... }` test struct literals across 6 files, and adding `AuthKeyRing` to `AuthenticatedUser` breaks `auth/mod.rs:285` (`FromRequestParts` tested against `&Arc<Config>`).
2. **`openidconnect 3.5.0` vs. `reqwest 0.12` adapter mismatch:** The design assumes `openidconnect 4.x` `AsyncHttpClient` `&http_adapter` borrowing and direct `HttpRequest` ↔ `reqwest::Request` conversion, whereas locked `openidconnect 3.5.0` (`oauth2 4.4.2`) uses `http 0.2.12` (incompatible with `reqwest 0.12.28` `http 1.5.0`) and takes `FnOnce` closures by value; OIDC error HTTP status mapping and `login_handler` discovery behavior are also unspecified.
3. **Cross-crate ownership & unclamped `/hx/entries` `limit`:** `EntryGql.imageUrl` (`crates/miniflux`) and `sanitize_image_url` (`crates/server`) lack a shared crate helper, `render_entries_list_html` passes `query.limit` unclamped into GraphQL (turning `?limit=200` into a 500 error and `?limit=0` into division by zero), and 2 of 4 `EntryViewModel` URL builders are missed.
4. **`rate_limit_middleware` state wiring & dual-stack IPv4-mapped IPv6:** `app.rs` passes `Arc<RateLimiter>` without `Config`, and `IpNet::contains` misses `::ffff:<ipv4>` peer addresses on dual-stack Linux sockets without `.to_canonical()`.
5. **`InFlightRegistry` missed-wakeup/leader-failure hang & `CachedEntryDetail` field contract:** `Notify::notified()` without `.enable()` and a completion flag risks a 10-second stall when a leader fails before the follower awaits; `CachedEntryDetail` must strip HTML from fallback `teaser` and preserve status/bookmark metadata.

---

## Findings

### Major

#### 1. `Config` struct update and `AuthKeyRing` extractor break existing callers & Task 1 `Files allowed`
- **Problem:**
  - `03-design.md` (§4) adds `pub trusted_proxies: Vec<ipnet::IpNet>` to `Config`. The workspace currently constructs `Config { ... }` with explicit field literals (no `Default` impl) in **34 test locations across 6 files**: `crates/server/src/app.rs` (3), `crates/server/src/rate_limit.rs` (1), `crates/server/src/auth/mod.rs` (1), `crates/server/src/routes/entries.rs` (16), `crates/server/src/routes/mod.rs` (5), and `crates/server/tests/e2e_tests.rs` (8).
  - `03-design.md` (§3) states `AuthenticatedUser` extracts precomputed `AuthKeyRing` from `AppState`. Adding `Arc<AuthKeyRing>: FromRef<S>` to `impl<S> FromRequestParts<S> for AuthenticatedUser` (`crates/server/src/auth/middleware.rs:40-44`) will break `test_authenticated_user_bypasses_when_auth_disabled` (`crates/server/src/auth/mod.rs:285`), which passes `&config` (`&Arc<Config>`) directly instead of `AppState`.
  - `04-tasks.md` Task 1 sets `Files allowed` to `config.rs`, `auth/crypto.rs`, `session.rs`, `temp_cookie.rs`, `middleware.rs`, `handlers.rs`, `app.rs`, `.env.example`, and `Cargo.toml`—omitting `rate_limit.rs`, `auth/mod.rs`, `routes/entries.rs`, `routes/mod.rs`, and `tests/e2e_tests.rs`. Under those file limits, `cargo test -p server` will not compile at Task 1.
- **Required update:**
  - Either store `AuthKeyRing` inside `Config` (or provide a test helper / `#[cfg(test)]` `Config::test_default(...)` or `impl Default for Config`) and explicitly include all affected test files in Task 1 `Files allowed`, plus update `auth/mod.rs:285` to pass a state providing both `Arc<Config>` and `Arc<AuthKeyRing>` (e.g., `AppState` or `Config`).

#### 2. `openidconnect 3.5.0` vs. `reqwest 0.12` HTTP adapter, `AsyncHttpClient` signature, and OIDC error/login contracts
- **Problem:**
  - Locked `Cargo.lock` pins `openidconnect = "3.5.0"` and `oauth2 = "4.4.2"` (using `http = "0.2.12"`), alongside `reqwest = "0.12.28"` (using `http = "1.5.0"`).
  - `03-design.md` (§2) specifies converting `openidconnect::HttpRequest` ↔ `reqwest::Request` and calling `.request_async(&http_adapter).await`. In `openidconnect 3.5.0` / `oauth2 4.4.2`:
    1. `openidconnect::HttpRequest` and `HttpResponse` use `http 0.2` `Method`, `HeaderMap`, and `StatusCode`, which are incompatible Rust types with `reqwest 0.12` (`http 1.5`). The adapter must explicitly translate method names, header names/values as `&str`/`&[u8]`, URL strings, and numeric status codes (or pass `openidconnect::reqwest::async_http_client`—though building an explicit `reqwest 0.12` closure avoids `reqwest 0.11` pool fragmentation).
    2. `AsyncHttpClient` in `oauth2 4.4.2` is implemented for `FnOnce(HttpRequest) -> Pin<Box<dyn Future<...>>>` consumed **by value** (not `AsyncHttpClient<'c>` borrowed via `&http_adapter`, which was introduced in `openidconnect 4.x` / `oauth2 5.x`). Passing `&http_adapter` to `discover_async` or `exchange_code(...).request_async(...)` will fail trait bounds unless the closure itself is passed by value (`|req| oidc_service.execute_http(req)`).
  - `03-design.md` (§2) also leaves three contract details unspecified:
    1. Whether `login_handler` / `generate_auth_request` (`crates/server/src/auth/oidc.rs:54`, `handlers.rs:24`) continues building the `/authorize` URL synchronously or switches to async `OidcService` discovery (and whether `discover_endpoints` / `generate_auth_request` public signatures change).
    2. What HTTP status code `callback_handler` (`crates/server/src/auth/handlers.rs:89-96`) returns when ID token verification fails (`missing id_token`, bad signature, wrong `nonce`/`iss`/`aud`, expired `exp`)—currently all `exchange_token` errors return `500 Internal Server Error`, whereas `401 Unauthorized` or `400 Bad Request` may be expected by Task 3 / Case 2 tests.
    3. `OidcTempData` literals in `crates/server/src/auth/mod.rs:97, 123, 245` (including `test_oidc_callback_state_mismatch_fails`) and `crates/server/src/auth/temp_cookie.rs:46` all need the new `nonce` field.
- **Required update:**
  - Specify the exact `http 0.2` ↔ `http 1.5` (`reqwest 0.12`) bridging and by-value closure call for `openidconnect 3.5.0` (`discover_async` and `request_async`).
  - Clarify `login_handler` / `generate_auth_request` behavior, exact `callback_handler` failure status codes (`401`/`502` vs `500`), and `nonce` updates across all 4 `OidcTempData` test literals.

#### 3. Cross-crate ownership for `EntryGql.imageUrl` vs. `sanitize_image_url`, unclamped `/hx/entries` `limit`, and missed `entry.url` builders
- **Problem:**
  - `03-design.md` (§7) adds `EntryGql.imageUrl` in `crates/miniflux/src/schema/{types.rs, query.rs}` using "first safe image extraction", while `03-design.md` (§6) places `sanitize_image_url` (`https://` and non-SVG `data:image/(png|jpeg|jpg|gif|webp);base64,`, blocking `http://`, `data:image/svg+xml`, `//`) in `crates/server/src/routes/entries.rs` / `ui-components`. `crates/miniflux` does **not** depend on `server` or `ui-components` (and `crates/miniflux/Cargo.toml` has no `regex` dependency), and `04-tasks.md` Task 6 vs. Task 7 have disjoint `Files allowed`. Leaving helper ownership unspecified risks divergent filtering between authenticated (`EntryGql.imageUrl`) and guest (`extract_first_image_url`) paths or duplicated logic.
  - `crates/server/src/routes/entries.rs:257` (`render_entries_list_html`) reads `let limit_val = query.limit.unwrap_or(50);` **without clamping**, unlike `executive_brief_htmx` (`entries.rs:698`, `.clamp(1, 100)`). Once `EntryFilterInput.limit > 100` is rejected in `crates/miniflux/src/schema/query.rs`, any request to `GET /hx/entries?limit=200` will fail with HTTP 500/upstream error, and `?limit=0` causes a division-by-zero panic at `entries.rs:391,393` (`offset / limit`).
  - `03-design.md` (§6) specifies `sanitize_http_url` on `entry.url` in `entries.rs`, but `crates/server/src/routes/entries.rs` constructs `EntryViewModel.url` in **4 places**: `entry_view_model_from_json` (`line 231`), `render_ai_entries_list` (`line 583`, feeding `ai_grouped_list.html:12`), `get_entry_detail_htmx` (`line 1446`, feeding `entry_detail.html:14,58`), and `mark_entry_read_htmx` (`line 1956`, feeding `entry_item.html:14`). Also, `index.html:1288-1299` (`safeHref`) checks lowercase `http://`/`https://` only, whereas Rust `sanitize_http_url` is case-insensitive.
- **Required update:**
  - Define one shared canonical location for `sanitize_http_url`, `sanitize_image_url`, and `extract_first_safe_image_url` (e.g., in `miniflux-client`, which `server` already imports, with consistent tracking-pixel + scheme checks), update `entry_view_model_from_json` to prefer `item.get("imageUrl")` before HTML fallback, and apply `sanitize_http_url` at all 4 `EntryViewModel` builders (plus case-insensitive scheme check in `safeHref`).
  - Clamp `limit_val` (`clamp(1, 100)`) and `offset_val` (`max(0)`) in `render_entries_list_html` (`entries.rs:257-258`).

#### 4. `rate_limit_middleware` State wiring and dual-stack IPv4-mapped IPv6 `ConnectInfo` matching
- **Problem:**
  - `crates/server/src/app.rs:114` registers `.layer(from_fn_with_state(rate_limiter, rate_limit_middleware))`, where `rate_limit_middleware` (`rate_limit.rs:82`) extracts only `State<Arc<RateLimiter>>`. `03-design.md` (§4) reads `config.trusted_proxies` inside `rate_limit_middleware`, without specifying whether `rate_limit_middleware` switches to `State(state): State<AppState>` (changing `app.rs:114`) or `RateLimiter` owns `trusted_proxies: Vec<IpNet>` (changing `RateLimiter::new` / `app.rs:78`). Notice `04-tasks.md` Task 4 lists `app.rs`, so the exact state contract must be stated so tests constructing `RateLimiter` directly stay aligned.
  - On dual-stack Linux / Raspberry Pi OS sockets (`HOST=::` or `0.0.0.0`), `ConnectInfo<SocketAddr>` can report an IPv4 proxy peer (like `127.0.0.1` or `172.17.0.1`) as an IPv4-mapped IPv6 address (`::ffff:127.0.0.1`). `ipnet::IpNet::contains(&peer_ip)` returns `false` when matching `127.0.0.1/32` (`V4`) against `::ffff:127.0.0.1` (`V6`) unless `peer_ip` is first normalized via `peer_ip.to_canonical()`, which would silently break `TRUSTED_PROXIES` behind `cloudflared`.
- **Required update:**
  - Specify whether `trusted_proxies` is passed to `rate_limit_middleware` via `AppState` or stored inside `RateLimiter`, and require `peer_ip.to_canonical()` before checking `trusted_proxies.iter().any(|net| net.contains(&canonical_peer))`.

#### 5. `InFlightRegistry` missed-wakeup / leader-failure stall race & `CachedEntryDetail` field contract
- **Problem:**
  - `03-design.md` (§8) proposes `executive_brief_htmx` await `let notified = notify.notified();` (created before re-checking `ai_service.db.get_page_brief_by_key(&page_key)`) with a 10s timeout. Two concurrency gaps exist in `tokio::sync::Notify`:
    1. `Notify::notify_waiters()` does **not** store a permit; it only wakes futures that are already created/enabled. If the leader finishes **with an error** (DB remains `None`) and drops `InFlightGuard` *between* `try_acquire` returning `Follower(notify)` and the follower reaching `notify.notified()`, `get_page_brief_by_key` returns `None` and the follower stalls for the full **10 seconds** waiting for a `notify_waiters()` that already happened.
    2. In Tokio, calling `notify.notified()` alone does not register the future in the waiter queue until polled or explicitly enabled via `tokio::pin!(notified); notified.as_mut().enable();` before the re-check.
  - For `MinifluxCache` (`cache.rs`, `pages.rs`, `entries.rs`):
    1. `03-design.md` (§8) states `get_entry_detail_htmx` extracts `(hero_image, body_images)` and `teaser` once on cache miss and stores `CachedEntryDetail`. When GraphQL `summary` is empty, `entries.rs:1366-1370` currently falls back to raw HTML `content` for `teaser_str`; `CachedEntryDetail` must explicitly store stripped text (`strip_html_tags`) for `teaser` rather than caching raw HTML `content`, and must include `status`, `is_bookmarked`, `author`, `reading_time`, `url`, `published_at`, `feed_title`, `title`, and `summary` to preserve `entries.rs:1330-1470`.
    2. `index_page` (`pages.rs`) and `render_entries_list_html` (`entries.rs`) in **guest mode** (`auth_enabled = false`) must still select `content` on cache miss to populate `EntryViewModel.sanitized_content` (required by `#entries-guest-data` in `entry_list.html:240`), then discard raw `content` before inserting slim `EntryViewModel`s into `CachedEntriesPage`.
- **Required update:**
  - Track a completion state on the in-flight entry (or re-check whether `key` is still active in `InFlightRegistry` after `tokio::pin!(notified); notified.as_mut().enable();`), returning immediately if the leader already finished without writing a brief; also explicitly specify `CapacityFull` handling in `executive_brief_htmx` (`render_brief_error`) and `run_prefetch` (early return).
  - Document the exact `CachedEntryDetail` and `CachedEntriesPage` field schemas (including `strip_html_tags` on detail `teaser` fallback and guest-mode `content` fetch-then-discard).

---

### Minor

1. **Upfront `Content-Length > 512 KiB` rejection vs. capped read (`crates/server/src/ai/extractor.rs`, `03-design.md` §5):**
   `03-design.md` leaves an ambiguous "or" (`rejecting upfront if Content-Length > MAX_HTML_BYTES or passing capped buffer to parse_html`). Prefer streaming up to `MAX_HTML_BYTES = 512 * 1024` and decoding via `String::from_utf8_lossy(&buf)` instead of rejecting upfront, because modern news pages with large inline scripts/JSON hydration often exceed 512 KiB before `</head>` or after `<article>`.
2. **`SsrSafeDnsResolver` `lookup_host` port tuple (`crates/server/src/ai/extractor.rs`, `03-design.md` §5):**
   `reqwest::dns::Resolve::resolve` receives a hostname without a port (`name.as_str()`). Call `tokio::net::lookup_host((name.as_str(), 0)).await` (not bare `name.as_str()`) so `ToSocketAddrs` resolves cleanly.
3. **`RateLimiter` sync signature & mutex poisoning (`crates/server/src/rate_limit.rs`):**
   Clarify whether `RateLimiter::check` and `bucket_count` become synchronous `fn` (updating `.await` call sites in `rate_limit.rs:147, 153, 202, 209, 237`) or stay `async fn` wrapping `std::sync::Mutex`, and use `.unwrap_or_else(|e| e.into_inner())` on lock acquisition.
4. **`ammonia::Builder` `&mut self` chaining (`crates/ui-components/src/sanitizer.rs`):**
   `ammonia 4.2.5` `Builder` methods return `&mut Self`, so `LazyLock::new` must bind `let mut builder = Builder::default();` before configuring and returning `builder`.

---

## Verifiable References

- Workspace files inspected:
  - `crates/server/src/config.rs`, `crates/server/src/app.rs`, `crates/server/src/rate_limit.rs`, `crates/server/src/cache.rs`, `crates/server/src/main.rs`
  - `crates/server/src/auth/{oidc.rs,crypto.rs,temp_cookie.rs,session.rs,handlers.rs,middleware.rs,mod.rs}`
  - `crates/server/src/ai/{mod.rs,extractor.rs,prefetch.rs,llm.rs}`
  - `crates/server/src/routes/{pages.rs,entries.rs,mod.rs}` and `crates/server/tests/e2e_tests.rs`
  - `crates/miniflux/src/schema/{mod.rs,query.rs,types.rs}` and `crates/miniflux/src/summary.rs`
  - `crates/ui-components/src/{sanitizer.rs,templates.rs}` and `crates/ui-components/templates/{base.html,pages/index.html,partials/entry_list.html,partials/entry_detail.html,partials/entry_item.html,partials/ai_grouped_list.html}`
  - `Cargo.lock` (`openidconnect 3.5.0`, `oauth2 4.4.2`, `http 0.2.12` & `1.5.0`, `reqwest 0.12.28`, `ammonia 4.2.5`, `async-graphql 7.2.1`, `axum 0.8.9`, `hkdf 0.12.4`, `ipnet 2.12.2`, `url 2.5.8`)
- Primary external sources:
  - `openidconnect 3.5.0` & `oauth2 4.4.2` (`HttpRequest`, `HttpResponse`, `AsyncHttpClient`, `CoreClient::from_provider_metadata`, `CoreIdTokenVerifier`, `AuthType::RequestBody`): <https://docs.rs/openidconnect/3.5.0/openidconnect/> · <https://docs.rs/oauth2/4.4.2/oauth2/>
  - Pocket-ID OIDC endpoints & `client_secret_post`: <https://pocket-id.org> · <https://github.com/pocket-id/pocket-id>
  - `reqwest 0.12` (`dns::Resolve`, `Response::chunk`, `redirect::Policy::none`): <https://docs.rs/reqwest/0.12/reqwest/>
  - `tokio::sync::Notify` (`enable`, `notify_waiters`): <https://docs.rs/tokio>
  - `ammonia 4.2.5` (`Builder::set_tag_attribute_value`, `link_rel`, `url_schemes`): <https://docs.rs/ammonia/4.2.5/ammonia/struct.Builder.html>
  - `async-graphql 7.2` (`SchemaBuilder::limit_depth`, `limit_complexity`): <https://docs.rs/async-graphql/7.2.1/async_graphql/struct.SchemaBuilder.html>
  - `axum 0.8` (`ConnectInfo`, `into_make_service_with_connect_info`): <https://docs.rs/axum/0.8/axum/extract/struct.ConnectInfo.html>
  - `htmx 1.9.12` SRI, `allowEval`, `htmx:configRequest`: <https://htmx.org/docs/#installing> · <https://htmx.org/docs/#config>
  - `lucide@0.576.0` UMD bundle & Tailwind Play CDN v3 CSP: <https://lucide.dev> · <https://tailwindcss.com>
