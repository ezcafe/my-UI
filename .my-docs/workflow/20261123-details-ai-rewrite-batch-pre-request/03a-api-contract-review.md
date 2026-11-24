Result: clean

# API Contract Review: Details Page AI Rewrite with Batch Pre-request

**Date:** 2026-11-23  
**Workflow Slug:** `20261123-details-ai-rewrite-batch-pre-request`  
**Review Stage:** `api-contract-review`  
**Target Design:** `.my-docs/workflow/20261123-details-ai-rewrite-batch-pre-request/03-design.md`  

---

## 1. Executive Summary

This review assesses the API contract specifications outlined in `03-design.md` for generating, caching, and serving article-level AI rewrites (`r`) alongside executive titles (`t`), feed-card briefs (`b`), and page overview (`top`).

- **Review Verdict:** `Result: clean`.
- **Summary:** The proposed API contracts across LLM structured output, word truncation, fallback cascades, and Axum/HTMX route integration are architecturally robust, backwards-compatible, and aligned with system throughput and latency requirements. Four non-blocking implementation hardening recommendations are noted below.

---

## 2. API Contract Evaluation

### 2.1 Batch LLM JSON Schema (`top`, `items` with `i`, `t`, `b`, `r`)

- **Schema Definition:**
  ```json
  {
    "top": "Page-level executive overview synthesizing major news...",
    "items": [
      {
        "i": 101,
        "t": "Short punchy title strictly in one sentence",
        "b": "Executive brief strictly in 3 sentences.",
        "r": "Conclusion and primary impact in first two sentences.\n\n• Supporting detail or key figure.\n• Implication or forward-looking context."
      }
    ]
  }
  ```
- **Rust Serde Representation:**
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
  pub struct BatchAiItem {
      #[serde(rename = "i")]
      pub id: i64,
      #[serde(rename = "t")]
      pub suggested_title: String,
      #[serde(rename = "b")]
      pub brief: String,
      #[serde(default, rename = "r")]
      pub rewrite: Option<String>,
  }

  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
  pub struct BatchAiResponse {
      #[serde(default)]
      pub top: String,
      #[serde(default)]
      pub items: Vec<BatchAiItem>,
  }
  ```
- **Contract Verification:**
  - **Backwards Compatibility:** Using `#[serde(default, rename = "r")]` allows graceful ingestion of legacy responses or LLM responses where `"r"` may be omitted or empty without throwing deserialization errors.
  - **Prompt Clarity:** The prompt instructions explicitly enforce inverted pyramid formatting (first 2 sentences for key outcome/impact, followed by 1–2 bullet points for context/data), strict length boundary (< 110 words, sweet spot 30–60 words), and strict multilingual parity (Vietnamese source &rarr; Vietnamese rewrite; English source &rarr; English rewrite).
  - **Compact Token Budget:** By using single-letter JSON keys (`i`, `t`, `b`, `r`), JSON structural overhead is capped at ~25 tokens per item.

### 2.2 Input Truncation Contract (300 Words Cap)

- **Target Location:** Input extraction pipeline in `crates/server/src/ai/prefetch.rs` / `ArticleExtractor`.
- **Rule:** Input content for each article candidate is capped at the first 300 words before passing to `format_batch_prompt_structured`.
- **Contract Verification:**
  - **Context Sufficiency:** 300 words provides roughly 1,200–1,500 characters of leading context, capturing the headline, lede, and core facts necessary for an executive inverted-pyramid rewrite.
  - **Token Cap Enforcement:** Capping at 300 words per article guarantees that a 20-item batch prompt stays within ~6,000–7,000 input tokens, safely below LLM context window limits and CLI `ARG_MAX` buffer thresholds.
  - **Fallback to Teaser:** When full extracted content is unavailable or shorter than 50 words, falling back to the RSS teaser ensures prompt input validity.

### 2.3 Fallback Contract (Regex Fallback & Missing Rewrite Cascades)

