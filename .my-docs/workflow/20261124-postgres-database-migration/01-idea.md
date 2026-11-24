# Idea framing: Switch to PostgreSQL latest & optimize for news apps

**Key takeaway:** We will replace the embedded SQLite store with PostgreSQL 17 (latest), containerized via Docker Compose, using async connection pooling in Rust (`sqlx`), and optimizing schema/indexes specifically for high-throughput news reader workloads (instant article summary lookups, batch reads, and time-ordered feeds). Existing SQLite data will be discarded as requested.

---

## 1. Context & Work Sizing

- **Work sizing:** Project with written design (replaces data storage layer across `crates/server`, configuration, Docker Compose, and unit/integration tests).
- **Current state:** `crates/server` uses `rusqlite` + `r2d2_sqlite` with blocking calls wrapped in `tokio::task::spawn_blocking` to store `article_ai_summaries` and `page_executive_briefs_v2`.
- **Target state:** Native async PostgreSQL client using `sqlx` (or `deadpool-postgres`), running `postgres:latest` in Docker Compose, with schema and indexes tailored for read-heavy news ingestion and summary caching.

---

## 2. Framing

### Goals
- **Postgres latest container:** Add an official PostgreSQL 17 service (`postgres:latest` or `postgres:17-alpine`) to `docker-compose.yml` and `docker-compose.local.yml` with health checks and volume persistence.
- **Native async database layer:** Replace synchronous `rusqlite`/`r2d2` with an async Rust driver (`sqlx` with connection pooling) in `crates/server/src/ai/db.rs`.
- **News app optimizations:**
  - Fast B-Tree primary lookups for article IDs (`entry_id`) and page keys (`page_key`).
  - Indexing on `created_at` for chronological feed ordering and cleanup.
  - Native Postgres Full-Text Search (`tsvector` / GIN index) or trigram (`pg_trgm`) readiness for searching article summaries and headlines.
  - High-concurrency upserts (`ON CONFLICT (entry_id) DO UPDATE`).
- **Clean slate:** Discard old SQLite tables and files without data migration scripts.
- **Configuration & environment:** Provide clear `DATABASE_URL` settings in `.env.example` and config parsing.

### Non-Goals
- Migrating historical records from `data/summaries.db` (explicitly ignored per user instruction).
- Modifying the upstream Miniflux server or its separate database.

### Anti-Goals
- Heavy ORM frameworks (like Diesel with complex DSLs) that add build overhead when straightforward async SQL meets all needs.
- Complex multi-node database clusters or external caching layers (e.g. Redis) that add unnecessary operational burden for a single news reader server.

### Constraints & Assumptions
- **Rust async runtime:** Tokio + Axum. Database calls should be non-blocking.
- **Has UI (tentative):** `no` (pure backend DB & configuration migration).
- **Environment variables:** Use `DATABASE_URL` with fallback defaults for local Docker workflows.

---

## 3. Playback & Intent

We understand the primary intent:
1. You want a production-grade, scalable PostgreSQL database backing `my-ui` rather than embedded SQLite files.
2. Because this database holds generated AI summaries and page briefs, starting clean without migrating old SQLite rows is preferred and faster.
3. News applications have distinct read/write characteristics: bursty writes when feeds refresh, frequent batch reads when rendering article lists, and queries sorting by publication/creation time. The schema and connection pool should reflect this workload.

*Skip option:* If this framing matches your vision and you want to jump straight to execution, you can say **"just start"** and we will proceed directly.

---

## 4. Open Question for HITL

To optimize the schema and driver setup for your exact news application workflow:

**What specific news search or filtering features do you plan to run most frequently against the database (for example: full-text search across article summaries/titles, time-window aggregations like top daily news, or strictly fast key-value lookups by article ID)?**
