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
git commit -m "feat(step15): -P perl-regexp via pcre2 crate (feature-gated)

IN-SCOPE:
- Added pcre2 as optional dependency and defined perl-regexp feature flag (D 15.1)
- Preserved existing panic for default build (D 15.2)
- Matcher::Pcre2 implementation gated behind perl-regexp with JIT/caseless/utf (D 15.3, D 15.4)
- Testsuite differential harness extended with 'requires_feature' check (D 15.7)
- 3 new PCRE2 differential tests covering lookaround and backreferences (0072-0074)
- 2 unit tests for lookahead and backreference under feature gate

OUT-OF-SCOPE (debito esplicito):
- JIT stack limit fallback not implemented (delegated to crate)
- Syntax divergence between GNU PCRE and Rust PCRE2 not handled

Testcase aggiunti: 3. Totali: 74."

echo "Committing to outer repo..."
cd "/Volumes/Extreme Pro/Claude/testag-grep"
git add diary/02-mapping-table.md diary/03-translation-log.md
git commit -m "docs(step15): translation log and mapping table for PCRE2

IN-SCOPE:
- diary/02-mapping-table.md: updated D-M10 status from confirmed to implemented
- diary/03-translation-log.md: appended -P feature gate description and fallback logic

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 74."

echo ""
echo "✅ TUTTO VERDE! ECCO GLI HASH REALI:"
echo "Rgrep (inner) Hash: $(git -C rgrep rev-parse --short HEAD)"
echo "Outer Hash:         $(git rev-parse --short HEAD)"
