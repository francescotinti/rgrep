#!/bin/bash
set -e

echo "Running build and tests for rgrep..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test --test proptest_differential

echo "Committing to rgrep..."
git add -A
git commit -m "feat(step16): differential proptest integration

IN-SCOPE:
- Added 'proptest' dev-dependency to Cargo.toml (D 16.1)
- Implemented tests/proptest_differential.rs with 3 strategies (literal, anchored, char_class) (D 16.2, D 16.3)
- Configured 1024 cases per strategy for deep progressive property checking (D 16.4)
- Added skip logic for BSD grep environment (D 16.7)
- Run helper methods for capturing exit codes and output streams (D 16.6)

OUT-OF-SCOPE (debito esplicito):
- Fuzz testing / cargo-fuzz integration

Testcase aggiunti: 3 proptest strategies (1024 cases each)."

echo ""
echo "✅ TUTTO VERDE! ECCO L'HASH RGREP:"
echo "Rgrep Hash: $(git rev-parse --short HEAD)"
