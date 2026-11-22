# Ideation Framing: AI Feature Gate & Docker Build Fix

## Goals
- **Gate AI behind authentication and AI toggle**: Ensure AI features are active only when both authentication is enabled (`AUTH_ENABLED=true`) and AI is enabled (`AI_ENABLED=true`). If either is off, AI features (UI digest filter, AI summaries, and endpoints) are disabled.
- **Fix Docker build for Raspberry Pi OS**: Resolve the `rustc 1.85.1` compiler mismatch in `Dockerfile` so that the image builds cleanly on Raspberry Pi OS (arm64/armv7) and other architectures.
- **Maintain backward compatibility & test coverage**: Ensure existing tests and new unit/integration tests verify the strict gating logic and Docker configuration.

## Non-Goals
- Altering the AI summary prompts or database schema.
- Changing authentication mechanisms or Miniflux sync logic.

## Technical Scope
1. **Server Configuration & Routes**:
   - In `crates/server/src/config.rs`: calculate `ai_enabled = auth_enabled && map.get("AI_ENABLED")...` so AI cannot be active when authentication is turned off.
   - In `crates/server/src/routes/entries.rs`: add configuration verification to `get_entry_ai_summary_htmx` so unauthenticated or AI-disabled requests return a 404 / feature disabled response.
2. **Dockerfile**:
   - Update builder image from `rust:1.85-slim-bookworm` to `rust:1-slim-bookworm` (or `rust:slim-bookworm` / `rust:1.100-slim-bookworm`) to satisfy crate requirements (e.g., `lock_api 0.5.0` and `parking_lot 0.13.0` which require rustc >= 1.95).
3. **Verification**:
   - Add TDD unit and route integration tests testing all permutations (`AUTH=on/AI=on`, `AUTH=off/AI=on`, `AUTH=on/AI=off`, `AUTH=off/AI=off`).
   - Run `cargo test` across all crates to ensure verify-pass.
