# 03-design: Docker & Docker Compose Design Specification

**Result:** done

## Overview

This specification details the Docker and Docker Compose architecture for `my-ui`, providing clean, reproducible, and secure execution across both local development and production environments.

---

## Technical Options & Architectural Choices

### Decision: Compose Topology & Configuration Organization

#### Option A: Unified Compose with Profiles
- **Explanation**: A single `docker-compose.yml` containing multiple services or environments controlled via Compose profiles (`--profile local` vs `--profile prod`).
- **Example**:
  ```yaml
  services:
    my-ui-local:
      build: .
      profiles: ["local"]
      volumes:
        - ./data:/app/data
    my-ui-prod:
      image: my-ui:latest
      profiles: ["prod"]
      volumes:
        - my_ui_data:/app/data
  ```
- **Pros**: Single configuration file to track.
- **Cons**: Convoluted commands requiring explicit `--profile` flags; mixing production and local definitions can lead to accidental misconfigurations.

#### Option B (Recommended & Confirmed): Dedicated `docker-compose.local.yml` and Standard Production `docker-compose.yml`
- **Explanation**: Keep production as the default `docker-compose.yml` (so standard `docker compose up -d` deploys production safely), and use `docker-compose.local.yml` for local development workflows (`docker compose -f docker-compose.local.yml up -d`).
- **Example**:
  ```bash
  # Local development:
  docker compose -f docker-compose.local.yml up -d

  # Production deployment:
  docker compose up -d
  ```
- **Pros**:
  - **Zero ambiguity**: Impossible to run development settings by mistake on production.
  - **Standard convention**: Standard `docker-compose.yml` matches automated deployment tools (Portainer, Dockge, GitOps).
  - **Clean separation**: Local development mounts host `./data` directly, while production uses named Docker volumes and strict security options.
- **Cons**: Minor duplication of the core service container port and env definitions across the two compose files.

---

## Detailed Specifications

### 1. Multi-Stage `Dockerfile`

The Dockerfile implements a clean two-stage build:

1. **Builder Stage (`rust:1.85-slim-bookworm`)**:
   - Installs build essentials (`gcc`, `g++`, `libc6-dev`, `pkg-config`, `make`) needed for compiling C source bundled in `rusqlite`.
   - Creates working directory `/build`.
   - Copies workspace manifests (`Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`).
   - Copies full source tree and builds the release binary (`cargo build --release --bin server`).
   - Strips unneeded debug symbols from the final binary to minimize size.

2. **Runner Stage (`debian:bookworm-slim`)**:
   - Updates APT and installs runtime packages:
     - `ca-certificates`: Required for outgoing HTTPS requests to Miniflux and Pocket-ID OIDC endpoints.
     - `curl`: Required for Docker container health check probes.
     - `sqlite3`: Useful utility for inspecting database status when needed.
   - Creates a dedicated non-root user and group: `appuser:appuser` with UID/GID `10001`.
   - Pre-creates `/app/data` with ownership `appuser:appuser` (mode 750) so SQLite WAL and DB files can be created without permission failures.
   - Copies `/build/target/release/server` to `/usr/local/bin/server` (executable mode 755).
   - Sets non-sensitive default environment variables:
     - `HOST=0.0.0.0`
     - `PORT=3000`
     - `AI_DB_PATH=/app/data/summaries.db`
   - Defines a native `HEALTHCHECK`:
     ```dockerfile
     HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
       CMD curl -f http://127.0.0.1:3000/health || exit 1
     ```
   - Drops privileges: `USER 10001:10001`.
   - Exposes port `3000`.
   - Declares volume `/app/data`.
   - Sets entrypoint: `ENTRYPOINT ["/usr/local/bin/server"]`.

---

### 2. Context Filtering (`.dockerignore`)

Prevents local build artifacts, repository history, secrets, and developer files from inflating the build context:
- `target/`
- `.git/`
- `.github/`
- `.my-docs/`
- `.gemini/`
- `.env*` (with exception for `.env.example`)
- `data/`
- `*.md`
- `*.sqlite*`
- `*.db*`

---

### 3. Local Development (`docker-compose.local.yml`)

Designed for developer agility and transparent local inspection:
- **Service Name**: `my-ui`
- **Container Name**: `my-ui-local`
- **Build Context**: `.` with `Dockerfile`
- **Environment File**: `.env` (developer supplies their active credentials)
- **Port Mapping**: `"${PORT:-3000}:3000"`
- **Host Mount**: `./data:/app/data` (allows developer to inspect SQLite database directly on the host machine)
- **Restart Policy**: `unless-stopped`

```yaml
services:
  my-ui:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: my-ui-local
    restart: unless-stopped
    env_file:
      - .env
    ports:
      - "${PORT:-3000}:3000"
    volumes:
      - ./data:/app/data
```

---

### 4. Production Deployment (`docker-compose.yml`)

Hardened for production reliability, container isolation, and resource stability:
- **Service Name**: `my-ui`
- **Container Name**: `my-ui`
- **Build Context**: `.` with `Dockerfile` (or tagged image `my-ui:latest`)
- **Environment File**: `.env`
- **Port Mapping**: `"${PORT:-3000}:3000"`
- **Data Persistence**: Named volume `my_ui_data:/app/data`
- **Restart Policy**: `always`
- **Security Hardening**:
  - `security_opt: [ "no-new-privileges:true" ]`
- **Logging Limits**:
  - `driver: json-file`
  - `options: { max-size: "10m", max-file: "3" }`
- **Healthcheck**: Aligned with container `/health` endpoint.

```yaml
services:
  my-ui:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: my-ui
    restart: always
    security_opt:
      - no-new-privileges:true
    env_file:
      - .env
    ports:
      - "${PORT:-3000}:3000"
    volumes:
      - my_ui_data:/app/data
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

volumes:
  my_ui_data:
    driver: local
```

---

### 5. Documentation (`README.md`)

Structured according to clarity and reading flow principles:
1. **Header & Brief**: What `my-ui` is (modern, fast Miniflux web frontend).
2. **Prerequisites**: Docker Engine 24+, Docker Compose v2.20+, optional Rust toolchain for native building.
3. **Configuration Guide**:
   - Explanation of copying `.env.example` to `.env`.
   - Variable breakdown with descriptions and defaults (without hardcoding secrets).
4. **Local Development with Docker Compose**:
   - Starting up: `docker compose -f docker-compose.local.yml up -d --build`
   - Viewing logs: `docker compose -f docker-compose.local.yml logs -f`
   - Stopping: `docker compose -f docker-compose.local.yml down`
5. **Production Deployment with Docker Compose**:
   - Starting up: `docker compose up -d --build`
   - Managing updates: `docker compose pull && docker compose up -d`
   - Stopping: `docker compose down`
6. **Data Persistence**:
   - Explanation of `data/summaries.db` and volume management (`docker volume inspect ...`).
7. **Troubleshooting**:
   - Healthcheck inspection (`docker inspect --format='{{json .State.Health}}' my-ui`).
   - SQLite permission errors and solutions.
   - Upstream connectivity to Miniflux / Pocket-ID.

---

## Proposed Has (parent applies to card)

- **Has UI**: no
- **Has API**: no
- **Has DB**: no
- **Has auth/PII/schema**: no
