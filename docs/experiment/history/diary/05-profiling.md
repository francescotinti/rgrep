# Diary 05 — Profile-driven performance work (Ondata 5)

> Wave 5 of the rgrep porting experiment. Profile-driven targeted fixes
> on the rgrep binary, executed by the Architect-Auditor directly
> (no Gemini relay) per D-Ondata-5.12. Baseline `a1243e2` (post-Ondata 4),
> post-fix commit to be assigned. See `rgrep/PROFILE_REPORT.md` for the
> raw profile findings and `rgrep/PERF_REPORT.md` "Ondata 5 post-fix"
> section for the validated numbers.

This diary answers the **5 skill questions for the legacy-port skill in
its perf-engineering variant**, as scoped by D-Ondata-5.7.

---

## 1. Methodology — tool, why, signal amplification

**Tool**: `samply 0.13.1` (Markus Stange / Mozilla). Chosen over the
SPEC's other candidates because:

- `cargo-flamegraph` on macOS arm64 requires `sudo` + SIP-relaxed: it
  shells out to `dtrace`, which only works with the root entitlement
  System Integrity Protection grants to the kernel.
- `cargo-instruments` (Apple Instruments wrapper) works but emits a
  proprietary `.trace` bundle that does not lend itself to scripted
  offline analysis — we wanted Python-parseable JSON.
- samply uses userspace `task_for_pid` and produces Gecko-format JSON
  loadable in Firefox Profiler. With `--unstable-presymbolicate` it also
  emits a `.syms.json` sidecar with the full symbol table, which let us
  write a 200-line Python analyser that walks
  `threads[].samples.stack → stackTable → frameTable → funcTable →
  stringArray` and joins each address-shaped name to its symbol via
  `bisect` over `(rva, size)` ranges. The analyser lives in
  `target/bench_fixtures/analyze.py` (gitignored) and is reproducible.

**Signal amplification**: D-Ondata-5.3 listed three options. We hit an
unforeseen obstacle on Option A (the bash-loop pattern): samply on
macOS cannot attach to `bash` because the system-signed `/bin/bash`
binary rejects `DYLD_INSERT_LIBRARIES`, which samply needs to siphon
out the mach task port. We fell back to a hybrid:

1. Generated `target/bench_fixtures/giant.c` = `grep.c` concatenated 60×
   (5.4 MB) — D-Ondata-5.3 Option B substrate.
2. Compiled a 10-line Rust helper (`profile_run`) with `rustc -O` that
   loops `Command::new(rgrep).args(...).output()` `N` times. Because the
   helper is cargo-built (not system-signed), samply can attach to it,
   and via process-tree tracking samply also collects samples from the
   spawned `rgrep` children. The helper source and binary live in
   `target/bench_fixtures/` (gitignored).
3. `PROFILE_RUN_ITERS=30 samply record --rate 4000
    --save-only --unstable-presymbolicate ... -- ./profile_run ./rgrep ...`
   gives 1500–2500 samples on the rgrep process per scenario, with
   process-spawn overhead siloed in the parent thread and analytically
   separable from the rgrep search hot path.

**Sampling rate**: 4 kHz (vs the 1 kHz default). The 30-iteration
harness completes in ~30 ms × 30 = ~900 ms of total runtime; 4 kHz buys
us ~3600 samples per profile, enough to resolve individual frames at
the 1–3 % self-time granularity we needed for the comparative
analysis.

---

## 2. Top findings — 3 hot frames, how surprising vs the pre-profile
hypothesis

The Ondata 5 SPEC (in `NEXT_STEPS.md`) listed the pre-profile
hypothesis per scenario:

