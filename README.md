# my-ui

`my-ui` is a modern, fast web frontend for the [Miniflux](https://miniflux.app/) RSS reader. It provides a clean, responsive reader interface with optional OIDC authentication (Pocket-ID), local AI article summarization, and a robust PostgreSQL backend for high-performance indexing, search, and caching.

---

## Prerequisites

Before running `my-ui`, make sure you have the following installed on your host machine:

- **Docker Engine**: version 24.0 or newer.
- **Docker Compose**: version 2.20 or newer (or `docker-compose` standalone).
- **Rust Toolchain** *(optional)*: Rust 1.85+ if compiling natively outside Docker.

---

## Configuration

The application reads configuration from environment variables. A template file is provided at `.env.example`.

### 1. Create your environment file

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

### 2. Configure environment variables

Open `.env` and fill in your values:

- **Server settings**:
  - `PORT`: HTTP port to listen on inside container (default: `3000`).
  - `HOST`: Bind address (default: `0.0.0.0`).
  - `APP_BASE_URL`: Public base URL of the application (e.g. `http://localhost:3000`).
  - `SESSION_SECRET`: Random 32+ character key for session cookie encryption.
- **Database (PostgreSQL 19)**:
  - `DATABASE_URL`: Connection string to PostgreSQL (default for local: `postgres://my_ui:my_ui_password@localhost:5432/my_ui`, or `postgres://my_ui:my_ui_password@db:5432/my_ui` inside Docker Compose).
- **Authentication (Pocket-ID OIDC)**:
  - `AUTH_ENABLED`: Set to `true` to require login, or `false` for public/single-user mode.
  - `OIDC_ISSUER_URL`: Issuer URL of your Pocket-ID instance.
  - `OIDC_CLIENT_ID`: Registered OIDC client ID.
  - `OIDC_CLIENT_SECRET`: Registered OIDC client secret.
  - `OIDC_REDIRECT_URL`: OAuth callback URL (e.g. `http://localhost:3000/auth/callback`).
- **Miniflux upstream**:
  - `MINIFLUX_BASE_URL`: Base URL of your Miniflux server (e.g. `https://miniflux.example.com`).
  - `MINIFLUX_API_KEY`: API key generated in Miniflux settings.
- **AI digest**:
  - `AI_ENABLED`: Set to `true` to enable article summarization, `false` to disable.
  - `LLM_CLI_CMD`: Local LLM CLI binary (default: `llm`).
  - `LLM_MODEL`: Model name (default: `gpt-4o-mini`).
  - `LLM_EXTRA_ARGS`: Optional extra arguments passed to CLI (e.g. `"--thinking-budget 0"`).

### 3. Generate a secure `SESSION_SECRET`

When `AUTH_ENABLED=true`, `SESSION_SECRET` is required to encrypt session cookies using AES-256-GCM. It must be **at least 32 characters long**.

Generate a random secret using any of the following commands:

```bash
# Using OpenSSL (Recommended):
openssl rand -hex 32

# Or using Python:
python3 -c "import secrets; print(secrets.token_hex(32))"

# Or using /dev/urandom:
head -c 32 /dev/urandom | base64
```

Copy the output and paste it into your `.env`:

```bash
SESSION_SECRET=your_generated_random_string_here
```

*(Note: If `AUTH_ENABLED=false`, a built-in default secret is automatically used for guest mode).*

---

## AI News Digests, Executive Briefs & Rewrites

`my-ui` integrates local or cloud LLMs to provide intelligent news digest capabilities tailored for high-volume RSS feeds:

### Key AI Features

1. **Page-Level Executive Briefs**:
   - Synthesizes up to 50 unread articles into an actionable, strictly 3-sentence executive overview.
   - Intelligently groups content by feed source and domain context.
   - Stored in PostgreSQL (`page_executive_briefs_v2`) and invalidated automatically when unread counts or filters change.
2. **Article AI Summaries**:
   - Concise 1–2 sentence digests (<35 words) generated on-demand as you scroll through feeds.
3. **Suggested Headlines**:
   - Clean, objective headline suggestions for fast, distraction-free scanning.
4. **Article Rewrites**:
   - Long-form article rewrites that strip boilerplate, sponsor messages, and navigation fluff.
   - Eagerly generated for top-priority news items and loaded on-demand for reader view.
5. **Magazine Editorial Layout**:
   - Switchable 3-column magazine layout featuring AI executive briefs, headline pills, and source badges.
6. **Token Optimization & Cost Control**:
   - Intelligent batching and strict word-count limits (130-word article snippets for summarization).
   - Eagerly generates full rewrites only for the top article to conserve token budgets.
   - Deduplicates requests and reuses cached summaries across page refreshes.
7. **Dual LLM Backends**:
   - **Native REST API (Recommended)**: Built-in async HTTP client with keep-alive connection pooling for OpenAI (`gpt-4o-mini`, etc.) and Google Gemini (`gemini-2.5-flash`, etc.) with sub-second response times.
   - **CLI Mode**: Pipes prompts directly to local CLI tools like Simon Willison's [`llm`](https://llm.datasette.io/).
   - **Graceful Extractive Fallback**: If LLM requests fail or time out (5s), `my-ui` automatically generates extractive text summaries without interrupting the reader.

### Configuring AI Backends

In your `.env`:

```bash
AI_ENABLED=true

# Option A: Native REST API (Sub-second responses via keep-alive connection pool)
LLM_PROVIDER=openai
LLM_API_KEY=your-api-key-here
LLM_MODEL=gpt-4o-mini
LLM_API_BASE_URL=https://api.openai.com/v1

# Option B: Google Gemini REST API
# LLM_PROVIDER=gemini
# LLM_API_KEY=your-gemini-api-key-here
# LLM_MODEL=gemini-2.5-flash

# Option C: CLI Mode (Default fallback)
# LLM_CLI_CMD=llm
# LLM_MODEL=gpt-4o-mini
# LLM_EXTRA_ARGS="--thinking-budget 0"
```

---

## Database Architecture (PostgreSQL 19)

`my-ui` uses an asynchronous PostgreSQL backend via `sqlx` with native connection pooling. All database tables and indexes are created idempotently on server startup in `AiDb::new`.

### Core Tables and Indexing Strategy

1. **`article_ai_summaries`**:
   - **Instant Lookups**: Clustered B-Tree primary key on `entry_id BIGINT`.
   - **Batch Fetches**: Efficient multi-row lookups using PostgreSQL `ANY($1::bigint[])`.
   - **Full-Text Search (FTS)**: A stored generated column `search_vector tsvector` automatically combines the suggested headline, summary, and article rewrite with English stemming. A `GIN` index (`idx_article_summaries_search_gin`) powers fast text searches with `ts_rank` relevance ranking.
   - **Chronological Feeds**: Descending index on `created_at TIMESTAMPTZ` (`idx_article_summaries_created_at`) enables quick time-window filtering.
2. **`page_executive_briefs_v2`**:
   - Stores synthesized page briefs keyed by deterministic hash `page_key TEXT PRIMARY KEY`.
   - Indexed on `created_at TIMESTAMPTZ DESC` for rapid retention pruning and latest digest access.

---

## Running with Docker Compose

`my-ui` is containerized alongside official PostgreSQL in Docker Compose.

### Local Development

Use `docker-compose.local.yml` for local development:

```bash
# Start PostgreSQL and web service
docker-compose -f docker-compose.local.yml up -d --build

# View logs
docker-compose -f docker-compose.local.yml logs -f

# Stop containers
docker-compose -f docker-compose.local.yml down
```

### Production Deployment

Use `docker-compose.yml` for production deployments. It includes security options (`no-new-privileges:true`), log rotation (10MB limit), automated database health checks, and persistent Docker volumes:

```bash
# Start all production services
docker-compose up -d --build

# View container status and health
docker-compose ps

# View service logs
docker-compose logs -f

# Gracefully stop services
docker-compose down
```

---

## Database Backup and Restore Guide

All database data is safely stored in the `my_ui_postgres_data` Docker volume. Use standard PostgreSQL utilities (`pg_dump` and `pg_restore` or `psql`) executed through Docker Compose to perform backups and restores without stopping the database service.

### 1. Compressed Custom Format Backup (`-Fc`) — Recommended

The PostgreSQL custom archive format (`-Fc`) is compressed, supports parallel multi-core restoration, and allows selective table extraction:

```bash
# Create a compressed backup file
docker-compose exec -T db pg_dump -U my_ui -d my_ui -Fc > my_ui_backup_$(date +%Y%m%d_%H%M%S).dump
```

To restore from a compressed custom format archive:

```bash
# Restore with clean table replacement using pg_restore
docker-compose exec -T db pg_restore -U my_ui -d my_ui --clean --if-exists --no-owner < my_ui_backup_20261124_090000.dump
```

### 2. Plain Text SQL Backup (`.sql`)

If you prefer human-readable SQL scripts:

```bash
# Create plain SQL dump
docker-compose exec -T db pg_dump -U my_ui -d my_ui --clean --if-exists > my_ui_backup_$(date +%Y%m%d_%H%M%S).sql

# Restore plain SQL dump
docker-compose exec -T db psql -U my_ui -d my_ui < my_ui_backup_20261124_090000.sql
```

### 3. Automated Daily Backup via Cron

You can add a simple daily cron job on your host machine to automate database backups and maintain a 7-day retention policy:

```bash
# Edit crontab
crontab -e

# Add daily backup at 03:00 AM:
0 3 * * * cd /path/to/my-ui && docker-compose exec -T db pg_dump -U my_ui -d my_ui -Fc > /var/backups/my_ui_$(date +\%Y\%m\%d).dump && find /var/backups -name "my_ui_*.dump" -mtime +7 -delete
```

### 4. Verify Database Records After Restore

After restoring a backup, verify that tables and records are present:

```bash
# List all tables and indexes
docker-compose exec -T db psql -U my_ui -d my_ui -c "\dt"

# Count saved summaries and verify FTS index
docker-compose exec -T db psql -U my_ui -d my_ui -c "SELECT count(*) FROM article_ai_summaries;"

# Count executive briefs
docker-compose exec -T db psql -U my_ui -d my_ui -c "SELECT count(*) FROM page_executive_briefs_v2;"
```

---

## Troubleshooting

- **Database container health check**:
  Verify PostgreSQL is ready to accept connections:
  ```bash
  docker-compose exec db pg_isready -U my_ui -d my_ui
  ```
- **Web service health endpoint**:
  Verify web application responsiveness:
  ```bash
  docker-compose exec my-ui curl -I http://127.0.0.1:3000/health
  ```
- **Inspect database container logs**:
  ```bash
  docker-compose logs db
  ```
- **Upstream connectivity issues**:
  Check that the container can reach your Miniflux and Pocket-ID instances:
  ```bash
  docker-compose exec my-ui curl -I https://miniflux.example.com
  ```
