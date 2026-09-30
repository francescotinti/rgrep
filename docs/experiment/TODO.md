# TODO — cosa resta da fare

> Backlog attivo dell'esperimento `testag-grep` (GNU grep → Rust).
> Aggiornato: 2026-09-30. Codice `rgrep` su `main` = `f2710b4` (pubblicato su
> GitHub, CI verde 4/4). Ultimo audit registrato: `f2710b4` (anchor).
> Cosa è già stato fatto: [DONE.md](DONE.md). Come è fatto il progetto:
> [ARCHITECTURE.md](ARCHITECTURE.md).

Questo file sostituisce il vecchio `NEXT_STEPS.md` come **unico backlog**.
Le SPEC contrattuali dei nuovi step si scrivono qui, nella sezione
"Step attivi", con il formato storico (D-decisioni numerate, testcase
obbligatori, acceptance criteria, out-of-scope). Alla chiusura, lo step
migra in [DONE.md](DONE.md) e la SPEC integrale resta consultabile nel
commit che l'ha introdotta. Le SPEC storiche (Step 0–19, Ondate 1–7) sono in
[history/NEXT_STEPS.md](history/NEXT_STEPS.md).

## Regole di ingaggio (invariate)

- Un solo step alla volta è 🚧 PRONTO; gli altri sono 🔒 LOCKED.
- Nessuna promozione senza richiesta esplicita del Decider.
- Ogni step = un commit nel subrepo `rgrep` (branch `main`), gate verdi
  prima del commit: build 0 warning, `cargo test`, `cargo clippy --tests
  -- -D warnings`, `cargo fmt --check`.
- Mai modificare gli `expected_stdout` di un testcase per farlo passare:
  aprire `SPEC-Q` o `DESIGN-Q`.
