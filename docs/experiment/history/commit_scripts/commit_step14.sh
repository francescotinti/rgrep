#!/bin/bash
set -e

echo "Running build and tests..."
cd "/Volumes/Extreme Pro/Claude/testag-grep/rgrep"
cargo build
cargo test

echo "Committing to rgrep..."
git add src/runner.rs tests/cases/0069_mmap_base.toml tests/cases/0070_mmap_line_number.toml tests/cases/0071_mmap_fallback_stdin.toml tests/testsuite.toml
git commit -m "feat(step14): memory mapped I/O --mmap

IN-SCOPE:
- --mmap via memmap2 for regular files (D 14.1)
- Automatic fallback to BufReader if stdin or metadata fails (D 14.2)
- Isolated unsafe block exclusively for MmapOptions::map (D 14.3)
- search_buffer byte-for-byte extraction to guarantee identical output to bufread_search (D 14.4)
- 3 new differential testcases (0069-0071)
- 2 unit tests covering mmap file mapping and stdin fallback

OUT-OF-SCOPE:
- No changes to default non-mmap paths

Testcase aggiunti: 3. Totali: 71."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/03-translation-log.md
git commit -m "docs(step14): translation log entry for memory mapped IO

IN-SCOPE:
- diary/03-translation-log.md: added entry documenting --mmap, memmap2, and isolated unsafe block

OUT-OF-SCOPE:
- No code changes

Testcase aggiunti: 0. Totali: 71."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
