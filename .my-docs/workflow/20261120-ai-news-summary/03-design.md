# Design: AI News Batch Summary

**Result:** done

## Key Takeaways
- **Feed-grouped digest**: Adds an AI-driven news digest for the latest 15 articles grouped by feed source (`GET /hx/entries?status=ai`), supporting search query forwarding (`search`).
- **Viewport-triggered lazy summaries**: Summaries load lazily as cards enter the viewport using HTMX `hx-trigger="revealed"` targeting canonical REST route `GET /hx/entries/{id}/ai-summary`.
- **High-concurrency SQLite pooling**: Uses `r2d2` + `r2d2_sqlite` connection pooling with Write-Ahead Logging (`WAL`), `busy_timeout=5000ms`, `user_version` migration tracking, and `ON CONFLICT` upserts.
- **Robust content extraction & CLI runner**: Extracts clean text from `<article>` tags with instant fallback on 403/Cloudflare blocks, feeding an isolated LLM CLI process throttled to 2 concurrent runs with zero reasoning budget.
- **Guaranteed UI unsticking & security**: Endpoints require `AuthenticatedUser` and verify entry ownership via Miniflux before accessing cache or LLM. Endpoints always return HTTP 200 with replacement HTML so HTMX skeletons never freeze.

---

## Architecture Options

### Option A: Integrated AI Service in `crates/server` (Recommended)
- **Explanation**: Implement a cohesive `crate::ai` module directly in `crates/server`. It handles pooled SQLite caching, `<article>` tag web extraction, and process-isolated LLM CLI invocation. Axum handlers in [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) invoke the service through `AppState`.
- **Concrete Example**:
  ```rust
  // crates/server/src/ai/mod.rs
  pub struct AiService {
      db: Arc<AiDb>,
      extractor: ArticleExtractor,
      llm: LlmCliRunner,
      semaphore: Arc<tokio::sync::Semaphore>,
  }

  impl AiService {
      pub async fn get_or_create_summary(
          &self,
          entry_id: i64,
          url: &str,
          title: &str,
          teaser: &str,
      ) -> String {
          // Fast cache check via connection pool
          if let Some(cached) = self.db.get_summary(entry_id).await {
              return cached;
          }

          // Bound concurrent LLM CLI spawns
          let _permit = self.semaphore.acquire().await.ok();
          let content = self.extractor.extract_content(url, title, teaser).await;
          let (summary, model) = match self.llm.summarize(title, &content).await {
              Some(ai_summary) => (ai_summary, self.llm.model_name()),
              None => (
                  miniflux_client::summary::extract_summary(&content, 2),
                  "extractive-fallback".to_string(),
              ),
          };

          self.db.save_summary(entry_id, &summary, &model).await.ok();
          summary
      }
  }
  ```
- **Pros**:
  - Direct integration with Axum routing, `AppState`, and server configuration.
  - Zero cross-crate dependency overhead or unnecessary trait indirection.
  - Straightforward unit and integration testing via standard Cargo workflows.
- **Cons**:
  - Web scraping and CLI subprocess management reside inside `crates/server`.

### Option B: Standalone Workspace Crate `crates/ai-engine`
- **Explanation**: Move SQLite storage, scraping, and CLI subprocess logic into a separate `crates/ai-engine` crate, exposing an async trait to `crates/server`.
- **Concrete Example**:
  ```rust
  // crates/ai-engine/src/lib.rs
  #[async_trait]
  pub trait AiEngine: Send + Sync {
      async fn summarize(&self, entry_id: i64, url: &str, title: &str, teaser: &str) -> String;
  }
  ```
- **Pros**:
  - Pure modularity; `crates/server` remains strictly an HTTP/GraphQL gateway.
- **Cons**:
  - High workspace churn and boilerplate for a single targeted feature.
  - Duplicates data structures and increases build complexity.

---

## Recommendation
**Option A (Integrated AI Service in `crates/server`)** is recommended. It conforms cleanly to the existing codebase layout, avoids boilerplate crate boundaries, and enables rapid TDD verification.

---

## Detailed Component Design

### 1. Configuration & Feature Toggle (`crates/server/src/config.rs`)
- **Fields added to `Config` struct**:
  - `ai_enabled: bool` (parses `AI_ENABLED`, default `false`)
  - `llm_cli_cmd: String` (parses `LLM_CLI_CMD`, default `"llm"`)
  - `llm_model: String` (parses `LLM_MODEL`, default `"gpt-4o-mini"`)
  - `llm_extra_args: Vec<String>` (parses `LLM_EXTRA_ARGS`, e.g. `"--thinking-budget 0"`)
  - `ai_db_path: String` (parses `AI_DB_PATH`, default `"data/summaries.db"`)
