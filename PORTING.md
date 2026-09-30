# Porting Overview

> Updated 2026-09-30: 2,379 Rust source lines (including inline tests),
> 137 manifest cases and 46 default / 48 PCRE2 Rust tests. GNU differences
> and actual executed coverage are tracked in `GNU_VERIFICATION.md`.
> Functional scope is complete; full GNU compatibility is not claimed.

> Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
> AI-assisted port — Architect: Claude Opus 4.7 (Anthropic) · Implementer: Gemini Antigravity (Google)

`rgrep` is an AI-assisted port of GNU grep from C to Rust. This document
summarizes **what** was delivered. The internal methodology used to drive
the port is intentionally not documented here.

---

## Source vs target

| | Origin | Target |
|---|---|---|
| Language | C | Rust 2024 edition |
| Reference | [GNU grep](https://git.savannah.gnu.org/cgit/grep.git) (~4.5K LOC C, 5 modules) | `rgrep` (~2.4K LOC Rust, 7 modules) |
| Build | autotools + gnulib | cargo |
| Crate ecosystem | n/a | `regex`, `aho-corasick`, `walkdir`, `globset`, `memmap2`, `pcre2` (optional) |

**Source size**: ~4500 C lines versus 2379 Rust lines including inline tests.
Crate implementations are excluded; this is not an algorithmic equivalence metric.

## Feature parity (vs GNU grep)

### Pattern selection
- `-E` / `--extended-regexp` (default flavor in `rgrep`, see Divergences)
- `-G` / `--basic-regexp` via in-house BRE→ERE translator
- `-F` / `--fixed-strings` via `aho-corasick` for multi-pattern literal search
- `-P` / `--perl-regexp` via `pcre2` crate (feature-gated; opt-in build)
- `-e PATTERN` (repeatable, OR semantics)
- `-f FILE` (patterns from file, line-by-line)

### Matching control
- `-i` / `--ignore-case`, `--no-ignore-case`
- `-v` / `--invert-match`
- `-w` / `--word-regexp`
- `-x` / `--line-regexp`

### Output control
- `-n` / `--line-number`
- `-b` / `--byte-offset`
- `-H` / `--with-filename`, `-h` / `--no-filename`
- `-c` / `--count`
- `-l` / `--files-with-matches`, `-L` / `--files-without-match`
- `-o` / `--only-matching`
- `--color[=WHEN]` with `GREP_COLORS` env parsing
- `-T` / `--initial-tab`
- `--label=LABEL`
- `--line-buffered`

### Context control
- `-A NUM` / `--after-context`
- `-B NUM` / `--before-context`
- `-C NUM` / `--context`
- `--group-separator=STRING`, `--no-group-separator`

### File traversal
- `-r` / `--recursive` (no symlink follow)
- `-R` / `--dereference-recursive` (follow symlinks)
- `-d ACTION` / `--directories={read,recurse,skip}`
- `-D ACTION` / `--devices={read,skip}`

### Filtering
- `--include=GLOB` (whitelist)
- `--exclude=GLOB` (blacklist)
- `--exclude-dir=GLOB` (subtree pruning)
- `--exclude-from=FILE`

### Limits
- `-m NUM` / `--max-count` (per-file early-exit)
- `-q` / `--quiet`, `--silent`
- `-s` / `--no-messages`

### Binary handling
- `-a` / `--text` (force text mode)
- `-U` / `--binary` (force binary mode, no CRLF stripping)
- `--binary-files={binary,text,without-match}`
- `-I` (alias for `--binary-files=without-match`)
- NUL-byte heuristic detection

### NUL data
- `-z` / `--null-data` (record separator = NUL instead of newline)
- `-Z` / `--null` (NUL after filename instead of `:`)

### Performance
- `--mmap` opt-in via `memmap2` crate (single isolated `unsafe` block;
  output byte-equivalent to `BufReader` path; automatic fallback for
  stdin/devices/empty files)

### Exit codes (POSIX)
- `0` — match found
- `1` — no match
- `2` — error (regex syntax, file not found, IO failure)

## Test surface

| Type | Count |
|---|---|
| Manifest testcases | 137 (see GNU_VERIFICATION.md for actual parity/skip counts) |
| Property-based comparisons | 3072 per GNU run; skipped on BSD |
| Library unit tests | 38 default, 40 with PCRE2 |
| Other Rust tests | 3 boundary tests + 2 harness tests + 3 properties |

The differential harness invokes `rgrep` and the system `grep` with
identical args+stdin, then compares stdout byte-for-byte and exit codes.
Property-based tests use `proptest` to generate random valid patterns and
inputs across three strategies (literal, anchored regex, character classes).

## Documented divergences (vs GNU grep canonical)

1. **Default regex flavor = ERE (`-E`)** instead of BRE. Modern CLI
   convention (ripgrep, ag). `-G` triggers a BRE→ERE pre-processor for
   compatibility.
2. **`--color` default = `never`** instead of `auto`. Avoids TTY auto-
   detection complexity; `--color=auto` still supported explicitly.
3. **`-P` requires opt-in build** (`--features perl-regexp`). Default
   build emits parity error message and exit 2, matching upstream
   "compiled without PCRE support" behavior.

## Build

```bash
cargo build --release                  # default
cargo build --release --features perl-regexp  # with PCRE2 support
```

## License

Open-source educational and practical use. Built on top of well-known
Rust ecosystem crates (each retains its own license).
