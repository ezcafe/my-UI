# Ideation: Article Detail AI Rewrite with Impact & Fallacy Analysis

**Sizing:** Project with written design  
**Has UI (tentative):** yes (details drawer template update: remove original text, add impact & fallacy highlight blocks)  
**Has API (tentative):** yes (rewrite HTMX endpoint and LLM request payload)  
**Has DB (tentative):** yes (PostgreSQL schema migration for impact and fallacies)  
**Has auth/PII/schema (tentative):** yes (PostgreSQL table `article_ai_summaries` schema update)  

---

## 1. Intent & Playback

### Summary of Requirements
1. **Detail Page Viewport**:
   - Strictly show the **Executive Brief** card and the **AI Rewrite**.
   - Completely remove the raw original article content (`sanitized_content`) from the details drawer.
   - The external "Original" link at the top remains for users who want to visit the source publication directly.
2. **Extended AI Generation (Same Request)**:
   - In the exact same LLM request that produces the article rewrite, also request:
     - **Impact analysis**: What are the practical effects, consequences, or stakes of this news?
     - **Logical fallacy identification**: Detect and name any logical fallacies or rhetorical biases in the article (e.g., ad hominem, straw man, false dilemma, hasty generalization, cherry-picking, emotional appeal). If the reporting is objective and logically sound, explicitly indicate "Không phát hiện" / "None".
3. **UI Highlight Block**:
   - Render the impact and fallacy information in a highlight block with the same visual style as the Executive Brief (`rounded-lg p-4 bg-muted/40 border border-border mb-6`).
4. **LLM Request Review & Best Practices**:
   - Review the current LLM request and propose concrete improvements adhering to 2026 LLM prompt and context engineering best practices.

---

## 2. Review of Current LLM Request & Flow Best Practices

### Current State
Currently in `crates/server/src/ai/llm.rs`:
```rust
let prompt = format!(
    "Rewrite the following article in an inverted pyramid style:\nTitle: {}\nContent: {}",
    title, content
);
```
- **Unstructured plain text**: The model returns raw text without a formal schema.
- **Missing schema contract**: Asking for multiple distinct facets (rewrite, impact, fallacy) in unstructured text risks parsing failures, inconsistent delimiters, or omitted sections.
- **Language drift risk**: Without explicit language instructions, multilingual articles might trigger unwanted translations.

