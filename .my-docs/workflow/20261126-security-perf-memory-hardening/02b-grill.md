# Grill log

**Result:** frontier-empty

**Grill open** = ¬Notes `grill-skip` ∧ (no `02b` ∨ Result ≠ `frontier-empty`) (incl. template **`pending`** alone — does not close). **Closed** on Notes `grill-skip` (parent after Parent-relay **or** labeled solo after AskQuestion) **or** Result `frontier-empty` only — Result `skipped` does **not** close — [../grill.md](../grill.md) · [../derive-next.md](../derive-next.md) · [../solo-entry.md](../solo-entry.md) · [../hitl.md](../hitl.md).

## Round log

### Round 1

❓ **Q1** - **Trusted Proxy Configuration for Cloudflare Tunnel**: How should `rate_limit_middleware` identify real visitor IPs behind `cloudflared`? If `my-ui` runs natively on Raspberry Pi OS, `cloudflared` connects from `127.0.0.1` / `::1`; if `my-ui` runs in Docker Compose (`docker-compose.yml`), `cloudflared` connects from a Docker bridge IP (e.g., `172.17.0.1`). Should we default `TRUSTED_PROXIES` to `127.0.0.1,::1`, allow overriding/extending via an explicit `TRUSTED_PROXIES` env var (comma-separated IPs/CIDRs, e.g. adding your Docker bridge subnet), read `CF-Connecting-IP` first (falling back to `X-Forwarded-For` / `X-Real-IP`) only when the TCP peer matches `TRUSTED_PROXIES`, and otherwise rate-limit by the direct peer IP? (Alternatively: default `TRUSTED_PROXIES` to empty/disabled unless explicitly set, or let us know if `cloudflared` hits Docker over a bridge CIDR so we document that in `.env.example`.)

➡️ Yes — default `TRUSTED_PROXIES` to `127.0.0.1,::1`, add optional `TRUSTED_PROXIES` IP/CIDR env config, and trust `CF-Connecting-IP` (then `X-Forwarded-For`/`X-Real-IP`) only from trusted peers.

**Answer (user 2026-11-26):** Confirmed — default `TRUSTED_PROXIES=127.0.0.1,::1`, allow IP/CIDR override via env, and trust `CF-Connecting-IP` (then `X-Forwarded-For`/`X-Real-IP`) only from trusted peers.

---

❓ **Q2** - **Tailwind Play CDN, SRI Coverage, and `eval` in CSP**: `https://cdn.tailwindcss.com` (Play CDN) serves dynamic/unpinned JavaScript, so browsers cannot verify a Subresource Integrity (`integrity=...`) hash for it (unlike `htmx.org@1.9.12` and a pinned `lucide` version). Also, `index.html` line 51 uses `hx-vals="js:{entry_ids: getVisibleEntryIds()}"` which calls `new Function()` (`eval`), and Tailwind Play CDN can require `'unsafe-eval'` in some setups. Should we keep Tailwind Play CDN (without SRI, keeping `'unsafe-inline'` as requested and adding `'unsafe-eval'` only if required by Play CDN), remove `hx-vals="js:..."` via an `htmx:configRequest` listener, and pin + add SHA-384 SRI to `htmx.org@1.9.12` and `lucide` (or vendor `htmx`/`lucide` locally into the server)? (Alternatively: replace Play CDN with a compiled local Tailwind CSS file so we can drop Play CDN and `'unsafe-eval'` completely.)

➡️ Yes — keep Tailwind Play CDN without SRI, refactor `hx-vals="js:..."` out of `index.html`, and add pinned SHA-384 SRI (or local vendoring if offline-preferred) for `htmx` and `lucide` under our softer CSP.

**Answer (user 2026-11-26):** Confirmed — keep Tailwind Play CDN (no SRI), refactor `hx-vals="js:..."` out of `index.html`, and pin + add SHA-384 SRI to `htmx@1.9.12` and `lucide`.

---

❓ **Q3** - **RSS Image Origins (`img-src`) in Content-Security-Policy**: Many RSS feeds embed external article images (`hero_image` and `body_images` in `entry_detail.html` / `entry_list.html`). Should the CSP `img-src` directive allow `'self' https: data:` (blocking cleartext `http://` image requests under your HTTPS Cloudflare Tunnel), or allow `'self' https: http: data:` so legacy `http://` feed thumbnails still load?

➡️ Yes — restrict `img-src` to `'self' https: data:` (block cleartext `http:` images while still permitting `http://` and `https://` external article links in `href`).

**Answer (user 2026-11-26):** Confirmed — restrict `img-src` to `'self' https: data:` (block cleartext `http:` images; still allow `http`/`https` in external `href` links).

---

### Frontier Status

All remaining design decisions (route guards, `openidconnect` `CoreIdTokenVerifier` + `AuthType::RequestBody` + `nonce` + JWKS rotation cache, HKDF domain separation, SSRF IP/DNS + streaming bounds, `ammonia` attribute configuration, `entry_list.html` JS quote-breakout removal, GraphQL depth/complexity/limit caps, `InFlightRegistry` with `Notify`, slim `MinifluxCache` structs, and `tracing` logging) follow directly from codebase evidence, primary crate documentation, and the confirmed Round 1 decisions. **Frontier empty.**