- Audit con checklist 12/12; il verdetto e il nuovo anchor si registrano in
  [DONE.md → Registro audit](DONE.md#registro-audit).

## Step attivi

Uno step scritto, **🔒 LOCKED** in attesa di promozione del Decider.

---

# 🔒 Step BD — Diagnostica binaria: messaggio su stderr e `-c` completo

**Stato**: 🔒 LOCKED — SPEC scritta dall'Architect il 2026-09-30, baseline
`f2710b4`. Da promuovere a 🚧 PRONTO dal Decider.
**Chiude**: divergenze GNU 0054 e 0067 (TODO P2). Prerequisito di P4
(strict mode) insieme allo Step per `-T`.
**Evidenza**: `reports/binary-diag-evidence-20260930.txt` (31 combinazioni
di flag, tre binari, stdout/stderr/exit separati), `gnu-grep/src/grep.c`
righe 1561–1575 e 1650–1655, `gnu-grep/NEWS` release 3.5 e 3.8,
`gnu-grep/tests/binary-file-matches`.

## Goal

Allineare rgrep a GNU grep ≥3.5 sul messaggio "binary file matches":
canale **stderr**, formato `rgrep: NAME: binary file matches`, stdout
vuoto. Nello stesso ramo di codice correggere `-c` su file binario, che
oggi si ferma al primo match (rgrep 1, GNU e BSD 2).

## Comportamento osservato (dal report di evidenza)

| Invocazione | GNU 3.12 | BSD 2.6.0 | rgrep oggi | Esito atteso |
|---|---|---|---|---|
| `foo bin.dat` | stderr `grep: bin.dat: binary file matches`, stdout vuoto, exit 0 | stdout `Binary file bin.dat matches` | come BSD | come GNU |
| `-s foo bin.dat` | messaggio **non** soppresso (NEWS 3.8, bug 51860) | come sopra | come BSD | come GNU |
| `-q foo bin.dat` | nulla, exit 0 | nulla | nulla | invariato |
| `-c foo bin.dat` | `2` | `2` | **`1`** | `2` |
| `-l` / `-L` | solo nome file / nulla, nessun messaggio | uguale | uguale | invariato |
| `-o`, `-n`, `-H`, `-m1`, `-A1`, `-v zzz`, `-U`, `--color=always` | solo il messaggio su stderr | solo il messaggio su stdout | come BSD | come GNU |
| `-a`, `--binary-files=text`, `-z` | righe stampate, nessun messaggio | uguale | uguale | invariato |
| `-I`, `--binary-files=without-match` | nulla, exit 1 | uguale | uguale | invariato |
| `nomatch bin.dat` | nulla, exit 1 | uguale | uguale | invariato |
| stdin senza `--label` | `grep: (standard input): binary file matches` | `Binary file (standard input) matches` | come BSD | come GNU |
| stdin con `--label=LBL` | `grep: LBL: binary file matches` | `Binary file LBL matches` | come BSD | come GNU |
| due file binari | un messaggio per file, ordine argomenti | uguale (stdout) | come BSD | come GNU |

Riferimento upstream (`grep.c` 1650–1655): il messaggio è emesso una sola
volta per file a fine `grep()`, con `error (0, 0, "%s: binary file
matches", input_filename ())`, solo se `binary_files == BINARY` e
`!out_quiet` (cioè non `-q`, non `-c`, non `-l`/`-L`) e c'è stato almeno
un match dopo la rilevazione del NUL. `-s` non entra nella condizione.

## D-decisioni (contratto)

### D BD.1 — Canale e formato

Il messaggio va su **stderr**, terminato da `\n`, nel formato
`rgrep: {name}: binary file matches`, dove `{name}` è il nome file come
passato in argv, oppure `(standard input)` per stdin, oppure il valore di
`--label` se presente. Prefisso `rgrep:` come gli altri diagnostici del
runner (`eprintln!("rgrep: {}: {e}", …)`), non `grep:`. Stdout non
riceve nulla per quel file. Exit code invariato (0 se match).

### D BD.2 — Condizioni di emissione

Emettere il messaggio se e solo se: il file è stato rilevato binario
(`binary_action == Binary`, NUL nel primo chunk, delimitatore `\n`) **e**
almeno una riga ha matchato **e** nessuna di `-q`, `-c`, `-l`, `-L` è
attiva. `-s` **non** sopprime il messaggio. `-o`, `-n`, `-b`, `-H`, `-h`,
`-m`, `-A/-B/-C`, `-v`, `-U`, `--color` non lo alterano: nessun output di
riga viene prodotto per un file binario in modalità `binary`.

### D BD.3 — Un messaggio per file, poi stop

Dopo il primo match su file binario la lettura del file si interrompe
(come oggi: `LineOutcome::Return(true)`), salvo `-c` (D BD.4). Con più
file, un messaggio per ogni file binario con match, nell'ordine di
elaborazione. Flush di stdout prima di scrivere su stderr non richiesto:
il harness confronta i due canali separatamente.

### D BD.4 — `-c` su file binario conta tutte le righe

Nel ramo `is_binary` di `process_line`, con `output.count` la decisione
diventa `LineOutcome::Continue` (si continua a contare) invece di
`Break`. `-l`/`-L` restano `Break`. Il conteggio finale stampa `N` come
per i file di testo, senza messaggio. Parità attesa con GNU **e** BSD
(entrambi 2 sul fixture). Il fast path `pure_count_quiet` resta escluso
per `is_binary` (nessuna modifica a quella condizione).

### D BD.5 — Harness: verifica di stderr per rgrep

Aggiungere al manifest il campo opzionale `expected_stderr_contains:
String`, verificato **solo su rgrep** come sottostringa byte della sua
stderr (stesso meccanismo di `gnu_stderr_contains`, che continua a
riferirsi all'oracolo). Se assente, stderr di rgrep non è verificata
(comportamento attuale). Aggiornare `tests/README.md` con il campo.
Nessun'altra modifica al harness.

### D BD.6 — Testcase (TDD-first: rossi prima dell'implementazione)

Modifiche ai case esistenti, autorizzate da questa SPEC (non sono
"expected cambiati per far passare"):

- `0054_binary_default.toml`: `expected_stdout = ""`,
  `expected_stderr_contains = ": binary file matches\n"`, rimuovere
  `gnu_difference`/`gnu_stdout`/`gnu_stderr_contains`, aggiungere
  `skip_if_bsd = true`, `skip_reason = "BSD grep prints the binary-match
  message on stdout (GNU >= 3.5 uses stderr)"`.
- `0067_label_binary.toml`: idem, con
  `expected_stderr_contains = "rgrep: BIN: binary file matches\n"`.

Nuovi case, numerati da `0076` in poi (ultimo manuale: `0075`), registrati
in `tests/testsuite.toml`:

| # | Nome | Args / stdin | Atteso | Oracoli |
|---|---|---|---|---|
| 0076 | binary -s keeps message | `-s foo {FIXTURES}/bin.dat` | stdout `""`, stderr contiene `: binary file matches`, exit 0 | GNU; `skip_if_bsd` |
| 0077 | binary -c counts all lines | `-c foo {FIXTURES}/bin.dat` con `foo\0bar\nfoo again\n` | stdout `2\n`, exit 0 | GNU **e** BSD |
| 0078 | binary -q silent | `-q foo {FIXTURES}/bin.dat` | stdout `""`, stderr non contiene `binary file` (verificare con un unit test, il harness non ha negazione), exit 0 | GNU e BSD |
| 0079 | binary -l no message | `-l foo {FIXTURES}/bin.dat` | stdout `{FIXTURES}/bin.dat\n`, exit 0 | GNU e BSD |
| 0080 | binary -o message only | `-o foo {FIXTURES}/bin.dat` | stdout `""`, stderr contiene `: binary file matches`, exit 0 | GNU; `skip_if_bsd` |
| 0081 | binary two files | `foo {FIXTURES}/a.dat {FIXTURES}/b.dat` entrambi binari | stdout `""`, stderr contiene `a.dat: binary file matches\nrgrep: ` (ordine), exit 0 | GNU; `skip_if_bsd` |
| 0082 | binary stdin default label | stdin `foo\0bar\n`, args `foo` | stdout `""`, stderr contiene `rgrep: (standard input): binary file matches\n` | GNU; `skip_if_bsd` |
| 0083 | binary -m1 message | `-m1 foo {FIXTURES}/bin.dat` | stdout `""`, stderr contiene il messaggio | GNU; `skip_if_bsd` |

Unit test in `src/runner.rs` (≥3): formato del messaggio con nome file,
con `(standard input)`, con label; `-q` non emette nulla su stderr
(catturare la scrittura tramite il writer già iniettato in `PrintCtx` se
esiste, altrimenti `// DESIGN-Q:` sul modo di catturare stderr e fermarsi).

### D BD.7 — Conteggi attesi post-implementazione

Manifest 129 → 137. GNU default: `known_gnu_divergences` 4 → **2**
(restano 0064/0065), parità 121 → 131 (8 nuovi + 0054 + 0067), skip
invariati. BSD default: skip 16 → 23 (0054, 0067, 0076, 0080, 0081, 0082,
0083), parità 112 → 115 (0077, 0078, 0079). Riportare i numeri reali
osservati nel commit e in `GNU_VERIFICATION.md`, non questi valori attesi.
`RGREP_STRICT_GNU=1` deve fallire **solo** su 0064 e 0065.

### D BD.8 — File toccati e budget

- `src/runner.rs`: solo il ramo `if is_binary { … }` di `process_line`
  (righe 346–357 a `f2710b4`) e gli unit test. ≤15 LOC production nette.
- `tests/diff_runner.rs`: campo `expected_stderr_contains` + controllo
  (≤12 LOC). `tests/README.md`: una voce.
- `tests/cases/0054`, `0067`, `0076`–`0083`, `tests/testsuite.toml`.
- `GNU_VERIFICATION.md`: tabella conteggi e paragrafo divergenze (da 4 a 2).
- `PORTING.md`: nessuna modifica (il messaggio non è tra le divergenze
  di design elencate).
- Vietato toccare `cli.rs`, `matcher.rs`, `output.rs`, il fast path
  `pure_count_quiet`, la rilevazione binaria in `bufread_search`.

### D BD.9 — Gate e commit

Gate: build 0 warning, `cargo test` con `RGREP_ORACLE=/opt/homebrew/bin/ggrep`
(default e `--features perl-regexp`) e con `/usr/bin/grep`, clippy
`--tests --all-features -D warnings`, fmt. Un solo commit:

```
fix(binary-diag): route binary-match diagnostic to stderr and count all lines with -c

IN-SCOPE:
- ...
OUT-OF-SCOPE (debito esplicito):
- ...
Testcase aggiunti: 8. Totali: 137.
```

### D BD.10 — Out-of-scope (debito esplicito, registrato in backlog)

- **NUL come terminatore dopo la rilevazione** (`nul_zapper` in
  `grep.c` 1572): GNU, una volta rilevato il binario, tratta i NUL come
  fine riga; `-cv zzz late.dat` dà 3 su GNU, 2 su BSD, 1 su rgrep oggi
  (2 dopo D BD.4). Divergenza GNU-only da documentare, non da
  implementare qui (→ P6).
- **Rilevazione oltre il primo chunk**: GNU controlla ogni buffer
  (`grep.c` 1561) e usa `file_must_have_nulls` per gli sparse file; rgrep
  guarda solo il primo `fill_buf` (intero file se slurped <16 MB, 8 KB su
  stdin/pipe/file grandi). Un NUL tardivo su stdin >8 KB produce output
  di testo in rgrep e messaggio in GNU (→ P6).
- **Encoding error come binario** (`encoding_error_output`): rilevante
  solo in locale UTF-8; il harness usa `LC_ALL=C` (→ P6, bassa priorità).
- `-T` (Step separato, P3).

## Checklist Auditor

1. Build, clippy, fmt verdi; test verdi con GNU (default + PCRE2) e BSD.
2. Un solo commit, formato integrale, `Testcase aggiunti: 8. Totali: 137`.
3. D BD.1–BD.9 applicate letteralmente; D BD.10 non implementata.
4. Diff di `src/runner.rs` confinato al ramo `is_binary` + unit test.
5. Live exec delle 12 righe della tabella "Comportamento osservato" con
   `ggrep`, `rgrep` e `/usr/bin/grep`: stessa tabella del report di
   evidenza, colonna rgrep ora uguale a GNU.
6. `[summary]` GNU: `known_gnu_divergences=2`; strict mode fallisce solo
   su 0064/0065.
7. Nessun `gnu_difference` residuo su 0054/0067.
8. `GNU_VERIFICATION.md` e `tests/README.md` aggiornati con numeri reali.
9. Nessuna modifica fuori da D BD.8.
10. `expected_stderr_contains` verificato solo su rgrep; `gnu_stderr_contains` invariato.
11. Unit test ≥3 presenti e significativi.
12. Anchor aggiornato in `DONE.md`, step spostato da TODO a DONE.

---

## Backlog ordinato per priorità

### ~~P1 — Chiudere il ciclo di audit sui commit post-anchor~~ ✅ chiuso 2026-09-30

Audit di `ab26dc4`, `92285c7`, `9e9a09a`, `f2710b4` registrato in
[DONE.md](DONE.md#audit-commit-post-anchor--2026-09-30--ab26dc4f2710b4--approved);
anchor spostato a `f2710b4`.

Resta aperta la **rilettura indipendente** degli audit di Ondata 6-bis,
Ondata 7 e verifica GNU, che Codex ha dichiarato come autoverifiche: i gate
sono stati rieseguiti verdi su `f2710b4` (stesso codice di `3e9d38f`), ma
la checklist 12/12 contro le SPEC emendate non è stata ripercorsa da un
Auditor diverso. Priorità bassa: nessuna evidenza contraria emersa.

### P2 — SPEC "diagnostica binaria" (casi 0054, 0067) → **Step BD scritto** 🔒

SPEC completa in [Step attivi](#-step-bd--diagnostica-binaria-messaggio-su-stderr-e--c-completo).
Decisione dell'Architect: allineare a GNU ≥3.5 (stderr), marcare
`skip_if_bsd` i case che dipendono dal canale, mantenere copertura BSD con
i case neutri (`-c`, `-l`, `-q`). Motivazione: l'oracolo di riferimento
del port è GNU (snapshot upstream 3.12 in `gnu-grep/`, CI Linux), BSD è
solo l'oracolo di comodo su macOS; la modalità strict deve poter diventare
verde. Scoperta collaterale, inclusa nello step: `-c` su file binario
conta 1 riga invece di tutte (GNU e BSD concordano su tutte).

### P3 — SPEC "allineamento -T" (casi 0064, 0065) 🔒

- rgrep antepone sempre un `\t` alla riga con `-T`, anche senza prefisso.
- GNU non emette il tab quando non c'è alcun prefisso (`-T foo` → `foo`),
  e con `-n`/`-H` allinea il prefisso con padding + tab in posizione diversa.

La SPEC deve riprodurre la regola GNU (`print_line_head` in `grep.c`:
tab solo dopo un prefisso, padding numerico a larghezza fissa), con
testcase GNU-only (`skip_if_bsd`, BSD non supporta `-T`). Tocca
`src/runner.rs` / `src/output.rs`.

### P4 — Modalità strict come gate

Dopo P2 e P3: rimuovere i campi `gnu_difference`/`gnu_stdout`/
`gnu_stderr_contains` dai quattro case, verificare che
`RGREP_STRICT_GNU=1 cargo test --test diff_runner` sia verde su Linux e
renderlo il comando eseguito da `.github/workflows/verify.yml` nei job GNU.

### P5 — Divergenze di design ancora "aperte, accettate"

Decisioni prese all'inizio e mai riconfermate formalmente:

- **D-NEW-1** default ERE (`-E`) invece di BRE. Da confermare o da
  invertire con `-G` default e translator BRE→ERE già esistente.
- **D-NEW-3** `--color` default `never` invece di `auto`. Il diario Step 17
  prevedeva un eventuale passaggio ad `auto` mai eseguito (richiede
  rilevamento TTY e testcase con `--color=never` esplicito).

Il Decider chiude entrambe con una riga in [DONE.md](DONE.md) oppure apre
uno step.

### P6 — Debiti minori registrati negli audit

- Audit Step 11-bis: `D 11.12` (≥3 unit test dedicati alla rilevazione
  binaria) segnato come "carried-over debt" e mai chiuso esplicitamente.
  Lo Step BD ne aggiunge ≥3 sul messaggio; la rilevazione resta scoperta.
- **Semantica binaria GNU non replicata** (emersa scrivendo lo Step BD,
  evidenza in `reports/binary-diag-evidence-20260930.txt`), tre voci
  distinte, tutte GNU-only o dipendenti dalla dimensione dell'input:
  1. dopo la rilevazione GNU tratta i NUL come terminatori di riga
     (`nul_zapper`): `-cv zzz` su `foo\nfoo\0bar\n` dà 3 su GNU, 2 su BSD;
  2. GNU rileva i NUL su ogni buffer e sugli sparse file, rgrep solo sul
     primo chunk (8 KB su stdin/pipe/file >16 MB);
  3. GNU considera binario anche l'output con errori di encoding in
     locale UTF-8 (`encoding_error_output`); irrilevante con `LC_ALL=C`.
  Nessuna delle tre ha oggi un testcase; da decidere se documentarle
  come divergenze accettate (`D-NEW-6..8`) o aprire uno Step.
- `diary/04a-divergences.md` storico non elenca le quattro divergenze GNU
  del 2026-09-30: ora sono in [ARCHITECTURE.md](ARCHITECTURE.md#divergenze-documentate).
- Housekeeping outer: directory non tracciate `.serena/`, `.vexp/`,
  `_archive/` e working tree di `gnu-grep/` non pulito. Decidere se
  ignorarle in `.gitignore` o rimuoverle.

### P7 — Performance (rinviato, non prioritario)

Tutte le voci sotto sono esplicitamente **fuori scope** finché P2–P4 non
sono chiuse. Ordine indicativo di leva, dai profili di Ondata 5/7:

| Candidato | Scenario che ne beneficerebbe | Nota |
|---|---|---|
| `PERF-FIX-O7-2` zero-copy: passare `&chunk[..=pos]` a `process_line` senza `extend_from_slice` | count/scan su file grandi (`memmove` 19,9% self post-O7) | ≤10 LOC, richiede firma `&[u8]` |
| Engine swap `regex::bytes::Regex` | `regex_simple`, `invert_match` | ~150 LOC, ridisegna `MatchEngine` |
| Sostituire `walkdir` con `read_dir` manuale, o walk parallela `rayon` | `fixed_string_F`, `recursive_walk` (syscall-bound al 37%) | ondata "fs walker" / "parallelism" |
| Harness bench FFI (senza process spawn) | tutti gli scenari piccoli (spawn ≈ 25–60% del tempo) | richiede API pubblica di `runner` |
| Fixture >10 MB per `mmap_vs_bufread` | conferma del vantaggio mmap | fixture generata in `target/` |
| `dhat` per il tracking allocazioni | validazione "zero alloc su ASCII" | dev-dep opzionale |

Gate per qualsiasi ondata perf: baseline Criterion nominata, confronto con
coppie alternate pre/post (`tools/compare_binaries.py`), ≥5% su almeno uno
scenario line-bound, nessuna regressione mediana >5%, risultati sfavorevoli
registrati.

## Cosa NON è in backlog (scope-out permanente)

- `GREP_OPTIONS`, `--rpm-*`, locale multibyte legacy (Shift-JIS ecc.),
  gnulib portability layer, fallback DFA→POSIX interno di GNU grep.
  Motivazione in [ARCHITECTURE.md](ARCHITECTURE.md#scope-out).
