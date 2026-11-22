# 01-idea: Docker & Docker Compose Setup for Local and Production

## Intent

Provide Docker and Docker Compose configurations to build and run `my-ui` seamlessly in both local development and production environments, accompanied by a comprehensive `README.md` guide.

---

## 1. Scope and Invariants

- **Has UI (tentative)**: no
- **Has API**: no
- **Has DB**: no
- **Has auth/PII/schema**: no

### Goals
- **Multi-stage Dockerfile**: Efficient, secure build of the Rust workspace binary (`server`) with minimal image size and non-root runtime user.
- **Local Docker Compose (`docker-compose.yml` / `docker-compose.local.yml`)**: Easy one-command startup for local testing, mounting SQLite database persistence, and loading local environment variables.
- **Production Docker Compose (`docker-compose.prod.yml`)**: Production-ready container configuration with restart policies, logging bounds, security hardening, and volume mounts for persistent data.
- **Comprehensive `README.md`**: Clear, step-by-step instructions for:
  - Prerequisites (Docker, Docker Compose, Rust toolchain if building natively).
  - Environment configuration (`.env.example` -> `.env`).
  - Running locally with Docker Compose.
  - Running in production with Docker Compose.
  - Data persistence (`data/summaries.db`).
  - Common troubleshooting and commands (logs, restarts, updates).

### Non-Goals
- Modifying Rust application code, routes, UI templates, or database schemas.
- Hosting an automated CI/CD pipeline or Kubernetes manifests (can be future work).
- Bundling hardcoded credentials or secrets into images or compose files.

### Anti-Goals
- No root user execution inside production containers.
- No heavy images containing full compiler toolchains in final runtime artifacts.
- No breaking changes to existing local `cargo run` workflows.

---

## 2. Key Technical Decisions & Trade-Offs

### Decision 1: Docker Compose File Architecture
How to organize the Docker Compose files for local and production modes.

- **Option A (Recommended)**: Base `docker-compose.yml` with environment-specific overrides (`docker-compose.local.yml` and `docker-compose.prod.yml`).
  - **Explanation**: A common base file defines the service and volumes, while local overrides enable debug ports or local volume mounts, and production overrides specify restart policies and resource controls.
  - **Concrete Example**:
    ```bash
    # Local run:
    docker compose -f docker-compose.yml -f docker-compose.local.yml up -d
    # Production run:
    docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
    ```
  - **Pros**: Clean separation of concerns, DRY (no duplicate service definitions), standard Docker Compose pattern.
  - **Cons**: Requires specifying `-f` flags or setting `COMPOSE_FILE` in environment.

- **Option B**: Two independent compose files (`docker-compose.local.yml` and `docker-compose.prod.yml`) or standalone `docker-compose.yml` with Compose Profiles.
  - **Explanation**: Keep local and prod in completely separate files, or use `--profile local` / `--profile prod`.
  - **Pros**: Explicit and isolated commands.
  - **Cons**: Duplicates container build settings and environment wiring across multiple files.

### Decision 2: Base Container Image for Runtime
Which operating system base image to choose for the final stage of the multi-stage Dockerfile.

- **Option A (Recommended)**: `debian:bookworm-slim` with `ca-certificates`.
  - **Explanation**: A lightweight Debian base matching standard Rust glibc compilation, pre-packaged with security updates and standard SSL certificates. Ensures full compatibility with `rusqlite` (bundled), `rustls-tls`, and any external CLI tools (e.g. `llm`).
  - **Concrete Example**:
    ```dockerfile
    FROM rust:1.85-slim-bookworm AS builder
    ...
    FROM debian:bookworm-slim AS runner
    RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates sqlite3 curl && rm -rf /var/lib/apt/lists/*
    ```
  - **Pros**: Rock-solid glibc compatibility, easy troubleshooting, small footprint (~80 MB total).
  - **Cons**: Slightly larger than Alpine (~20 MB vs ~80 MB).

- **Option B**: `alpine:3.20` using musl target (`x86_64-unknown-linux-musl`).
  - **Explanation**: Compile statically with musl for minimal image size.
  - **Pros**: Tiny image size (~25 MB).
  - **Cons**: Slower Rust compile times for musl cross-compilation, potential linking quirks with bundled C libraries (like SQLite) or dynamic CLI tools.

### Decision 3: Local Auxiliary Services (Miniflux / Postgres)
Whether the local compose should include optional local Miniflux and Postgres containers for self-contained testing.

- **Option A (Recommended)**: Focus primary compose on `my-ui`, with an optional `miniflux` service under an opt-in profile (e.g. `profiles: ["with-miniflux"]`).
  - **Explanation**: Users who already have an existing Miniflux instance (e.g. cloud or another host) can just run `my-ui`. Users who need a complete offline lab can start Miniflux + Postgres with a single `--profile with-miniflux` flag.
  - **Concrete Example**:
    ```yaml
    services:
      my-ui:
        build: .
        ports:
          - "${PORT:-3000}:3000"
      miniflux:
        image: miniflux/miniflux:latest
        profiles: ["with-miniflux"]
    ```
  - **Pros**: Flexible for both self-contained testing and production deployments.
  - **Cons**: Adds a few lines of configuration to the compose file.

- **Option B**: Only define `my-ui` service, leaving all external services external.
  - **Explanation**: `my-ui` expects external `MINIFLUX_BASE_URL` and `MINIFLUX_API_KEY` without providing any companion service.
  - **Pros**: Simplest possible compose configuration.
  - **Cons**: Developers without an active Miniflux instance cannot test end-to-end locally out-of-the-box.

---

## 3. User Confirmed Decisions

- **Decision 1 (Compose Architecture)**: Option 1B — `docker-compose.local.yml` for local development and `docker-compose.yml` for production.
- **Decision 2 (Companion Services)**: Option 2B — Strictly `my-ui` only (external Miniflux / Pocket-ID).
- **Decision 3 (Runtime Base Image)**: Option 3A — Multi-stage build with `debian:bookworm-slim` runtime.

## Shared understanding

- [x] User confirmed (brainstorm hard-gate)
  - User selection: `"1B docker-compose.local.yml + docker-compose.yml for prod, 2B, 3A"`
