# Ideation: Details Page AI Rewrite with Batch Pre-request

**Sizing:** Project with written design  
**Has UI (tentative):** yes (details view template and layout update)  
**Has API (tentative):** yes (LLM batch prompt and response parsing)  
**Has DB (tentative):** yes (SQLite schema migration to store rewrites)  
**Has auth/PII/schema (tentative):** yes (schema migration `user_version = 4`)  

---

## 1. Playback & Intent Confirmation

### Intent Summary
Currently, the article details page renders the raw sanitized article content (`sanitized_content`) below an Executive Brief card. Under this feature:
1. **Details view replacement**: The main article body on the details page will display a concise, high-impact **AI Rewrite** instead of the original raw HTML content.
2. **Writing & cognitive standards**:
   - **Length**: Under 110 words (30–40s read time). Ideal target is 30–60 words in 2–3 scannable bullet points or paragraphs.
   - **Structure**: Inverted Pyramid (Kim tự tháp ngược). The conclusion and core impact must be stated in the first 2 sentences.
   - **Language**: Language parity (Vietnamese source &rarr; Vietnamese rewrite; English source &rarr; English rewrite).
3. **Batch LLM processing**: The rewrite will be produced in the exact same batch LLM request that currently creates executive titles (`t`), briefs (`b`), and the page-level overview (`top`), avoiding extra per-article API calls.
4. **Server pre-requesting**: When there are enough entries for a page (or when a page is fetched), the server will pre-request and cache the batch results in SQLite.

### How-it-gets-made in one line
Extend the batch LLM prompt and JSON schema with a rewrite field (`r`), migrate SQLite (`user_version = 4`) to persist rewrites, update the detail page template to render the rewrite with an optional fallback/toggle to original content, and trigger server-side batch prefetching when a page of entries is loaded.

---

## 2. Framing

### Goals
- **Lead with impact**: Present users with the conclusion and significance within the first two sentences, cutting reading fatigue.
- **Fast, digestible reading**: Deliver rewrites within the 30–60 word "sweet spot" (< 110 words max), readable in 30–40 seconds.
- **Zero additional roundtrips**: Generate `rewrite` simultaneously with titles and briefs in a single batch LLM call per page.
- **Instant detail loading**: Pre-request and persist rewrites in SQLite so navigating to details is instant and local.
- **Graceful degradation**: Fall back cleanly if an entry has not yet been processed by the LLM or if LLM generation fails.

### Non-Goals
- Changing the list view or executive brief card on the main page.
- Removing external links to the original article source.
- Generating multiple rewrite styles or interactive chat rewriting per article.

### Anti-Goals
- Do not make blocking LLM calls on user click when opening the details panel.
- Do not completely delete user access to the full original article text (provide a way to inspect original text when desired).
- Do not mix languages (strict multilingual adherence to source text language).

### Constraints
- **SQLite backwards compatibility**: Use `PRAGMA user_version` migration (version 4) with `ALTER TABLE article_ai_summaries ADD COLUMN rewrite TEXT;`.
- **Token limits**: Adding ~50 words per item across 20 items increases output tokens by ~1,000–1,300 tokens. LLM timeout and max tokens must accommodate this batch payload.
- **Annuaire UI consistency**: Detail page typography and styling must follow existing design rules (`annuaire-theme.css`, Work Sans font, tabular figures, dark mode support).

---

## 3. Technical Approach Options

### Area A: LLM Batch Schema & Response Contract

#### Option A1 (Recommended): Extend compact JSON schema with `"r"` field
- **Explanation**: Add an `"r"` (rewrite) key to each item in the batch JSON schema alongside `"i"` (id), `"t"` (title), and `"b"` (brief).
- **Concrete example**:
  ```json
  {
    "top": "Page-level executive overview...",
    "items": [
      {
        "i": 101,
        "t": "OpenAI ra mắt mô hình lý luận mới",
        "b": "Mô hình mới giải quyết bài toán phức tạp tốt hơn 40%. Chi phí API giảm một nửa. Doanh nghiệp bắt đầu thử nghiệm từ hôm nay.",
        "r": "OpenAI chính thức phát hành mô hình lý luận thế hệ mới với hiệu năng xử lý tác vụ phức tạp tăng 40% trong khi chi phí vận hành giảm 50%.\n\n• Tốc độ suy luận đạt chuẩn thực tế, giảm thiểu ảo giác thông tin xuống mức thấp kỷ lục.\n• Hỗ trợ tích hợp trực tiếp qua API từ hôm nay cho tất cả khách hàng trả phí."
      }
    ]
  }
  ```
