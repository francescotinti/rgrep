#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/matcher.rs
git commit -m "fix(step11-hotfix): add without_match field to mock Config

IN-SCOPE:
- Add missing \`without_match\` field to mock Config in src/matcher.rs
- Fixes cargo test failure from bcbe045

OUT-OF-SCOPE:
- No feature changes"

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C . rev-parse --short HEAD)"
