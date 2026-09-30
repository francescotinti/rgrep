#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/cli.rs src/runner.rs tests/cases/0054_binary_default.toml tests/cases/0055_binary_force_text.toml tests/cases/0056_binary_without_match.toml tests/cases/0057_binary_files_text.toml tests/cases/0058_binary_files_binary.toml tests/testsuite.toml
git commit -m "feat(step11): binary file handling -a -U --binary-files

IN-SCOPE:
- Binary detection via NUL byte heuristic in first buffer (D 11.1)
- -a / --text: force text mode, ignore NULs (D 11.2)
- -U / --binary: force binary mode, no CRLF stripping (D 11.3)
- --binary-files=TYPE (binary, text, without-match) (D 11.4)
- 'Binary file <name> matches' output on binary match (D 11.5)
- 5 new differential testcases (0054-0058)

OUT-OF-SCOPE (debito esplicito):
- Performance on massive NUL files
- Fallback logic to other charsets

Testcase aggiunti: 5. Totali: 58."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md
git commit -m "docs(step11): translation log entry for binary handling

IN-SCOPE:
- diary/03-translation-log.md: added NUL heuristic and binary flags

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 58."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
