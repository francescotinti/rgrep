#!/bin/bash
set -e

echo "Running GNU test conversion script..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
python3 tools/convert_gnu_tests.py

echo "Applying manual curation to expected failures..."
python3 tools/curate.py

echo "Running cargo build and diff_runner tests..."
cargo build
cargo test --test diff_runner

echo "Committing rgrep..."
git add -A
git commit -m "test(step18): import and convert gnu-grep testsuite

IN-SCOPE:
- Added tools/convert_gnu_tests.py to auto-parse gnu-grep/tests/*.sh (D 18.1)
- Auto-evaluates stdin/args via system grep to populate expected_stdout dynamically
- Applied exclude_patterns for locale, IFS, binary formats and init.sh esoterica (D 18.4)
- Imported generated testcases into tests/testsuite.toml with 'gnu_' prefix (D 18.2)
- Added tools/README.md

OUT-OF-SCOPE (debito esplicito):
- Tests relying on C helpers (get-mb-cur-max) or specific locales are skipped.
- No code changes to rgrep core yet (awaiting manual curation of test results).

Testcase aggiunti: TBD. Totali: TBD."

echo ""
echo "✅ TUTTO VERDE! ECCO L'HASH RGREP:"
echo "Rgrep Hash: $(git rev-parse --short HEAD)"
