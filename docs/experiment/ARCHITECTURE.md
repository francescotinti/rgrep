# ARCHITECTURE — il progetto e la sua evoluzione

> Come è fatto `rgrep`, perché è fatto così, e come ci è arrivato.
> Aggiornato: 2026-09-30 (`92285c7`). Backlog: [TODO.md](TODO.md).
> Cronologia consegne: [DONE.md](DONE.md). Fonti integrali: il modello
> semantico del C ([history/diary/01](history/diary/01-semantic-model.md)),
> la tabella di mappatura con le D-decisioni
> ([history/diary/02](history/diary/02-mapping-table.md)) e il log di
> traduzione ([history/diary/03](history/diary/03-translation-log.md)).

## 1. Origine: GNU grep in C

Sorgente di riferimento: snapshot upstream in `gnu-grep/` (~4 500 LOC C,
149 test shell). Cinque moduli, ciascuno con una strategia di porting
decisa prima di scrivere codice:

| Modulo C | LOC | Ruolo | Strategia | Destinazione Rust |
|---|---|---|---|---|
| `grep.c` | 3 036 | CLI (`getopt_long`), buffer I/O page-aligned (`fillbuf`), loop di ricerca (`grepbuf`), output (`print_line_head`, `prtext`), traversal (`grepfile` su `fts`) | selective port | `cli.rs` + `runner.rs` + `output.rs` |
| `dfasearch.c` | 599 | DFA veloce con fallback a regex POSIX per backreference | adapter | crate `regex` (+ `fancy-regex` per backref) |
| `kwsearch.c` | 238 | Aho-Corasick / Boyer-Moore per `-F` | adapter | crate `aho-corasick` |
| `pcresearch.c` | 421 | PCRE2 JIT per `-P` | adapter, feature-gated | crate `pcre2` |
| `searchutils.c` | 219 | word boundary multibyte, case fold | selective port | delegato a `regex` (`\b`, `(?i)`) |

Modello semantico del C che il port doveva preservare (diario 01):

- **Dispatch dei matcher a runtime** in base a `-E/-F/-G/-P`, con
  downgrade dinamico (`-F` con `-w` o case-insensitive multibyte ricade su
  regex).
- **Buffer, non righe**: GNU grep non carica mai il file intero; legge
  blocchi da 96 KiB allineati alle pagine, cerca il delimitatore `eolbyte`
  (`\n` o `\0` con `-z`) e tiene i residui. Nessun `mmap` (rimosso
  storicamente).
- **Rilevazione binaria** con `buf_has_encoding_errors` sul primo blocco.
- **Edge case** trattati esplicitamente: input = output (stesso inode),
  loop di symlink con `-R`, device e FIFO, righe più grandi della RAM,
  UTF-8 non valido in PCRE.
