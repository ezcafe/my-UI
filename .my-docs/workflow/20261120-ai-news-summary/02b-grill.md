# Grill Log: Decision Tree & Frontier Analysis

**Result:** frontier-empty

## Decision Tree & Resolution

All primary product and architectural decisions have been systematically evaluated and resolved through prior user confirmations, constraints, and codebase exploration:

### 1. UI Presentation & Layout
- **Decision**: How should the batch of 15 news items grouped by feed source be displayed?
- **Resolution**: **Option A: Feed-Grouped Glass Cards with HTMX Viewport Revealing**. Grouped under distinct feed headers with spatial glass styling (`ar-glass-panel`), loading AI summaries on scroll via `hx-trigger="revealed"`.
- **Status**: **Settled** (Explicit user selection in `01-idea.md`).

### 2. Filter Bar Integration & Placement
- **Decision**: Where should the `[AI]` button be placed, and how does it behave when AI is disabled?
- **Resolution**: Place `[AI]` immediately before `[Unread]` in the filter bar (`[AI]`, `[Unread]`, `[Today]`, `[Starred]`, `[All Titles]`). When `AI_ENABLED=false`, the button is hidden and endpoints are inactive.
- **Status**: **Settled** (Explicit user selection in `01-idea.md`).

### 3. Content Extraction Pipeline
- **Decision**: How should full article content be extracted and cleaned?
- **Resolution**: Parse `<article>` tag content (extract body within `<article>`, strip inner tags/scripts/nav, falling back to body/teaser if `<article>` tag is absent). Truncate to Title + first 400–600 words.
- **Status**: **Settled** (Explicit user requirement in ideation provenance: `"Update \"Strip HTML boilerplate (tags, nav, ads, footers)\" to \"Parse <artical> tag content\""`).

### 4. Scraper Failure & Anti-Bot Fallbacks
- **Decision**: What happens if an external article URL returns HTTP 403, Cloudflare challenge, paywall, or times out?
- **Resolution**: Immediately fall back to RSS Title + excerpt summary. Never block, stall, or throw errors in the UI.
- **Status**: **Settled** (Mandatory constraint in `01-idea.md`).

### 5. LLM CLI Execution & Prompt Specification
- **Decision**: How is the summary generated, and with what prompt/budget?
- **Resolution**: Fast lightweight model executed via CLI (`LLM_CLI_CMD` / `LLM_MODEL`) with thinking/reasoning budget set to 0. Constraint: 1–2 sentences under 35 words focusing on what changed or why it matters. Wrapped in a 5-second timeout with fallback to extractive summary.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

### 6. Persistence Store & Re-Summarization Prevention
- **Decision**: How are summaries stored across users and reloads?
- **Resolution**: Bundled SQLite table `article_ai_summaries(entry_id INTEGER PRIMARY KEY, summary TEXT NOT NULL, model TEXT NOT NULL, created_at TEXT NOT NULL)`. Check cache before remote fetch or LLM CLI.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

### 7. Viewport Throttling & Concurrency
- **Decision**: How to prevent a flood of 15 simultaneous LLM CLI spawns during rapid scrolling?
- **Resolution**: Tokio `Semaphore(2)` in server state to serialize/throttle active CLI executions, while SQLite cache hits return instantly without acquiring semaphore permits.
- **Status**: **Settled** (Engineering best practice; only one viable path).

---

## Frontier Status
- **Open questions**: None.
- **Frontier state**: **Empty**. All branches of the decision tree are fully resolved and agreed upon.