- **Pros**:
  - Clean separation: `"b"` remains for short list-card summaries, while `"r"` holds the inverted pyramid detail rewrite.
  - No breakage in existing list view logic or tests.
- **Cons**:
  - Slightly larger JSON response payload per batch (~1,000 extra output tokens for 20 items).

#### Option A2: Merge brief and rewrite into a single multi-sentence field
- **Explanation**: Use a single summary field that serves as both the brief and the rewrite.
- **Pros**: Smaller response token count.
- **Cons**: Compromises either the brevity needed for feed list cards (3 short sentences) or the depth needed for the detail view rewrite (inverted pyramid lead + 2-3 bullets).

---

### Area B: Detail View UI & Access to Original Content

#### Option B1 (Recommended): Primary AI Rewrite with Collapsible Original Text
- **Explanation**: The detail page displays the AI Rewrite as the primary article content. Below the rewrite, a collapsed `<details>` accordion labeled "Original Article" allows the user to expand and read the full raw text if needed, while keeping the top "Original" button for external browser navigation.
- **Concrete example**:
  ```html
  <!-- AI Rewrite (Primary Content) -->
  <div class="prose dark:prose-invert max-w-none text-foreground text-sm leading-relaxed mb-8">
    {{ entry.rewrite|safe }}
  </div>

  <!-- Collapsible Original Content -->
  <details class="border-t border-border pt-4 group">
    <summary class="text-xs font-semibold text-muted-foreground hover:text-foreground cursor-pointer select-none">
      Show original article text
    </summary>
    <div class="mt-4 prose dark:prose-invert max-w-none text-muted-foreground text-xs leading-relaxed">
      {{ sanitized_content|safe }}
    </div>
  </details>
  ```
- **Pros**:
  - Satisfies the core requirement of showing the rewrite instead of raw text.
  - Safe safety-net: users can instantly verify facts against original text without navigating away.
  - Zero clutter: original text is tucked away until intentionally opened.
- **Cons**:
  - Slightly more DOM nodes than completely stripping original content.

#### Option B2: Complete Replacement (External Link Only)
- **Explanation**: Remove `sanitized_content` entirely from the detail page DOM; only render `rewrite`. The only way to view original content is clicking the external "Original" link to the publisher's website.
- **Pros**: Minimal DOM size.
- **Cons**: If the original site is paywalled, deleted, or slow, the user cannot read the original RSS content text cached by Miniflux.

---

### Area C: Server Pre-request Trigger Condition

#### Option C1 (Recommended): Page-fetch driven prefetch with uncached threshold
- **Explanation**: Whenever a page is loaded (or requested via HTMX pagination/filter), collect entries on that page. If there are uncached entries, trigger a background batch prefetch job for all uncached entries on that page (batch size up to page limit, e.g. 20).
- **Pros**:
  - Aligns with existing `AiPrefetchService` architecture in `crates/server/src/ai/prefetch.rs`.
  - Batching happens naturally around the page boundary (20 items).
- **Cons**:
  - If a user opens a brand new feed with only 1 or 2 entries, it still batches those available entries.

### Decision Confirmed: Option B (Complete Replacement)
- **Selection**: User confirmed **Option B**.
- **Behavior**: The detail page will strictly display the AI rewrite as the sole body content. The raw `sanitized_content` is omitted from the details view, leaving the external "Original" link as the sole reference to the source publisher.
- **Batching & Prefetching**: Rewrites will be batched into the same LLM request as titles and briefs, with server-side pre-requesting when enough data is available.

---

## 4. Open Questions for the User (HITL)

### Skip Option
If this framing and the recommended approaches look good, you can simply reply:
> **"just start"**

We will record `ideation-skip` and proceed immediately to formal design.

### Clarifying Question
If you'd like to refine the behavior before design, please share your thoughts on this scenario:

> **When a reader opens an article detail page and the AI rewrite is presented as the primary content:**  
> Would you prefer having a collapsible **"Show original article text"** accordion at the bottom for quick fact-checking/verification, or should original content be **strictly excluded** from the page (leaving the top "Original" external link as the sole way to view the source)?
