# EXPERIMENT_PLAN — testag-grep

> Piano storico iniziale (maggio 2026). Per stato corrente post-Ondata 7 e
> verifica GNU del 2026-09-30 vedere README.md e SESSION_HANDOFF.md.

> Mappatura dell'esperimento `grep → Rust` rispetto alle 4 fasi della skill
> `legacy-port` e al workflow multi-agent (Architect + Implementer + Decider)
> validato su `rawk` (One True Awk → Rust).
>
> _Generato con assistenza AI — Claude Opus 4.7 (1M context), 2026-05-03._

## 0. Snapshot di partenza

L'esperimento **non parte da zero**. Stato baseline al 2026-05-03:

| Componente | Stato | LOC |
|---|---|---|
| `gnu-grep/` (upstream snapshot) | C, 5 file principali, 149 testcase | ~4.5K |
| `minigrep.c` (reference minimale) | C, single-file | ~30 |
| `rgrep/src/cli.rs` (Gemini, pre-experiment) | clap-derive, copre ~50 flag | 207 |
| `rgrep/src/runner.rs` (Gemini, pre-experiment) | I/O orchestration | 316 |
| `rgrep/src/matcher.rs` (Gemini, pre-experiment) | match logic | 203 |
| `rgrep/main.rs` + `lib.rs` | wiring | 15 |
| **Totale Rust** | | **~741** |

Il lavoro pre-experiment di Gemini è **input**, non output: i prossimi step
auditano, formalizzano e completano questa baseline applicando il metodo della
skill.

## 1. Strategia di porting (decision tree skill)

GNU grep è **misto**: ha sia funzionalità con equivalente crate maturo, sia
algoritmi proprietari (DFA Boyer-Moore di kwsearch). Decisione per modulo:

| Modulo C | LOC | Strategia | Rationale |
|---|---|---|---|
| `grep.c` (orchestrazione + CLI) | 3036 | **F — Selective** | Sostituito da `clap` + `walkdir` + `regex` ecosystem |
| `dfasearch.c` | 599 | **A — Adapter** | Sostituito da `regex` crate (DFA-based, RE2 lineage) |
| `kwsearch.c` | 238 | **A — Adapter** | Sostituito da `aho-corasick` per `-F` fixed strings |
| `pcresearch.c` | 421 | **A — Adapter** | Sostituito da `pcre2` crate per `-P` (opzionale) |
| `searchutils.c` | 219 | **F — Selective** | Helper word-boundary, case-fold → in-house Rust |

> **Compressione attesa**: 4513 LOC C → ~1500 LOC Rust (compressione 65-70%
> grazie a `regex` + `clap` + `walkdir` + `globset` che fanno il lavoro pesante).
> Stima coerente con esperimento wget2 (compressione 90% sul CLI).

## 2. Fasi della skill — mappatura

| Fase skill | Stato | Deliverable in questo repo |
|---|---|---|
| **Fase 1 — Comprensione semantica** | retroattivo, prodotto in Step 1 | [diary/01-semantic-model.md](diary/01-semantic-model.md) |
| **Fase 2 — Mappatura idiomatica** | retroattivo, prodotto in Step 2 | [diary/02-mapping-table.md](diary/02-mapping-table.md) |
| **Fase 3 — Traduzione TDD** | iterativa Step 3..15 | [diary/03-translation-log.md](diary/03-translation-log.md) (incrementale) |
| **Fase 4 — Differential testing** | Step 0 (infra) + Step 16 (proptest) | [diary/04a-divergences.md](diary/04a-divergences.md) |
| **Sintesi finale** | Step 17 | [diary/99-conclusions.md](diary/99-conclusions.md) |

## 3. Differential testing — disegno

Pattern bonus skill: **process-spawn**, no FFI.

- **Oracolo**: `/usr/bin/grep` di sistema. Su macOS = BSD grep (rilevante per
  divergenze tipo case-insensitive ASCII vs locale, BRE escape rules,
  `--include`).
- **Harness**: `tests/diff_runner.rs` integration test che:
  1. Per ogni `tests/cases/NNNN_<name>.txt`:
     - parsa header TOML in-line con `args`, `stdin`, `expected_stdout`,
       `expected_exit_code`, `skip_if_bsd` (opzionale)
     - esegue `rgrep <args>` + `grep <args>` su stdin canned
     - confronta byte-per-byte
  2. Outcome enum: `Match`, `Differ(rgrep, grep)`, `BothFail`, `OnlyRgrepOk`,
     `OnlyGrepOk`. Invariante (analoga skill): mai `OnlyRgrepOk` su input
     standard.
