#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/main.rs src/runner.rs tests/cases/0047_max_count.toml tests/cases/0048_max_count_zero.toml tests/cases/0049_quiet_match.toml tests/cases/0050_quiet_no_match.toml tests/cases/0051_no_messages.toml tests/cases/0052_no_match_exit.toml tests/cases/0053_missing_file_exit.toml tests/testsuite.toml
git commit -m "feat(step10): limits -m -q -s + exit code semantics (D-NEW-2 fix)

IN-SCOPE:
- -m NUM: stop reading file after NUM matches (D 10.1)
- -q: quiet mode, early exit on first match globally (D 10.2)
- -s: suppress stderr for nonexistent files (D 10.3)
- Canonical Exit Codes: 0=Match, 1=NoMatch, 2=Error (D 10.4)
- Fix D-NEW-2 latente bug from Step 3
- 7 new differential testcases (0047-0053)
- 0052 tests no-match -> exit 1
- 0053 tests missing file -> exit 2
- 3 unit tests for limit control and RunResult

OUT-OF-SCOPE (debito esplicito):
- POSIX-strict interactions with other edge cases

Testcase aggiunti: 7. Totali: 53."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md diary/04-divergences.md
git commit -m "docs(step10): translation log + D-NEW-2 fix completion

IN-SCOPE:
- diary/03-translation-log.md: Limit control updates
- diary/04-divergences.md: mark D-NEW-2 as fixed in Step 10

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 53."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
