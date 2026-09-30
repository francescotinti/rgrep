# Adding a new testcase

> rgrep — Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
> AI-assisted port: Claude Opus 4.7 (Anthropic) + Gemini Antigravity (Google)


1. Create a new TOML file in `cases/` with a zero-padded 4-digit ID (e.g., `0006_name.toml`).
2. Follow this structure:
```toml
name = "my test name"
args = ["-i", "pattern"]
stdin = "input\ndata\n"
expected_stdout = "matched\noutput\n"
expected_exit_code = 0

# Optional fields
# skip_if_bsd = true
# skip_reason = "BSD grep handles this differently"
```
3. Add the path to the `cases` array in `testsuite.toml`.
4. Run `cargo test` to verify against the system's `grep`.

## Explicit oracle and coverage (2026-09-30)

Use `RGREP_ORACLE=/absolute/path/to/grep cargo test -- --nocapture`.
GNU and BSD are recognized once per test executable; unknown implementations
are rejected. The harness compares raw stdout bytes and exit codes even for
nonzero exits. `LC_ALL=C` is the default (a case may override it).

The summary separates `parity`, `rgrep_only`, `skipped`,
`known_gnu_divergences`, and `failed`; these sum to the manifest total.
The three 1024-case properties execute against GNU and explicitly skip on BSD.
A green BSD run therefore does not establish property-based GNU parity.

Additional manifest fields:

- `oracle_args`: equivalent oracle command, used to omit rgrep's `--mmap`
  optimization flag, which modern GNU grep does not accept.
- `rgrep_only` and `forbids_feature`: test the expected exit 2 for `-P` in
  builds without PCRE2; skip this case in a PCRE2-enabled build.
- `gnu_difference`, `gnu_stdout`, `gnu_stderr_contains`: a documented GNU
  discrepancy with separately verified outputs on both sides. Existing rgrep
  expectations remain enforced. An unexpected change or resolution fails.
  These cases are reported as known differences, never as parity passes.
- `expected_stderr_contains`: byte substring that **rgrep's** stderr must
  contain (`{FIXTURES}` is expanded). Never checked on the oracle, whose
  stderr wording differs (`grep:` vs `rgrep:` prefix). Used by the binary
  diagnostic cases (0054, 0067, 0076, 0080–0083), which are `skip_if_bsd`
  because BSD grep prints that message on stdout.

Set `RGREP_STRICT_GNU=1` to reject every known GNU difference. Two remain:
0064/0065 implement different `-T` alignment. The binary diagnostics
(0054/0067) were aligned to GNU >= 3.5 by Step BD. See `GNU_VERIFICATION.md`.
The obsolete generic `expected_to_fail` escape hatch has been removed;
unknown manifest fields are rejected instead of silently ignored.

`.github/workflows/verify.yml` runs Linux/GNU and macOS/BSD with default and
PCRE2 builds and uploads the full per-case coverage logs. It needs only this
repository; the external `gnu-grep` source tree is needed for benchmarks,
not the test suite. Local buffer-boundary tests cover streamed long records,
CRLF, NUL, EOF and offsets independently of the oracle.
