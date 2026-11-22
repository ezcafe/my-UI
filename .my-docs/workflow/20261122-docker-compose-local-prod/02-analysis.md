# 02-analysis: Docker and Docker Compose Architecture Analysis

**Result:** done

## Context

The `my-ui` project is a high-performance web dashboard and reader for Miniflux RSS feeds, written in Rust. It is structured as a Cargo workspace:
- `crates/server`: Core web application powered by Axum 0.8, providing web pages, HTMX endpoints, and GraphQL proxying.
- `crates/miniflux`: Miniflux API client communicating over HTTPS using `reqwest` and `rustls-tls`.
- `crates/ui-components`: UI templates compiled directly into the binary using Askama 0.12.

### Runtime and Operational Requirements
- **Executable binary**: `server` binary compiled in release mode via `cargo build --release --bin server`.
- **Operating system & glibc**: The binary links dynamically against glibc and statically bundles SQLite C source code via `rusqlite` (`features = ["bundled"]`).
- **Network ports**: Default port `3000`, listening on host `0.0.0.0`.
- **Healthcheck endpoint**: `/health` returning `{"status": "ok"}` with HTTP 200.
- **Data storage**: SQLite database for AI digests at `data/summaries.db` (configurable via `AI_DB_PATH`).
- **Environment variables**: Configured via `.env.example` including `PORT`, `HOST`, `APP_BASE_URL`, `SESSION_SECRET`, `AUTH_ENABLED`, `OIDC_*`, `MINIFLUX_*`, and `AI_*`.

---

## Key Invariants & Constraints

1. **Security & Least Privilege**:
   - The production container must run as an unprivileged non-root user (e.g., `appuser` with UID `10001`).
   - Secret files (`.env`, `.env.*`) must never be baked into container images.
   - `no-new-privileges: true` must be enforced in production container security options.

2. **Persistence**:
   - The SQLite database directory (`/app/data`) must be persistent across container recreations.
   - For local development, bind-mounting `./data:/app/data` allows direct inspection on the host.
   - For production, a named Docker volume (`my_ui_data:/app/data`) ensures standard Docker volume management, proper ownership permissions, and automated backup compatibility.

3. **Multi-Stage Build Efficiency**:
   - The builder stage must contain the full Rust toolchain (`rust:1.85-slim-bookworm` or compatible) and C build tooling needed by `rusqlite` (e.g. `gcc`, `libc6-dev`).
   - The runner stage must be minimal (`debian:bookworm-slim`), containing only essential runtime dependencies (`ca-certificates` for TLS verification, `curl` for container health checks, and `sqlite3` for database diagnostics).
   - The final container image size should remain under ~120 MB.

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **File permissions on mounted volume** | SQLite cannot create or write to `data/summaries.db` if the non-root container user lacks write access. | Explicitly create `/app/data` in the Dockerfile with ownership `appuser:appuser`, and configure documentation detailing permission handling. |
| **Accidental secret inclusion in image** | Sensitive `.env` files containing `SESSION_SECRET` or `MINIFLUX_API_KEY` could be copied during `COPY . .`. | Maintain a strict `.dockerignore` file excluding `.env`, `.env.*` (except `.env.example`), `.git`, and `target`. |
| **TLS/SSL certificate errors** | Container fails to communicate with external Miniflux or OIDC identity provider over HTTPS. | Install and update `ca-certificates` package in the runner stage. |
| **Log disk exhaustion in production** | Container logs grow indefinitely without rotation. | Configure `logging` in `docker-compose.yml` with `json-file` driver, `max-size: 10m`, and `max-file: 3`. |
| **Large Docker build context** | Slow build startup and heavy caching overhead if `target/` directory is sent to Docker daemon. | Ignore `target/` and `.git/` in `.dockerignore`. |

---

## Open Questions

None. All architectural decisions (Compose separation 1B, single service 2B, multi-stage Debian runner 3A) were reviewed and confirmed by the user in `01-idea.md`.
