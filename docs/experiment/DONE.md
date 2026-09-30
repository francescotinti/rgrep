# DONE — cosa è stato fatto

> Registro di ciò che l'esperimento `testag-grep` ha consegnato e verificato.
> Aggiornato: 2026-09-30. Da fare: [TODO.md](TODO.md). Architettura:
> [ARCHITECTURE.md](ARCHITECTURE.md). Testi integrali di SPEC e audit:
> [history/](history/).

## Stato verificato al 2026-09-30

| Metrica | Valore |
|---|---|
| Rust `src/*.rs` (con test inline) | 2 379 righe (era 741 nella baseline Gemini pre-esperimento) |
| Test Rust: default / PCRE2 | 42 / 44 |
| Casi nel manifest differenziale | 129 |
| GNU 3.12 default: parità / solo rgrep / skip / divergenze note | 121 / 1 / 3 / 4 |
| GNU 3.12 PCRE2: parità / solo rgrep / skip / divergenze note | 124 / 0 / 1 / 4 |
| BSD 2.6.0 default: parità / solo rgrep / skip | 112 / 1 / 16 |
| BSD 2.6.0 PCRE2: parità / solo rgrep / skip | 112 / 0 / 17 |
| Confronti generativi (proptest) per build GNU | 3 072 (6 144 nella sessione) |
| Gruppi benchmark Criterion | 10 |
| Ondata 7, count amplificati (coppie alternate) | −9,15% / −11,98% / −10,33% |
| CI GitHub (Linux + macOS × default + PCRE2) | verde 4/4, run `36766955016` su `92285c7` |
| Step chiusi | 17 principali + 8 `-bis` + 2 riapertura = 27, tutti ✅ APPROVED |
| Ondate chiuse | 1, 2, 3, 4, 5, 6, 6-bis, 7 + verifica GNU, tutte ✅ |

Dati grezzi: `reports/verification-20260930.json`,
`reports/o6bis-20260930.json`, `reports/o7-paired-20260930.json`,
`reports/ci-first-run-20260930.json`.

## Cronologia

### Fase 0 — Baseline pre-esperimento (fino al 2026-05-03)

Gemini Antigravity aveva già prodotto `cli.rs` (207 LOC), `runner.rs` (316)
e `matcher.rs` (203): circa 741 righe, senza test differenziali. Questo
codice è stato trattato come **input** da auditare, non come output.

### Fase 1 — Porting funzionale, Step 0–17 (2026-05-03, ~20 h)

Implementer Gemini via relay-shell, Architect/Auditor Claude Opus 4.7.
Un commit per step; sei step hanno richiesto uno step `-bis` di recupero.

| Step | Commit rgrep | Contenuto | Esito audit |
|---|---|---|---|
| 0 | `04f94dc` | Harness differenziale `tests/diff_runner.rs`, 5 casi TOML | 🟡 → 0-bis `bbe49b7` (gitignore, AppleDouble, DESIGN-Q formale) |
| 1 | outer `28743a8` | `diary/01-semantic-model.md` (7 sezioni, 28 citazioni `*.c:line`) | ✅ |
| 2 | outer `27af1f4` | `diary/02-mapping-table.md`: 13 D-G, 15 D-M, tabella costrutti | ✅ |
| 3 | `1094344` | Engine `-E`/`-G`/`-F`, translator BRE→ERE, aho-corasick | 🟡 → 3-bis `fc044cd` (scratch file, D-NEW-2 documentata) |
| 4 | `2d78e76` | `-e` multipli, `-f FILE`, fixture files nel harness | ✅ |
| 5 | `6a3d6ab` | `-n -b -H -h -c -l -L` | ✅ |
| 6 | `4ad2fda`+`980a024` | `-o`, `--color`, parsing `GREP_COLORS` | 🟡 → 6-bis `e1a1d89`+`dce7cb1` (import inutilizzato, flag `skip_if_bsd`) |
| 7 | `b52d887` | `-r -R -d -D` via walkdir | 🟡 BUILD RED → 7-bis `651f415` (brace mancante, `sort_output`, exit 2) |
| 8 | `053a684` | `--include/--exclude/--exclude-dir/--exclude-from` via globset | ✅ |
| 9 | `b994875` | `-A -B -C`, `--group-separator`, ring buffer `VecDeque` | ✅ |
| 10 | `9bb0412` | `-m -q -s`, exit code 0/1/2 canonici (`RunResult`), chiude D-NEW-2 | ✅ |
| 11 | `bcbe045` | `-a -U --binary-files`, euristica NUL | 🟡 BUILD RED → 11-bis `5b3649b` |
| 12 | `85d6a9c` | `-z -Z`, `read_until` con delimitatore programmabile | 🟡 → 12-bis `2f158f2` (testcase non committati) |
| 13 | `3aa2e9a` | `-T`, `--label`, `--line-buffered` | ✅ |
| 14 | `b7dc89c` | `--mmap` via memmap2, unico blocco `unsafe`, `search_buffer` condiviso | 🟡 → 14-bis `eb7db7e` |
| 15 | `43a3f5e` | `-P` via pcre2, feature `perl-regexp` | 🟡 → 15-bis outer `2d22d15` |
| 16 | `6ccd72a` | Proptest differenziale: 3 strategie × 1024 casi | ✅ (D-NEW-4: nessuna divergenza) |
| 17 | outer `28c9b82` | `99-conclusions.md` + `metrics.md` | ✅ 🏁 esperimento chiuso (74 casi, 32 unit, 100% verdi) |