| Scenario | Pre-profile hypothesis | Top hot frame in profile | Surprise |
|---|---|---|---|
| `regex_simple` | "regex crate glue, per-line alloc" | `memchr_aligned` (32.2 %), then `Utf8Chunks::next` (19.1 %) | regex glue itself is only 4.9 % self |
| `invert_match` | "regex crate negation + count machinery" | `memchr_aligned` (37.8 %), `Utf8Chunks::next` (22.0 %) | regex engine is **0.6 %** — `^$` is a trivially fast DFA |
| `fixed_string_F` | "aho-corasick glue + WalkDir/IO loop overhead" | FS syscalls 37 % combined (`__open` 14.8 %, `fstatfs` 13.3 %, `__getdirentries64` 9.1 %) | aho-corasick is essentially free; the workload is **directory traversal**, not search |

**Top three universal hot frames across all line-mode scenarios**
(`regex_simple`, `invert_match`, `literal_count`):

1. `core::slice::memchr::memchr_aligned` — 32–38 % self. The newline
   scanner inside `BufRead::read_until`. Already optimised
   (usize-aligned), but no SIMD — the platform `memchr` (used by the
   `memchr` crate on macOS arm64 via NEON) would be faster.
2. `<core::str::lossy::Utf8Chunks as Iterator>::next` — 19–22 % self.
   The byte-by-byte scan inside `String::from_utf8_lossy`. The std's
   own `core::str::run_utf8_validation` has a usize-block ASCII
   fast-path that `Utf8Chunks::next` does not.
3. `read` (syscall) — 6–9 % self. One per 8 KB chunk of the input file
   on the `BufReader` path. Replaced by zero syscalls on the
   `Cursor`-over-`Vec<u8>` path (mmap or slurp), which is the
   PERF-FIX-O5-2 lever.

---

## 3. Surprises — what we expected to be hot and wasn't

- **`regex` engine was much less hot than expected**. The Ondata 4
  PERF_REPORT.md attributed the `regex_simple` slowness to "the regex
  crate's general NFA/DFA machinery"; the profile shows the engine
  itself is 4.9 % self and 16.5 % total. The actual cost is in the
  **per-line glue** (`read_until` + `from_utf8_lossy`) which is shared
  with the literal-pattern scenarios.

- **`Cow::Borrowed` allocation overhead was a non-issue**. On ASCII
  input the lossy validation succeeds and `from_utf8_lossy` returns
  `Cow::Borrowed` with zero allocation. The 22 % cost we saw is **pure
  validation work**, not allocation.

- **`_platform_memmove` showed up at 5–7 % self** — we hadn't expected
  it. This is the per-line copy inside `read_until` from the BufReader's
  internal 8 KB buffer into the caller's `Vec<u8>` line buffer. mmap
  avoids this because `Cursor::fill_buf` returns the whole remaining
  slice as a no-op reference, with no intermediate Vec.

- **`fixed_string_F` is not a search-cost problem**. The PERF_REPORT.md
  hypothesis was "aho-corasick glue + WalkDir overhead" with the
  emphasis implicitly on the matcher. The profile shows the opposite:
  the matcher is ~2 % self, and the bottleneck is `walkdir` traversal
  syscalls. This reframes the rgrep-vs-BSD-grep delta on this scenario
  entirely: we're not slower at *searching*, we're slower at *walking
  the directory tree*. Closing that gap requires replacing `walkdir`
  with a hand-rolled `read_dir` loop or parallelising via `rayon`,
  both explicit out-of-scope for Ondata 5.

- **The control (`literal_count`) is not a "different beast"**. Its
  profile is essentially identical to `regex_simple`'s on the line-mode
  hot path; the +5.5 % delta vs BSD grep (vs +30.8 % for `regex_simple`)
  is **not** about search-engine cost. The cost structure is the same;
  the *absolute* numbers move because the test corpus is different
  (5 short files vs 1 longer file). This was the most
  generalisable insight of the wave.

---

## 4. Fix(es) applied — hypothesis → patch → bench delta

### PERF-FIX-O5-1 — UTF-8 validation fast path

- **Hypothesis**: replace `String::from_utf8_lossy(&buffer)` with
  `std::str::from_utf8(&buffer).map_or_else(lossy_fallback,
  Cow::Borrowed)` to hit the usize-block ASCII fast-path in
  `core::str::run_utf8_validation` instead of the byte-by-byte
  `Utf8Chunks::next`. Expected −5 to −10 % on line-mode scenarios.
