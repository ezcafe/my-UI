# Gate A: Framing & Scope Review

## Evaluation

- **Clarity of Goal**: Clear and explicit.
  - Replace the raw article body on the details page with a high-impact, concise AI Rewrite.
  - Follow strict writing standards: Inverted pyramid structure (impact and conclusion in the first 2 sentences), 30–60 words target (under 110 words max), and language parity (Vietnamese source &rarr; Vietnamese rewrite, English &rarr; English).
  - Integrate rewrite generation directly into the existing batch LLM prompt and compact JSON schema (`"r"` field) alongside titles (`"t"`) and briefs (`"b"`), avoiding redundant LLM requests.
  - User explicitly confirmed **Option B** (complete replacement in detail view, external link only for original content).
  - Pre-request and persist batch rewrites in SQLite via database migration (`user_version = 4`), enabling sub-5ms local loading when opening the detail view.
- **Scope & Boundaries**:
  - Primary detail view replaces raw content with `rewrite`.
  - Non-goals: No alteration to feed list layout or main page executive cards; no interactive per-article rewriting chat.
  - Anti-goals: No blocking LLM calls on detail view click; no translation across different languages.
- **Feasibility & Constraints**:
  - SQLite schema migration: `ALTER TABLE article_ai_summaries ADD COLUMN rewrite TEXT;` bumped to `user_version = 4`.
  - Token bounds: Adding ~50 words per item across a 20-item batch adds ~1,000–1,300 output tokens. Max timeout (90s) and token allowances are sufficient.
  - UI styling: Adheres to Annuaire theme typography (`annuaire-theme.css`, Work Sans, tabular figures, dark mode support).
- **Risks & Mitigations**:
  - Latency / Timeout in LLM: Mitigated by background prefetch and fallback extraction.
  - Cache misses on instant click: Handled by graceful fallback to existing summary/brief.
  - JSON parsing with additional field: Supported by `serde_json` and updated regex fallback parser.

## Result

Result: ok