- **Placeholders added to `.env.example`**:
  ```bash
  # AI Digest Configuration
  AI_ENABLED=true
  LLM_CLI_CMD=llm
  LLM_MODEL=gpt-4o-mini
  LLM_EXTRA_ARGS="--thinking-budget 0"
  AI_DB_PATH=data/summaries.db
  ```

### 2. Persistent SQLite Cache (`crates/server/src/ai/db.rs`)
- **Dependencies**:
  - `rusqlite = { version = "0.32", features = ["bundled"] }`
  - `r2d2 = "0.8"`
  - `r2d2_sqlite = "0.25"`
- **Connection Pool & Pragmas**:
  - Uses `r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>` with a pool size of 8 connections.
  - Configures pragmas on every pooled connection:
    - `PRAGMA journal_mode = WAL;` (Write-Ahead Logging enables concurrent readers alongside writers; see [SQLite WAL Documentation](https://www.sqlite.org/wal.html)).
    - `PRAGMA busy_timeout = 5000;` (Prevents `SQLITE_BUSY` errors by waiting up to 5000ms for locks; see [SQLite busy_timeout](https://www.sqlite.org/pragma.html#pragma_busy_timeout)).
    - `PRAGMA synchronous = NORMAL;` (Safe and optimal for WAL mode).
- **Schema & Migration Versioning**:
  - Evaluates `PRAGMA user_version;` at initialization. If `version < 1`, executes:
    ```sql
    CREATE TABLE IF NOT EXISTS article_ai_summaries (
      entry_id INTEGER PRIMARY KEY,
      summary TEXT NOT NULL,
      model TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    );
    CREATE INDEX IF NOT EXISTS idx_article_ai_summaries_created_at ON article_ai_summaries(created_at);
    PRAGMA user_version = 1;
    ```
- **Upsert on Insert**:
  - Avoids constraint violations during concurrent requests for the same entry:
    ```sql
    INSERT INTO article_ai_summaries (entry_id, summary, model, created_at)
    VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    ON CONFLICT(entry_id) DO UPDATE SET
      summary = excluded.summary,
      model = excluded.model,
      created_at = excluded.created_at;
    ```
  - Reference: [SQLite UPSERT Documentation](https://www.sqlite.org/lang_UPSERT.html).
- **Execution Model**:
  - All database queries run inside `tokio::task::spawn_blocking` with connections checked out from the `r2d2` pool.
  - Concurrent cache reads execute simultaneously without mutex contention.

### 3. Article Content Extractor (`crates/server/src/ai/extractor.rs`)
- **Requirement**: Parse `<article>` tag content (extract body within `<article>`, strip inner tags/scripts/nav, falling back to body/teaser if `<article>` tag is absent).
- **Extraction Flow**:
  1. If `url` is empty or invalid, return `title` + `teaser`.
  2. Perform HTTP GET with `reqwest::Client` (3-second timeout, `User-Agent: Mozilla/5.0 SpatialRSS/1.0`).
  3. If HTTP status is not 200 (403 Forbidden, 429, Cloudflare challenge, paywall) or network times out: immediately return `title` + `miniflux_client::summary::extract_summary(teaser, 2)`.
  4. On HTTP 200: parse DOM using `scraper::Html`.
  5. Search for `<article>` element:
     - If found: extract inner text, stripping `<script>`, `<style>`, `<nav>`, `<header>`, `<footer>`, `<aside>`.
     - If absent or extracted text < 50 chars: fall back to `<body>` text or `teaser`.
  6. Collapse excessive whitespace, prepend `Title: {title}\n\n`, and truncate to first 500 words.

### 4. LLM CLI Runner (`crates/server/src/ai/llm.rs`)
- **Execution**: Spawns CLI subprocess using `tokio::process::Command`.
- **Prompt Structure**:
  ```
  Summarize what changed or why this news matters in 1 to 2 concise sentences (strictly under 35 words). Do not include introductory filler.

  Article Title: {title}
  Article Content:
  {content}
  ```
- **CLI Command**:
  ```bash
  {LLM_CLI_CMD} -m {LLM_MODEL} {LLM_EXTRA_ARGS} "{prompt}"
  ```
- **Safeguards**:
  - `kill_on_drop(true)` on child process.
  - `tokio::time::timeout(Duration::from_secs(5))` wraps execution.
  - Concurrency capped via `Arc<tokio::sync::Semaphore>` (2 permits).
  - On CLI failure, exit code != 0, or timeout: fall back to `miniflux_client::summary::extract_summary(content, 2)` and log model as `"extractive-fallback"`.

### 5. UI Templates & HTMX Interactivity (`crates/ui-components`)
- **Filter Bar (`templates/pages/index.html`)**:
  - Render `[AI]` filter button before `[Unread]` when `ai_enabled` is true:
    ```html
    <button 
      id="filter-ai"
      data-status="ai"
      hx-get="/hx/entries?status=ai" 
      hx-target="#entries-list" 
      hx-swap="innerHTML" 
      hx-include="[name='search']"
      onclick="setActiveFilter(this)"
      class="filter-pill px-3 py-1 rounded-full text-xs font-medium bg-white/5 text-slate-400 hover:text-white border border-white/10 transition-colors flex items-center gap-1.5">
      <i data-lucide="sparkles" class="w-3.5 h-3.5 text-indigo-400"></i>
      AI
    </button>
    ```
- **Grouped Summary List Partial (`templates/partials/ai_grouped_list.html`)**:
  - Renders up to 15 articles grouped by feed title (`Vec<FeedSummaryGroupViewModel>`).
  - Glass cards styled with `ar-glass-panel rounded-xl border border-white/10`.
  - Lazy summary placeholder with canonical REST route:
    ```html
    <div class="ai-summary-slot mt-3 pt-3 border-t border-white/5"
         hx-get="/hx/entries/{{ item.id }}/ai-summary"
         hx-trigger="revealed"
         hx-swap="outerHTML">
      <div class="skeleton-pulse text-xs text-indigo-300/60 flex items-center gap-2 py-1">
        <i data-lucide="sparkles" class="w-3.5 h-3.5 animate-spin"></i>
        <span>Generating brief...</span>
      </div>
    </div>
    ```
- **Summary Result Partial (`templates/partials/ai_summary_card.html`)**:
  - Replaces `.ai-summary-slot` with `.ai-summary-card`:
    ```html
    <div class="ai-summary-card mt-3 pt-3 border-t border-white/5">
      <div class="text-xs leading-relaxed text-slate-300 flex items-start gap-2">
        <i data-lucide="sparkles" class="w-3.5 h-3.5 text-indigo-400 shrink-0 mt-0.5"></i>
        <p class="font-normal">{{ summary }}</p>
      </div>
    </div>
    ```
- **Guaranteed Replacement on Failure**:
  - If scraping or LLM fails, the endpoint returns `AiSummaryCardTemplate` with extractive summary (HTTP 200).
  - If entry does not exist or cannot be accessed, returns `<div class="ai-summary-card text-xs text-slate-500 italic mt-3 pt-3 border-t border-white/5">Summary unavailable</div>` (HTTP 200) so the outerHTML swap completes and never freezes the loading spinner.

### 6. HTTP & HTMX Endpoints (`crates/server/src/routes/entries.rs`)
- **`GET /hx/entries?status=ai` (Batch Digest List)**:
  - **Authentication**: Requires `_user: AuthenticatedUser`. Redirects to `/login` if unauthenticated.
  - **Feature Gate**: If `!config.ai_enabled`, returns `handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()))`.
  - **Query Forwarding**: Checks `query.search`. If present and non-empty, appends `search: "<escaped_query>"` to GraphQL query filter. Supports optional `feed_id` and `category_id`.
  - **Execution**: Fetches up to 15 latest entries (`limit: 15`), groups them by feed source into `Vec<FeedSummaryGroupViewModel>`, and renders `AiGroupedListTemplate`.
- **`GET /hx/entries/{id}/ai-summary` (Canonical REST Lazy-Load)**:
  - **Authentication & Multi-User Isolation**: Requires `_user: AuthenticatedUser`.
  - **Ownership Verification**: Queries Miniflux/GraphQL for entry `{id}` under the user's session.
    - If entry does not exist or does not belong to the user: returns an empty/unavailable summary replacement card with HTTP 200 without reading local cache or executing LLM.
  - **Summary Generation**:
    - Calls `AppState.ai_service.get_or_create_summary(id, &url, &title, &content)`.
    - Returns HTTP 200 with `AiSummaryCardTemplate` ensuring the `outerHTML` swap succeeds.

---

## Proposed Has (parent applies to card)
- **Has UI**: yes (Filter button, grouped cards, HTMX revealed slots, pulsing skeletons)
- **Has API**: yes (`/hx/entries?status=ai`, `/hx/entries/{id}/ai-summary`)
- **Has DB**: yes (SQLite persistence for `article_ai_summaries`, WAL mode, connection pool)
- **Has auth/PII/schema**: yes (Authenticated endpoints, config schema, SQLite table schema & versioning)
