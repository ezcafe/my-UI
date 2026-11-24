Result: clean

# API Contract Review: Batch Rewrite & Decoupled Detail Analysis

- **Workflow Slug:** `20261124-batch-rewrite-and-detail-analysis`
- **Review Stage:** `api-contract-review`

---

## 1. Executive Summary

This review assesses the API contract changes in `03-design.md`:
1. Batch LLM response schema: replacing brief `"b"` with inverted-pyramid rewrite `"r"`.
2. Asynchronous detail analysis endpoint: `GET /hx/entries/{id}/analysis` returning HTML fragment of impact and fallacy analysis.

- **Review Verdict:** `Result: clean`.

---

## 2. API Contract Evaluation

### 2.1 Batch LLM Contract
- Schema:
  ```json
  {
    "top": "Page synthesis brief...",
    "items": [
      {
        "i": 101,
        "t": "Punchy headline in one sentence",
        "r": "Inverted pyramid rewrite: lead with outcome and impacts in first 2 sentences, followed by 2-3 bullet points."
      }
    ]
  }
  ```
- Serde mapping:
  `#[serde(rename = "r", alias = "b")] pub rewrite: String`
  Using `alias = "b"` ensures that if any legacy response provides `"b"`, deserialization succeeds without error.
- Backward compatibility:
  Extractive fallback sets `rewrite` to extracted lead content if LLM fails.

### 2.2 Dedicated Detail Analysis Contract
- Route: `GET /hx/entries/{id}/analysis`
- Request parameters: `id: i64` (path)
- Response: HTML fragment (`text/html`) containing:
  - If analysis present:
    ```html
    <div id="ai-analysis-box" class="space-y-6">
      <div class="rounded-lg p-4 bg-muted/40 border border-border">...Impact...</div>
      <div class="rounded-lg p-4 bg-muted/40 border border-border">...Fallacies...</div>
    </div>
    ```
  - If LLM analysis fails: graceful fallback card or hidden box so page is not broken.
- In-flight deduplication:
  Guarded by `AiService.in_flight` key `analysis:{id}` to prevent duplicate LLM calls on rapid clicks.

## Result

Result: clean
