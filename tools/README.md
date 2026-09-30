# tools/

> Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
> AI-assisted port: Claude Opus 4.7 (Anthropic) + Gemini Antigravity (Google)

## convert_gnu_tests.py

Script to automatically parse and extract differential testcases from the `gnu-grep/tests` shell scripts into `rgrep`'s TOML format.

## Usage

```bash
cd rgrep
python3 tools/convert_gnu_tests.py
```

This will read `../gnu-grep/tests/*.sh`, extract `echo ... | grep ...` patterns, and generate `gnu_NNNN_*.toml` files in `tests/cases/`, appending them to `tests/testsuite.toml`.

## Limitations

Because the script cannot execute the actual GNU shell script to capture `expected_stdout`, it generates cases with `expected_to_fail = true` by default. Manual curation is required to set the proper stdout expectations or skip them if they rely on GNU extensions.

The current harness deliberately rejects the legacy `expected_to_fail` field.
Imported cases must be reviewed and curated before adding them to the manifest;
the historical converter is not an unattended ingestion path.

## Performance verification

`summarize_bench.py BASELINE [--compare PRE_BASELINE] [--json OUTPUT]` exports
named Criterion estimates from `target/criterion` and computes median changes
from the same pair. It refuses incomplete comparisons.

`compare_binaries.py BEFORE AFTER [--output OUTPUT]` runs 31 alternating pairs
per CLI workload with three warmups and verifies output/exit parity. Run from
the repository root after `cargo bench --bench search -- line_scan` generates
the amplified fixture. Reports include raw times, binary hashes and bootstrap
intervals; negative deltas mean less elapsed time. No external Python packages
are required.
