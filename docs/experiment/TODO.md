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

Nessuno. Stato: 🔒 tutto LOCKED in attesa di promozione.

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

### P2 — SPEC "diagnostica binaria" (casi 0054, 0067) 🔒

Divergenza reale rispetto a GNU grep 3.12, oggi registrata nel manifest come
`gnu_difference` e rifiutata da `RGREP_STRICT_GNU=1`.

- rgrep stampa `Binary file X matches` su **stdout** (comportamento GNU
  storico, ≤3.4, e BSD grep 2.6.0).
- GNU ≥3.5 stampa `grep: X: binary file matches` su **stderr**, stdout vuoto.

La SPEC deve decidere: allineare a GNU 3.12 (stderr) accettando la
divergenza da BSD, oppure mantenere stdout e documentarla come `D-NEW-6`.
Vincoli: TDD-first con oracolo GNU (`RGREP_ORACLE=/opt/homebrew/bin/ggrep`
sul Mac, `/usr/bin/grep` su Linux), nessuna modifica degli expected senza
SPEC-Q, aggiornare `skip_if_bsd` se si sceglie stderr. Tocca
`src/runner.rs` (emissione del messaggio) ed eventualmente il case 0067
(`--label`). Riferimento: [GNU_VERIFICATION.md](../../GNU_VERIFICATION.md).

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
  Verificare se i test aggiunti nelle ondate successive lo coprono.
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
