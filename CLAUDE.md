# CLAUDE.md — rgrep

AI-assisted port of **GNU grep** (C, ~4.5K LOC) to Rust (~2.4K LOC), driven
by a multi-agent Architect/Implementer/Decider workflow. Full experiment
documentation lives in `docs/experiment/`: [TODO.md](docs/experiment/TODO.md)
(what is left, active SPECs), [DONE.md](docs/experiment/DONE.md) (what was
delivered, audit register with anchor hash), [ARCHITECTURE.md](docs/experiment/ARCHITECTURE.md)
(modules, flow, testing, evolution). Operating rules in Italian are in the
outer workspace `CLAUDE.md`; this file is the English summary for this repo.

## Commands (run from this directory)

```bash
cargo build                                        # 0 warnings required
cargo test                                         # differential + proptest + unit
cargo clippy --tests -- -D warnings                # gate: exit 0
cargo fmt --check                                  # gate: exit 0
cargo test --features perl-regexp                  # enables -P (pcre2, needs libpcre2)
RGREP_ORACLE=/opt/homebrew/bin/ggrep cargo test --locked -- --nocapture   # GNU 3.12 on macOS
RGREP_ORACLE=/usr/bin/grep cargo test --locked -- --nocapture             # BSD on macOS, GNU on Linux
RGREP_ORACLE=/opt/homebrew/bin/ggrep RGREP_STRICT_GNU=1 cargo test --test diff_runner  # fails on the 4 known divergences
cargo bench                                        # criterion; needs sibling ../gnu-grep/ fixtures
python3 tools/summarize_bench.py <baseline> --json reports/<file>.json
python3 tools/compare_binaries.py <pre-binary> <post-binary> --output reports/<file>.json  # after `cargo bench --bench search -- line_scan`
```

## Layout

- `src/cli.rs` clap-derive `Config` (5 sub-structs) · `src/matcher.rs` `MatchEngine` trait, `Engine` enum (regex / fancy-regex / aho-corasick / pcre2), `Matcher` · `src/runner.rs` file resolution, walkdir+globset, `search_file` (slurp ≤16 MB → `Cursor`, `--mmap`, `BufReader` fallback), `bufread_search`, `read_delimited` (memchr), `process_line` · `src/output.rs` `GREP_COLORS`, `highlight()` → `Cow` · `src/error.rs` thiserror → exit 2.
- `tests/diff_runner.rs` process-spawn differential harness · `tests/proptest_differential.rs` 3 strategies × 1024 · `tests/buffer_boundaries.rs` · `tests/testsuite.toml` authoritative manifest (137 cases) · `tests/cases/NNNN_<name>.toml`.
- `benches/search.rs` 10 criterion groups · `tools/` converters and bench helpers · `reports/` versioned JSON/TXT evidence · `.github/workflows/verify.yml` Linux+macOS × default+PCRE2.

## Rules

- One step = one commit on `main`, conventional style, body with `IN-SCOPE` / `OUT-OF-SCOPE (debito esplicito)` / `Testcase aggiunti: N. Totali: M.`
- All four gates green before every commit. Never commit with warnings or red tests.
- TDD-first: add failing TOML cases first, then implement.
- Never edit a case's `expected_stdout` to make it pass: open a `SPEC-Q` (spec) or `// DESIGN-Q:` (code) and stop.
- Every case must be registered in `tests/testsuite.toml`; unknown manifest fields are rejected; `expected_to_fail` no longer exists.
- Do not start work on a 🔒 LOCKED step; only the Decider promotes to 🚧 in `docs/experiment/TODO.md`. Audit verdicts go to `docs/experiment/DONE.md`.
- Code, inline comments and commit messages in English; `docs/experiment/` in Italian.

## Testing schema

Case fields: `name`, `args`, `stdin`, `expected_stdout`, `expected_exit_code`. Optional: `skip_if_bsd` + `skip_reason`, `requires_feature` / `forbids_feature` (`"perl-regexp"`), `rgrep_only`, `oracle_args` (used for `--mmap`, rejected by modern GNU), `fixture_files` with `{FIXTURES}` placeholder, `env`, `sort_output`, `gnu_difference` + `gnu_stdout` + `gnu_stderr_contains`, `expected_stderr_contains` (rgrep's own stderr only). The oracle is chosen with `RGREP_ORACLE`; GNU/BSD identity is detected once and unknown implementations are rejected. Summary counts parity / rgrep-only / skipped / known GNU divergences / failed, summing to the manifest total. Property tests run only against GNU and skip explicitly on BSD. Details: `tests/README.md`, `GNU_VERIFICATION.md`.

## Gotchas

- `-h` is `--no-filename` (clap `disable_help_flag`); `-P` without the feature exits 2 with GNU's message.
- Known GNU 3.12 divergence (technical debt, see TODO P3): cases 0064/0065 `-T` alignment. The binary-match message follows GNU ≥3.5 (`rgrep: NAME: binary file matches` on stderr), so those cases are `skip_if_bsd`.
- On an ExFAT volume `cargo build`/`clippy` print `warning: rgrep (…) generated N warning` from the hard-link cache: not lints. Use `CARGO_TARGET_DIR=/tmp/rgrep-target` on APFS if in doubt. AppleDouble `._*` files are gitignored.
- Cargo output saved to a file contains ANSI escapes: filter with `awk`, not `grep`.
- Benchmarks are process-spawn: 1–3 ms of startup on 4–7 ms scenarios. A speedup claim needs a named baseline plus alternating pre/post pairs (`tools/compare_binaries.py`), ≥5% on a line-bound scenario and no median regression >5%. Sequential runs drift by tens of percent.
- Profiling on macOS: `samply` with a compiled helper (`cargo-flamegraph` needs sudo/SIP; `bash` rejects `DYLD_INSERT_LIBRARIES`).
