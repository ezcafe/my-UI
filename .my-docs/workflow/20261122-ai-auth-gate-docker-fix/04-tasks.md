# Tasks (TDD-ready)

Each task defines exact behavior, failing test name first, allowed files, and done evidence.

## Planned tests

<!-- ≥1 named case → planned tests present (Prefer tdd-review). Empty / N/A → not present. -->

| Case | Failing test name | Covers |
|------|-------------------|--------|
| Config AI Gating | `test_config_ai_disabled_when_auth_disabled` | `crates/server/src/config.rs` |
| Config Matrix Permutations | `test_config_ai_matrix_permutations` | `crates/server/src/config.rs` |
| Route AI Summary Gating | `test_hx_entries_ai_summary_disabled_when_ai_flag_off` | `crates/server/src/routes/entries.rs` |
| Route AI Summary Unauthenticated & Guest Gating | `test_hx_entries_ai_summary_disabled_in_guest_mode` | `crates/server/src/routes/entries.rs` |

---

## Task 1: Red Unit Tests for Config AI Gating

- **Behavior**: Add unit tests in `crates/server/src/config.rs` asserting that `ai_enabled` is strictly `false` whenever `auth_enabled` is `false`, regardless of `AI_ENABLED`. Also verify all 4 permutations of auth and AI flags.
- **Failing test name**:
  - `test_config_ai_disabled_when_auth_disabled`
  - `test_config_ai_matrix_permutations`
- **Files allowed**:
  - `crates/server/src/config.rs`
- **Done evidence**:
  - Tests fail initially against current code (where `AUTH_ENABLED=false` and `AI_ENABLED=true` currently yields `ai_enabled=true`).

---

## Task 2: Red Route Integration Tests for AI Endpoints

- **Behavior**: Add route integration tests in `crates/server/src/routes/mod.rs` asserting that requests to `/hx/entries/{id}/ai-summary` return 404 / NotFound when `ai_enabled` is `false` (both for authenticated users and guest mode).
- **Failing test name**:
  - `test_hx_entries_ai_summary_disabled_when_ai_flag_off`
  - `test_hx_entries_ai_summary_disabled_in_guest_mode`
- **Files allowed**:
  - `crates/server/src/routes/mod.rs`
- **Done evidence**:
  - Tests fail initially because `get_entry_ai_summary_htmx` executes `ai_service.get_or_create_summary` without checking `config.ai_enabled`.

---

## Task 3: Green Implementation for Config & Entry Routes

- **Behavior**:
  - Update `Config::from_map` to calculate `ai_enabled = auth_enabled && map.get("AI_ENABLED")...`.
  - Update `get_entry_ai_summary_htmx` in `crates/server/src/routes/entries.rs` to extract `State(config): State<Arc<Config>>` and `headers: HeaderMap`, returning `handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()))` if `!config.ai_enabled`.
- **Files allowed**:
  - `crates/server/src/config.rs`
  - `crates/server/src/routes/entries.rs`
- **Done evidence**:
  - All red unit and route integration tests pass (`cargo test -p server`).

---

## Task 4: Dockerfile Builder Base Image Update

- **Behavior**: Update Docker builder base image in `Dockerfile` from `rust:1.85-slim-bookworm` to `rust:1-slim-bookworm`.
- **Files allowed**:
  - `Dockerfile`
- **Done evidence**:
  - `Dockerfile` specifies `FROM rust:1-slim-bookworm AS builder`.

---

## Task 5: Full Verification & Quality Gate

- **Behavior**: Run full workspace test suite, linter (`cargo clippy`), and build check.
- **Files allowed**: None (read/verify only).
- **Done evidence**:
  - `cargo test --workspace` passes cleanly.
  - `cargo clippy --workspace -- -D warnings` reports zero warnings.
