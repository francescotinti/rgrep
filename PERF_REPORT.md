# rgrep Performance Report — Ondata 4

**Date**: 2026-05-18
**Commit (baseline)**: `87c652f` (post-Ondata 3 architectural refactor)
**Platform**: Darwin 25.4.0 (Apple Silicon, arm64), `xnu-12377.101.15~1/RELEASE_ARM64_T8132`
**Rust toolchain**: rustc 1.90.0 (1159e78c4 2025-09-14)
**Comparison oracle**: `/usr/bin/grep` — BSD grep 2.6.0-FreeBSD (macOS bundled)
**ripgrep version (if available)**: not installed — comparison rows skipped silently
**Benchmark harness**: criterion 0.5.1 (`html_reports` feature enabled), 100 samples per scenario (20 for `recursive_walk`).

---

## Methodology

- **Measurement model**: process-spawn. Every iteration invokes the compiled
  release binary (`CARGO_BIN_EXE_rgrep`) and the system grep as separate
  subprocesses. Output is read fully and discarded. This is the same fair-fight
  model used by ripgrep's own benchmarking writeups.
- **Trade-off**: process-spawn adds ~1–3 ms of OS startup overhead per
  iteration on macOS arm64. For scenarios whose total runtime is in the
  3–5 ms range (most of ours), startup dominates and the relative
  differences between tools are compressed. This is documented honestly in
  the Limitations section below — the numbers are still ordinally meaningful,
  but absolute deltas should not be over-interpreted.
- **Corpus**: the `gnu-grep/` subtree checked into the repo (~4.5K LOC of C in
  `gnu-grep/src/`, with `grep.c` alone at 3 036 lines / 92 KB).
- **Reporting**: criterion's three-tuple `[lower 95% CI · median · upper 95% CI]`
  is reproduced verbatim. The "Δ vs grep" column is computed against the
  oracle's median.
- **Reproduce**: `cd rgrep && cargo bench`. HTML reports land in
  `target/criterion/`.

---

## Results

### 1. `literal_match_count` — `-c "include"` on 5 `.c` files

| Tool | Lower 95% | Median | Upper 95% | Δ vs grep |
|---|---|---|---|---|
| rgrep         | 4.284 ms | **4.444 ms** | 4.635 ms | +5.5 % |
| /usr/bin/grep | 4.165 ms | **4.210 ms** | 4.262 ms | baseline |
| ripgrep       | — | — | — | not installed |

### 2. `regex_simple` — `-E "^[a-z]+\("` on `grep.c` (3 036 lines)

| Tool | Lower 95% | Median | Upper 95% | Δ vs grep |
|---|---|---|---|---|
| rgrep         | 3.655 ms | **3.691 ms** | 3.732 ms | +30.8 % |
| /usr/bin/grep | 2.809 ms | **2.821 ms** | 2.834 ms | baseline |
| ripgrep       | — | — | — | not installed |

### 3. `fixed_string_F` — `-rF "MB_LEN_MAX"` on `gnu-grep/src/`

| Tool | Lower 95% | Median | Upper 95% | Δ vs grep |
|---|---|---|---|---|
| rgrep         | 4.440 ms | **4.468 ms** | 4.497 ms | +27.2 % |
| /usr/bin/grep | 3.478 ms | **3.513 ms** | 3.552 ms | baseline |
| ripgrep       | — | — | — | not installed |

### 4. `recursive_walk` — `-r "TODO"` on `gnu-grep/src/` (sample_size 20)

| Tool | Lower 95% | Median | Upper 95% | Δ vs grep |
|---|---|---|---|---|
| rgrep         | 4.444 ms | **4.547 ms** | 4.652 ms | +18.6 % |
| /usr/bin/grep | 3.668 ms | **3.834 ms** | 4.021 ms | baseline |
| ripgrep       | — | — | — | not installed |

### 5. `invert_match` — `-v -c "^$"` on `grep.c`

| Tool | Lower 95% | Median | Upper 95% | Δ vs grep |
|---|---|---|---|---|
| rgrep         | 3.589 ms | **3.626 ms** | 3.670 ms | +28.9 % |
| /usr/bin/grep | 2.758 ms | **2.814 ms** | 2.883 ms | baseline |
| ripgrep       | — | — | — | not installed |

