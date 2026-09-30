#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/runner.rs tests/cases/0064_initial_tab.toml tests/cases/0065_initial_tab_line_number.toml tests/cases/0066_label_stdin.toml tests/cases/0067_label_binary.toml tests/cases/0068_line_buffered.toml tests/testsuite.toml
git commit -m "feat(step13): misc flags -T, --label, --line-buffered

IN-SCOPE:
- -T / --initial-tab: prepend \t before matched line content/prefixes (D 13.1)
- --label=LABEL: override '(standard input)' for stdin matches and binary warnings (D 13.2)
- --line-buffered: explicit stdout flush after every line (D 13.3)
- 5 new differential testcases (0064-0068) to verify misc flag behaviors

OUT-OF-SCOPE (debito esplicito):
- Tab stop alignment calculations (GNU grep complex logic)
- Interactive TTY testing for line-buffered (only static flush verified)

Testcase aggiunti: 5. Totali: 68."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md
git commit -m "docs(step13): translation log entry for misc flags

IN-SCOPE:
- diary/03-translation-log.md: added entry for -T, --label, --line-buffered

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 68."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
