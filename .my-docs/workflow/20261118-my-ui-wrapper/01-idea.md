# Idea framing — my-ui wrapper

## Goals

- Build a Rust monorepo that wraps open-source self-hosted projects with a custom spatial glassmorphism UI
- First integration: **Miniflux RSS reader** (via its REST API, learned from OpenAPI spec)
- Provide a **GraphQL** API layer that proxies to upstream REST APIs
- Server-rendered frontend using **HTMX** + Rust templates — no SPA
- **Pocket-ID (OIDC)** authentication gates access to the app
- Miniflux URL + API key configured via server-side `.env` — one deployment = one Miniflux instance
- UI follows the **AR spatial glassmorphism** design from reference files (dark glass panels, blur, indigo/purple accents, 720px spatial windows)

## Non-goals

- Not a re-implementation of Miniflux — just a UI skin
- No owned database — pure stateless proxy with auth gate
- No per-user Miniflux config — single instance per deployment
- No mobile-native app — web only (but spatial/AR aesthetic)

## Anti-goals

- No client-side JS framework (React, Vue, etc.) — HTMX only
- No REST API — GraphQL only for the frontend-to-backend interface
- Never store Miniflux API keys in the database — `.env` only

## Constraints

- Rust for backend (Axum or Actix-web likely)
- HTMX for interactivity
- async-graphql for the GraphQL layer
- Pocket-ID OIDC for auth (https://pocket-id.org/docs/api/endpoints)
- Miniflux REST API v2.2.0 (from OpenAPI spec)
- Monorepo with Cargo workspaces
- Design system: dark glassmorphism from ar-list.html / ar-detail.html reference files

## Open-source wrapper pattern

The repo is designed to be **extensible** — Miniflux is the first service, but the architecture should support adding more upstream services in the future (each with its own GraphQL resolvers and HTMX templates).
