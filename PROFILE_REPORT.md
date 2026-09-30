# rgrep Profile Report — Ondata 5

**Date**: 2026-05-18
**Baseline commit**: `a1243e2` (post-Ondata 4 benchmark wave)
**Platform**: Darwin 25.4.0 (Apple Silicon, arm64), 12 cores
**Rust toolchain**: rustc 1.90.0 (1159e78c4 2025-09-14), release build
**Profiler**: samply 0.13.1 (`cargo install samply`)
**Amplification technique**: Option B — single invocation on a 60×-concatenated
fixture (`gnu-grep/src/grep.c` repeated 60 times = 5.4 MB) wrapped in a
30-iteration helper harness compiled standalone with `rustc -O` so samply can
profile the process tree (`bash`-based loops are blocked on macOS because
`bash` is system-signed and rejects samply's DYLD instrumentation).

---

## 1. Methodology

### Tool

samply was chosen over `cargo-flamegraph` because the latter requires `sudo` +
SIP-relaxed on macOS arm64 (it shells out to `dtrace`). samply uses the
userspace `task_for_pid` entitlement and profiles any cargo-installed /
locally-built binary at the requested sampling rate without elevated
privileges. Profiles are emitted as Gecko-format JSON loadable in the
Firefox Profiler (`https://profiler.firefox.com`).

Sampling rate: 4000 Hz. Higher than the 1000 Hz default because the target
runs are short (~30 ms total for the 30-iteration harness), so we need
~120 samples per scenario before stack aggregation to get statistically
useful self-time distributions.

### Signal amplification

D-Ondata-5.3 Option A (bash `for` loop) was blocked: samply on macOS cannot
attach to `bash` (system-signed binary rejects `DYLD_INSERT_LIBRARIES`).
We fell back to a **hybrid Option A/B**:

1. Generated `target/bench_fixtures/giant.c` = `grep.c` concatenated 60 times
   (5.4 MB, 182160 lines). Off the git tree (`target/` is gitignored).
2. Compiled a 5-line Rust helper (`profile_run`) with `rustc -O` that just
   loops `Command::new(rgrep).args(...).output()` `N` times. Cargo-built ⇒
   samply can attach.
3. `samply record --rate 4000 --iteration-count 1 --save-only` invokes the
   helper; the helper spawns 30 rgrep children.
4. samply traces the parent + all children. `--unstable-presymbolicate`
   emits `<name>.syms.json` sidecars so we can resolve addresses to symbol
   names offline (the live UI uses these but we wanted scriptable analysis).

This gives us ~1500–2500 samples on the `rgrep` process per scenario, with
process-spawn overhead siloed in the `profile_run` thread (visible but
analytically separable from the rgrep search hot path).

### Per-scenario analysis

A 200-line Python analyser (`target/bench_fixtures/analyze.py`) walks the
Gecko `threads[].samples.stack → stackTable → frameTable → funcTable →
stringArray` chain, joins each address-shaped name with the matching library
symbol table from the `.syms.json` sidecar (via `bisect` over `(rva, size)`
ranges), and emits "top N by self" and "top N by total" tables per process.
The script is in `target/bench_fixtures/` (gitignored) and is reproducible.

### Profile artefacts (gitignored, not committed)

```
target/profiles/regex_simple.json       250 KB   30-iter on giant.c
target/profiles/regex_simple.syms.json   27 KB   symbol table
target/profiles/invert_match.json       218 KB
target/profiles/invert_match.syms.json   22 KB
target/profiles/fixed_string.json       131 KB
target/profiles/fixed_string.syms.json   19 KB
target/profiles/literal_count.json      230 KB
target/profiles/literal_count.syms.json  19 KB
target/bench_fixtures/giant.c           5.4 MB   grep.c × 60
target/bench_fixtures/profile_run.rs    600 B    harness source
target/bench_fixtures/profile_run       478 KB   rustc -O binary
target/bench_fixtures/analyze.py        5 KB     analyser
```

### Reproducibility

```bash
cd rgrep
# 1. Fixtures
mkdir -p target/bench_fixtures target/profiles
for i in $(seq 1 60); do cat ../gnu-grep/src/grep.c; done > target/bench_fixtures/giant.c

# 2. Harness (source is at target/bench_fixtures/profile_run.rs in this repo's
#    profile session; recreate verbatim from the snippet at end of section 1)
rustc -O target/bench_fixtures/profile_run.rs -o target/bench_fixtures/profile_run

# 3. Profile each scenario
cargo build --release
PROFILE_RUN_ITERS=30 samply record --rate 4000 --save-only --unstable-presymbolicate \
  -o target/profiles/regex_simple.json -- \
  ./target/bench_fixtures/profile_run ./target/release/rgrep -E '^[a-z]+\(' \
  target/bench_fixtures/giant.c

# (repeat with the other 3 scenarios — see per-scenario section for exact args)

# 4. Analyse
python3 target/bench_fixtures/analyze.py target/profiles/regex_simple.json --top 15
```

### Harness source (`target/bench_fixtures/profile_run.rs`)

```rust
use std::process::Command;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { eprintln!("usage: profile_run <binary> [args...]"); std::process::exit(2); }
    let n: usize = std::env::var("PROFILE_RUN_ITERS").ok().and_then(|s| s.parse().ok()).unwrap_or(50);
    for _ in 0..n {
        let _ = Command::new(&args[0]).args(&args[1..]).output().expect("spawn failed");
    }
}
```

---

## 2. Per-scenario findings

All percentages are samples-as-fraction-of-the-rgrep-process-samples; the
parent `profile_run` thread (which only forks/waits) is reported separately
in section 4 for completeness but excluded from the search-cost tables.

### 2.1 `regex_simple` — `-E '^[a-z]+\(' giant.c`

Samples on rgrep: **2358**. Pattern compiles via `regex` crate (ERE,
ASCII-only, anchored start, character class with quantifier, escaped `(`).

**Top self frames** (% of leaf-frame samples):

| %     | Frame                                                                              |
|-------|------------------------------------------------------------------------------------|
| 32.2  | `core::slice::memchr::memchr_aligned`                                              |
| 19.1  | `<core::str::lossy::Utf8Chunks as Iterator>::next`                                 |
|  7.3  | `regex_automata::dfa::search::find_fwd`                                            |
|  6.3  | `read` (syscall)                                                                   |
|  6.3  | `std::io::read_until`                                                              |
|  5.5  | `_platform_memmove`                                                                |
|  4.9  | `<regex::Regex as rgrep::matcher::MatchEngine>::engine_is_match`                   |
|  3.4  | `alloc::string::String::from_utf8_lossy`                                           |
|  3.3  | `rgrep::runner::bufread_search`                                                    |
|  3.2  | `rgrep::runner::process_line`                                                      |
|  2.3  | `regex_automata::meta::strategy::Core::search_half`                                |
|  2.0  | `regex_automata::dfa::automaton::Automaton::start_state_forward`                   |

**Top total frames** (% of samples that have this frame anywhere in the
stack):

| %     | Frame                                                       |
|-------|-------------------------------------------------------------|
| 99.0  | `rgrep::runner::run`                                        |
| 96.4  | `rgrep::runner::bufread_search`                             |
| 50.3  | `std::io::read_until`                                       |
| 32.2  | `memchr_aligned`                                            |
| 22.5  | `String::from_utf8_lossy`                                   |
| 19.1  | `Utf8Chunks::next`                                          |
| 16.5  | `regex::Regex::engine_is_match`                             |
| 11.6  | `regex_automata::meta::strategy::Core::search_half`         |
|  9.2  | `regex_automata::dfa::search::find_fwd`                     |

**Candidate fixes**:

1. **HIGH** — drop UTF-8 lossy validation off the hot path. `Utf8Chunks::next`
   uses a byte-by-byte scan; `std::str::from_utf8` has a usize-block ASCII
   fast path (`run_utf8_validation` in `core::str::validations`). Source
   files are ~100 % ASCII in practice; the fast-path hit rate should be
   essentially 100 %. Hypothesis: ~5–15 % on this scenario.
2. **MEDIUM** — eliminate `_platform_memmove` (5.5 %): `BufReader::read_until`
   does an intermediate copy from the page cache into the `Vec<u8>` line
   buffer. A `fill_buf` + `memchr` + manual slicing rewrite avoids the
   move but adds ~30 LOC and pulls in the `memchr` crate as a direct
   dep — blocked by D-Ondata-5.10 (no new runtime deps).
3. **LOW** — regex DFA start-state caching: `start_state_forward` shows 2.0 %.
   Already cached by `regex_automata` internally; no actionable hook from
   rgrep's side.

### 2.2 `invert_match` — `-v -c '^$' giant.c`

Samples on rgrep: **1996**. Pattern is the literal regex `^$` (empty-line
match) inverted via `-v` and reduced to a count via `-c`.

**Top self frames**:

| %     | Frame                                                                              |
|-------|------------------------------------------------------------------------------------|
| 37.8  | `core::slice::memchr::memchr_aligned`                                              |
| 22.0  | `<core::str::lossy::Utf8Chunks as Iterator>::next`                                 |
|  7.8  | `read`                                                                             |
|  7.3  | `_platform_memmove`                                                                |
|  6.2  | `std::io::read_until`                                                              |
|  3.9  | `rgrep::runner::bufread_search`                                                    |
|  3.7  | `rgrep::runner::process_line`                                                      |
|  2.9  | `alloc::string::String::from_utf8_lossy`                                           |
|  2.8  | `<regex::Regex as rgrep::matcher::MatchEngine>::engine_is_match`                   |
|  0.6  | `regex_automata::dfa::search::find_fwd`                                            |

**Top total frames** (relevant subset):

| %     | Frame                                                       |
|-------|-------------------------------------------------------------|
| 25+   | `String::from_utf8_lossy`                                   |
| 22.0  | `Utf8Chunks::next`                                          |

The regex engine itself is **0.6 %** here — `^$` is a degenerate pattern
that the DFA short-circuits in a single state transition. The hot path is
**100 % I/O + UTF-8 validation**. We never need a `&str` view at all in
this mode (count-only, no print, pattern can match on bytes), so the
validation is pure waste.

**Candidate fixes**:

1. **HIGH** — same as 2.1 fix #1 (UTF-8 fast path). Hypothesis: ~10–20 %
   on this scenario specifically because validation is a larger fraction
   of the total.
2. **MEDIUM** — `is_match_bytes` API: feed `&[u8]` directly to the matcher
   when `output.count || output.quiet || output.files_with_matches ||
   output.files_without_match`. Each engine impl (`regex`, `fancy`,
   `aho-corasick`, `pcre2`) can accept bytes — `regex` would need to use
   `regex::bytes::Regex` instead of `regex::Regex`. ~50 LOC across
   `matcher.rs` and `runner.rs`. Hypothesis: another ~5–10 % on top of
   fix #1 for the count/quiet modes specifically.
3. **LOW** — fixed empty-line shortcut: detect `^$` as a special case and
   replace with `buffer.iter().all(|b| *b == b'\n')`. Pattern-specific,
   not generalisable. Rejected.

### 2.3 `fixed_string_F` — `-rF MB_LEN_MAX ../gnu-grep/src/`

Samples on rgrep: **263**. Lowest sample count because the scenario is
dominated by directory enumeration, not search. The recursive walk over
`gnu-grep/src/` opens ~20 files and reads each.

**Top self frames**:

| %     | Frame                                                                              |
|-------|------------------------------------------------------------------------------------|
| 14.8  | `__open` (syscall)                                                                 |
| 13.3  | `fstatfs` (syscall)                                                                |
|  9.1  | `__getdirentries64` (syscall)                                                      |
|  8.4  | `core::slice::memchr::memchr_aligned`                                              |
|  7.2  | `<core::str::lossy::Utf8Chunks as Iterator>::next`                                 |
|  6.1  | `read`                                                                             |
|  3.8  | `memchr::memmem::searcher::searcher_kind_neon`                                     |
|  3.4  | `_platform_memmove`                                                                |
|  2.7  | `std::io::read_until`                                                              |
|  2.3  | `aho_corasick::automaton::try_find_fwd`                                            |

**Surprises**:

- The hot path here is **dominated by FS syscalls** (~37 % combined: open +
  fstatfs + getdirentries). This is `walkdir::WalkDir` traversing the
  directory tree. The actual matching is only ~6 % (aho-corasick neon
  search + try_find_fwd ~6 %).
- BSD grep's lead on this scenario (+27.2 % per PERF_REPORT.md) is therefore
  **not** about search speed — it's about how cheaply each side does the
  directory walk. BSD grep uses a hand-rolled `opendir`/`readdir` loop
  with minimal allocations; `walkdir` does more bookkeeping per entry
  (DirEntry struct, metadata caching, follow-link decisions).

**Candidate fixes**:

1. **HIGH** — UTF-8 fast path (7.2 % self). Smaller win here in relative
   terms but still useful.
2. **MEDIUM** — `walkdir` configuration: disable `follow_links` metadata
   probing if not requested. Looking at the source, we already only set
   `follow_links(config.filter_opts.dereference_recursive)` which is `false`
   in this scenario, so this is already optimal.
3. **REJECTED** — replace `walkdir` with a hand-rolled traversal. Would
   plausibly close ~10–15 % but it's a 100+ LOC change and D-Ondata-5.10
   forbids architectural reorganisation.
4. **REJECTED** — parallel walk via `rayon`. Explicit out-of-scope
   (D-Ondata-5.10), reserved for a future "parallelism" wave.

### 2.4 `literal_count` (control) — `-c include giant.c`

Samples on rgrep: **2434**. Best-case scenario in PERF_REPORT.md (+5.5 %
vs BSD grep). Single file, literal pattern routed through aho-corasick.

**Top self frames**:

| %     | Frame                                                                              |
|-------|------------------------------------------------------------------------------------|
| 33.5  | `core::slice::memchr::memchr_aligned`                                              |
| 19.2  | `<core::str::lossy::Utf8Chunks as Iterator>::next`                                 |
|  8.6  | `read`                                                                             |
|  8.6  | `memchr::memmem::searcher::searcher_kind_neon`                                     |
|  5.2  | `std::io::read_until`                                                              |
|  5.1  | `_platform_memmove`                                                                |
|  5.0  | `<regex::Regex as rgrep::matcher::MatchEngine>::engine_is_match`                   |
|  4.0  | `rgrep::runner::process_line`                                                      |
|  3.6  | `rgrep::runner::bufread_search`                                                    |
|  3.0  | `alloc::string::String::from_utf8_lossy`                                           |

Pattern: same top-2 frames as `regex_simple` and `invert_match` (memchr +
Utf8Chunks). The regex engine `engine_is_match` is **5.0 %** vs **4.9 %**
on regex_simple — essentially identical. So *the regex engine itself is
not the differentiator* between the +5.5 % and +30.8 % rows; the
differentiator is something else in the per-line path.

---

## 3. Allocation hotspots

samply does not track heap allocations by default on macOS (no
malloc-stack-logging integration). However, the leaf samples on the
allocator are visible:

- `libsystem_malloc.dylib!0x2aa1c` (and siblings): negligible (<1 %) on
  all 4 scenarios.
- `alloc::string::String::from_utf8_lossy` self-time: 3.0–3.4 % across
  the 3 line-mode scenarios. This includes the Cow construction when
  validation succeeds (the fast path returns `Cow::Borrowed`, no
  allocation). When validation fails, a new `String` is allocated. On
  pure-ASCII input (our case), the validation never fails ⇒ zero
  allocation ⇒ the 3 % cost is **pure validation overhead, not allocation**.

This is consistent with the byte-by-byte `Utf8Chunks::next` scan being
the dominant cost: it does the validation work, and the Cow wrapper is
~free on top of it.

---

## 4. Comparative analysis

What do the **3 worst scenarios share** that the **control does not**?

Looking at the per-scenario `engine_is_match` self-time:

| Scenario              | `engine_is_match` self | Δ vs BSD grep |
|-----------------------|------------------------|---------------|
| `regex_simple`        |  4.9 %                 | +30.8 %       |
| `invert_match`        |  2.8 %                 | +28.9 %       |
| `fixed_string_F`      |  2.3 % (aho-corasick)  | +27.2 %       |
| `literal_count` (ctrl)|  5.0 %                 |  +5.5 %       |

The engine cost is **higher on the control** than on two of the three
"worst" scenarios. The matching cost is **not** the differentiator.

Looking at `Utf8Chunks::next`:

| Scenario              | Utf8Chunks self |
|-----------------------|-----------------|
| `regex_simple`        | 19.1 %          |
| `invert_match`        | 22.0 %          |
| `fixed_string_F`      |  7.2 %          |
| `literal_count` (ctrl)| 19.2 %          |

UTF-8 validation is a **constant ~19–22 % tax on the line-mode scenarios**,
and dropping to ~7 % only on `fixed_string_F` because that scenario is
syscall-bound rather than line-bound.

Looking at `memchr_aligned`:

| Scenario              | memchr_aligned self |
|-----------------------|----------------------|
| `regex_simple`        | 32.2 %               |
| `invert_match`        | 37.8 %               |
| `fixed_string_F`      |  8.4 %               |
| `literal_count` (ctrl)| 33.5 %               |

Same pattern: ~33–38 % on line-mode, drops on syscall-bound.

### Key finding

The **per-line glue cost** (`read_until` → `memchr_aligned` → `from_utf8_lossy`
→ `Utf8Chunks::next`) is essentially **identical across all 4 line-mode
scenarios at ~50–60 % of total time**. The differentiator between the
control's +5.5 % and the worst scenarios' +30 % is **not in our code at all** —
it's in the absolute baseline: the control runs on **5 short files
(20 KB total)** where rgrep + BSD grep are both dominated by process-spawn
overhead, while the worst scenarios run on **a single ~5 MB synthetic file
(or the equivalent ~92 KB grep.c at criterion time)** where BSD grep's
hand-rolled inner loop pulls ahead.

Therefore: **a fix that reduces the per-line glue cost helps all
line-mode scenarios proportionally**, and the easiest such fix targets
the `Utf8Chunks` byte-by-byte validation.

The walking-bound scenario (`fixed_string_F`) is structurally different
and would need a different fix (walkdir replacement or parallel walk),
both out-of-scope per D-Ondata-5.10.

---

## 5. Selected fix(es)

Two fixes were applied and validated against the Ondata 4 baseline
(`a1243e2`) via a full `cargo bench` re-run. Numbers below are the
post-fix median vs the Ondata 4 PERF_REPORT.md median.

### PERF-FIX-O5-1 — UTF-8 validation fast path

**File**: `src/runner.rs`, in `bufread_search`.

**Hypothesis**: `String::from_utf8_lossy(&buffer)` calls
`Utf8Chunks::new(buffer).next()` which scans byte-by-byte. For ASCII input
(our case, ~100 %) this leaves the usize-block fast-path of
`core::str::run_utf8_validation` on the table. Switching to
`std::str::from_utf8` first, with `from_utf8_lossy` as a fallback only on
error, should reduce the validation cost by 4–8× per line.

**Expected delta**: −5 to −10 % median on `regex_simple`, `invert_match`,
`literal_count`.

**Validated delta**: marginal, ~−1.5 % on `regex_simple`,
~−1 % on `invert_match`, within criterion's noise envelope on most
scenarios. Honest verdict: **the fix is a correct optimisation but its
effect on `cargo bench` is below the 5 % bar in isolation**. The
attribution misread the profile: `Utf8Chunks::next` shows at 19–22 % on
the *amplified* 5.4 MB fixture, but on the 92 KB `grep.c` corpus used by
criterion the absolute time spent in validation is small relative to the
process-spawn overhead (~50 % of the 3.5 ms total). The fix is retained
because (a) it is unambiguously cheaper than the original code on every
input, (b) it costs ~5 LOC, and (c) it combines positively with
PERF-FIX-O5-2 below.

**Diff size**: 1 `use std::borrow::Cow` import + 3 LOC in `bufread_search`
+ 6 LOC of attribution comment = **10 LOC**.

### PERF-FIX-O5-2 — slurp-and-cursor for small regular files

**File**: `src/runner.rs`, in `search_file`.

**Hypothesis**: PERF_REPORT.md §7 showed `--mmap` is ~12 % faster than
the default `BufReader` path on a 92 KB file, with a substantially
tighter distribution. Profile shows the streaming `BufReader` path
spends 5–7 % self in `_platform_memmove` (per-line copy from the 8 KB
internal buffer into the line `Vec<u8>`) and another 6–8 % in the `read`
syscall (one per 8 KB chunk). For files small enough to fit in
memory (we cap at 16 MB), reading the whole content into a single
`Vec<u8>` and parsing via `Cursor` consolidates I/O into one syscall and
removes the per-line refill overhead — replicating the mmap shape without
the mmap syscall.

**Expected delta**: −8 to −12 % median on small-file `-c` scenarios
(`literal_match_count`, `bufread_default`, `color_*_count_only`). Marginal
on scenarios that already use mmap or that are syscall-bound on directory
traversal (`fixed_string_F`).

**Validated delta** (median, vs Ondata 4 `a1243e2` baseline):

| Scenario                           | Pre (ms) | Post (ms) | Δ %     | Status |
|------------------------------------|----------|-----------|---------|--------|
| `literal_match_count`              | 4.444    | 4.023     | **−9.5 %**  | ✅ ≥5 % |
| `mmap_vs_bufread/bufread_default`  | 4.141    | 3.579     | **−13.6 %** | ✅ ≥5 % |
| `cow_impact/color_never_count_only`| 4.174    | 3.515     | **−15.8 %** | ✅ ≥5 % |
| `mmap_vs_bufread/mmap_explicit`    | 3.648    | 3.479     | −4.7 %  | sub-5 % |
| `cow_impact/color_never_with_output`| 3.642   | 3.554     | −2.4 %  | sub-5 % |
| `invert_match`                     | 3.626    | 3.586     | −1.1 %  | sub-5 % |
| `recursive_walk`                   | 4.547    | 4.558     | +0.2 %  | flat   |
| `regex_simple`                     | 3.691    | 3.720     | +0.8 %  | flat   |
| `cow_impact/color_always_count_only`| 3.699   | 3.736     | +1.0 %  | flat   |
| `cow_impact/color_always_with_output`| 3.707  | 3.783     | +2.0 %  | flat   |
| `fixed_string_F`                   | 4.468    | 4.586     | +2.7 %  | flat   |

**Result**: D-Ondata-5.6 criterion "Δ ≥5 % sulla mediana del scenario
target" is met on **three** scenarios (literal_match_count, bufread_default,
color_never_count_only). The largest regression is +2.7 % on
`fixed_string_F`, well **below** the 5 % regression bar.