- **Exit code**: 0 match, 1 nessun match, 2 errore (`-q` può tacere
  l'errore).

## 2. Struttura del port Rust

Repo `rgrep/` (git annidato nel workspace, branch `main`, pubblicato su
GitHub `francescotinti/rgrep`). Edizione Rust 2024, toolchain verificata
1.98.1.

```
rgrep/
├── Cargo.toml                 feature opzionale perl-regexp = ["dep:pcre2"]
├── src/
│   ├── main.rs        28 LOC  parse → run → exit code
│   ├── lib.rs         14 LOC  esporta i moduli
│   ├── cli.rs        370 LOC  clap-derive: Config → 5 sub-struct
│   ├── matcher.rs    603 LOC  trait MatchEngine, enum Engine, Matcher
│   ├── runner.rs   1 201 LOC  resolve_files, walk_recursive, search_file,
│   │                          bufread_search, read_delimited, process_line
│   ├── output.rs     117 LOC  GrepColors (GREP_COLORS), highlight() → Cow
│   └── error.rs       46 LOC  thiserror → exit 2
├── tests/
│   ├── diff_runner.rs        256 LOC  harness differenziale process-spawn
│   ├── proptest_differential.rs 100  3 strategie × 1024 casi
│   ├── buffer_boundaries.rs   62      confini buffer senza oracolo
│   ├── testsuite.toml                 manifest autoritativo (129 casi)
│   └── cases/NNNN_<name>.toml
├── benches/search.rs         251 LOC  Criterion, 10 gruppi, process-spawn
├── tools/                    convert_gnu_tests.py, curate.py,
│                             compare_binaries.py, summarize_bench.py
├── reports/                  JSON/TXT versionati di bench, profili, verifica, CI
├── .github/workflows/verify.yml   Linux+macOS × default+PCRE2
└── docs/experiment/          questa documentazione
```

Dipendenze runtime e la decisione che le giustifica:

| Crate | Sostituisce | Decisione |
|---|---|---|
| `clap` (derive) | `getopt_long` | D-G10: `disable_help_flag`, perché `-h` in grep è `--no-filename` |
| `regex` | DFA custom + regex POSIX | D-G1, D-G11: nessun fallback DFA→NFA replicato; D-G13 case-fold via `(?i)` |
| `fancy-regex` | fallback POSIX per backreference | D-M6 (Step 19): usato solo se `regex` rifiuta il pattern |
| `aho-corasick` | `kwsearch.c` | D-M7/M8: multi-pattern `-F`; D-M9 fallback a regex con `-w` |
| `pcre2` (opzionale) | `pcresearch.c` | D-M10: senza feature `-P` esce 2 con lo stesso messaggio di GNU |
| `walkdir` | `fts_read` | D-G2: scelto al posto di `ignore` per non onorare `.gitignore` (parità `--exclude`) |
| `globset` | glob C | D-G3: `--include/--exclude/--exclude-dir` |
| `memmap2` | — (GNU non usa mmap) | Step 14: `--mmap` opt-in, unico `unsafe` del progetto |
| `memchr` | `memchr_aligned` di std | Ondata 7: scansione delimitatori SIMD (NEON/SSE2) |
| `thiserror` | exit code ad hoc | D-G6: un solo tipo errore mappato su 2 in `main` |

## 3. Flusso di esecuzione

```
argv ──clap──▶ Config {pattern_opts, output_opts, context_opts, filter_opts, binary_opts}
   │
   ├─▶ Matcher::new(patterns, opts)
   │     • validazione per-pattern PRIMA dell'unione con `|` (D-NEW-5)
   │     • -G: translator BRE→ERE (preserva \1..\9)
   │     • -w: wrapping \b(?:…)\b ; -x: ^(?:…)$ ; -i: (?i)
   │     • scelta Engine: AhoCorasick (-F puro) | Regex | Fancy (backref) | Pcre2 (-P)
   │
   └─▶ runner::run
         ├─ resolve_files / walk_recursive   (walkdir + globset, -d/-D, symlink -R)
         ├─ search_file
         │    ├─ file regolare, 0 < len < 16 MB  → read_to_end + Cursor     (PERF-FIX-O5-2)
         │    ├─ --mmap                          → memmap2 + Cursor          (Step 14)
         │    └─ altrimenti                      → BufReader
         ├─ bufread_search(reader)
         │    ├─ rilevazione binaria sul primo chunk (NUL), -a/-U/--binary-files
         │    ├─ read_delimited(reader, delim, &mut buf)  helper memchr condiviso (O7)
         │    ├─ fast path pure_count_quiet: Matcher::is_match_bytes(&[u8])  (O6)
         │    └─ path generale: from_utf8 → fallback lossy (O5-1) → process_line
         └─ process_line
              ├─ ring buffer VecDeque per -B, contatore per -A, separatori
              ├─ prefissi file:riga:offset (-H/-h/-n/-b/-T/-Z/--label)
              ├─ -o via find_match_offsets ; highlight() Cow<str> se --color
              └─ -m / -q early-exit, -l/-L, -c
   ▼
RunResult {MatchFound=0, NoMatch=1, Error=2}
```

Invarianti mantenute dal C:

- Il delimitatore è preservato nel buffer di riga (semantica `read_until`),
  `-z` cambia solo il byte.
- L'output passa da `write_all` di byte, mai da `println!`: i NUL interni
  restano intatti.
- Il path `--mmap`, quello slurp e quello `BufReader` producono output
  byte-identico perché convergono in `bufread_search` su un `BufRead`.
- Il fallback su `BufReader` è automatico per stdin, device, file vuoti o
  senza metadati.

## 4. Architettura del testing

Il differential testing è il contratto dell'esperimento: **l'oracolo è il
binario `grep` di sistema**, invocato per processo (niente FFI).

- **Harness** (`tests/diff_runner.rs`): per ogni caso TOML esegue `rgrep`
  e l'oracolo con gli stessi `args` e `stdin`, confronta stdout **byte per
  byte** ed exit code, anche quando l'exit non è zero. Fixture temporanee
  con placeholder `{FIXTURES}`, `env`, `sort_output` per ordine di
  `readdir` non deterministico.
- **Oracolo esplicito**: `RGREP_ORACLE=/path/grep`; l'identità GNU/BSD e la
  versione sono rilevate una volta per eseguibile di test; implementazioni
  sconosciute vengono rifiutate. `LC_ALL=C` di default.
- **Schema del manifest** (`tests/testsuite.toml`, campi ignoti rifiutati):
  `name`, `args`, `stdin`, `expected_stdout`, `expected_exit_code`;
  opzionali `skip_if_bsd`+`skip_reason`, `requires_feature`,
  `forbids_feature`, `rgrep_only`, `oracle_args` (per `--mmap`, che GNU
  moderno non accetta), `fixture_files`, `env`, `sort_output`,
  `gnu_difference`+`gnu_stdout`+`gnu_stderr_contains`. Il campo storico
  `expected_to_fail` è stato eliminato.
- **Conteggi separati** nel riepilogo: parità, solo-rgrep, skip, divergenze
  GNU note, falliti. Sommano al totale. Una divergenza nota non è mai
  contata come parità; se si "risolve" da sola, il test fallisce (va
  riclassificata). `RGREP_STRICT_GNU=1` la rifiuta.
- **Proptest** (`tests/proptest_differential.rs`): pattern letterali,
  regex ancorate, classi di caratteri; 1024 casi ciascuna, eseguiti solo
  con oracolo GNU e skip esplicito su BSD.
- **Test senza oracolo**: `buffer_boundaries.rs` (record lunghi in
  streaming, CRLF, NUL, EOF senza terminatore, offset) e unit test inline
  (34 default, 36 con PCRE2), inclusi i contratti del reader `read_delimited`.
- **CI** (`verify.yml`): matrice `ubuntu-latest`/`macos-latest` ×
  `default`/`perl-regexp`, toolchain 1.98.1, `fmt` + `build --locked` +
  `clippy -D warnings` + `test`, log per caso caricati come artefatti.

Benchmark (`benches/search.rs`): Criterion in modalità process-spawn
(equità con l'oracolo, ma 1–3 ms di spawn su scenari da 4–7 ms), 10 gruppi
fra cui `line_scan` amplificato (grep.c ×60) e `fixed_boolean`; i bench
falliscono su exit ≠ 0/1; baseline nominate e riassunte in JSON da
`tools/summarize_bench.py`; conferma causale con coppie alternate pre/post
in `tools/compare_binaries.py`.

## 5. Divergenze documentate

Rispetto a GNU grep canonico:

| ID | Tipo | Stato |
|---|---|---|
| D-NEW-1 | Default ERE (`-E`) invece di BRE; `-G` attiva il translator | scelta di design, aperta e accettata |
| D-NEW-2 | Exit code 0/1/2 assenti nella baseline Gemini | bug latente, chiuso Step 10 |
| D-NEW-3 | `--color` default `never` invece di `auto` | scelta di design, aperta e accettata |
| D-NEW-4 | Nessuna divergenza dai 3072 proptest | informativa |
| D-NEW-5 | `-e "[" -e "]"` unite prima della validazione formavano `[|]`, regex valida | bug reale, chiuso Step 19 |
| GNU 0054/0067 | Messaggio "binary file matches" su stdout (rgrep, GNU storico, BSD) contro stderr (GNU ≥3.5) | debito aperto, [TODO P2](TODO.md) |
| GNU 0064/0065 | `-T`: tab anteposto anche senza prefisso; padding diverso | debito aperto, [TODO P3](TODO.md) |
| — | `-P` senza feature: exit 2 con messaggio GNU | intenzionale (D-M10) |

## 6. Scope-out

Non portati per decisione esplicita (diario 02 §5): `GREP_OPTIONS`
(deprecata), `--rpm-*` e flag vendor-specifici, il layer di portabilità
gnulib, il trattamento dei locale multibyte legacy (`mb_clen`, Shift-JIS:
rgrep tratta solo unibyte e UTF-8), la catena di fallback interna
DFA → regex POSIX (delegata a `regex`/`fancy-regex`). Persa anche la
micro-ottimizzazione di `dfa_comp` con cache dei literal obbligatori.

## 7. Evoluzione

### 7.1 Da 741 a 2 379 righe

| Tappa | Cosa cambia nell'architettura |
|---|---|
| Baseline Gemini | Tre file monolitici, nessun test, exit code sbagliati |
| Step 0 | Harness differenziale process-spawn: da qui ogni feature nasce da un testcase rosso |
| Step 1–2 | Modello semantico e mappatura scritti **dopo** la baseline ma **prima** delle feature: le D-decisioni diventano il contratto |
| Step 3 | Nasce `Engine` (Regex/AhoCorasick); translator BRE→ERE in-house |
| Step 9 | Context lines con `VecDeque` al posto delle pending lines di `prtext` |
| Step 10 | `RunResult` e mapping esplicito degli exit code |
| Step 12 | `read_until` con delimitatore programmabile; output via `write_all` |
| Step 14 | `search_buffer` estratto e condiviso da mmap e BufReader |
| Step 15 | Variante `Pcre2` dietro `cfg(feature)` |
| Step 19 | Validazione per-pattern; variante `Fancy` per backreference |
| Ondata 2 | `error.rs` con `thiserror`; funzioni lunghe decomposte |
| Ondata 3 | `Config` in 5 sub-struct; trait `MatchEngine` con `engine_is_match` / `engine_find_offsets` / `engine_highlight`; `highlight()` restituisce `Cow` |
| Ondata 5 | `search_file` sceglie slurp+`Cursor` per file <16 MB; validazione UTF-8 con `from_utf8` prima del lossy |
| Ondata 6 | `engine_is_match_bytes` nel trait con override per engine; loop `pure_count_quiet` senza `&str` |
| Ondata 7 | `read_delimited` privato con `fill_buf` + `memchr`, usato da entrambi i loop |
| Verifica GNU | Harness con oracolo esplicito, schema rigido, conteggi separati, strict mode, CI |

### 7.2 Cosa hanno insegnato i profili

Sequenza delle ipotesi e di ciò che le misure hanno mostrato:

1. **Ondata 4** (solo misura, BSD grep come confronto): rgrep +5% sul
   count letterale, +27–31% su regex e `-rF`. Ipotesi: "glue del regex
   engine". `--mmap` batte `BufReader` del 12% anche su 92 KB,
   contro la previsione. Il `Cow` di Ondata 3: −1,8%, nel rumore.
2. **Ondata 5** (samply, fixture 5,4 MB): il regex engine era il 5% del
   tempo; il costo era `memchr_aligned` (32–38%) dentro `read_until` e
   `Utf8Chunks::next` (19–22%) dentro `from_utf8_lossy`. `fixed_string_F`
   era per il 37% syscall di `walkdir`. Il vantaggio di mmap era in realtà
   il `fill_buf` a slice intera di `Cursor`: replicato senza mmap con lo
   slurp. I tre "worst target" della SPEC non si sono mossi; hanno vinto
   scenari non elencati.
3. **Ondata 6**: byte-path per count/quiet, guadagno marginale perché O5
   aveva già preso il grosso; l'API resta come deliverable architetturale.
4. **Ondata 6-bis**: scoperto che `regex_simple` misurava un errore;
   ripgrep ora installato e più lento di rgrep in process-spawn (startup).
5. **Ondata 7**: `memchr_aligned` 46,3% → 4,8% self, ma l'helper inlinato
   assorbe 29,3% e `memmove` sale a 19,9%: il cambio di simboli non è uno
   speedup. La prova causale sono le 31 coppie alternate: −9/−12% sui
   count amplificati, workload piccoli piatti, `fixed_quiet` +3,5%.

Residuo noto: copie (`memmove`) e validazione UTF-8 restano materiali;
il walker è syscall-bound. Sono i candidati di [TODO P7](TODO.md).

## 8. Architettura del processo

Il progetto è anche un esperimento di metodo (skill `legacy-port`,
gemello di `rawk`). Tre ruoli: **Architect/Auditor** scrive SPEC con
D-decisioni e audita con checklist 12/12 senza scrivere codice;
**Implementer** legge un solo step 🚧, aggiunge prima i testcase rossi,
implementa e fa un solo commit; **Decider** umano promuove gli step e
decide i trade-off. Step 0–19: Implementer Gemini senza terminale, con il
Decider che incolla blocchi shell atomici (`&&` chain, `pipefail`).
Ondate: Implementer Claude con terminale. Ondate 6-bis/7 e verifica GNU:
Codex, con audit dichiarato come autoverifica.

Artefatti del processo: le SPEC integrali con le D-decisioni in
[history/NEXT_STEPS.md](history/NEXT_STEPS.md), i verdetti in
[history/AUDIT_LOG.md](history/AUDIT_LOG.md), gli script di commit del
relay-shell in [history/commit_scripts/](history/commit_scripts/), il
piano iniziale con le predizioni in
[history/EXPERIMENT_PLAN.md](history/EXPERIMENT_PLAN.md). Le regole
operative vigenti sono nel `CLAUDE.md` del workspace outer.
