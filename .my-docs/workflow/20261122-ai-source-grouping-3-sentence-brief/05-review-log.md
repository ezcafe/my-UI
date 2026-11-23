# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

---

## Adversarial

**Result:** clean

### 1. Attack Surface & Threat Analysis

- **Subprocess & Prompt Injection**:
  - `LlmCliRunner::format_batch_prompt_structured` streams the prompt directly into `child.stdin` (`Stdio::piped()`), dropping `stdin` immediately upon write completion to signal EOF.
  - Eliminates OS argument length limitations (`ARG_MAX`) and protects against prompt inspection in system process listings.
  - Subprocess execution uses `tokio::process::Command::new(&self.cli_cmd)` directly without shell wrappers (`sh -c`), preventing command injection.
  - `kill_on_drop(true)` with strict 90s timeout ensures no zombie or runaway processes linger on timeout or cancellation.
  - Multilingual directive prevents cross-language hallucination or translation artifacts.

- **Parser Resilience & Graceful Fallback**:
  - `parse_batch_response` extracts JSON enclosed in markdown code fences (` ```json `), strips trailing commas, and extracts structured items and top brief.
  - On empty, unparseable, or non-zero process exits, the runner gracefully falls back to `fallback_extractive_structured(articles)`.
  - Fallback strictly generates 3 concise sentences using `miniflux_client::summary::extract_summary(..., 3)`.

- **Concurrency & Deadlock Resilience**:
  - `InFlightGuard` in `executive_brief_htmx` employs RAII `Drop` semantics to remove the page key from the shared `in_flight` mutex set upon completion or error, preventing stalled in-flight entries.
  - `Semaphore(2)` limits concurrent CLI subprocess invocations, preventing CPU and memory starvation under burst load.

- **UI Click Event Isolation**:
  - Moving `onclick="openEntryDetail({{ entry.id }})"` to the outer `.annuaire-row` element and removing it from the inner child `<div>` eliminates nested handler invocation.
  - The external link button explicitly retains `onclick="event.stopPropagation(); markArticleRead({{ entry.id }})"`, ensuring link navigation does not accidentally trigger the detail drawer opening.

---

## Quality

**Result:** clean

### 1. Five-Axis Assessment

- **Correctness**:
  - Prompt formatters across single-article (`format_prompt`) and batch (`format_batch_prompt_structured`) mandate strictly 3 sentences.
  - Extractive fallbacks (`fallback_extractive_structured`) extract exactly 3 sentences.
  - Source grouping groups articles by `source` (`"General"` fallback) and sorts within each group by `published_at` descending (latest first).
  - Cache reuse: articles already in SQLite with `model != "extractive-fallback"` are passed as `(Existing Brief)` to guide page synthesis without redundant item generation.
  - Only uncached articles are persisted in SQLite, protecting high-quality summaries from accidental overwrite.
  - Outer row click target operates correctly in static templates and dynamic JS rendering (`index.html`).
  - Pagination navigation cleanly removed `border-t border-border`.
  - 179 unit, integration, and e2e tests pass; `cargo clippy --workspace --all-targets` is 100% clean with 0 warnings.

- **Readability**:
  - Clean abstractions: `BatchArticleInput<'a>` and `PrefetchCandidate` are strongly typed and well-documented.
  - Backward-compatible helper methods (`format_batch_prompt`, `summarize_batch`) ensure clean interoperability.
  - TDD tests in `llm.rs`, `entries.rs`, and `templates.rs` are explicit, readable, and assert both behavior and edge cases.

- **Architecture**:
  - Clean separation between route orchestration (`entries.rs`), LLM prompts/dispatch (`llm.rs`), prefetch worker (`prefetch.rs`), and UI templates (`ui-components`).
  - Single-pass hybrid batch architecture achieves 100% full-page synthesis with minimal token generation.

- **Security**:
  - SQL statements use parameterized binds (`params!`).
  - Subprocess piping prevents command-line expansion.
  - Secrets and tokens remain redacted.

- **Performance**:
  - Cache reuse eliminates redundant token generation for previously summarized articles.
  - Single LLM call per page view keeps latency minimal.
  - SQLite WAL mode and connection pooling ensure fast non-blocking reads and writes.

---

## Merged

**Result:** clean

### Summary of Completed Review Stages

1. **Adversarial Review**: `clean` (0 Critical, 0 Major defects)
2. **Quality Review**: `clean` (0 Critical, 0 Major defects)
3. **Lens: API**: `clean` (0 Critical, 0 Major defects; see `05-lens-api.md`)
4. **Lens: DB**: `clean` (0 Critical, 0 Major defects; see `05-lens-db.md`)

All review axes and conditional lenses are clean. No defects require fixing.
