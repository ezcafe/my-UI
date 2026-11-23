clean

# API Contract Review: AI Source Grouping, 3-Sentence Brief & UI Polish

**Stage:** `api-contract-review`  
**Status:** `clean`  
**Date:** 2026-11-23  

## 1. Executive Summary

The API contract review evaluated the interfaces, data types, query structures, and failure semantics defined in `03-design.md` across four architectural boundaries:
1. **Public HTTP / HTMX Endpoint** (`/hx/executive-brief`)
2. **Internal GraphQL Schema Queries** (`entriesResult` with `publishedAt` and `feed { title }`)
3. **Internal Rust AI Service Interfaces** (`BatchArticleInput`, `LlmBackend`, and `AiPrefetchService`)
4. **LLM Input / Output JSON Data Contract** (`BatchAiResponse`, `BatchAiItem`)

**Conclusion:** `clean` (Zero Critical or Major contract violations). The design is backward-compatible, robust against missing or malformed data, and preserves resilient failure contracts across client and server.

---

## 2. API Contract Analysis

### A. HTTP & HTMX Endpoint Contract (`/hx/executive-brief`)
- **Route & Parameters**: `GET /hx/executive-brief?status={status}&limit={limit}&...`
  - Maintains strict backwards compatibility with existing frontend triggers in `entry_list.html`.
- **Response Format**:
  - Primary HTML payload: Swaps into `#page-executive-brief`.
  - Out-Of-Band (OOB) updates: Generates `<span id="entry-title-{id}" hx-swap-oob="outerHTML">...</span>` for suggested titles.
  - Failure / Graceful Degradation: Returns status `200 OK` with `<div id="page-executive-brief" class="hidden"></div>`. This prevents client-side rendering breakage during LLM timeouts or empty feed states.
- **Contract Recommendation (Minor)**: Ensure OOB spans are emitted for all entries on the page that have suggested titles (both newly generated and cached in SQLite), preventing desync when client-side swaps occur.

### B. GraphQL Schema & Query Contract
- **Query Extension**:
  ```graphql
  entriesResult(filter: { ... }) {
    entries {
      id
      title
      summary
      url
      publishedAt
      feed {
        title
      }
    }
  }
  ```
- **Field Availability & Nullability**:
  - `published_at` (`publishedAt`) is defined on `EntryGql` as a `String` (RFC3339).
  - `feed` is defined on `EntryGql` as `Option<FeedGql>` with `title: String`.
- **Contract Recommendation (Minor)**: Route deserialization must defensively handle null/missing feeds (`feed: None`) by defaulting to `"Unknown Source"`, and gracefully handle invalid or missing dates when sorting descending.

### C. Internal Rust Service Contract (`crates/server/src/ai/`)
- **Strong Typing**: The introduction of `BatchArticleInput<'a>`:
  ```rust
  pub struct BatchArticleInput<'a> {
      pub id: i64,
      pub title: &'a str,
      pub source: &'a str,
      pub published_at: &'a str,
      pub content_or_brief: &'a str,
      pub is_cached: bool,
  }
  ```
  replaces brittle raw tuples `(i64, &str, &str)` with a self-documenting, strongly-typed contract.
- **Cross-Service Compatibility**:
  - `AiPrefetchService` in `crates/server/src/ai/prefetch.rs` also invokes `ai.llm.summarize_batch`.
  - To prevent compilation breakage or prefetch disruption, `BatchArticleInput` should provide a helper (e.g., `BatchArticleInput::new(...)` or `From` tuple) or default empty source/date fields when invoked from background prefetch routines.

### D. LLM Input / Output Contract
- **Prompt Specification**:
  - Strict 3-sentence constraint (`"b": executive brief, strictly 3 sentences (what changed, context, why it matters)`).
  - Explicit multilingual directive: Source language is strictly preserved for each item and synthesis.
- **Output Schema**:
  - `{"top": "...", "items": [{"i": 101, "t": "...", "b": "..."}]}`
  - Server-side filtering: The server filters `items` against the uncached article ID set before writing to SQLite, ensuring the contract is defensively enforced even if the LLM emits cached IDs.

---

## 3. Findings & Severity Assessment

| # | Item | Category | Severity | Resolution |
|---|------|----------|----------|------------|
| 1 | GraphQL `feed` and `publishedAt` null safety | Robustness | Minor | Fallback to `"Unknown Source"` and safe chronological sort in route parser |
| 2 | `AiPrefetchService` signature alignment | Internal API | Minor | Provide constructor / default mapping for `BatchArticleInput` |
| 3 | OOB suggested title span emission | UI Contract | Minor | Emit OOB spans for both newly generated and cached item titles on page |

**Result:** `clean` — Zero blocking Critical/Major issues. Proceed to DB design review and implementation.
