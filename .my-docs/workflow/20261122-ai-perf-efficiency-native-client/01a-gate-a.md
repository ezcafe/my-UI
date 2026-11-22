# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear and explicit.
  - Optimize AI processing efficiency and pipeline throughput across the system.
  - Implement fast-path skipping of external HTTP scraping when RSS teaser has $\ge 50$ words.
  - Lower word truncation limit from 250 words down to 100 words and eliminate duplicate `Title: ...` formatting.
  - Use standard input (`stdin`) piping for `LlmCliRunner` rather than shell arguments.
  - Correct prefetch storage to save page-level executive briefs with the deterministic `page_key`.
  - Synthesize page-level executive briefs from cached article summaries when all items on an unread page are already cached in SQLite.
  - Lower prefetch trigger threshold from 5 uncached items to 1.
  - Introduce a native HTTP REST client (`reqwest`) supporting OpenAI-compatible and Gemini REST APIs with automatic fallback to `LlmCliRunner`.
- **Scope & Boundaries**:
  - Goals, non-goals, and anti-goals are explicitly defined in `01-idea.md`.
  - User confirmed the complete scope: "Both: All quick-wins (scraping fast-path, token reduction, cache fixes) + Native HTTP REST client (OpenAI & Gemini) with CLI fallback".
- **Feasibility & Constraints**:
  - `reqwest` is already present with `json` and `rustls-tls` features in `crates/server/Cargo.toml`.
  - Backward compatibility is preserved: users without API keys seamlessly continue using `LlmCliRunner`.
  - Environment variables (`LLM_API_KEY`, `LLM_API_BASE_URL`, `LLM_PROVIDER`) configure the HTTP client cleanly without logging secrets.
- **Risks & Mitigations**:
  - Provider REST API failures/timeouts: mitigated by 30s timeout and automatic fallback to `LlmCliRunner`.
  - Inconsistent JSON formatting: mitigated by requesting JSON schemas (`response_format` / `responseMimeType`) and existing robust regex parser fallbacks.

## Result

Result: ok