### Fase 2 — Riapertura, Step 18–19 (2026-05-04)

| Step | Commit rgrep | Contenuto | Esito |
|---|---|---|---|
| 18 | `78aac88` | `tools/convert_gnu_tests.py`: importa `gnu-grep/tests/*.sh` in TOML; scopre D-NEW-5 | ✅ |
| 19 | `b9e797e` | Fix 12 bug noti: validazione per-pattern dei `-e` (D-NEW-5), backreference `\1..\9` via fallback `fancy-regex`, +3 unit test | ✅ (36 test) |

### Fase 3 — Ondate di hardening (2026-05-18)

Implementer Claude Opus 4.7 single-shot (senza relay), Auditor Claude.

| Ondata | Commit | Contenuto | Esito |
|---|---|---|---|
| 1 | `106a138` | `clippy --fix` + `cargo fmt`; 28 lint actionable → 2 | ✅ |
| 2 | `25e6817` | Refactor strutturale: tipo errore `thiserror`, decomposizione funzioni, naming; pedantic ~18 residui | ✅ |
| 3 | `87c652f` | Architettura: `Config` diviso in 5 sub-struct, trait `MatchEngine` + enum `Engine`, `highlight()` → `Cow`; pedantic 0 | ✅ |
| 4 | `a1243e2` | Suite Criterion (7 gruppi, process-spawn), `PERF_REPORT.md`, `diary/04b`. Misura-only. mmap ~12% più veloce su 92 KB; `Cow` senza effetto misurabile | ✅ |
| 5 | `6ab7be1` | Profiling samply 4 kHz su fixture 5,4 MB; PERF-FIX-O5-1 (`from_utf8` prima di `from_utf8_lossy`) e O5-2 (slurp ≤16 MB + `Cursor`); −9,5/−13,6/−15,8% su tre scenari | ✅ |
| 6 | `759524b` | `MatchEngine::engine_is_match_bytes` con override per i 4 engine; fast path `pure_count_quiet` senza validazione UTF-8; guadagno marginale (−1,2/−2,4%) | ✅ |

### Fase 4 — Correzioni metodologiche, Ondata 7 e verifica GNU (2026-09-30)

Esecutore Codex; audit come autoverifica dichiarata.

| Ondata | Commit | Contenuto | Esito |
|---|---|---|---|
| 6-bis | `723aab3` | Il bench `regex_simple` passava una regex invalida e misurava exit 2: corretto; i bench ora falliscono su exit ≠ 0/1; nuovi gruppi `fixed_boolean` (`-cF`/`-qF`) e `line_scan` (grep.c ×60); baseline nominata `o6bis-20260930` con JSON versionato; colonna ripgrep finalmente popolata | ✅ |
| 7 | `3de67b0` | `memchr = "2"` diretto; helper privato `read_delimited` condiviso dai due loop di `bufread_search`; 5 test nuovi (contratto reader + CLI); 31 coppie alternate pre/post con bootstrap: −9,15/−11,98/−10,33% sui count amplificati, `fixed_quiet` +3,52% [1,41; 5,22] dichiarato | ✅ |
| Verifica GNU | `3e9d38f` | Oracolo esplicito `RGREP_ORACLE` con identità GNU/BSD; conteggi parità/skip/divergenze separati; confronto byte anche con exit ≠ 0; `oracle_args` per `--mmap`; `rgrep_only`/`forbids_feature`; `gnu_difference` con output GNU registrato e `RGREP_STRICT_GNU`; rimosso `expected_to_fail`; test confini buffer; workflow CI | ✅ locale |

### Fase 5 — Pubblicazione (2026-09-30, sera)

| Commit | Contenuto |
|---|---|
| `ab26dc4` | `fix(ci)`: Bash 3 su macOS rifiutava un array vuoto sotto `set -u`; parametri posizionali |
| `92285c7` | Snapshot della documentazione outer in `docs/experiment/` (poi riorganizzato nei file presenti) |

- Push su `https://github.com/francescotinti/rgrep`.
- Prima run CI `36766671569` (su `3e9d38f`): Linux default e PCRE2 verdi,
  macOS PCRE2 verde, macOS default fallito prima della build (Bash 3).
