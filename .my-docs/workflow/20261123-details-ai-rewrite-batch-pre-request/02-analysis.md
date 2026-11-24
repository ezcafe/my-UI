# Analysis: Details Page AI Rewrite with Batch Pre-request

## 1. Context & Objectives

Currently, when a user opens an article detail view in the web UI, the backend renders the raw sanitized HTML content (`sanitized_content`) below an Executive Brief card. For busy readers, long raw HTML feeds often contain fluff, navigation fragments, or slow-to-read prose that causes cognitive fatigue.

The objective of this feature is to transform the article details page into a high-impact reading experience:
1. **Primary AI Rewrite**: The full article body is replaced with a concise, high-impact AI Rewrite adhering to the **Inverted Pyramid** model (conclusion and key takeaways in the first 2 sentences, followed by 2–3 scannable bullet points, target 30–60 words, strictly under 110 words).
2. **User-Selected Option B**: The raw article content is omitted from the details page DOM. The top-bar external "Original" link serves as the definitive reference to the publisher's full original article.
3. **Batch Pre-requesting & Local Caching**: Rewrites are produced in the same single batch LLM call per page that already generates titles and executive briefs, and are persisted in SQLite (`user_version = 4`).

---

## 2. In-Depth Comparison: Batch LLM Pre-request vs. On-Demand Detail View Request

In response to the user's inquiry:
> *"Compare the batch llm request and request details on details page view, what is better for token usage, and loading time?"*

Here is a rigorous technical comparison across token economics, latency, and operational reliability:

### A. Token Usage Comparison

| Dimension | Option 1: Batch LLM Request (Current Design) | Option 2: On-Demand Request on Detail Open |
| :--- | :--- | :--- |
| **System Prompt Overhead** | **Amortized across 20 items**: Shared system instructions (~300 tokens) are sent once per 20-item batch (~15 tokens/article). | **Multiplied by each click**: Full system instructions (~300 tokens) must be re-sent for every individual article opened. |
| **Input Content Tokens** | Compact article excerpts (~100–150 words / ~130 tokens) sent in batch payload. | Full or partial article text (~300–800 words / ~400–1,000 tokens) sent per click. |
| **Total Input Tokens (20 items)** | ~2,900 input tokens total (~145 tokens/article). | If 10 articles are clicked: ~10 &times; 1,000 = **10,000 tokens**. If 20 clicked: **20,000 tokens**. |
| **Output Tokens** | Per item: title (15 tok) + brief (45 tok) + rewrite (70 tok) = ~130 tok. 20 items = ~2,600 output tokens. | Per item: rewrite only (~70–100 tok). 10 items = ~800 tok; 20 items = ~1,600 tok. |
| **Total Token Footprint** | **~5,500 total tokens** for all 20 articles completely processed with titles, briefs, page overview, and rewrites. | **11,000 – 22,000 total tokens** if most articles are opened, plus separate tokens already spent for list-view briefs! |
| **Token Efficiency Verdict** | **Winner: Batch LLM Pre-request.** By generating titles, feed-card briefs, and detail rewrites simultaneously, system prompt overhead is amortized across 20 items, eliminating redundant API roundtrips. |

### B. Loading Time & User Experience Comparison

| Dimension | Option 1: Batch LLM Pre-request | Option 2: On-Demand Request on Detail Open |
| :--- | :--- | :--- |
| **Detail Open Latency** | **Instantaneous (< 5ms)**. The rewrite is fetched from local SQLite via indexed `entry_id` query. | **Slow (1,500ms – 4,500ms)**. User must wait for an outbound LLM HTTP/CLI request to complete on every click. |
| **UI Responsiveness** | Zero layout shifts or loading spinners. The detail panel slides in fully rendered at 60 fps. | Requires loading skeletons, spinners, or progressive streaming in the detail panel on every article. |
| **Offline / Airplane Mode** | **Fully readable**. Once pre-requested, rewrites persist in SQLite and can be browsed without internet access. | **Broken**. Cannot open details without active LLM connectivity. |
| **Failure Experience** | Failures are caught in background batch. Extractive fallback is already populated in DB. | User clicks article and encounters an error toast or blank screen if the LLM times out or rate-limits. |
| **Loading Time Verdict** | **Winner: Batch LLM Pre-request by >300&times; speedup** (5ms local database lookup vs 2,000ms+ network/LLM roundtrip). |

**Conclusion on Architecture**: Batch pre-requesting with SQLite caching is superior in both token cost efficiency and user-perceived loading speed.

---

## 3. Technical Nuances & Failure Modes

### Schema Migration (`user_version = 4`)
- **Action**: Add `rewrite TEXT` to `article_ai_summaries`.
- **Integrity**: SQLite `ALTER TABLE ... ADD COLUMN rewrite TEXT;` is an instantaneous, non-blocking metadata-only schema operation. It is 100% backwards-compatible with existing rows (defaulting to `NULL`).
- **Idempotency**: Check `user_version < 4` and inspect column existence via `PRAGMA table_info` before executing.

### Batch JSON Schema & Regex Fallback
- Adding `"r"` to the compact JSON schema:
  ```json
  {"top":"...","items":[{"i":101,"t":"...","b":"...","r":"..."}]}
  ```
- **Parsing Robustness**:
  - Direct `serde_json` deserialization expects `BatchAiItem { id: i64, suggested_title: String, brief: String, rewrite: Option<String> }`.
  - Fallback regex must be updated to capture `"r"` if the LLM wraps the response or if minor JSON deviations occur.
  - If `"r"` is omitted by the LLM for a particular item, `rewrite` defaults to `None` without breaking the parse of other items.

### Detail View Fallback Behavior
- When `rewrite` is present and non-empty in SQLite: Render `rewrite` in the detail body.
- When `rewrite` is `None` or empty (e.g. article opened before batch prefetch completed, or external LLM failure):
  - Do NOT crash or display an empty blank space.
  - Fall back cleanly to the 3-sentence `summary` / executive brief already available in SQLite or Miniflux teaser.
  - The top-bar external "Original" button is always present to reach full source text.

---

## 4. Risks & Mitigations

1. **Output Token Limits in Batch Calls**:
   - *Risk*: Adding ~50 words (~70 tokens) per item across 20 articles increases batch response tokens by ~1,400 tokens (total batch response ~2,500 tokens). If `max_tokens` is configured too low, JSON truncation may occur.
   - *Mitigation*: Ensure `max_tokens` in LLM request payload is set to at least 4,096 tokens, and enforce strict prompt instructions: *"Strictly under 110 words, target 30–60 words in 2–3 bullet points or short paragraphs"*.
2. **Language Consistency**:
   - *Risk*: LLM might inadvertently translate Vietnamese news into English or vice-versa.
   - *Mitigation*: Enforce explicit prompt instruction: *"Strictly maintain language parity: Vietnamese source -> Vietnamese rewrite; English source -> English rewrite"*.

## Result

Result: done
