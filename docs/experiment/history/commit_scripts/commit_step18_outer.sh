#!/bin/bash
set -e

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/
git commit -m "docs(step18): translation log and new divergences for GNU tests

IN-SCOPE:
- diary/03-translation-log.md: documented Step 18 conversion and dataset expansion
- diary/04-divergences.md: registered any D-NEW-N findings (TBD)

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0."

echo ""
echo "✅ TUTTO VERDE! ECCO L'HASH OUTER:"
echo "Outer Hash: $(git rev-parse --short HEAD)"
