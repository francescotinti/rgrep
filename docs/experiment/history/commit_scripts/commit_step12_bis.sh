#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add tests/cases/0059_null_data.toml tests/cases/0060_null_filename.toml tests/cases/0061_null_data_only_matching.toml tests/cases/0062_null_filename_files_with_matches.toml tests/cases/0063_null_data_cross_file.toml tests/testsuite.toml src/runner.rs
git commit -m "chore(step12-bis): commit pending testcases + fix unused warning"

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md
git commit -m "docs(step12-bis): translation log entry for NUL handling"

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
