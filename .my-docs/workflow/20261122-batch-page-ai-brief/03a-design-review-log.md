Result: clean

# Cross-Cutting Design Review: Batch Page AI Brief

## 1. Executive Summary & Review Verdict

This review re-evaluates the updated `03-design.md` and `04-tasks.md` following the fixes applied in `design-update`. It specifically verifies that all findings from the previous review round have been completely resolved, and that the architecture adheres to the Annuaire design system, security standards, performance bounds, and multilingual requirements.

- **Overall Verdict**: **clean**.
- **Previous Findings**: All 3 prior findings (1 Major, 2 Minor) have been fully and properly resolved.
- **Isolated Reviews Status**:
  - `03a-api-contract-review.md`: **clean**
  - `03a-db-design-review.md`: **clean**
- **Advance-on-clean**: All criteria for Design clean are met.

---

## 2. Verification of Prior Review Findings

### Finding 1 [Major]: Annuaire Typography & Line Clamping on Swapped Titles
- **Prior Problem**: Out-Of-Band (`hx-swap-oob`) replacement span used `font-serif font-medium text-foreground`, stripping Annuaire font tokens, removing `line-clamp-2`, and omitting the original title tooltip.
- **Verification**:
  - `03-design.md` Section 2 (lines 50–51) and Section 3.E Step 11 (lines 215–220) now explicitly specify:
    ```html
    <span id="entry-title-{id}"
          hx-swap-oob="outerHTML"
          class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors"
          title="{escaped_original_title}">
      {escaped_suggested_title}
    </span>
    ```
  - Section 4 point 2 (lines 274–276) aligns the initial wrapper in `entry_item.html` with identical classes and dynamic `title` tooltip binding.
  - `04-tasks.md` Task 4.1, 4.2, and 4.3 include explicit implementation and TDD verification requirements for these classes, tooltip attributes, and XSS escaping.
- **Status**: **RESOLVED & VERIFIED CLEAN**.

---

### Finding 2 [Minor]: Skeleton Padding Mismatch Causing Micro Layout Shift
- **Prior Problem**: The skeleton placeholder had `p-3.5` (14px) whereas the completed brief card was documented with `p-4` (16px), introducing a 2px vertical layout shift upon lazy loading.
- **Verification**:
  - `03-design.md` Section 2 (line 41), Section 3.E Step 11 (line 212), and Section 4 point 3 (line 278) all standardize padding to `p-3.5`:
    `class="rounded-lg p-3.5 bg-muted/40 border border-border mb-4"`
  - `04-tasks.md` Task 4.1, 4.2, and 4.3 specify `p-3.5` padding across skeleton and completed cards.
- **Status**: **RESOLVED & VERIFIED CLEAN**.

---

### Finding 3 [Minor]: Empty Entries Guard for Skeleton Card
- **Prior Problem**: If `entries.is_empty()`, the skeleton card would still render and fire an unnecessary network request to `/hx/executive-brief`.
- **Verification**:
  - `03-design.md` Section 3.E Step 3 (line 200) documents an explicit guard returning `200 OK` with an empty response immediately if `entries.is_empty()` without invoking LLM or DB.
  - Section 4 point 1 (lines 263–264) documents guarding the skeleton inclusion in `entry_list.html` with `{% if !entries.is_empty() && is_brief_eligible %}`.
  - `04-tasks.md` Tasks 4.1, 4.2, and 4.3 incorporate TDD tests and implementation for empty state guards.
- **Status**: **RESOLVED & VERIFIED CLEAN**.

---

## 3. Verification of Cross-Cutting Architecture

### A. Multilingual Prompt Enforcement
- **Status**: **VERIFIED CLEAN**.
- **Evidence**: `03-design.md` Section 3.D strictly enforces source language preservation per article and dominant language page synthesis. Compact JSON keys (`"top"`, `"items"` with `"i"`, `"t"`, `"b"`) minimize token usage. `04-tasks.md` Milestone 3 covers TDD test verification for prompt generation and code fence stripping.

### B. Error Handling & Graceful Skeleton Collapse
- **Status**: **VERIFIED CLEAN**.
- **Evidence**: `03-design.md` Section 3.E Step 9 guarantees that upon timeout, CLI failure, or malformed JSON, the server responds with `200 OK` and `<div id="page-executive-brief" class="hidden"></div>`. HTMX replaces the skeleton with a hidden node, cleanly collapsing the card container without infinite pulsing animations. Article detail drawer strictly reads from cache with zero CLI subprocess invocations.

### C. Rate Limiting, Concurrency & Transactions
- **Status**: **VERIFIED CLEAN**.
- **Evidence**: `AiService` in-flight deduplication on `page_key` prevents thundering herds. `LlmCliRunner::semaphore` with capacity 2 bounds concurrent subprocesses. `AiDb::save_batch_results` bundles summary updates and page brief upsert inside a single SQLite transaction (`conn.transaction()?`), minimizing disk fsync operations.

---

## 4. Final Review Conclusion

All requirements, UI/UX consistency, and backend architecture contracts have been satisfied. No open Critical or Major issues remain.

- Result token: `clean`