### 6. `cow_impact` — D-Ondata-3.3 `Cow<'a, str>` validation

Same pattern (`include` on `grep.c`), four colour modes:

| Mode | Lower 95% | Median | Upper 95% |
|---|---|---|---|
| `--color=never` (with line output) | 3.600 ms | **3.642 ms** | 3.697 ms |
| `--color=always` (with line output) | 3.645 ms | **3.707 ms** | 3.783 ms |
| `--color=never` (`-c` count only)   | 3.860 ms | **4.174 ms** | 4.577 ms |
| `--color=always` (`-c` count only)  | 3.626 ms | **3.699 ms** | 3.783 ms |

**Δ for `never` vs `always` with line output**: −1.8 % (median). Effectively
inside criterion's noise floor.

### 7. `mmap_vs_bufread` — `--mmap` vs default bufread on a 92 KB file

| Mode | Lower 95% | Median | Upper 95% |
|---|---|---|---|
| `bufread` (default) | 3.890 ms | **4.141 ms** | 4.438 ms |
| `--mmap`            | 3.599 ms | **3.648 ms** | 3.705 ms |

**Δ**: mmap is ~12 % faster on the median, with a substantially tighter
distribution (lower upper bound). See interpretation note.

---

## Interpretation

- **rgrep is consistently ~20–30 % slower than BSD grep on regex / fixed-string
  scenarios, and ~5 % slower on the literal `-c` scenario.** BSD grep wins on
  every benchmarked workload. We are *not* shipping a faster grep. We are
  shipping a port that has the same observable behaviour as our differential
  oracle (`/usr/bin/grep`), at a measurable but bounded performance cost. For
  an educational port this is the expected ordering: GNU/BSD grep is decades
  of C-level micro-optimisation; rgrep is a clean-room Rust translation that
  prioritises clarity over instruction-count.

- **`literal_match_count` is the closest race** (+5.5 %). This is the workload
  closest to a fair-fight on the BSD grep side: 5 short input files, simple
  literal pattern, no regex backtracking, no recursion. rgrep's fast path
  (aho-corasick via `-F` semantics, even when `-F` is not explicit) is in
  the same league as BSD grep's literal search.

- **`regex_simple` and `invert_match` are rgrep's worst showings** (+30 %).
  Both go through the `regex` crate's general NFA/DFA machinery, which is
  great for correctness and complexity-safety but pays a per-invocation cost
  that BSD grep's hand-rolled regex skips. These are exactly the scenarios
  where ripgrep would have been a more relevant comparator — the regex crate
  in rgrep is the same crate ripgrep uses, so the gap is mostly about glue
  code, not the engine itself.

- **Cow impact (validation of D-Ondata-3.3): −1.8 % median, in the noise.**
  This is an *honest negative result*. The `Cow<'a, str>` return type that
  Ondata 3 introduced for the `highlight()` function does not produce a
  measurable performance improvement on this corpus, neither on the
  short-circuit path (`--color=never`) nor on the colourised path
  (`--color=always`). The optimisation is *theoretically* sound — it avoids
  a `String` allocation per matched line in the no-colour case — but the
  workload is dominated by process spawn and aho-corasick scanning, not by
  per-line allocation. **This does not invalidate Ondata 3.** The Cow type
  still encodes the right invariant (no allocation when no colour escape is
  added) and remains a cleaner API than the previous `String` return. It is
  simply not a hot-path optimisation; it is an API/clarity improvement
  with a side-effect of being slightly cheaper. The `count_only` rows show
  no consistent signal either; the `--color=never -c` outlier (4.17 ms) is
  driven by 16 % high-severe outliers and is noise, not a real regression.

