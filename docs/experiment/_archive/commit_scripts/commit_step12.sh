#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/runner.rs tests/cases/0059_null_data.toml tests/cases/0060_null_filename.toml tests/cases/0061_null_data_only_matching.toml tests/cases/0062_null_filename_files_with_matches.toml tests/cases/0063_null_data_cross_file.toml tests/testsuite.toml
git commit -m "feat(step12): NUL handling -z null-data and -Z null filename

IN-SCOPE:
- -z / --null-data: read_until customizable delimiter (D 12.1)
- -Z / --null: NUL byte after filenames (D 12.2)
- Binary-safe output via io::Write::write_all to preserve internal NULs (D 12.4)
- 5 new differential testcases (0059-0063) handling NULs natively via TOML \\u0000
- 3 new unit tests for NUL behavior

OUT-OF-SCOPE:
- None

Testcase aggiunti: 5. Totali: 63."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md
git commit -m "docs(step12): translation log entry for NUL handling

IN-SCOPE:
- diary/03-translation-log.md: added NUL handling (-z, -Z) details

OUT-OF-SCOPE:
- No code changes

Testcase aggiunti: 0. Totali: 63."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