**Caveat on the SPEC-defined "3 worst" targets**: the three scenarios that
D-Ondata-5.2 listed as profile targets (`regex_simple`, `invert_match`,
`fixed_string_F`) did **not** individually hit the ≥5 % bar. Their
bottleneck on the criterion corpus turned out to be the per-line
`memchr_aligned` cost (32–38 % self in the profile), which can only be
materially improved by introducing the `memchr` crate as a direct
runtime dependency — explicitly forbidden by D-Ondata-5.10. The
validation runs against `mmap_vs_bufread/bufread_default` cover the same
hot path (`BufRead::read_until` → `memchr_aligned` → per-line memcpy) and
show the fix is structurally correct; the criterion noise floor on the
3 worst scenarios is just wide enough to swallow the absolute Δ.
The relative position vs BSD grep on `regex_simple` improved
(`+30.8 %` → `+28.4 %`) because BSD grep's own median drifted upward in
the re-bench, but this is not a strong claim — both sides are within
each other's noise envelopes.

**Diff size**: 19 LOC in `search_file` (15 logic + 4 attribution comment).

**Test impact**: zero. All 36 tests pass with both fixes applied
(`cargo test` IDENTICO baseline `a1243e2`).

### Rejected candidates

- **`is_match_bytes` API** (section 2.2 fix #2): would add another ~5–10 %
  on count/quiet modes specifically, but requires `regex::bytes::Regex`
  alongside `regex::Regex` (or a full swap), which would more than double
  the matcher dispatch surface and very likely exceed the 50 LOC cap.
  **Deferred to Ondata 6+**.

- **`memchr` crate as direct dep** (section 2.1 fix #2): blocked by
  D-Ondata-5.10 ("no new runtime deps"). The `memchr` crate is in the
  transitive graph (via `regex`) but not exposed.

- **`walkdir` replacement** (section 2.3 fix #3): blocked by
  D-Ondata-5.10 ("no module reorganisation"). ~100 LOC and would
  conflict with the `aho-corasick` integration in `process_line`.

- **Parallel walk via `rayon`** (section 2.3 fix #4): explicit
  out-of-scope per D-Ondata-5.10 ("Threading"), reserved for Ondata 6+.

---

## 6. Future work (deferred, in scope of Ondata 6+)

- **`is_match_bytes` API**: ~5–10 % win on count/quiet modes. ~50 LOC.
  Tracked as Ondata 6 candidate.
- **`walkdir` replacement with hand-rolled `read_dir`**: ~10–15 % win on
  `fixed_string_F` and `recursive_walk`. Architectural change, requires
  its own design wave.
- **Parallel walk via `rayon`**: explicit reserved future "parallelism"
  wave. Would target `recursive_walk` and `fixed_string_F`.
- **`memchr` crate direct dep + manual `fill_buf` loop**: would replace
  `BufRead::read_until` with a NEON-aware scan. ~30 LOC + Cargo.toml.
  Plausible Ondata 7 if NEON memchr gives >5 % over the existing std
  `memchr_aligned`.
- **Allocation tracking via `dhat`**: dev-dep only, optional feature.
  Would let us validate "zero allocations on ASCII input after FIX-O5-1"
  empirically. ~20 LOC + Cargo.toml `[features]` block.
- **`regex::bytes::Regex` engine swap**: dedicated "engine swap" wave.
  Would close most of the `regex_simple` / `invert_match` gap; requires
  rewriting `Matcher` and the trait. ~150 LOC.

# Ondata 7 re-profile — 2026-09-30

Same rustc 1.98.1, `-vc '^$'` on grep.c ×60, 60 child runs, 4000 Hz.
Pre-change is `723aab3`; post-change uses the shared memchr reader. Existing
section 1 harness/analyser commands apply with PROFILE_RUN_ITERS=60 and
`target/bench_fixtures/line_scan.c`. Full text summaries are committed in
`reports/o7-profile-before.txt` and `reports/o7-profile-after.txt`; raw
samply JSON remains gitignored in `target/profiles/o7-{before,after}.json`.

| Metric | Before | After |
|---|---:|---:|
| rgrep samples | 2923 | 2331 |
| std memchr_aligned self | 46.3% | 4.8% |
| new read_delimited self (includes inlined scanner) | — | 29.3% |
| memmove self | 10.8% | 19.9% |

Do not treat disappearing symbols or changing sample shares as absolute
speedups: scanner work is partly attributed to the inlined helper. The
paired wall-time benchmark independently measures about −12% for this
workload. Copies and UTF-8 validation remain material; no claim that all
scanning now consumes <15% is made. The older fixed_string_F profile had
8.4% memchr and was filesystem-bound, not a 32–38% scanner target.
