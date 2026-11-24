# Ideation: Replace Brief with Rewrite in Batch LLM & On-Demand Impact/Fallacy Analysis

**Sizing:** Medium change with written design  
**Has UI (tentative):** yes (remove Executive Brief from detail page, show rewrite immediately on load, lazy-load impact & fallacies)  
**Has API (tentative):** yes (update batch prompt schema to generate rewrite `r` instead of brief `b`; add/update analysis endpoint for impact and fallacies)  
**Has DB (tentative):** yes (persist rewrite from batch into `article_ai_summaries`, update impact/fallacies on analysis return)  
**Has auth/PII/schema (tentative):** no (database columns `rewrite`, `impact`, `fallacies` already exist in `article_ai_summaries`)  

---

## 1. Intent Summary & Playback

### User Requirement
- In LLM batch request: Replace "executive brief" with "rewrite text". Each article processed in the batch will receive a punchy suggested title (`t`) and a full inverted-pyramid rewrite (`r`).
- In article details page:
  1. Remove the "Executive Brief" card entirely.
  2. Show the article's rewrite text immediately on initial page load (zero latency since it is pre-generated in batch or stored in DB).
  3. After initial load, trigger an asynchronous LLM request for Impact Analysis and Fallacy & Objectivity Check, swapping them in when ready (or displaying them immediately if already cached in DB).

### Reading Eye Flow & How It Works in One Line
Batch LLM generates suggested title `t` and inverted-pyramid rewrite `r`; the details page renders the rewrite instantly on load without the "Executive Brief" card, then triggers an HTMX background request to fetch and render impact and fallacy analysis cards.

---

## 2. Framing

### Goals
- **Instant Detail Reading**: Readers immediately see the full rewritten article in inverted-pyramid style without waiting for a second LLM roundtrip.
- **Remove Duplicate Briefs**: Eliminate redundant "Executive Brief" cards on the detail page, simplifying the layout to focus on the rewrite.
- **Targeted Background Analysis**: Keep impact analysis and logical fallacy checks focused on a separate lightweight prompt/endpoint that streams or swaps in seamlessly.
- **Multilingual Fidelity**: Maintain strict language parity (Vietnamese sources produce Vietnamese rewrite/impact/fallacies; English sources produce English).

### Non-Goals
- Changing the top-level executive brief (`top`) displayed on the main page/magazine view.
- Removing or altering the original article link button ("Original").
- Modifying the underlying PostgreSQL database schema (columns `rewrite`, `impact`, and `fallacies` already exist).

### Anti-Goals
- Do not block the detail page initial render while waiting for LLM impact/fallacy analysis.
- Do not re-request rewrite generation on detail page open if the rewrite is already present.

### Constraints
- Database column `summary` in `article_ai_summaries` is `NOT NULL`. When batch generates rewrite `r`, `summary` should be populated with the rewrite (or its core summary) to maintain full compatibility with list views.
- Must preserve HTMX graceful fallbacks if LLM analysis fails.

---

## 3. Technical Approach Options

### Area A: LLM Batch Prompt & JSON Schema

#### Option A1 (Recommended): Replace `"b"` with `"r"` in batch prompt
- **Explanation**: In `LlmCliRunner::format_batch_prompt_structured`, instruct the LLM to return `t` (suggested title, 1 sentence) and `r` (inverted-pyramid rewrite: 2 impact lead sentences followed by 2-3 bullet points). Drop the 3-sentence brief `"b"`.
- **Concrete Example**:
  ```json
  {
    "top": "Page synthesis brief...",
    "items": [
      {
        "i": 101,
        "t": "OpenAI Launches New Search Product",
        "r": "OpenAI launched SearchGPT today, entering direct competition with Google Search. The tool provides real-time conversational search with attributed web citations.\n\n• Available initially to a prototype test group of 10,000 users.\n• Direct publisher partnerships announced with major news organizations."
      }
    ]
  }
  ```
- **Pros**: Matches user request directly. Detail view has the complete rewrite ready immediately upon batch execution.
- **Cons**: Slightly higher token output per batch item (~40-60 tokens per item instead of ~25 tokens).

#### Option A2: Keep `"b"` and add `"r"` alongside it
- **Explanation**: Generate both a 3-sentence brief `"b"` and a rewrite `"r"`.
- **Pros**: Leaves legacy brief intact.
- **Cons**: Wastes prompt and output tokens; user explicitly requested replacing brief with rewrite text.

---

### Area B: Details Page & On-Demand Impact/Fallacy Flow

#### Option B1 (Recommended): Inline Rewrite on Load + Dedicated HTMX Analysis Partial
- **Explanation**:
  1. `entry_detail.html`:
     - Delete `{% include "brief_card.html" %}`.
     - Render the `rewrite` text directly in `#ai-rewrite-box` on initial load.
     - Below the rewrite, include an analysis container:
       - If `impact` and `fallacies` are already cached in DB, render the Impact & Fallacy cards immediately.
       - If not cached, render an HTMX placeholder:
         `<div id="ai-analysis-container" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="load" hx-swap="outerHTML">...skeleton...</div>`
  2. Route `/hx/entries/:id/analysis`:
     - Calls LLM with a focused prompt requesting only `impact` and `fallacies` for the article.
     - Saves the analysis to `article_ai_summaries` in `AiDb`.
     - Returns HTML containing the Impact and Fallacy cards.
- **Concrete Example**:
  ```html
  <div class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
    {{ rewrite }}
  </div>

  {% if has_analysis %}
    {% include "analysis_cards.html" %}
  {% else %}
    <div id="ai-analysis-container" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="load" hx-swap="outerHTML">
      <!-- Lightweight skeleton for impact & fallacies -->
    </div>
  {% endif %}
  ```
- **Pros**:
  - Zero initial delay for reading the rewrite.
  - LLM request for impact and fallacies is focused, fast, and does not re-generate the rewrite.
  - Cached articles render everything on initial load without any extra HTTP requests.
- **Cons**: Requires adding or refining the dedicated `/analysis` route and prompt.

#### Option B2: Keep `/rewrite` route but have it return only impact and fallacies
- **Explanation**: Reuse the existing `/hx/entries/:id/rewrite` URL but change its template to append impact and fallacies into the page.
- **Pros**: Reuses existing route name.
- **Cons**: Misleading endpoint name (`/rewrite` that doesn't return rewrite, but impact and fallacies); cleaner to name it `/analysis` or keep `/rewrite` with clear semantics.

---

## 4. Recommendation & Plan

We recommend **Option A1** (replace `"b"` with `"r"` in batch schema) and **Option B1** (inline rewrite on initial load + dedicated background request for impact and fallacies).

---

## 5. Confirmation Question for User

Before proceeding with Gate A and detailed design, please confirm:
1. Does the proposed structure (Option A1 + Option B1) align with your expectations?
2. When saving the batch result to database, shall we store the rewrite text in both `rewrite` and `summary` (to keep `summary` column satisfied and list views updated with the latest rewrite text)?
