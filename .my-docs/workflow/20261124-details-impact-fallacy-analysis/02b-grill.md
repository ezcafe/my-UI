# Grill: Article Detail AI Rewrite with Impact & Fallacy Analysis

- **Stage:** grill
- **Status:** completed
- **Result:** `frontier-empty`

---

## 1. Design Tree

```
Article Detail AI Rewrite with Impact & Fallacy Analysis
├── LLM Prompt Contract & Schema
│   └── Option 1A: Strict JSON Schema (`rewrite`, `impact`, `fallacies`) [Settled by user]
├── Database Persistence Architecture
│   └── Option 2A: Dedicated columns (`impact TEXT`, `fallacies TEXT`) in PostgreSQL [Settled by user]
├── UI Presentation & Details Drawer Layout
│   ├── Remove raw content (`sanitized_content`) completely [Settled by user]
│   └── Option 3B: Two separate cards styled like brief [Settled by user]
└── Fallacy Detection Behavior
    └── Reassuring indicator when no fallacies detected [Settled by user]
```

---

## 2. Settled Decisions (Round 1)

### Q1 - LLM Prompt Contract & Schema
- **Decision:** **Option 1A (Strict JSON Schema)**.
- **Settled Details:**
  Prompt requests raw JSON:
  ```json
  {
    "rewrite": "...",
    "impact": "...",
    "fallacies": "..."
  }
  ```
  Parser uses dual-pass deserialization: direct `serde_json::from_str` then markdown fence stripping (` ```json ... ``` `), with safe fallback to prevent drawer rendering failure.

---

### Q2 - Storage Architecture in PostgreSQL
- **Decision:** **Option 2A (Dedicated Columns in `article_ai_summaries`)**.
- **Settled Details:**
  Schema updated via idempotent DDL in `AiDb::new`:
  ```sql
  ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;
  ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;
  ```
  `AiSummaryRecord` extended with `impact: Option<String>` and `fallacies: Option<String>`. Both PostgreSQL pool and in-memory mock store and retrieve these fields.

---

### Q3 - UI Highlight Block Presentation & Layout
- **Decision:** **Option 3B (Two separate card boxes styled like Executive Brief)**.
- **Settled Details:**
  - `sanitized_content` is eliminated from `entry_detail.html`.
  - Impact Card: `rounded-lg p-4 bg-muted/40 border border-border mb-6` with `trending-up` icon and title "Impact Analysis".
  - Fallacy Card: `rounded-lg p-4 bg-muted/40 border border-border mb-6` with `shield-alert` icon and title "Logical Fallacy & Objectivity Check".
  - Rewrite Box: `#ai-rewrite-box` follows with inverted pyramid text.
  - While loading, skeleton blocks mirror the card heights and structure.

---

### Q4 - Reassuring Indicator for Objective Articles
- **Decision:** **Reassuring indicator displayed when no fallacies detected**.
- **Settled Details:**
  When the article is objective and free of fallacies, the fallacy card explicitly presents:
  - Vietnamese: *"Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu."*
  - English: *"No logical fallacies detected — Objective, evidence-based reporting."*

---

## 3. Verdict

- **Grill Result:** `frontier-empty`
- **Proceed to:** `design`