- **Patch**: 10 LOC in `src/runner.rs::bufread_search`. Adds
  `use std::borrow::Cow;`, swaps the validation call, retains identical
  observable semantics on every code path (the `Cow` value is consumed
  the same way the previous `Cow` from `from_utf8_lossy` was).
- **Bench delta**: −1.5 % on `regex_simple`, −1.1 % on `invert_match`,
  flat elsewhere. **Under the 5 % bar in isolation.** Honest verdict:
  the optimisation is correct but its absolute effect on the 92 KB
  criterion corpus is masked by process-spawn overhead (~50 % of the
  3.5 ms total). Retained because (a) unambiguously cheaper on every
  input, (b) costs ~10 LOC, (c) composes positively with FIX-O5-2.

### PERF-FIX-O5-2 — slurp-and-cursor for small regular files

- **Hypothesis**: PERF_REPORT.md §7 showed `--mmap` is ~12 % faster than
  `BufReader` on a 92 KB file with a substantially tighter distribution.
  The mmap win comes from `Cursor::fill_buf` returning the whole remaining
  slice (one fill per file, no per-line `_platform_memmove` from a
  refill buffer, no per-chunk `read` syscall). Replicate the I/O shape
  *without* the mmap syscall by `read_to_end`-ing the file into a
  `Vec<u8>` capped at 16 MB and feeding `Cursor::new(bytes)` to the
  same `bufread_search` function.
- **Patch**: 19 LOC in `src/runner.rs::search_file`. Guarded by
  `metadata.is_file() && metadata.len() > 0 && metadata.len() <
  16 MB`. Falls through to the existing `BufReader` path for larger
  files and for non-regular files (pipes, FIFOs, device nodes).
