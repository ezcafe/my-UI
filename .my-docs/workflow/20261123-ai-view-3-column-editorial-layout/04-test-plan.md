# Test Plan

## Planned TDD Test Cases

1. **`test_entry_list_template_renders_3_column_magazine_layout`**
   - Verify `#entries-magazine-view` contains `#mag-col-brief`, `#mag-col-center`, and `#mag-col-right`.
   - Verify container query style / classes are applied for responsive fluid columns.
   - Verify `#mag-col-brief` contains `#page-executive-brief`.

2. **`test_center_column_renders_priority_1_and_priority_2`**
   - Verify `#mag-col-center` contains Priority 1 hero card with left image and right title layout.
   - Verify `#mag-col-center` contains Priority 2 featured grid for entries 1 to 3 with top thumbnail and category badge.

3. **`test_right_column_renders_priority_3_cards`**
   - Verify `#mag-col-right` contains Priority 3 cards for entries 4 onwards.
   - Verify each card has title, category, and source metadata.

4. **`test_executive_brief_card_template_editorial_styling`**
   - Verify `executive_brief_card.html` renders `#page-executive-brief` with single narrative paragraph and editorial header.

## Result: done
