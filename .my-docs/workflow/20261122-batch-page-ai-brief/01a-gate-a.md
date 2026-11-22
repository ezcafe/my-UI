# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear. Streamline AI operations in My News:
  - Multilingual fidelity: AI summaries and titles match the source article language.
  - Page-level batching: Exactly 1 LLM call per page (e.g., page 2 of 3) generating page summary, item briefs, and punchy suggested titles.
  - Loading skeleton: Display an Annuaire-compliant loading skeleton for the Executive Brief card while awaiting batch generation.
  - Dynamic updates: Instant Out-Of-Band (OOB) swap updates both the Executive Brief card and the list item titles when the batch returns (confirmed by user: Option 1).
  - Filter scoping: Executive Brief appears exclusively on `unread` feed. Completely hidden on `today`, `read`, `starred`, `all`, and search.
  - Zero-LLM detail views: Opening an article detail view never spawns a new LLM call, reusing cached briefs or falling back to the extractive teaser.
- **Scope & Boundaries**:
  - Goals, non-goals, and anti-goals are explicitly defined in `01-idea.md`.
  - User confirmed Option 1: Instant OOB Swap for summarized titles when the page batch LLM call returns.
- **Feasibility & Constraints**:
  - Leverages existing `AiDb`, `LlmCliRunner`, and HTMX architecture.
  - Backwards-compatible SQLite schema evolution (version 3 for keyed page executive briefs).
  - Clean fallbacks ensure zero UI disruption if AI is disabled or fails.
- **Risks & Mitigations**:
  - LLM format deviations: mitigated by robust regex parser with fallback to extractive teasers and original titles.
  - Page navigation latency: mitigated by async HTMX lazy-loading (`hx-get="/hx/executive-brief"`) and persistent caching.

## Result

Result: ok
