# Gate A: Framing Review

**Result:** ok

---

## 1. Scope and Clarity Review

- **Executive Brief Duplication**: `/hx/executive-brief` returns duplicate markup (`#page-executive-brief` and `#list-executive-brief`). The endpoint response and `executive_brief_card.html` will be cleaned up so only a single brief card is returned.
- **Instant Detail Title & Brief from Cache**: `openEntryDetail(id)` currently fails cache validation for AI articles and displays empty gray skeleton boxes. We will fix cache validation and populate the title and brief immediately from available DOM elements or client cache upon opening.
- **Database Reuse for Analysis**: `get_entry_analysis_htmx` currently falls through to LLM generation if either impact or fallacies is missing. We will ensure cached impact and fallacies are reused directly from the database whenever present, avoiding redundant LLM calls.
- **Rewrite Rules in Prompts**: Update single and batch rewrite prompts to enforce concise inverted pyramid structure (first 2 sentences lead with outcome and impacts, 2-3 bullets, under 110 words, 30-60 sweet spot, no opening/closing boilerplate).

## 2. Verdict

- Scope is clear, bounded, and directly satisfies user requirements.
- No schema changes or external infrastructure migrations required.
- **Gate A status:** `ok`
