# Frontier Grill

## Frontier Questions & Resolutions

1. **Executive Brief formatting**
   - *Question:* Should Executive Brief in the left column be a single narrative paragraph or broken into bullet points?
   - *Resolution:* User confirmed: "single narrative paragraph".

2. **Column Distribution & Proportions**
   - *Question:* What proportions should the 3 columns take in the magazine grid?
   - *Resolution:*
     - Left column (Executive Brief): `minmax(240px, 1fr)` (compact editorial sidebar).
     - Center column (Priority 1 + 2): `minmax(420px, 2.6fr)` (wide lead content area).
     - Right column (Priority 3): `minmax(240px, 1.1fr)` (secondary editorial sidebar).
     - On smaller container widths (< 800px), cleanly stack vertically via container queries.

3. **Priority Breakdown in Items**
   - *Question:* How many items should each priority slice receive?
   - *Resolution:*
     - Priority 1: 1 item (`loop.index0 == 0`).
     - Priority 2: Up to 3 items (`loop.index0 >= 1 && loop.index0 <= 3`).
     - Priority 3: Remaining items (`loop.index0 >= 4`).

## Frontier Status: frontier-empty
## Result: frontier-empty
