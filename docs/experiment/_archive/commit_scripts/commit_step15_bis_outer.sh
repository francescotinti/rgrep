#!/bin/bash
set -e

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/02-mapping-table.md diary/03-translation-log.md
git commit -m "docs(step15-bis): translation log + mapping table for PCRE2 feature gate

IN-SCOPE:
- diary/03-translation-log.md: pcresearch.c -> pcre2 crate (feature-gated bridge)
- diary/02-mapping-table.md: D-M10 status updated to implemented

OUT-OF-SCOPE (debito esplicito):
- No code changes
- Cannot retitle 43a3f5e (no rebase policy)

Testcase aggiunti: 0. Totali: 74."

echo ""
echo "✅ TUTTO VERDE! ECCO L'HASH OUTER:"
echo "Outer Hash: $(git rev-parse --short HEAD)"