- Seconda run CI `36766955016` (su `92285c7`): **4/4 verdi**. È la prima
  esecuzione della suite con oracolo GNU su Linux.

## Verifica GNU: cosa dicono davvero i numeri

- Le 129 righe del manifest si sommano esattamente in ogni configurazione:
  parità + solo-rgrep + skip + divergenze note + falliti = 129.
- Gli 8 fallimenti del vecchio harness contro GNU 3.12 sono stati classificati:
  3 casi `--mmap` (opzione rifiutata da GNU moderno → `oracle_args`),
  1 caso `-P` senza feature (→ `rgrep_only` + `forbids_feature`),
  4 divergenze semantiche reali (→ `gnu_difference`, restano debito).
- Su BSD i tre proptest saltano esplicitamente: un run BSD verde **non**
  prova parità generativa. Prima del 2026-09-30 questo era invisibile.
- `RGREP_STRICT_GNU=1` fallisce esattamente sui quattro casi noti.

## Correzioni metodologiche riconosciute

Registrate perché cambiano il peso delle evidenze storiche:

1. `regex_simple` (Ondate 4–6) misurava una regex invalida con exit 2. I
   tempi storici di quello scenario non sono confrontabili.
2. Le percentuali di Ondata 6 non condividono sempre la stessa baseline
   (Criterion sovrascrive `base/`): i delta storici sono qualificati, non
   riutilizzabili come gate.
3. La deriva fra run sequenziali era enorme anche per BSD invariato
   (−39,5% sul literal count): solo le coppie alternate pre/post hanno
   isolato un beneficio credibile per Ondata 7.
4. "Test verdi" e "audit APPROVED" non bastavano a dimostrare copertura:
   proptest saltati su BSD, xfail generico che nascondeva regressioni,
   exit non-zero che bypassava il confronto stdout.

## Lezioni apprese

Dal diario di conclusioni (Step 17), dall'appendice del 2026-09-30 e dai
diari performance (Ondate 4–5):

- **L'AI traduce bene, gestisce male il processo.** Il codice core
  (matcher, walker, mmap isolato, clap) è stato corretto al primo tentativo;
  i fallimenti sistematici sono stati commit outer dimenticati (Step 12, 14,
  15), build rossa committata (Step 7, 11), titoli `hotfix` impropri. Il
  rimedio è stato il pattern relay-shell a blocco unico con `&&` chain e
  `pipefail`, poi due blocchi separati (rgrep / outer).
- **L'oracolo originale in brute-force batte il TDD progettato.** Lo Step 18
  (import automatico della test-suite GNU) ha fatto emergere D-NEW-5, che 74
  casi disegnati a mano non avevano trovato.
- **Misurare prima di ottimizzare.** Il `Cow` di Ondata 3 era zero-impact;
  il regex engine era il 5% del tempo, non il collo di bottiglia; il costo
  vero era `read_until` + validazione UTF-8; `fixed_string_F` era
  syscall-bound sul walker, non sulla ricerca.
- **I "worst target" scritti nella SPEC possono essere le leve sbagliate.**
  I fix di Ondata 5 hanno vinto su scenari che la SPEC non elencava.
- **Separare ondata di refactor e ondata di misura** obbliga a riportare
  i risultati negativi.
- **Il verde non basta.** Serve rendere visibili skip, xfail e oracolo:
  altrimenti la copertura dichiarata supera quella eseguita.

## Registro audit

Procedura e checklist 12/12 in [history/AUDIT_LOG.md](history/AUDIT_LOG.md)
(template "Audit Ondata N"). Da qui in avanti i verdetti si appendono in
questa sezione; l'anchor è sempre l'ultimo verdetto ✅.

| Data | Oggetto | Verdetto | Anchor rgrep | Auditor |
|---|---|---|---|---|
| 2026-05-03 | Step 0–17 | ✅ (6 PARTIAL recuperati con -bis) | `6ccd72a` | Claude Opus 4.7, indipendente da Gemini |
| 2026-05-04 | Step 18–19 | ✅ | `b9e797e` | Claude Opus 4.7 |
| 2026-05-18 | Ondate 1–6 | ✅ 12/12 ciascuna | `759524b` | Claude Opus 4.7 (Implementer e Auditor in sessioni distinte) |
| 2026-09-30 | Ondata 6-bis, Ondata 7, verifica GNU | ✅ | `3e9d38f` | **Codex, autoverifica**: non indipendente |
| — | `ab26dc4`, `92285c7` | non auditati | — | vedi [TODO.md P1](TODO.md#p1--chiudere-il-ciclo-di-audit-sui-commit-post-anchor) |

**Anchor corrente: `3e9d38f`.** HEAD `main`: `92285c7`.