- **Property-based** (Step 16, `proptest`):
  - Genera pattern regex valido + corpus di testo random
  - Property: `rgrep <pattern>` output == `grep <pattern>` output, modulo
    feature documentate come divergenza in 02-mapping-table.md
  - Range progressivo come da skill warning (start strict → expand → catch
    bugs).

## 4. Backlog ordinato

Mantenuto in [NEXT_STEPS.md](NEXT_STEPS.md). Riepilogo:

```
Step 0  🚧 Test harness + diff baseline + audit infra
Step 1  🔒 Phase 1 retroactive: 01-semantic-model.md
Step 2  🔒 Phase 2 retroactive: 02-mapping-table.md (D-decisioni globali)
Step 3  🔒 Regex engine flavor: -E (default rgrep), -G (BRE), -F (aho-corasick)
Step 4  🔒 Pattern sources: -e multipli, -f file, OR logic
Step 5  🔒 Output formatting: -n -b -H -h -c -l -L
Step 6  🔒 Output formatting: -o, --color (env GREP_COLORS)
Step 7  🔒 Recursion: -r/-R, -d, -D
Step 8  🔒 Filtering: --include, --exclude, --exclude-dir, --exclude-from
Step 9  🔒 Context: -A -B -C, --group-separator, --no-group-separator
Step 10 🔒 Limits: -m, -q, -s
Step 11 🔒 Binary handling: -a, -U, --binary-files
Step 12 🔒 NUL handling: -z (null-data), -Z (null filename suffix)
Step 13 🔒 Misc: -T, --label, --line-buffered
Step 14 🔒 mmap path: --mmap (memmap2)
Step 15 🔒 -P perl-regexp via pcre2 crate (opzionale)
Step 16 🔒 Differential proptest cross-implementation
Step 17 🔒 99-conclusions.md + metrics
```

## 5. Stima effort

Per ~4.5K LOC C → ~1.5K Rust target, con baseline pre-existing:

- Step 0-2 (setup + retroactive docs): **3-4h** Architect + 2-3h Implementer
- Step 3-15 (gap-closing): **15-25h** Implementer (1-2h per step)
- Step 16 (proptest): **3-5h**
- Step 17 (conclusions): **1-2h** Architect

**Totale: ~30-40h interazione**, distribuiti su ~17 commit + eventuali -bis.

## 6. Cosa ci aspettiamo di scoprire (predictions)

Documentate qui per check post-experiment:

1. **`regex` crate vs DFA C**: parità totale su input ASCII; divergenze su
   Unicode word-boundary (regex crate ha `\b{wb}` opzionale, GNU grep usa
   POSIX `[[:<:]]`).
2. **BSD grep vs rgrep**: `--include` di BSD non supporta multi-pattern,
   rgrep deve allinearsi a GNU. Skip BSD per quei testcase.
3. **`-P` pcre2**: 100% delegato al crate `pcre2`, nessun lavoro algoritmico.
4. **Color**: `GREP_COLORS` env var parsing è il pezzo più "manuale" rimasto.
5. **Performance**: `regex` crate dovrebbe essere within 2× di GNU grep su
   pattern semplici, e **più veloce** su pattern alternation grazie a literal
   prefiltering.
6. **Bug latenti scoperti**: aspettatamente 1-3 (rawk ne ha trovato 1 con
   proptest a range esteso).

## 7. Quando l'esperimento è "chiuso"

Criteri di completamento (skill: "se la suite di diary è incompleta,
l'esperimento non è chiuso"):

- ✅ Tutti i 17 step in `NEXT_STEPS.md` ✅ APPROVED nel `AUDIT_LOG.md`
- ✅ `diary/01..04` + `99-conclusions.md` presenti e coerenti
- ✅ `cargo test` verde con N differential testcase + property-based suite
- ✅ Almeno 50 testcase XML/TOML con tasso di parity ≥85% vs `grep` di sistema
- ✅ Almeno una divergenza nuova catturata, classificata e spiegata in `04a-divergences.md`