- **Bench delta**:
  - `literal_match_count`: **−9.5 %** ✅
  - `bufread_default` (`mmap_vs_bufread`): **−13.6 %** ✅
  - `color_never_count_only` (`cow_impact`): **−15.8 %** ✅
  - `mmap_explicit`: −4.7 %
  - Other line-mode scenarios: flat to slightly improved
  - Largest regression: +2.7 % on `fixed_string_F` (well below the 5 %
    cap; this scenario is FS-walking-bound, not I/O-bound, so the
    slurp benefit doesn't apply but the extra allocation slightly costs)

D-Ondata-5.6 criterion "Δ ≥5 % sulla mediana del scenario target" is
met on **three scenarios**. The largest regression is +2.7 %, below the
5 % cap.

**Caveat — the 3 SPEC-defined targets did not individually hit ≥5 %.**
`regex_simple`, `invert_match`, `fixed_string_F` all moved within
criterion's noise envelope. Their bottleneck on the criterion corpus
turned out to be the per-line `memchr_aligned` cost (32–38 % self in
the profile), which can only be materially improved by introducing the
`memchr` crate as a direct runtime dependency — explicitly forbidden by
D-Ondata-5.10. The validation runs against `mmap_vs_bufread/bufread_default`
cover the same hot path and confirm the fix is structurally correct;
the criterion noise floor on the 3 SPEC-defined worst scenarios is
just wide enough to swallow the absolute Δ.

---

## 5. Generalizability — patterns transferable to other ports

### G.1 — Profile-driven > spec-driven for performance work in legacy ports

The original Ondata 4 PERF_REPORT.md hypothesised that `regex_simple` was
slow because of "regex crate glue overhead, per-line allocation". The
profile showed that:
- The regex engine itself is 4.9 % self, not the bottleneck.
- Per-line allocation is effectively zero on ASCII input (Cow fast-path).
- The actual hot path is `read_until` + `memchr_aligned` + `from_utf8_lossy`,
  which is **shared across all line-mode scenarios regardless of
  pattern type**.

**Pattern**: when a port is "20–30 % slower than the C oracle", do not
guess where the time is going. Profile first, even on a noisy
process-spawn benchmark. The pre-profile hypothesis ranking and the
post-profile hot-frame ranking diverged on every single scenario in
this wave. Document the divergence — it is itself a port-methodology
insight.

### G.2 — `Utf8Chunks::next` vs `run_utf8_validation` is a stdlib smell

`std::String::from_utf8_lossy` uses `Utf8Chunks::next` internally, which
is a byte-by-byte scan with no usize-block ASCII fast path. `std::str::from_utf8`
uses `core::str::run_utf8_validation` which **does** have the fast path.
For ASCII-heavy inputs (typical for source code, logs, CSV) the
`from_utf8` path is materially faster.

**Pattern**: in a Rust port of a C library where the C version operates
on `char*` directly, if the Rust version converts byte buffers to
`&str` for matching/parsing, use `std::str::from_utf8` first and
`from_utf8_lossy` only as the fallback. The cost is ~5 LOC and a `Cow`
import; the benefit on ASCII workloads can be 5–15 % on the validation
phase alone.

This generalises to: **rawk** (One True Awk → Rust) was reported in its
own diary as having an "is-utf8" hot spot in the line-reading path;
the same fix should apply there.

### G.3 — "Slurp + Cursor" replicates mmap's shape without the syscall

The mmap result in Ondata 4 (`-12 %` on `bufread_default` vs mmap)
turned out to be replicable on the `BufReader` path by reading the
whole file into a `Vec<u8>` and feeding it to `Cursor`. The mmap
benefit is **not** about the OS page-fault story — it is about
`Cursor::fill_buf` returning the whole remaining slice in one no-op
call. Any I/O strategy that produces a `BufRead` impl with whole-buffer
`fill_buf` will achieve the same win, regardless of whether the bytes
come from mmap or `read_to_end`.

**Pattern**: for small/medium files in a search-style workload, prefer
`read_to_end` + `Cursor` over streaming `BufReader`. Cap the slurp at
a size you can afford to keep resident (we used 16 MB) and fall back
to `BufReader` for larger inputs.

This generalises to: any C-to-Rust port of a line-oriented utility
(grep, awk, sed, head, tail, wc) on files small enough to fit in
memory. The pattern is materially cheaper than the streaming default,
without paying the mmap-syscall + unsafe-block cost.

### G.4 — Profile attribution differs by corpus size

The samply profile ran on a 5.4 MB amplified fixture; criterion runs
on the original 92 KB fixture. On the amplified fixture `Utf8Chunks::next`
showed at 19–22 %, suggesting a ~5–10 % win was achievable on
criterion. The criterion delta turned out to be 1–2 %. The reason:
on the small corpus, process-spawn overhead is a much larger fraction
of total time, so eliminating an X % cost saves X % × (1 − spawn_frac)
on the bench.

**Pattern**: amplify-then-profile on a synthetic corpus, but **validate
on the bench corpus**, and adjust expectations by the spawn-overhead
fraction. Document the divergence honestly — the fix can still be
correct and worth keeping even if it does not move the bench needle.

### G.5 — Spec-defined "worst targets" can be the wrong levers

Ondata 5's SPEC named `regex_simple`, `invert_match`, `fixed_string_F`
as profile targets because they had the largest delta vs BSD grep in
Ondata 4. But the profile showed those scenarios are bound by costs
that the SPEC's own D-5.10 forbade us from touching (`memchr_aligned`
needs the `memchr` crate; `walkdir` traversal needs `rayon` or a
custom walker). The fixes that actually landed (FIX-O5-2 slurp) won on
*different* scenarios that the SPEC didn't list as targets.

**Pattern**: a "worst targets" list at SPEC time is a hypothesis about
where the leverage is. The profile is the source of truth. When they
diverge, ship the fix where the leverage actually is, document the
SPEC-vs-profile divergence, and add a SPEC-Q to the next wave to
re-scope.

This was the most generalisable methodology insight of Ondata 5. It
mirrors the legacy-port skill's "Phase 4d (post-port performance work)"
guidance: never let the SPEC's pre-profile guesses constrain the
post-profile fixes.