### 2026 Flow & Context Engineering Best Practices
Reference: [Prompt Engineering as Context Engineering & Structured Outputs](https://collinwilkins.com)
1. **Strict Structured JSON Schema**:
   Enforce a deterministic JSON schema (e.g. `{"rewrite": "...", "impact": "...", "fallacies": "..."}`). This prevents brittle regex splitting and guarantees all fields are present.
2. **Explicit Fallacy Taxonomy & Calibration**:
   Provide clear guidance on identifying informal fallacies while avoiding false positives on neutral factual reporting (e.g. explicitly require the model to output "None" / "Không phát hiện" when no fallacy is present).
3. **Multilingual Consistency**:
   Maintain strict language parity (Vietnamese source $\rightarrow$ Vietnamese output; English source $\rightarrow$ English output).
4. **Single-Pass Efficiency**:
   Bundle rewrite, impact, and fallacy into one inference call to avoid extra latency and token overhead.

---

## 3. Goals, Non-Goals, and Constraints

### Goals
- **Clean, distraction-free reading**: Only the executive brief, highlight block (impact & fallacy), and the inverted pyramid rewrite appear in the drawer.
- **Zero extra LLM latency**: Bundle rewrite + impact + fallacy into a single LLM invocation.
- **Deterministic parsing**: Use JSON structured outputs for reliable extraction and caching.
- **Persistent caching**: Cache impact and fallacy data in PostgreSQL alongside the rewrite so repeat views load in $<50\text{ms}$.
- **Annuaire style compliance**: Match existing UI standards (`annuaire-theme.css`, Work Sans typography, tabular numbers, clean border cards).

### Non-Goals
- Changing the feed list view or main page layout.
- Calling external third-party fact-checking APIs outside the configured LLM.
- Disabling the external link to the source publisher.

### Anti-Goals
- Do not show raw HTML content anywhere in the details drawer.
- Do not make separate LLM calls for rewrite, impact, and fallacy.
- Do not invent fallacies when the article is fair and well-evidenced.

### Constraints
- **PostgreSQL Database**: Must update `article_ai_summaries` table schema using safe `ADD COLUMN IF NOT EXISTS` migrations without breaking existing data.
- **Mock DB compatibility**: In-memory / mock database (`AiDb::new_mock()`) used in unit tests must support the new fields.
- **Graceful degradation**: If LLM generation fails or times out, provide a safe fallback UI without crashing.

---

## 4. Technical Decision Options

### Area 1: LLM Prompt Contract & Schema

#### Option 1A (Recommended): Strict JSON Output Contract
- **Explanation**: Instruct the LLM to output a raw JSON object containing `rewrite`, `impact`, and `fallacies`.
- **Concrete Example**:
  ```json
  {
    "rewrite": "Lead sentence in inverted pyramid style...\n\n• Key bullet 1\n• Key bullet 2",
    "impact": "Tác động trực tiếp đến ngành công nghệ và người tiêu dùng: làm giảm chi phí vận hành...",
    "fallacies": "Không phát hiện (Bài báo trình bày số liệu khách quan)"
  }
  ```
- **Pros**:
  - Deterministic parsing; no fragile string splitting or regex parsing of markdown markers.
  - Aligns with modern LLM structured decoding standards.
- **Cons**:
  - Requires fallback parsing in case the LLM wraps the JSON in triple backticks ` ```json ... ``` ` (already handled by our robust parser).

#### Option 1B: Markdown Delimited Sections (`### Rewrite`, `### Impact`, `### Fallacies`)
- **Explanation**: Ask LLM for plain text with header delimiters.
- **Pros**: Slightly fewer tokens than JSON formatting.
- **Cons**: Fragile parsing; models often change heading levels, punctuation, or omit headers.

---

### Area 2: PostgreSQL Storage Architecture

#### Option 2A (Recommended): Dedicated Columns in `article_ai_summaries`
- **Explanation**: Add `impact TEXT` and `fallacies TEXT` columns to `article_ai_summaries`.
- **Concrete Example**:
  ```sql
  ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;
  ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;
  ```
- **Pros**:
  - Explicit schema, strongly typed in Rust `AiSummaryRecord` and sqlx queries.
  - Easy to index or query individually in future features.
- **Cons**:
  - Requires updating sqlx queries and mock storage structs.

#### Option 2B: Embed into `rewrite` column as JSON string
- **Explanation**: Store serialized JSON in the existing `rewrite` column without schema migration.
- **Pros**: No database schema change required.
- **Cons**: Pollutes the `rewrite` column; breaks backward compatibility with raw text rewrites; difficult to query or migrate cleanly.

---

### Area 3: Highlight Block Presentation in Details Drawer

#### Option 3A (Recommended): Two-Section Highlight Card (Impact & Critical Analysis)
- **Explanation**: Render a highlight card styled like `brief_card.html`, displaying both Impact Analysis and Fallacy/Critical Analysis in a clean, scannable format with distinctive icons (e.g. `zap` / `trending-up` for impact, `alert-circle` / `scale` for fallacy).
- **Concrete Example**:
  ```html
  <div class="rounded-lg p-4 bg-muted/40 border border-border mb-6 space-y-3 font-sans">
    <div class="flex items-start gap-2">
      <div class="p-1 rounded bg-muted text-foreground shrink-0 mt-0.5">
        <i data-lucide="trending-up" class="w-4 h-4 text-brand"></i>
      </div>
      <div>
        <span class="text-xs font-bold tracking-wide uppercase text-foreground">Impact Analysis</span>
        <p class="text-sm text-foreground/90 leading-relaxed mt-1">{{ impact }}</p>
      </div>
    </div>
    <div class="border-t border-border/50 pt-3 flex items-start gap-2">
      <div class="p-1 rounded bg-muted text-foreground shrink-0 mt-0.5">
        <i data-lucide="shield-alert" class="w-4 h-4 text-brand"></i>
      </div>
      <div>
        <span class="text-xs font-bold tracking-wide uppercase text-foreground">Fallacy & Objectivity Check</span>
        <p class="text-sm text-foreground/90 leading-relaxed mt-1">{{ fallacies }}</p>
      </div>
    </div>
  </div>
  ```
- **Pros**:
  - Clean visual hierarchy matching the Annuaire design system.
  - Compact and readable without cluttering the screen.
- **Cons**:
  - Adds a few more vertical lines than a single combined block.

#### Option 3B: Two Separate Independent Cards
- **Explanation**: Create two distinct cards: one dedicated card for Impact, and another card for Fallacies.
- **Pros**: Completely separate visual containers.
- **Cons**: Takes up twice the vertical card margin space and pushes the rewrite too far down.

---

## 5. Decision & Next Step

### Recommended Approach
- **Area 1**: Option 1A (Strict JSON schema).
- **Area 2**: Option 2A (Dedicated `impact` and `fallacies` columns in PostgreSQL).
- **Area 3**: Option 3A (Unified two-section highlight card matching executive brief style).

### Skip Option
If this plan matches what you want, simply reply:
> **"just start"**

We will stamp `ideation-confirmed` (or `ideation-skip`) and proceed directly to Gate A, Skim, and Design.

### Clarifying Question
> **For fallacy analysis:** If the LLM does not detect any logical fallacy (the article is neutral, factual, and well-supported), would you prefer the highlight block to display a reassuring note (e.g., *"Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"*), or would you prefer hiding the fallacy section entirely when none are found?
