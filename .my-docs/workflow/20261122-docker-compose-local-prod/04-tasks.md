# 04-tasks: Tasks & Verification Plan

**Result:** done

## Planned Tests

| Case | Test / Verification Command | Covers |
|---|---|---|
| **Health endpoint contract** | `cargo test -p server --lib app::tests::test_health_endpoint` (or endpoint route verification) | Verifies `/health` responds with HTTP 200 and status ok for container healthcheck probe |
| **.dockerignore exclusions** | Validation check verifying `.env`, `.git`, `target`, and `data` are ignored from context | Ensures build context remains lean and prevents secret leakage |
| **docker-compose.local.yml validation** | `docker compose -f docker-compose.local.yml config --quiet` | Confirms syntax, volume mappings, and environment file definitions for local dev |
| **docker-compose.yml validation** | `docker compose -f docker-compose.yml config --quiet` | Confirms syntax, named volume, logging bounds, and security options for production |
| **Dockerfile structure & buildability** | Syntax and multi-stage target check | Ensures builder and runner stages compile and produce a non-root container |

---

## Task Breakdown

### Task 1: Create `.dockerignore`
- **Behavior**: Ignore build artifacts, repository metadata, local databases, and secret `.env` files from Docker build context.
- **Files Allowed**: `.dockerignore`
- **Done Evidence**: File exists; verify exclusions with `git status` / pattern inspection.

### Task 2: Create Multi-Stage `Dockerfile`
- **Behavior**: Implement two-stage build:
  1. `rust:1.85-slim-bookworm` builder compiling `server` binary with release profile.
  2. `debian:bookworm-slim` runner with `ca-certificates`, `curl`, `sqlite3`, non-root user `appuser` (10001), `/app/data` directory ownership, `HEALTHCHECK` for `/health`, and default environment variables.
- **Files Allowed**: `Dockerfile`
- **Done Evidence**: File exists and adheres to all security and runtime invariants.

### Task 3: Create `docker-compose.local.yml`
- **Behavior**: Define local development configuration with `container_name: my-ui-local`, host bind mount `./data:/app/data`, `env_file: .env`, and port `"${PORT:-3000}:3000"`.
- **Files Allowed**: `docker-compose.local.yml`
- **Done Evidence**: Validated with `docker compose -f docker-compose.local.yml config`.

### Task 4: Create `docker-compose.yml` (Production)
- **Behavior**: Define production configuration with `container_name: my-ui`, named volume `my_ui_data:/app/data`, `security_opt: [ "no-new-privileges:true" ]`, json-file logging limits (10m, 3 files), and `restart: always`.
- **Files Allowed**: `docker-compose.yml`
- **Done Evidence**: Validated with `docker compose -f docker-compose.yml config`.

### Task 5: Create `README.md`
- **Behavior**: Author a complete, beginner-friendly operational guide covering prerequisites, environment setup from `.env.example`, running locally, running in production, volume management, and troubleshooting.
- **Files Allowed**: `README.md`
- **Done Evidence**: Markdown rendering, link validation, and completeness review.

### Task 6: Final Verification & Smoke Test
- **Behavior**: Execute compose configuration validations and inspect generated files against user requirements.
- **Files Allowed**: None (read-only verification).
- **Done Evidence**: All compose validations pass without errors.