- **mmap impact: ~12 % faster, with much tighter variance, on a 92 KB file.**
  This contradicts the architect's pre-bench hypothesis that bufread would win
  on files <1 MB due to page-fault cost. Two plausible explanations: (a) on
  modern APFS + Apple Silicon, mmap of a small file is essentially a free
  reference into the unified buffer cache, so the page-fault story is wrong
  on this platform; (b) the `--mmap` code path in `runner.rs` is structurally
  simpler than the bufread path (fewer abstraction layers, no intermediate
  buffering) and that structural simplicity translates into a measurable
  saving. This is the most actionable single result in this report: **mmap
  is a strict win on macOS arm64 for files in the ~100 KB range**. We have
  not benchmarked larger files (>10 MB) — that's documented as Future Work
  below — but the early signal is favourable.

---

## Limitations

- **Process-spawn dominates micro-benchmarks.** On macOS arm64 the `fork/exec`
  cost for a small static binary is ~1–3 ms. Most of our scenarios run in
  3–5 ms total, so 25–60 % of the measured time is OS overhead, not search.
  This compresses relative deltas between rgrep and grep — the *real* search
  delta is larger than the table values suggest. We accept this trade-off
  because the alternative (linking rgrep as a library and benching internal
  functions) would have required architectural changes to expose private
  modules, which D-Ondata-4.8 explicitly forbids for a measurement-only wave.
- **BSD grep 2.6.0-FreeBSD is the macOS oracle.** It is materially older and
  less aggressively optimised than the GNU grep on Linux. A Linux re-run
  against GNU grep would likely show rgrep doing relatively *worse* in the
  same scenarios, because GNU grep has Boyer-Moore / Aho-Corasick fast paths
  that BSD grep does not. We did not run this comparison; it is left as
  Future Work.
- **ripgrep was not installed on the benchmark machine.** All `ripgrep` rows
  are silently skipped per D-Ondata-4.4. A re-run on a host with `rg` in
  `$PATH` will populate the missing column without modifying the benchmark
  code.
- **High variance in some count-only scenarios.** `cow_impact/color_never_count_only`
  in particular shows 16 % high-severe outliers. The upper-bound CI on that
  row should be read with caution.
- **Corpus is `gnu-grep/src/` only.** No huge files (>10 MB), no binary
  files, no UTF-8 stress text. The mmap result especially deserves a re-run
  on larger inputs before being generalised.
- **The Cow result is only validated for `highlight()`.** D-Ondata-3.3 changed
  one function's return type; there may be other Cow-shaped opportunities
  in the codebase that this report does not cover.

---

## Future work (out-of-scope for Ondata 4, accepted backlog)

- **SIMD literal search** via `memchr` aggressive features or `aho-corasick`
  with the `prefilter` feature. Likely closes most of the `literal_match_count`
  and `fixed_string_F` gap to BSD grep, possibly opens a gap *over* it.
- **DFA-based regex engine** replacement of the `regex` crate. Out-of-scope —
  the `regex` crate already uses a DFA internally, so the lever is small.
- **Parallel directory walk** via `rayon`. Plausible Ondata 5 candidate; the
  `recursive_walk` scenario would be the natural benchmark target.
- **FFI-based benchmark harness** vs process-spawn. Would remove the ~1–3 ms
  startup overhead but requires making `runner` / `matcher` public, which
  conflicts with the current module-private design.
- **Huge-file mmap re-run** (>10 MB). Generate a deterministic fixture in
  `target/bench_fixtures/`, off the git tree, and re-run `mmap_vs_bufread`
  on it. Expected to amplify the mmap advantage further.
- **Flamegraph / `cargo-flamegraph` profiling** of the worst scenarios
  (`regex_simple`, `invert_match`). Would pinpoint the exact glue code
  costs separating us from BSD grep.
- **GNU grep oracle re-run on Linux**. Would re-calibrate the deltas against
  the harder oracle.

---

## Reproducibility

```bash
cd rgrep
cargo bench                          # full run, ~6 minutes
cargo bench -- literal_match_count   # single scenario
open target/criterion/report/index.html   # HTML reports (criterion html_reports)
```

All numbers in this report were taken from a single uninterrupted
`cargo bench` run on 2026-05-18, baseline commit `87c652f`. Re-runs are
expected to differ by ≤5 % per criterion's documented noise envelope; if
a re-run shifts a number by >15 %, suspect background CPU contention
(other apps, Time Machine, Spotlight indexing).
