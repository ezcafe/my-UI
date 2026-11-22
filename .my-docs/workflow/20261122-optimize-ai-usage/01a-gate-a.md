# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear. Optimize AI token usage and latency in My News (`AI_ENABLED=true`), display an Executive Brief summary card at the top of the list page, use AI-suggested titles on list rows (no subtitles), show Executive Brief in details panel, add structured debug logging, and minimize token costs via combined single-call prompting, input truncation, and persistent SQLite caching.
- **Scope & Boundaries**:
  - Goals, non-goals, and anti-goals are explicitly documented.
  - Confirmed user decision: Option 1 (Top "Executive Briefs" summary card + AI-suggested title on list rows, no subtitle).
- **Feasibility & Constraints**:
  - Leverages existing `AiDb` (SQLite) and `LlmCliRunner`.
  - Backwards-compatible schema addition (`suggested_title` in `article_ai_summaries`).
  - Graceful fallback when AI is disabled or fails.
- **Risks**:
  - Model response formatting inconsistency handled via key-prefix lines (`TITLE:` and `BRIEF:`) with fallback.
  - Latency impact mitigated via persistent SQLite cache and truncation.

## Result

Result: ok
