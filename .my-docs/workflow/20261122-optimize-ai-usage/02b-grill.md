# Grill: AI Optimization & Presentation

## Round 1 — Frontier Questions

❓ **Q1** - **List Page Top Executive Brief**: Should the top "Executive Briefs" card on the list page synthesize the top unread article's brief (or top 3 highlights) using cached summaries, falling back to a clean placeholder if no summaries exist yet?
- **Option A (Recommended)**: Yes. Display the top highlighted brief from currently cached unread items with a link to that article. This ensures zero latency overhead on list rendering and requires no extra LLM calls.
- **Option B**: Perform a separate batch LLM call to synthesize the whole page of 15–50 articles into a single multi-article digest. (Cons: Adds 1–3s latency to list loads and consumes extra tokens).

➡️ **Yes (Option A)**: Display top highlights from cached item briefs with zero latency impact.

---

❓ **Q2** - **List Row Title Population Strategy**: Should list rows use non-blocking cache lookups (showing AI-suggested titles when already cached in SQLite, and original titles otherwise), with full generation happening when opening articles or via background batching?
- **Option A (Recommended)**: Yes. Keep list loads instant (sub-50ms) by reading `suggested_title` directly from SQLite via batch query. If an item has not been summarized yet, display the original title seamlessly.
- **Option B**: Block the list page rendering until all visible articles have their AI titles generated via CLI LLM. (Cons: Extremely slow, 5–15s delay on list view).

➡️ **Yes (Option A)**: Instant list loads with graceful fallback to original title until summarized.

---

## Frontier Answers (Round 1)

1. **Q1: Top Executive Brief & Item Briefs Generation**:
   - **Answer**: Single LLM Call for items & Top Executive Brief.
   - **Design**: Micro-batch prompt generates top page-level synthesis + item titles + item briefs in a single call with compact JSON output (`top`, `items` with `i`, `t`, `b`).

2. **Q2: Timing & Prefetching Strategy**:
   - **Answer**: Option A (Background prefetch worker with on-demand fallback).
   - **Design**: Background worker periodically fetches latest unread entries, processes them in micro-batches (10–15 items) and caches into `AiDb`. List view reads instant cache lookups (<30ms) with original title fallback. Detail view triggers on-demand generation if uncached.

---

## Result

Result: frontier-empty