- **Regex Fallback:**
  ```rust
  static ref ITEM_REGEX: Regex = Regex::new(
      r#"\{[^{}]*?"i"\s*:\s*(\d+)[^{}]*?"t"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"[^{}]*?"b"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"(?:[^{}]*?"r"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)")?[^{}]*?\}"#
  ).expect("valid regex");
  ```
  - **Contract Verification:**
    - Step 1 parses standard JSON (`serde_json::from_str`).
    - Step 2 unwraps markdown fences (````json ... ````).
    - Step 3 captures regex matches even if trailing or wrapping JSON structure is malformed.
    - Capturing group 4 (`"r"`) is optional `(?: ... )?`, ensuring items without `"r"` continue to extract valid titles and briefs.
- **UI Fallback Cascade (Option 1A):**
  - If `rewrite` is available in SQLite: render `rewrite` as the primary article body (`is_rewrite = true`).
  - If `rewrite` is pending (cache miss, in-flight prefetch, or LLM omission): render the executive brief (`summary`) in `brief_card.html` or fallback teaser (`is_rewrite = false`).
  - The user never encounters a blank screen, blocking spinner, or broken UI.

### 2.4 Server Endpoint & Prefetch Integration (`GET /hx/entries/{id}`, `AiPrefetchService`)

- **Route Handler (`GET /hx/entries/{id}`):**
  - Handler checks `ai_service.db.get_summary_and_title(id).await`.
  - Reads `record.rewrite` in `< 5ms` from local SQLite without blocking on upstream networks.
  - Passes `rewrite`, `is_rewrite`, and `summary` into `EntryDetailTemplate`.
- **Prefetch Service (`AiPrefetchService`):**
  - Background task triggers on page load for uncached entries.
  - Unprocessed entry filtering (`get_unprocessed_entry_ids`) skips already completed items.
  - Concurrency lock on `batch_key` prevents thundering-herd duplication.
  - Atomic persistence via `save_batch_results` stores `top`, `t`, `b`, and `r` in a single SQLite transaction.

---

## 3. Findings & Implementation Hardening Recommendations

### Critical / Major Issues
- **None (0 findings).**

### Minor Implementation Recommendations (Non-blocking)

1. **Retain `pub summary: String` on `EntryDetailTemplate`:**
   - *Detail:* In `03-design.md` Section 2.F, the template struct snippet omitted `pub summary: String`. Because `entry_detail.html` includes `brief_card.html` when `!is_rewrite`, and `brief_card.html` references `{{ summary }}`, Askama template compilation requires `summary` as a field on `EntryDetailTemplate`:
     ```rust
     pub struct EntryDetailTemplate {
         pub entry: EntryViewModel,
         pub summary: String,
         pub rewrite: String,
         pub is_rewrite: bool,
         pub auth_enabled: bool,
     }
     ```
   - *Action:* Retain `pub summary: String` during implementation in `crates/ui-components/src/templates.rs`.

2. **Stateful Word Boundary Tracking in `truncate_words`:**
   - *Detail:* A naive check incrementing on every `ch.is_whitespace()` will count consecutive spaces, tabs, or `\n\n` separators as multiple words.
   - *Action:* Use a stateful transition (`in_word: bool`) or `split_whitespace()` boundary scanning so consecutive whitespace characters do not prematurely shorten the 300-word budget.

3. **Defensive Guard on Brief Card Inclusion:**
   - *Detail:* In `entry_detail.html`, guard the brief card with `{% if !is_rewrite && !summary.is_empty() %}` so that if both `rewrite` and `summary` happen to be empty, an empty executive card is not rendered.

4. **Sanitize Whitespace-only Rewrites:**
   - *Detail:* When extracting `record.rewrite`, check `record.rewrite.as_deref().filter(|s| !s.trim().is_empty())` to guarantee that whitespace-only strings are treated as `None`, correctly triggering the fallback path.

---

## 4. Conclusion & Next Steps

The API contract in `03-design.md` meets all functional, structural, and resilience criteria. Zero Critical or Major issues exist.

- **Stage Verdict:** `Result: clean`
- **Next Stage:** Proceed with parallel isolated sibling review (`db-design-review`).
