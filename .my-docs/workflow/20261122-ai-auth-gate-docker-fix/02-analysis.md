# Analysis

**Result:** done

## Context

The `my-ui` service is an Axum-based web reader for Miniflux RSS feeds. It provides an optional AI-driven summary and digest feature powered by an external LLM CLI tool and a local SQLite cache.

### Current AI Gating & Auth Logic

1. **Configuration (`crates/server/src/config.rs`)**:
   - `auth_enabled` determines whether OIDC authentication and session cookies are required.
   - `ai_enabled` currently parses `AI_ENABLED` independently of `auth_enabled`.
   - **Flaw**: When `AUTH_ENABLED=false` and `AI_ENABLED=true`, AI features remain enabled in guest mode without user authentication. The design requirement demands that AI features are active **only** when both authentication is enabled (`AUTH_ENABLED=true`) and AI is enabled (`AI_ENABLED=true`).
   - Gating rule: `ai_enabled = auth_enabled && map.get("AI_ENABLED")...`

2. **Routes & Endpoints (`crates/server/src/routes/entries.rs`)**:
   - `/hx/entries?status=ai` (`list_entries_htmx`): Checks `if query.status.as_deref() == Some("ai")`. If `!config.ai_enabled`, it returns `handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()))`.
   - `/hx/entries/{id}/ai-summary` (`get_entry_ai_summary_htmx`): Extracts `AuthenticatedUser`, `AppSchema`, `AiService`, and `id`.
   - **Flaw**: `get_entry_ai_summary_htmx` does **not** extract `Config` and does not check `config.ai_enabled`. A direct request to `/hx/entries/{id}/ai-summary` triggers `ai_service.get_or_create_summary(...)`, invoking the LLM runner or database query even when AI is disabled.

3. **Docker Builder Base Image (`Dockerfile`)**:
   - `Dockerfile` line 6 specifies `FROM rust:1.85-slim-bookworm AS builder`.
   - **Flaw**: Modern workspace dependencies (such as newer Axum, Tower, or synchronization crates like `parking_lot` / `lock_api`) require a newer Rust compiler (rustc >= 1.95 or latest stable). Pinning `rust:1.85-slim-bookworm` causes build failures when compiling on modern developer hosts and target architectures like Raspberry Pi OS (arm64/armv7).

---

## Risks

- **Direct Endpoint Bypass**: Direct requests to `/hx/entries/{id}/ai-summary` consume LLM budget or execute local commands if AI gating is not enforced at the route handler level.
  - *Mitigation*: Inject `State(config): State<Arc<Config>>` and `headers: HeaderMap` into `get_entry_ai_summary_htmx`, returning `handle_htmx_error` with 404 when `!config.ai_enabled`.
- **Inconsistent Auth State**: Enabling AI when auth is off risks leaking AI capabilities to unauthenticated public visitors.
  - *Mitigation*: Enforce `ai_enabled = auth_enabled && AI_ENABLED` inside `Config::from_map`.
- **Docker Toolchain Incompatibility**: A floating or overly specific compiler version could cause build drift or ABI mismatch with the Debian Bookworm runner.
  - *Mitigation*: Use `rust:1-slim-bookworm` to track stable 1.x while locking the Debian Bookworm base and glibc version.

---

## Open questions

All primary questions regarding product behavior and architecture were resolved during ideation framing:
- Should AI ever run without auth? No, AI strictly requires `AUTH_ENABLED=true`.
- What image should Docker use? `rust:1-slim-bookworm` for the builder stage.
