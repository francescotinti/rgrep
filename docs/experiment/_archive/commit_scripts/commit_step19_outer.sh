#!/bin/bash
set -e

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/
git commit -m "docs(step19): close D-NEW-5, revise D-M6 backref support

IN-SCOPE:
- diary/04-divergences.md: D-NEW-5 status updated to fixed
- diary/02-mapping-table.md: D-M6 revised (fancy-regex fallback)
- diary/03-translation-log.md: Step 19 entry

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 129."

echo ""
echo "✅ TUTTO VERDE! ECCO L'HASH OUTER:"
echo "Outer Hash: $(git rev-parse --short HEAD)"
