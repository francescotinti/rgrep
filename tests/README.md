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
