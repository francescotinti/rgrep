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
