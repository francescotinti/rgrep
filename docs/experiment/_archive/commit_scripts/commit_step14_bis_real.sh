#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add -A
git commit -m "chore(step14-bis): commit pending mmap testcases

IN-SCOPE:
- tests/cases/0069_mmap_base.toml: base verification for --mmap
- tests/cases/0070_mmap_line_number.toml: verification with -n
- tests/cases/0071_mmap_fallback_stdin.toml: automatic stdin fallback
- tests/testsuite.toml: manifest registration

OUT-OF-SCOPE (debito esplicito):
- No source code changes

Testcase aggiunti: 3. Totali: 71."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md
git commit -m "docs(step14-bis): translation log entry for mmap path

IN-SCOPE:
- diary/03-translation-log.md: appended --mmap features, MmapOptions::map unsafe isolation and fallback logic.

OUT-OF-SCOPE:
- No code changes

Testcase aggiunti: 0. Totali: 71."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH REALI:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
