#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/runner.rs
git commit -m "fix(step14-hotfix): fix unit test compilation errors

IN-SCOPE:
- Renamed obsolete 'process_file' calls to 'bufread_search' in unit tests
- Removed unavailable 'tempfile' crate usage in favor of std::env::temp_dir

OUT-OF-SCOPE:
- No feature changes"

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C . rev-parse --short HEAD)"
