# Analysis — my-ui wrapper

**Result:** done

## Context

A stateless Rust web server wrapping Miniflux behind a Pocket-ID OIDC auth gate, surfacing entries through a GraphQL API layer consumed by HTMX-rendered spatial glass UI. No database — all state lives in Miniflux. The repo uses Cargo workspaces for extensibility to future upstream services.

## Architecture summary

```
Browser (HTMX)
  ↕ HTML fragments + hx-* attributes
Rust server (Axum)
  ├─ OIDC middleware (Pocket-ID) ← session cookie / token
  ├─ GraphQL endpoint (/graphql) ← async-graphql
  │   └─ Resolvers → Miniflux REST client (reqwest)
  ├─ Template engine → HTML responses (Askama / Tera / Maud)
  └─ Static assets (CSS extracted from reference files)
```

## Key design decisions to resolve in Design phase

1. **Template engine** — Askama (compile-time, type-safe) vs Tera (runtime, familiar Jinja2) vs Maud (Rust macro DSL)
2. **Session storage** — in-memory (simple, lost on restart) vs signed cookie (stateless, fits no-DB constraint) vs Redis (external, overkill)
3. **HTMX + GraphQL bridge** — GraphQL normally returns JSON; HTMX expects HTML fragments. Options: (a) server-side resolvers return HTML strings, (b) separate REST-style template routes that internally query GraphQL, (c) thin JS layer that calls GraphQL and swaps DOM
4. **Monorepo crate layout** — how to split crates for extensibility

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **HTMX + GraphQL friction** — HTMX wants HTML, GraphQL returns JSON | High | Use template routes that call GraphQL resolvers internally (Option b above) — GraphQL serves as internal API boundary, templates consume it server-side |
| **OIDC token refresh** — Pocket-ID tokens expire, need silent refresh | Medium | Use `openidconnect` crate with refresh token flow; store tokens in signed cookie |
| **Miniflux API key security** — `.env` only, single API key shared for all users | Medium | Acceptable for single-instance deployment; document clearly that all OIDC-authenticated users share one Miniflux identity |
| **Article content sanitization** — Miniflux entry `content` is raw HTML | Medium | Use `ammonia` crate for HTML sanitization before rendering in templates |
| **AI summary card** — reference UI shows AI summary, but no AI service defined | Low | Placeholder for now; stub with entry summary from Miniflux or reading_time metadata |

## Open questions for Grill

1. **HTMX-GraphQL bridge pattern** — which of the three options (a/b/c) fits best for a no-SPA HTMX app?
2. **Template engine choice** — compile-time safety vs runtime flexibility?
3. **Session strategy** — signed cookie is simplest for no-DB, but is it sufficient for OIDC token storage?
4. **AI summary** — the reference UI has an "AI Executive Brief" card — is this in scope for v1 or deferred?
5. **Crate boundary** — should GraphQL schema types live in a shared crate, or per-service crate?
