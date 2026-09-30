# GNU/BSD verification — 2026-09-30

Executed locally on macOS arm64, rustc 1.98.1, GNU grep 3.12 (Homebrew)
and BSD grep 2.6.0-FreeBSD. After publication, the [first CI run](https://github.com/francescotinti/rgrep/actions/runs/36766671569)
**passed both Linux configurations and macOS with PCRE2**. The default macOS
job failed before the build because Bash 3 rejects an empty array under
`set -u`; the workflow now uses positional arguments. The report below
records the earlier local verification; see `reports/ci-first-run-20260930.json`
and [publication notes](docs/experiment/PUBLICATION.md) for remote status.

## Measured coverage

There are 129 manifest cases. Counts distinguish actual parity from explicit
exceptions; they sum to 129 in each configuration. The full local Rust suites
passed: 42 default, 44 with PCRE2 (BSD property functions return without
performing comparisons). Each GNU run actually executes 3072 generated
comparisons; both build variants were tested, for 6144 in this session.

| Oracle / build | Parity | rgrep only | Skipped | Known differences | Failed | Generated comparisons |
|---|---:|---:|---:|---:|---:|---:|
| gnu-default | 121 | 1 | 3 | 4 | 0 | 3072 |
| gnu-pcre | 124 | 0 | 1 | 4 | 0 | 3072 |
| bsd-default | 112 | 1 | 16 | 0 | 0 | 0 |
| bsd-pcre | 112 | 0 | 17 | 0 | 0 | 0 |

Machine-readable results: `reports/verification-20260930.json`.
Detailed local logs: `target/verification/{gnu,bsd}-{default,pcre}-full.log`.
The workflow uploads equivalent per-case logs as artifacts.

## Baseline findings and disposition

The first GNU execution of the old harness failed eight cases:

- Three `--mmap` cases: GNU 3.12 rejects the historical optimization option.
  `oracle_args` omits that flag while keeping the same input and expected
  output. rgrep still executes with `--mmap`; this is an equivalent-output
  check of rgrep's optimization, not a claim that GNU accepts that option.
- One unavailable-PCRE test: GNU's compiled capabilities differ. This now
  verifies rgrep alone without PCRE2 and skips when the feature is enabled.
- Four **unfixed semantic differences**: binary diagnostics (0054, 0067) go
  to stdout in rgrep but stderr in GNU; initial-tab alignment (0064, 0065)
  differs. They are still technical debt, not compatibility successes.
  Existing rgrep expected outputs were not changed. Separately recorded GNU
  outputs are checked as well, so these exceptions cannot hide arbitrary
  regressions. An unexpected resolution also fails and requires review.

The previously ignored `directory skip` case already passed on both oracles;
its obsolete expected-failure marker was removed. Generic expected failures
are no longer accepted by the manifest schema. Another old harness gap was
fixed: equal nonzero exits no longer bypass stdout comparison. A regression
test verifies that distinct invalid UTF-8 bytes remain distinct.

`RGREP_STRICT_GNU=1` was tested: it fails exactly the four known differences.
The normal CI is a regression gate with an explicit discrepancy inventory,
not a certificate of full GNU compatibility. Other intentional design
choices (such as ERE by default) remain documented in PORTING.md.

## Reproduction

```sh
RGREP_ORACLE=/opt/homebrew/bin/ggrep cargo test --locked -- --nocapture
RGREP_ORACLE=/opt/homebrew/bin/ggrep cargo test --locked --features perl-regexp -- --nocapture
RGREP_ORACLE=/usr/bin/grep cargo test --locked -- --nocapture
RGREP_ORACLE=/usr/bin/grep cargo test --locked --features perl-regexp -- --nocapture
# Expected to fail until the four semantic differences are resolved:
RGREP_ORACLE=/opt/homebrew/bin/ggrep RGREP_STRICT_GNU=1 cargo test --test diff_runner -- --nocapture
```

On Linux, select `/usr/bin/grep` for the GNU runs. The first remote CI run
validated both Linux configurations; no Linux runtime was used locally.
Next compatibility work should resolve binary diagnostic routing and `-T`
formatting in dedicated, tested steps rather than expanding the exceptions.
