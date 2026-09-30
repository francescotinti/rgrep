#!/bin/bash
set -e

echo "Cleaning up expected_to_fail from generated TOMLs..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
python3 tools/clean_expected_to_fail.py

echo "Running cargo build and cargo test..."
cargo build
cargo test

echo "Committing rgrep..."
git add -A
git commit -m "feat(step19): fix 12 known bugs (backref support + D-NEW-5 validation)

IN-SCOPE:
- Add fancy-regex dependency for backref support (D 19.2)
- Engine::Fancy variant with auto-fallback compile order (D 19.3)
- Per-pattern validation in -e multi to fix D-NEW-5 (D 19.1)
- Remove expected_to_fail=true from 12 gnu_ testcases (D 19.5)
- New regression testcase for D-NEW-5
- 3 unit tests for backref + multi-e validation

OUT-OF-SCOPE (debito esplicito):
- fancy-regex performance vs regex DFA
- General lookaround support outside backref tests
- Newly discovered bugs (Step 20 if needed)

Testcase aggiunti: 1. Totali: 129 (+12 da expected_to_fail → passing)."

echo ""
echo "✅ TUTTO VERDE! ECCO L'HASH RGREP:"
echo "Rgrep Hash: $(git rev-parse --short HEAD)"
