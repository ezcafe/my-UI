# Grill: Stress-testing Decisions

**Result:** frontier-empty

---

## 1. Decision Status

- **Executive Brief Response Cleanup**:
  - Settled: Remove duplicate markup in `executive_brief_card.html`. The endpoint will return a single clean executive brief card. In the client, `applyAiViewMode` or the swap event handles keeping views in sync.
- **Immediate Detail Rendering**:
  - Settled: Fix `index.html` cache validation (remove obsolete `'Executive Brief'` string requirement) and pass existing DOM title / brief into the opening skeleton so the title and brief are displayed immediately without waiting for the network.
- **Database Reuse for Analysis**:
  - Settled: In `get_entry_analysis_htmx` and `AiService`, check for existing analysis in the database before calling the LLM. If either impact or fallacies exists, reuse it and do not call the LLM.
- **Prompt Guidelines**:
  - Settled: Word count strictly under 110 words, sweet spot 30–60 words, 2–3 bullets, inverted pyramid structure (lead with outcome and impact in first 2 sentences), and plain language without opening or closing conversational boilerplate.

---

## 2. Frontier

- No open decisions remain. The user has provided explicit requirements and instruction to proceed.
- **Grill status:** `frontier-empty` (Grill closed).
