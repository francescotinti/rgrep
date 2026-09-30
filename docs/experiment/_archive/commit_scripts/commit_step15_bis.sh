#!/bin/bash
set -e

echo "Running build and tests for DEFAULT profile (no features)..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Running build and tests for PCRE2 profile (--features perl-regexp)..."
cargo build --features perl-regexp
cargo test --features perl-regexp

echo "Committing to rgrep..."
git add -A
git commit -m "fix(step15-hotfix): fix borrow of moved value raw_patterns

IN-SCOPE:
- Changed 'raw_patterns' to 'final_patterns' inside pcre2 builder to resolve borrow checker error.

OUT-OF-SCOPE:
- No feature changes"

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH REALI:"
echo "Rgrep (inner) Hash: $(git -C . rev-parse --short HEAD)"
