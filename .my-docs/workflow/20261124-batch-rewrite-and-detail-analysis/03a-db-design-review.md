Result: clean

# DB Design Review: Batch Rewrite & Decoupled Detail Analysis

- **Workflow Slug:** `20261124-batch-rewrite-and-detail-analysis`
- **Review Stage:** `db-design-review`

---

## 1. Executive Summary

This review assesses the database operations for storing batch rewrites and on-demand analysis results:
- Storage table: `article_ai_summaries`
- Columns involved: `entry_id` (PK), `summary` (TEXT NOT NULL), `suggested_title` (TEXT), `rewrite` (TEXT), `impact` (TEXT), `fallacies` (TEXT), `model` (TEXT NOT NULL), `created_at` (TIMESTAMPTZ).

- **Review Verdict:** `Result: clean`.

---

## 2. DB Design Evaluation

### 2.1 Table Schema & Constraints
- Column `summary`: Has `NOT NULL` constraint.
  - When batch items are inserted via `save_batch_summaries`, we store `item.rewrite` into both `summary` and `rewrite`.
  - This satisfies the NOT NULL constraint and ensures list views (which read `summary`) stay updated with the latest rewrite text.
- Columns `impact` and `fallacies`: Already exist as `TEXT` (added via `ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;` etc.).
  - Default to `NULL` on batch creation.
  - Updated via `save_analysis(entry_id, impact, fallacies)` when on-demand analysis finishes.
- Concurrency & Upserts:
  - `save_analysis` executes:
    ```sql
    UPDATE article_ai_summaries
    SET impact = $2, fallacies = $3
    WHERE entry_id = $1
    ```
    This is safe, idempotent, and non-blocking.
- Mock implementation:
  - In `AiDb::new_mock()`, update in-memory `HashMap` to store `rewrite`, `impact`, and `fallacies`.

## Result

Result: clean
