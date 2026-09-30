# NEXT_STEPS — testag-grep (workflow Architect ↔ Implementer)

> Workflow attivo. **Solo UN step alla volta è 🚧 PRONTO PER IMPLEMENTER.**
> Tutti gli altri sono 🔒 LOCKED finché l'Architect non li promuove dopo audit ✅.
>
> Riferimenti: [CLAUDE.md](CLAUDE.md), [EXPERIMENT_PLAN.md](EXPERIMENT_PLAN.md),
> [AUDIT_LOG.md](AUDIT_LOG.md). Skill: `legacy-port` (Anthropic Claude).

---

## Tag legenda

### ✅ Verifica GNU locale — 2026-09-30 — `3e9d38f` (Linux CI pending)

- Selezionare l'oracolo con `RGREP_ORACLE`, rilevarne identità/versione una
  sola volta; proprietà eseguite su GNU e skip esplicito su BSD.
- Il runner deve verificare stdout/exit di rgrep contro gli expected anche
  quando l'exit non è zero. Comparare byte, senza conversione UTF-8 lossy.
- Pubblicare totale, pass differenziali, verifiche solo rgrep, skip feature/OS,
  divergenze GNU note e fallimenti. Nessun xfail generico che nasconda regressioni.
- SPEC-Q risolta con evidenza GNU 3.12: `--mmap` è un'opzione di rgrep;
  comparare i tre casi con gli stessi args GNU senza `--mmap`, mantenendo
  tutti gli expected. Il testcase `-P unavailable` verifica soltanto rgrep
  senza feature e va saltato quando PCRE2 è compilato.
- `directory skip` passa già su GNU e BSD: rimuovere l'xfail obsoleto.
- Quattro divergenze reali preesistenti (0054/0067 messaggio binario su stdout
  vs stderr GNU; 0064/0065 tabulazione) restano debito esplicito. Non cambiare
  expected rgrep per far passare l'oracolo. Registrare separatamente gli output
  GNU osservati e validarli entrambi; conteggio «divergenze note», non «parità».
  Modalità strict opzionale deve fallire se una divergenza nota viene incontrata.
- Workflow Linux GNU + macOS BSD, default e PCRE2, con log archiviati. Nessun
  push o risultato CI remoto implicito; se Linux non è disponibile localmente,
  consegnare workflow verificato staticamente e dichiarare run non eseguita.
- Ambito: harness, fixture metadata, CI, documentazione. Nessun cambiamento
  semantico del motore in questa fase. Un commit dedicato dopo gate verdi.

## Sequenza autorizzata dal Decider — 2026-09-30

Il Decider ha approvato: consolidamento baseline → Ondata 6-bis benchmark →
Ondata 7 emendata → verifica GNU e chiusura documentale. Esecutore: Codex.
Questa sezione prevale sugli stati storici. Sequenza locale conclusa; CI Linux pending.

- ✅ **Ondata 6-bis APPROVED — `723aab3`**: correggere il pattern invalido di `regex_simple`
  (`^[a-z]+\\(` a runtime), far fallire i benchmark su exit diverso da 0/1,
  aggiungere `-cF`/`-qF`, salvare una baseline Criterion nominata e artefatti
  numerici verificabili. Nessuna modifica al motore. Un commit subrepo.
- ✅ **Ondata 7 APPROVED — `3de67b0`**: promossa dopo 6-bis; emendare prima i vincoli per test
  dei confini buffer, EOF, NUL ed errori, helper condiviso fra entrambi i
  loop O6, baseline attuale e target realmente line-bound. Il profilo di
  `fixed_string_F` misura 8,4% memchr, non 32–38%. Non usare i vecchi tempi
  di `regex_simple` come confronto: misuravano una regex invalida.
- ✅ **Verifica GNU locale — `3e9d38f`**: oracolo selezionabile, conteggi pass/skip/xfail visibili,
  esecuzione GNU locale e workflow Linux riproducibile. Divergenze reali
  devono restare visibili; vietato aggiornare expected per nasconderle.
- Documentazione finale: metriche attuali distinte da quelle storiche;
  nessuna attestazione di CI Linux senza una run osservata.

**Esito**: O6-bis `723aab3`, O7 `3de67b0`, verifica GNU `3e9d38f`.
Nessuno step PRONTO rimasto. Prossimo backlog: eseguire CI Linux; poi SPEC
dedicate per le quattro differenze GNU (diagnostica binaria, allineamento -T).
Non è stata pubblicata alcuna modifica remota.

### Evidenza baseline del 2026-09-30

`759524b`, rustc 1.98.1, macOS arm64: build, test default (32 unit + 1 harness
+ 3 proprietà), test PCRE2 (34 unit + 1 harness + 3 proprietà), Clippy
all-features e fmt verdi; bench compilabile. Le proprietà saltano su BSD:
il verde non implica 3072 confronti. La toolchain differisce dai report di
maggio (1.90.0). Le mediane O6 e i delta Criterion non condividono sempre
la stessa baseline: confronto storico da qualificare, non da ricostruire
inventando dati mancanti.

- 🚧 **PRONTO PER IMPLEMENTER** — Implementer può iniziare
- 🟢 **FATTO — AUDIT PENDING** — Implementer ha committato, aspetta audit
- 🟡 **BLOCKED — DESIGN-Q / SPEC-Q** — Implementer ha aperto domanda, aspetta Architect
- 🟡 **PARTIAL** — Audit ha trovato leftover; aprirà Step N-bis
- ✅ **APPROVED** — Audit ✅ in AUDIT_LOG.md, step chiuso
- 🔒 **LOCKED** — non avviabile finché lo step precedente non è ✅

## Format commit message obbligatorio

```
feat(stepN): <titolo breve in inglese>

IN-SCOPE:
- <bullet list di cosa è stato implementato>

OUT-OF-SCOPE (debito esplicito):
- <bullet list di cosa è stato deliberatamente lasciato fuori>

Testcase aggiunti: N. Totali: M.
```

---

# 🟡 Step 0 — Baseline test harness + differential infra (PARTIAL)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit**: `04f94dc` (in `rgrep/.git`)
**Leftover**: vedi Step 0-bis sotto.

## Goal

Stabilire l'infrastruttura di **differential testing** vs il binario `grep` di
sistema (process-spawn pattern, no FFI), introdurre il manifest-based test
suite layout, e verificare che la baseline rgrep esistente passa una suite
minima di 5 testcase confrontata con `grep` reale.

Questo step **non aggiunge feature**: solo plumbing di test. Il codice in
`rgrep/src/` non deve cambiare salvo refactor minimi indispensabili a far
girare i test esistenti.

## D-decisioni (contratto, non riapribile in questo step)

### D 0.1 — Differential strategy = process-spawn
Useremo `std::process::Command::new("grep")` per eseguire l'oracolo, non FFI.
Razionale: `grep` è installato di default su macOS/Linux, l'overhead di
spawn-per-test è accettabile per <500 testcase, evita di compilare GNU grep da
sorgente. Pattern bonus della skill (rawk Step 17 lesson).

### D 0.2 — Test layout: cases dir + manifest TOML
Struttura:
```
rgrep/tests/
├── diff_runner.rs               # integration test, legge manifest e itera
├── testsuite.toml               # manifest autoritativo
└── cases/
    ├── 0001_basic_literal.toml
    ├── 0002_basic_regex.toml
    ├── 0003_ignore_case.toml
    ├── 0004_invert_match.toml
    └── 0005_count.toml
```
Razionale: pattern testsuite manifest skill (rawk Step 17). Numerazione
zero-padded a 4 cifre, "buchi" liberi, manifest controlla l'ordine. NO
autodiscover di file fuori manifest.

### D 0.3 — Formato testcase TOML
```toml
# tests/cases/0001_basic_literal.toml
name = "basic literal match"
args = ["foo"]
stdin = "foo\nbar\nfoo bar\n"
expected_stdout = "foo\nfoo bar\n"
expected_exit_code = 0
# campi opzionali:
# skip_if_bsd = true   # skip su BSD grep (macOS), eseguire solo su GNU
# skip_reason = "..."
# expected_stderr_contains = "..."
```
Razionale: TOML è già nelle dev-dependencies plausibili e parsa con `toml`
crate (single-line value safe). Self-contained, niente file esterni di
fixture per ora.

### D 0.4 — Oracolo è il binario `grep` in `$PATH`
Il harness rileva il path con `which grep` (o `Command::new("grep")`). Su
macOS sarà BSD grep, su Linux CI sarà GNU grep. Documentare in commit message
quale variante è stata usata localmente.

### D 0.5 — Outcome enum del diff
```rust
enum DiffOutcome {
    Match,                                    // expected
    Differ { rgrep: String, oracle: String },// segnalato come failure
    BothFail { rgrep_code: i32, oracle_code: i32 },
    OnlyRgrepOk { oracle_code: i32 },         // INVARIANTE: mai accettabile
    OnlyOracleOk { rgrep_code: i32 },         // segnalato, può essere noto
}
```
Razionale: skill Phase 4a outcome enum. `OnlyRgrepOk` è bug grave (rgrep
accetta input che oracle rifiuta = rgrep non strict-by-spec).

### D 0.6 — Build precondition
`cargo build --manifest-path rgrep/Cargo.toml` deve produrre **0 warning**.
`cargo test --manifest-path rgrep/Cargo.toml` deve essere **verde** (sia
unit test che il nuovo integration test `diff_runner`).
Anti-pattern critico (rawk Step 13): mai committare con build red.

### D 0.7 — Skip BSD opt-in
Per ora supportiamo entrambi BSD e GNU grep. Un testcase con
`skip_if_bsd = true` viene **saltato** se `grep --version` non contiene "GNU".
Marcare con `eprintln!("[skip] {} (bsd grep)", name)` per visibilità.

### D 0.8 — File modificati attesi
```
rgrep/Cargo.toml                      (aggiungere dev-dep: toml = "0.8")
rgrep/tests/diff_runner.rs            (nuovo)
rgrep/tests/testsuite.toml            (nuovo)
rgrep/tests/cases/0001_basic_literal.toml
rgrep/tests/cases/0002_basic_regex.toml
rgrep/tests/cases/0003_ignore_case.toml
rgrep/tests/cases/0004_invert_match.toml
rgrep/tests/cases/0005_count.toml
```
**Nessun file in `rgrep/src/` deve essere toccato in questo step**, salvo
fix indispensabile per build verde (in tal caso aprire DESIGN-Q).

## Testcase obbligatori (TDD-first)

Aggiungere **prima** della logica del diff_runner:

| ID | Args | Stdin | Expected | Note |
|---|---|---|---|---|
| 0001 | `["foo"]` | `foo\nbar\nfoo bar\n` | `foo\nfoo bar\n` (exit 0) | match literale base |
| 0002 | `["^bar"]` | `foo\nbar\nbar baz\n` | `bar\nbar baz\n` (exit 0) | regex anchor |
| 0003 | `["-i", "FOO"]` | `Foo\nfoo\nbar\n` | `Foo\nfoo\n` (exit 0) | case-insensitive |
| 0004 | `["-v", "foo"]` | `foo\nbar\nbaz\n` | `bar\nbaz\n` (exit 0) | invert match |
| 0005 | `["-c", "foo"]` | `foo\nbar\nfoo\n` | `2\n` (exit 0) | count |

Il diff_runner deve, per ogni file in manifest:
1. Parsare TOML
2. Eseguire `grep <args>` con stdin
3. Eseguire `cargo run -- <args>` con stesso stdin (oppure `target/debug/rgrep`
   se già buildato — preferire path al binario per velocità)
4. Confrontare outcome
5. Failure del test Rust se `Differ`, `OnlyRgrepOk`, o se
   `BothFail.rgrep_code != BothFail.oracle_code`

## Acceptance criteria

- ✅ `cargo build --manifest-path rgrep/Cargo.toml` con **0 warning**
- ✅ `cargo test --manifest-path rgrep/Cargo.toml` verde
- ✅ I 5 testcase listati sono presenti, in manifest, e passano
- ✅ Almeno il testcase 0001 è eseguito su entrambi BSD e GNU (no skip)
- ✅ Il commit segue il format obbligatorio
- ✅ Aggiunto **un** README.md minimo in `rgrep/tests/` con istruzioni
  "come aggiungere un nuovo testcase" (max 30 righe)

Se uno o più dei 5 testcase **falliscono** sulla baseline rgrep esistente,
NON modificare `rgrep/src/`: marcare il testcase con `expected_to_fail = true`
nel TOML e aprire **SPEC-Q** nel commit message. L'Architect deciderà se
correggere il sorgente in uno step successivo o accettare la divergenza in
mapping-table.md.

## Out-of-scope (debito esplicito)

- Property-based testing (rinviato a Step 16)
- Test su file reali (non solo stdin) — rinviato a Step 5+
- Test su recursion/glob — rinviato a Step 7+
- CI integration (.github/workflows) — non in questo step
- Benchmark — non in questo esperimento

## Commit attesi

**Un solo commit** con titolo:
```
feat(step0): differential test harness vs system grep
```

Body conforme al format obbligatorio (vedi sopra).

## Quando finisci

1. `cargo build` 0 warning + `cargo test` verde verificati ✅
2. Cambia l'header di questo step da `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit <hash>`
3. **Stop**. Non avviare Step 1. Aspetta il trigger `audita` dal Decider.

---

# ✅ Step 0-bis — Hygiene + decision rationale leftover di Step 0

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `bbe49b7` — **Commit outer**: `5955087`

## Goal

Chiudere i leftover dell'audit di Step 0 (verdict 🟡 PARTIAL, vedi
[AUDIT_LOG.md](AUDIT_LOG.md)). Nessuna nuova feature, nessun nuovo testcase.
Focus: hygiene del repo + ratifica del cambio a `cli.rs` come D-decisione
formalmente accettata + commit dell'infra Architect nel repo outer.

## Contesto

Audit Step 0 ha approvato la sostanza tecnica (5/5 testcase verdi, build verde,
DiffOutcome enum corretto) ma rilevato **violazioni di processo** non di
codice:
1. `cli.rs` toccato senza che fosse strettamente necessario per Step 0
   (i 5 testcase non usano `-h`). Però il fix `disable_help_flag = true`
   risolve un conflitto reale tra `-h` clap auto-help e `-h` GNU grep
   `--no-filename` che colpirà gli step 5+. L'Architect lo **ratifica
   retroattivamente** come D-G10 invece di chiedere il revert.
2. `rgrep/.gitignore` mancante; 12+ macOS `._*` AppleDouble blob committati.
3. Repo outer `testag-grep/` ha 0 commit: CLAUDE.md, NEXT_STEPS.md,
   AUDIT_LOG.md, EXPERIMENT_PLAN.md, diary/ tutti untracked.

## D-decisioni (contratto)

### D 0-bis.1 — Ratifica del fix `cli.rs` come D-G10
**Aggiungere** in `diary/02-mapping-table.md` (sezione D-decisioni globali)
un nuovo elemento:
- **D-G10**: `disable_help_flag = true` su `Config` di clap.
  Razionale: GNU grep usa `-h` per `--no-filename`; default clap usa `-h` per
  `--help`. Senza `disable_help_flag`, clap intercetta `-h` come help e
  rgrep diverge dal comportamento GNU. `--help` resta funzionante (long form).

### D 0-bis.2 — Aggiungere `.gitignore` in `rgrep/`
Contenuto minimo (tutto su file separato `rgrep/.gitignore`):
```gitignore
target/
**/.DS_Store
**/._*
```
Razionale: prevenire pollution macOS metadata (`._*`, `.DS_Store`) e build
artifacts.

### D 0-bis.3 — Rimuovere AppleDouble files committati in `04f94dc`
Eseguire:
```bash
cd rgrep
git rm --cached -f tests/._* tests/cases/._* 2>/dev/null || true
git rm --cached -f .DS_Store tests/.DS_Store tests/cases/.DS_Store 2>/dev/null || true
```
Verificare con `git ls-files | grep -E "\._|\.DS_Store"` che non rimangano
tracked.

### D 0-bis.4 — Commit infra Architect nel repo outer
Il repo `/Volumes/Extreme Pro/Claude/testag-grep/.git` ha 0 commit.
**Aggiungere `.gitignore`** nel repo outer:
```gitignore
**/.DS_Store
**/._*
target/
rgrep/target/
```
Poi **stage e commit** dei file Architect (NON modificare `rgrep/`):
```
.gitignore
.vscode/settings.json   (se non sensibile)
CLAUDE.md
EXPERIMENT_PLAN.md
NEXT_STEPS.md
AUDIT_LOG.md
README.md
Cargo.toml
Cargo.lock
minigrep.c
src/main.rs
diary/01-semantic-model.md
diary/02-mapping-table.md
diary/03-translation-log.md
diary/04a-divergences.md
```
Commit message:
```
chore(architect): bootstrap multi-agent workflow infrastructure

IN-SCOPE:
- CLAUDE.md, EXPERIMENT_PLAN.md, NEXT_STEPS.md, AUDIT_LOG.md
- diary/ stubs (01-04)
- Outer repo .gitignore

OUT-OF-SCOPE:
- rgrep/ (separate git repo, owned by Implementer)
- gnu-grep/ snapshot (read-only reference)

Testcase aggiunti: 0. Totali: N/A (no test layer in outer repo).
```
**`gnu-grep/` e `rgrep/` NON vanno aggiunti al repo outer**: il primo è
read-only reference da gestire eventualmente come submodule in futuro,
il secondo è proprietà del workflow Implementer (suo `.git`).

### D 0-bis.5 — Note divergence: oracolo = ugrep
**Aggiungere** un'entry in `diary/04a-divergences.md` (sezione introduttiva,
NON come D-NEW-N — è una nota infrastrutturale):
```markdown
## Nota infra — Oracle dev machine = ugrep 7.5.0

Sul dev machine corrente, `which grep` punta a **ugrep 7.5.0** (third-party
GNU grep clone, generally compatible). Non è BSD grep né GNU grep canonico.
La logica `is_bsd_grep()` in `tests/diff_runner.rs` lo classifica come
"non-GNU" → comportamento equivalente a BSD per skip logic.

Implicazioni:
- Su CI Linux (eventualmente) `grep` sarà GNU grep canonico.
- I testcase con `skip_if_bsd = true` saranno saltati su dev machine ma
  eseguiti su Linux CI (consistente con il piano).
- Eventuali divergenze ugrep-vs-GNU vanno catalogate come D-NEW-N normali.
```

### D 0-bis.6 — File modificati attesi
```
.gitignore                                    (nuovo, repo outer)
rgrep/.gitignore                              (nuovo, repo rgrep)
diary/02-mapping-table.md                     (aggiungere D-G10)
diary/04a-divergences.md                       (aggiungere nota infra)
```
**Nessuna altra modifica al codice in `rgrep/src/` o `rgrep/tests/`.**
Se trovi che il `git rm --cached` di D 0-bis.3 richiede un commit in `rgrep/`
(perché il fix è nel suo .git), fai un secondo commit dedicato lì:
```
chore(step0-bis): add .gitignore and remove AppleDouble blobs
```
Quindi avrai **due commit** per Step 0-bis: uno in `rgrep/.git`, uno in
`testag-grep/.git`. Documenta entrambi gli hash quando aggiorni l'header.

## Acceptance criteria

- ✅ `rgrep/.gitignore` esiste con `target/`, `._*`, `.DS_Store`
- ✅ `cd rgrep && git ls-files | grep -E '\._|\.DS_Store'` ritorna **vuoto**
- ✅ Repo outer `testag-grep/.git` ha **almeno 1 commit** con i file Architect
- ✅ `.gitignore` outer esiste con `._*`, `.DS_Store`, `target/`
- ✅ `diary/02-mapping-table.md` contiene la riga D-G10
- ✅ `diary/04a-divergences.md` contiene la nota "Oracle dev machine = ugrep"
- ✅ `cargo test --manifest-path rgrep/Cargo.toml` ancora **verde**
  (regressione zero: la pulizia non deve rompere nulla)
- ✅ Entrambi i commit (`rgrep/` e outer) seguono il format obbligatorio

## Out-of-scope (debito esplicito)

- Refactor di `test_differential` per produrre 1 `#[test]` per case (cosmetic,
  rinviato a Step 4 o successivi se diventa scomodo)
- `gnu-grep/` come submodule del repo outer (rinviato)
- Configurazione CI cross-platform (rinviato a Step 17)

## Quando finisci

1. `cargo test --manifest-path rgrep/Cargo.toml` verde ✅
2. `git ls-files` in entrambi i repo non contiene `._*` né `.DS_Store`
3. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash> outer:<hash>`
4. **Stop**. Aspetta `audita` dal Decider.

---

# ✅ Step 1 — Phase 1 retroactive: semantic model

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit**: `28743a8` (outer)

## Goal

Phase 1 della skill `legacy-port` applicata retroattivamente: produrre
`diary/01-semantic-model.md` documentando il **modello mentale di GNU grep**
(non l'implementazione riga-per-riga). Output: documento standalone (4-7
pagine) che permetta a un secondo lettore di capire *cosa fa* grep e
*perché* certe decisioni sono state prese, senza dover leggere il C.

Niente codice Rust toccato. Niente test aggiunti.

## D-decisioni (contratto)

### D 1.1 — Scope di lettura
Leggere **solo** i file in `gnu-grep/src/`:
- `grep.c` (3036 LOC, main + dispatch + CLI parsing)
- `grep.h` (header pubblico)
- `dfasearch.c` (599 LOC) + `search.h`
- `kwsearch.c` (238 LOC) — Aho-Corasick / Boyer-Moore per `-F`
- `pcresearch.c` (421 LOC) — wrapper PCRE2 per `-P`
- `searchutils.c` (219 LOC) — helper word-boundary, case-fold

**NON leggere**: `gnu-grep/lib/`, `gnu-grep/gnulib/`, `gnu-grep/m4/`,
`gnu-grep/po/`, `gnu-grep/doc/`, `gnu-grep/tests/`. Out of scope.

### D 1.2 — Struttura obbligatoria del documento
Sostituire integralmente `diary/01-semantic-model.md` con queste 7 sezioni:

```
§1 API surface CLI               (tabella: flag groups → flag list → grep.c:NNN)
§2 Architettura del matching engine  (dispatch DFA / KWS / PCRE)
§3 Modello dati interno          (struct portanti semantica)
§4 Strategia memory management   (matchers cache, I/O buffer, mmap)
§5 Encoding e locale             (byte vs multibyte, BOM, word boundary)
§6 Edge cases (tabella ≥8 righe) (empty pattern, NUL, binary, symlink loop, ...)
§7 Exit codes                    (0/1/2)
```

### D 1.3 — Citazioni obbligatorie
**Ogni invariante o edge case** in §2/§4/§6 deve avere `file.c:linea` o
`:linea-linea`. Senza ≥15 citazioni distribuite, audit flagga 🟡 PARTIAL.

### D 1.4 — Lunghezza target
**4-7 pagine markdown** (~2000-4000 parole). Sotto 3 pagine = §6 sotto-pensato.
Sopra 7 = stai descrivendo l'implementazione invece del modello.

### D 1.5 — Lingua
Italiano. Codice citato e nomi di funzione/variabile in inglese inline.

### D 1.6 — Niente più del modello
Vietato in questo step:
- Suggerire mappature Rust (è Step 2)
- Scrivere codice Rust di esempio
- Toccare qualsiasi file fuori da `diary/01-semantic-model.md`

Annotazioni utili per Phase 2 → bullet in fondo sotto `## Note per Phase 2`,
massimo 10 bullet, niente codice.

### D 1.7 — Criterio di successo (autovalutazione)
Prima di committare, rileggi il documento e chiediti:
> "Un secondo lettore che NON conosce grep, leggendo solo questo file,
> capisce **come funziona** la pipeline matcher e **perché** la binary
> detection non legge l'intero file?"

Se sì → commit. Se no → espandi le sezioni manchevoli.

### D 1.8 — File modificati attesi
```
diary/01-semantic-model.md     (sovrascritto integralmente)
```

## Acceptance criteria

- ✅ `diary/01-semantic-model.md` sostituito con le 7 sezioni di D 1.2
- ✅ Almeno **15 citazioni** `file.c:line` distribuite tra §2/§4/§6
  (verifica: `grep -oE '[a-z_]+\.c:[0-9]+' diary/01-semantic-model.md | wc -l`)
- ✅ Tabella §6 edge cases con almeno **8 righe**
- ✅ Lunghezza tra 4-7 pagine markdown
- ✅ Nessuna mappatura Rust né codice di esempio
- ✅ Commit nel repo outer `testag-grep/.git` (NON in `rgrep/.git`)
- ✅ Commit message segue il format obbligatorio

## Out-of-scope (debito esplicito)

- Mappatura idiomatica Rust → Step 2
- Lettura di `gnu-grep/lib/`, `gnu-grep/gnulib/` → mai (skill: scope-out)
- Test → Step 16
- Codice rgrep → da Step 3 in poi

## Commit atteso

Un solo commit nel repo outer `testag-grep/.git`:
```
docs(step1): phase 1 semantic model of GNU grep

IN-SCOPE:
- diary/01-semantic-model.md: 7-section semantic model with N citations
- §6 edge cases table with N entries

OUT-OF-SCOPE (debito esplicito):
- Rust mapping decisions (Step 2)
- gnulib/lib portability layer (out of skill scope)
- No rgrep/ files touched

Testcase aggiunti: 0. Totali: 5.
```

## Quando finisci

1. Rileggi il documento contro D 1.7 (criterio di successo)
2. Conta citazioni: `grep -oE '[a-z_]+\.c:[0-9]+' diary/01-semantic-model.md | wc -l` ≥ 15
3. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit <hash>`
4. **Stop**. Aspetta `audita` dal Decider.

---

# ✅ Step 2 — Phase 2 retroactive: idiomatic mapping table

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit**: `27af1f4` (outer)

## Goal

Phase 2 della skill `legacy-port`: produrre `diary/02-mapping-table.md`
formalizzando **come ogni costrutto C di GNU grep si traduce in Rust**.
Output: documento standalone con D-decisioni numerate (D-G1..D-Gn globali +
D-Mxx per modulo) che diventeranno il **contratto** per gli step di
traduzione/audit (Step 3..15).

L'input è il `diary/01-semantic-model.md` di Step 1 + l'esistente
`rgrep/src/` (che è già una traduzione in essere — qui la formalizziamo
retroattivamente). Niente codice modificato, solo la mapping table.

## D-decisioni (contratto)

### D 2.1 — Sostituire integralmente `diary/02-mapping-table.md`
Il file attuale ha solo D-G10 + uno stub. Va riscritto da zero. Tutte le
decisioni esistenti (D-G1..D-G10) vanno **rivalidate e razionalizzate**,
non solo copiate.

### D 2.2 — Struttura obbligatoria del documento

```
§1 Decisioni globali (D-G1..D-Gn)
   Tabella: ID | Costrutto C | Scelta Rust | Razionale | Status
   Almeno 12 decisioni globali (espandere D-G1..D-G10 + nuove).

§2 Decisioni per modulo (D-Mxx)
   Una sotto-sezione per ognuno dei 5 file C originali:
   §2.1 grep.c       (orchestration → cli.rs + runner.rs)
   §2.2 dfasearch.c  (DFA → regex crate adapter)
   §2.3 kwsearch.c   (KW → aho-corasick adapter)
   §2.4 pcresearch.c (PCRE → pcre2 crate, feature-gated)
   §2.5 searchutils.c (helpers → in-house Rust)
   Per ogni modulo: strategia (A-F dalla skill), LOC C → LOC Rust target,
   D-decisioni specifiche.

§3 Tabella di traduzione costrutti
   Tabella riga-per-riga: ogni costrutto C menzionato in 01-semantic-model.md
   → mappa Rust + ID decisione.
   Almeno 25 righe.
   Esempi di costrutti da coprire:
   - `getopt_long` → clap derive
   - `fillbuf + grepbuf` → `BufRead::read_until`
   - `struct dfa_comp` → `regex::Regex`
   - `struct kwsearch` → `aho_corasick::AhoCorasick`
   - `EXIT_TROUBLE` → `std::process::ExitCode` o `enum ExitStatus`
   - `fts_read` → `walkdir::WalkDir`
   - `mb_clen / mb_goback` → byte-oriented + `regex` crate Unicode mode
   - `buf_has_encoding_errors` → in-house heuristic NUL-byte
   - `eolbyte` (default `\n`, `-z` → 0) → `delimiter: u8` config
   - ...

§4 Sketch API pubblica Rust
   Solo firme di funzione/struct, niente body. La `Config` esistente in
   `rgrep/src/cli.rs` è OK come riferimento ma vanno definite le firme
   target di `runner` e `matcher`. Esempio:
   ```rust
   pub struct Matcher { ... }
   impl Matcher {
       pub fn compile(patterns: &[String], opts: &MatchOptions) -> Result<Self, MatchError>;
       pub fn is_match(&self, line: &[u8]) -> bool;
       pub fn find(&self, line: &[u8]) -> Option<Range<usize>>;
   }
   pub fn search_reader<R: BufRead, W: Write>(
       reader: R, writer: W, matcher: &Matcher, opts: &OutputOptions
   ) -> Result<MatchStats, GrepError>;
   ```
   Niente più di ~30 righe di sketch totali.

§5 Cose esplicitamente NON portate (scope-out)
   Lista numerata, ognuna con razionale di una riga.
   Coprire almeno: GREP_OPTIONS deprecato, --rpm-* flags, gnulib portability
   layer, locale handling C-style, fallback chain interno DFA→Posix
   (sostituito da regex crate JIT).

§6 Punti di review per il Decider (umano)
   Lista bulleted di decisioni con trade-off non banali da validare prima
   di Step 3. Esempi:
   - "Usare `walkdir` o `ignore`?" — già risolto in D-G2 ma confermare
   - "Scope di `-P`: fallback compile-time error o runtime error?" — D 2.x
   - "Color env `GREP_COLORS` parsing: ad-hoc o crate?"
   Almeno 3 bullet, max 8.
```

### D 2.3 — Espandere e rivalidare D-G1..D-G10 esistenti
Le decisioni globali pre-esistenti devono essere **riformulate** in formato
contratto: ognuna con razionale completo (non solo "perché è meglio") + status
`confermato | da-validare | sperimentale`.

Aggiungere almeno **2 nuove D-G** non già presenti, ad esempio:
- D-G11: error type unificato (`thiserror::Error` con varianti per IO/Regex/Glob/PatternFile)
- D-G12: `eolbyte` (delimiter) come `u8` config (`b'\n'` default, `0` per `-z`)
- D-G13: fallback DFA→Posix non riprodotto (regex crate ha già il dispatch interno)
- D-G14: case-insensitivity = `(?i)` flag inline in pattern (no locale-aware fold)

(quante e quali aggiungere è judgment dell'Implementer — minimo 2, target ≥ 12 totali)

### D 2.4 — Per ogni gruppo di flag in §1 di 01-semantic-model.md, una decisione
La tabella §1 di Step 1 elenca 7 gruppi (Pattern Selection, Pattern Source,
Matching Control, Output Control, Context Control, File Traversal, Binary).
Per ognuno, almeno **una D-decisione** (per modulo o globale) che dica come
quel gruppo si mappa.

### D 2.5 — Numero target di decisioni
- D-Gn globali: **≥ 12** (pre-esistenti D-G1..D-G10 + 2 nuove minimo)
- D-Mxx per modulo: **≥ 15** distribuite tra i 5 sotto-§2
- Tabella §3 traduzione costrutti: **≥ 25 righe**

Skill dice 50-80 per librerie ~3K LOC. Per grep target ~25 + 12 = 37+ ≈
ragionevole (grep è più piccolo del riferimento parser/serializer).

### D 2.6 — Niente codice Rust di esempio salvo sketch
Vietato:
- Implementazioni di funzione (anche brevi)
- Test code
- Codice di rgrep modificato

Permesso:
- Sketch firme in §4 (≤30 righe)
- Snippet enum/struct definitions inline nelle D-decisioni (≤5 righe per snippet)

### D 2.7 — Lingua
Italiano. Nomi di crate, type, funzioni e attributi clap restano in inglese
inline.

### D 2.8 — File modificati attesi
```
diary/02-mapping-table.md     (sovrascritto integralmente)
```
**Nessun altro file toccato.** Niente in `rgrep/`, `gnu-grep/`, altri diary/.

### D 2.9 — Riferimenti cross-doc
Ogni D-decisione che ha origine da un edge case di Step 1 deve citare la
sezione: es. `(da 01-semantic-model.md §6 "Symlink Recursion Loop")`. Almeno
**5 cross-reference** verso 01-semantic-model.md.

## Acceptance criteria

- ✅ `diary/02-mapping-table.md` riscritto con le 6 sezioni di D 2.2
- ✅ Almeno **12 D-G globali** numerate (D-G1..D-G12+) con razionale + status
- ✅ Almeno **15 D-M decisioni per modulo** distribuite tra §2.1..§2.5
- ✅ Tabella §3 traduzione costrutti con **≥25 righe**
- ✅ §4 sketch API Rust ≤30 righe, solo firme
- ✅ §5 ≥5 voci scope-out con razionale
- ✅ §6 ≥3 punti di review per il Decider
- ✅ ≥5 cross-reference verso `01-semantic-model.md` (es. `§6 ...`)
- ✅ Commit nel repo outer `testag-grep/.git`
- ✅ Commit message segue il format obbligatorio

## Out-of-scope (debito esplicito)

- Implementazione delle decisioni → da Step 3 in poi
- Refactor di `rgrep/src/` per allinearlo alle decisioni → step successivi
- Test → Step 16
- Differential testing oltre la baseline → Step 16

## Commit atteso

Un solo commit nel repo outer `testag-grep/.git`:
```
docs(step2): phase 2 idiomatic mapping table for GNU grep → Rust

IN-SCOPE:
- diary/02-mapping-table.md: 6 sections, N D-G globali, M D-M per modulo
- §3 tabella traduzione costrutti con K righe
- §6 review points pendenti per il Decider

OUT-OF-SCOPE (debito esplicito):
- Implementation work (Steps 3-15)
- Refactor rgrep/src/ existing code
- No rgrep/ files touched

Testcase aggiunti: 0. Totali: 5.
```

## Quando finisci

1. Conta D-G globali: deve essere ≥12
2. Conta D-M decisioni per modulo: deve essere ≥15
3. Conta righe tabella §3: `awk '/^## §3/,/^## §4/' diary/02-mapping-table.md | grep -c '^|'` ≥ 27 (25 dati + 2 header)
4. Conta cross-reference: `grep -c '01-semantic-model' diary/02-mapping-table.md` ≥ 5
5. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit <hash>`
6. **Stop**. Aspetta `audita` dal Decider.

---

# 🟡 Step 3 — Regex engine flavors: -E, -G, -F (PARTIAL)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `1094344` — **Commit outer**: `6df436b`
**Leftover**: vedi Step 3-bis sotto.

## Goal

Primo step di **traduzione vera** (Phase 3 della skill): allineare
`rgrep/src/matcher.rs` alle decisioni di Step 2 per i 3 flavor di regex
fondamentali. `-E` deve essere il default coerente, `-G` deve tradurre
BRE in ERE prima di compilare con `regex` crate, `-F` deve usare
`aho-corasick` invece del fallback regex per match literali multi-pattern.

Verificare exit code 2 su pattern invalido (parity con GNU grep).
TDD-first: aggiungere prima i 5 nuovi testcase nel manifest, verificare
che falliscano per il motivo atteso, poi implementare.

## Riferimenti contratto

- D-G1 (regex crate)
- D-G8 (BRE → ERE pre-processor)
- D-G6 (exit codes via thiserror enum)
- D-M4, D-M5, D-M6 (dfasearch.c → regex crate adapter)
- D-M7, D-M8, D-M9 (kwsearch.c → aho-corasick)

## D-decisioni (contratto, non riapribile in questo step)

### D 3.1 — `-E` (Extended) è il default
GNU grep ha `-G` (BRE) come default storico, ma la moda corrente e la
scelta di rgrep (D-G1 + Phase 1 §1) è ERE. **rgrep usa `-E` come default**:
se nessuno tra `-E`, `-G`, `-F`, `-P` è specificato → comportamento ERE.
Documentare la divergenza da GNU come **D-NEW-1** in `04a-divergences.md`
(severità: parity / scelta_design).

### D 3.2 — `-F` implementation = `aho-corasick`
Aggiungere `aho-corasick = "1.1"` come dipendenza in `rgrep/Cargo.toml`.
Quando `-F` è attivo, il pattern (e i pattern multipli da `-e` o `-f`,
quando arriveranno in Step 4) sono trattati come **stringhe literali**:
- Split su `\n` per supportare `-F "foo\nbar"` (multi-pattern in un singolo arg)
- Costruire `aho_corasick::AhoCorasickBuilder::new().build(patterns)`
- Su match: come per `regex::Regex::find`

Edge case: se è anche attivo `-w` (word match), wrappare con regex `\b...\b`
e fallback a `regex` crate (D-M9). In Step 3 implementare il fallback solo
se i testcase lo richiedono — altrimenti scope-out a Step 5+ esplicito.

### D 3.3 — `-G` (BRE) → ERE translation layer
Implementare in `matcher.rs` (o nuovo module `bre_to_ere.rs` se >50 LOC) una
funzione:
```rust
fn bre_to_ere(pattern: &str) -> String;
```

**Algoritmo (semplificato, parity GNU)**:
- In BRE i metacaratteri `?+(){}|` sono **literals**; `\?\+\(\)\{\}\|` sono
  metacaratteri.
- In ERE è il contrario.
- Translation: scambiare il significato di backslash di questi 7 caratteri.
- Mantenere intatti: `.`, `*`, `^`, `$`, `[...]`, `\1..\9` (backref), `\b`,
  `\<`, `\>`.

Esempi obbligatori (vanno in unit test):
- `foo\?` (BRE) → `foo?` (ERE) — match optional `foo`
- `foo?` (BRE) → `foo\?` (ERE) — match literal `foo?`
- `\(a\|b\)` (BRE) → `(a|b)` (ERE)
- `(a|b)` (BRE) → `\(a\|b\)` (ERE)
- `^foo$` (BRE) → `^foo$` (ERE) — anchor invariato

**OUT-OF-SCOPE in Step 3**:
- Character class `[[:alpha:]]` translation (parity ASCII OK, multi-byte
  rinviato a step futuro)
- Bound `\{n,m\}` (BRE) → `{n,m}` (ERE) — implementare se semplice, altrimenti
  nota DESIGN-Q.

### D 3.4 — `-P` resta stub
**Non implementare** PCRE in questo step. Comportamento atteso:
- Se `-P` è specificato, rgrep esce con exit code 2 e stderr:
  `rgrep: -P only supported when compiled with --features perl-regexp`
- Step 15 implementerà la feature reale.

Aggiungere 1 testcase `0011_perl_regexp_unsupported.toml` con
`expected_exit_code = 2` e `skip_if_bsd = true` (BSD grep ha messaging
diverso).

### D 3.5 — Last flavor flag wins
Se l'utente passa più di un flavor (es. `-E -G`), **l'ultimo specificato vince**
(parity GNU grep). Implementazione: clap derive con `overrides_with_all` o
post-parsing che sceglie il flavor in base all'ordine di apparizione in argv.

Verificare con un testcase: `-G -E "foo|bar"` → ERE → match `foo` e `bar`.

### D 3.6 — Pattern invalido → exit code 2
Quando il pattern non compila (sia in regex che aho-corasick), rgrep deve:
- Stampare errore su stderr (formato consigliato:
  `rgrep: <regex error message>`)
- Exit code 2 (NON 1, che è "no match")

Implementazione: `Result<Matcher, GrepError>` propagato fino a `main()`,
match esplicito che fa `process::exit(2)`.

### D 3.7 — Testcase obbligatori (TDD-first)
Aggiungere **prima** dell'implementazione:

| ID | Args | Stdin | Expected stdout | Exit | Note |
|---|---|---|---|---|---|
| 0006 | `["-E", "foo\|bar"]` | `foo\nbaz\nbar\nfoobar\n` | `foo\nbar\nfoobar\n` | 0 | ERE alternation |
| 0007 | `["-G", "foo\\?"]` | `foo?\nfoo\nfo\n` | `foo?\nfoo\nfo\n` | 0 | BRE: `\?` = optional |
| 0008 | `["-F", "a.b"]` | `a.b\naxb\n` | `a.b\n` | 0 | F = literal string, not regex |
| 0009 | `["-E", "["]` | `irrelevant\n` | `` (empty) | 2 | invalid regex → exit 2 |
| 0010 | `["-Fw", "foo"]` | `foo\nfoobar\nbar\n` | `foo\n` | 0 | -F + -w = whole word literal |
| 0011 | `["-P", "foo"]` | `foo\n` | `` (empty) | 2 | -P stub error (skip_if_bsd) |

**Note testcase 0007**: in TOML il pattern `foo\\?` rappresenta la stringa
literale `foo\?`. Verificare che il TOML deserializzato passi `foo\?` come
arg al binario.

**Note testcase 0009**: alcuni `grep` di sistema potrebbero non restituire
exit 2 ma 1 su `[`. Su ugrep dev machine: probabilmente exit 2. Se diverge
dall'oracolo, marcare `skip_if_bsd = true` con motivo.

### D 3.8 — File modificati attesi
```
rgrep/Cargo.toml                       (+ aho-corasick = "1.1")
rgrep/src/matcher.rs                   (refactor + BRE→ERE + AhoCorasick branch)
rgrep/src/cli.rs                       (eventuale: overrides_with_all per flavor flags)
rgrep/src/runner.rs                    (eventuale: error propagation Result<>)
rgrep/src/main.rs                      (eventuale: exit code 2 mapping)
rgrep/tests/testsuite.toml             (+6 entries)
rgrep/tests/cases/0006_extended_regex.toml
rgrep/tests/cases/0007_basic_regex_bre.toml
rgrep/tests/cases/0008_fixed_string.toml
rgrep/tests/cases/0009_invalid_regex_exit2.toml
rgrep/tests/cases/0010_fixed_string_word.toml
rgrep/tests/cases/0011_perl_regexp_unsupported.toml
```

Se aggiungi `bre_to_ere.rs` come file separato, aggiungilo all'elenco.
Documenta in commit `IN-SCOPE`.

### D 3.9 — Build precondition (invariata)
- `cargo build` 0 warning di codice
- `cargo test` verde con i nuovi 6 testcase
- I 5 testcase di Step 0 devono **continuare a passare** (regressione zero)

### D 3.10 — Unit test per `bre_to_ere`
Oltre ai differential testcase, aggiungere unit test interni in
`matcher.rs` (o `bre_to_ere.rs`) per i 5 esempi obbligatori della D 3.3.
Almeno **5 unit test**. Eseguibili con `cargo test --lib`.

## Acceptance criteria

- ✅ `cargo build` 0 warning, `cargo test` verde (5 step-0 + 6 step-3 = **11 differential** + ≥5 unit)
- ✅ I 6 nuovi testcase passano (eventualmente con `skip_if_bsd`)
- ✅ `aho-corasick` aggiunta in `rgrep/Cargo.toml`
- ✅ Funzione `bre_to_ere` con almeno 5 unit test
- ✅ Exit code 2 verificato su pattern invalido (testcase 0009)
- ✅ `-P` stub error verificato (testcase 0011)
- ✅ Aggiunta entry **D-NEW-1** in `diary/04a-divergences.md` per il default ERE
- ✅ Aggiunta entry in `diary/03-translation-log.md` per la traduzione di
  `dfasearch.c` (D-M4..6) e `kwsearch.c` (D-M7..9), schema standard
- ✅ Commit nel repo `rgrep/.git`. **Anche** un secondo commit nel repo
  outer per i diary updates (03 + 04). Due commit totali:
  - rgrep/: `feat(step3): regex engine flavors -E, -G, -F`
  - outer/: `docs(step3): translation log + D-NEW-1 default ERE`

## Out-of-scope (debito esplicito)

- `-P` reale (Step 15)
- `-e` multipli e `-f` file (Step 4)
- BRE character class `[[:alpha:]]` Unicode-aware (rinviato)
- Performance benchmark vs GNU grep (mai in questo esperimento)
- Refactor di `runner.rs` oltre il minimo necessario per error propagation

## Note di implementazione

- L'esistente `rgrep/src/matcher.rs` (203 LOC pre-experiment) probabilmente
  ha già un dispatch per i flavor. **NON riscrivere da zero**: estendere e
  adattare. Se trovi che il codice esistente è incompatibile con D 3.x,
  lascia `// DESIGN-Q: ...` e tagga `🟡 BLOCKED — DESIGN-Q`.
- `regex` crate non supporta `\<` `\>` (word anchor GNU). Mappare a `\b`
  con check posizionale è OK. Se diverge dall'oracolo, documentare come
  D-NEW-2 in 04a-divergences.md.

## Commit attesi

**Due commit** (uno per repo, come da convenzione D 0.8 + D 0-bis.4):

In `rgrep/.git`:
```
feat(step3): regex engine flavors -E, -G, -F

IN-SCOPE:
- aho-corasick dependency for -F
- bre_to_ere() translation function with 5 unit tests
- -P stub error with exit 2
- last-flavor-wins logic for conflicting -E/-G/-F flags
- 6 differential testcases (0006-0011)

OUT-OF-SCOPE (debito esplicito):
- -P real implementation (Step 15)
- -e/-f multi-pattern (Step 4)
- POSIX character class Unicode mode

Testcase aggiunti: 6. Totali: 11.
```

In `testag-grep/.git`:
```
docs(step3): translation log + D-NEW-1 default ERE divergence

IN-SCOPE:
- diary/03-translation-log.md: 2 entries (dfasearch + kwsearch)
- diary/04a-divergences.md: D-NEW-1 (rgrep default = ERE, GNU = BRE)

OUT-OF-SCOPE:
- No code changes

Testcase aggiunti: 0. Totali: 11.
```

## Quando finisci

1. `cargo test --manifest-path rgrep/Cargo.toml` verde con 11 differential + ≥5 unit
2. Verifica con `grep -c '0006\|0007\|0008\|0009\|0010\|0011' rgrep/tests/testsuite.toml` ≥ 6
3. Verifica `diary/04a-divergences.md` contiene `D-NEW-1`
4. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash> outer:<hash>`
5. **Stop**. Aspetta `audita` dal Decider.

---

# ✅ Step 3-bis — Cleanup leftover di Step 3

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `fc044cd` — **Commit outer**: `96244cf`

## Goal

Chiudere i leftover dell'audit di Step 3 (verdict 🟡 PARTIAL, vedi
[AUDIT_LOG.md](AUDIT_LOG.md)). Niente nuove feature, niente nuovi testcase
di matching. Solo cleanup repo e documentazione del bug latente exit-code.

## Contesto

Audit Step 3 ha approvato la sostanza (11/11 differential + 9 unit verdi,
D 3.1..3.10 tutti applicati) ma rilevato:
1. `tests/debug_cli.rs` scratchpad accidentalmente committato in `1094344`
2. Bug latente exit-code (no-match=0 invece di 1, file-not-found=0 invece di 2)
   pre-esistente, da documentare come D-NEW-2 ma fix differito a Step 10.
3. Commit message body malformato in `1094344` — first-offense warning,
   nessuna remediation invasiva richiesta.
4. Mode bits 644→755 (filesystem noise) — tollerato.

## D-decisioni (contratto)

### D 3-bis.1 — Rimuovere `tests/debug_cli.rs`
File scratchpad debug con `fn main()`, non un vero test. Non rompe nulla
(cargo test esegue il binario, 0 test rilevati) ma è rumore.

```bash
cd rgrep
git rm tests/debug_cli.rs
```

Verificare con `cargo test` post-rimozione: deve restare verde con i 4
test suite originali (`debug_cli` deve sparire dall'output di `cargo test`,
mentre `diff_runner`, lib-tests, e doctest devono ancora girare).

### D 3-bis.2 — Documentare D-NEW-2 in `04a-divergences.md`
Aggiungere in `diary/04a-divergences.md` (sotto la sezione "Divergenze
intenzionali", o creare "Divergenze latenti / bug pre-esistenti" se serve):

```markdown
### D-NEW-2: Exit code semantics non conforme a GNU grep

- **Severità**: output_diverso / critico per shell pipelines
- **Categoria**: edge_spec
- **Causa**: bug_originale rgrep (pre-experiment)
- **Discovered in**: audit Step 3 (live exec)
- **Sintomo**:
  ```
  $ printf 'foo\n' | rgrep 'baz'      → exit 0 (atteso 1: no match)
  $ rgrep foo /nonexistent             → exit 0 (atteso 2: file error)
  ```
- **Atteso (GNU grep)**:
  - exit 0 = match found
  - exit 1 = no match
  - exit 2 = error (regex syntax, file not found, ...)
- **Stato attuale rgrep**:
  - exit 0 = sempre, salvo pattern syntax error → exit 2 (D 3.6)
- **Decisione**:
  - [x] documentato come D-NEW-2 (questo step)
  - [ ] fix in Step 10 (`-q`, `-s`, semantica exit code)
- **Status**: aperto, fix pianificato
```

Razionale: il bug è critico per qualsiasi script shell che usa rgrep in
pipeline (`if rgrep -q PATTERN file`). Va sistemato prima di Step 16
(differential proptest) altrimenti quel step esploderà di
`OnlyRgrepOk` outcomes.

### D 3-bis.3 — Eseguire `chmod 644` su file impattati da exFAT noise (opzionale)
```bash
cd rgrep
chmod 644 README.md src/lib.rs src/runner.rs C_TO_RUST_MIGRATION_SKILL.md 2>/dev/null || true
git add -u
# se git rileva i mode change come differenze, committali insieme a D 3-bis.1
```

Se `chmod 644` non ha effetto (filesystem exFAT non onora i mode bits),
documentare in commit message: `Note: chmod 644 ineffective on exFAT,
mode bits will continue to oscillate. Tolerated.`

Questa D è OPZIONALE. Se troppo macchinoso, skip esplicito nel commit
con motivazione.

### D 3-bis.4 — Commit format obbligatorio (lesson learned, no remediation)
**Niente** da fare per il commit `1094344` malformato (no amend, no
revert — è water under the bridge). Ma il commit di Step 3-bis e tutti
i futuri **devono** seguire il format obbligatorio integralmente:
```
<type>(stepN-bis): <titolo>

IN-SCOPE:
- <bullets>

OUT-OF-SCOPE (debito esplicito):
- <bullets>

Testcase aggiunti: N. Totali: M.
```
Ripeti questo formato a memoria. È contratto.

### D 3-bis.5 — File modificati attesi
```
rgrep/tests/debug_cli.rs              (RIMOSSO)
diary/04a-divergences.md               (aggiungere D-NEW-2)
```
Eventualmente: mode bits su 4 file rgrep (se chmod ha effetto).

**Niente codice rgrep/src/ toccato.** Niente nuovi testcase.

## Acceptance criteria

- ✅ `tests/debug_cli.rs` rimosso (`git ls-files tests/` non lo lista)
- ✅ `cargo test --manifest-path rgrep/Cargo.toml` ancora **verde** (regressione zero)
- ✅ `diary/04a-divergences.md` contiene `D-NEW-2`
- ✅ Due commit (uno per repo) **entrambi** con format obbligatorio completo:
  - rgrep/.git: `chore(step3-bis): remove debug scratchpad`
  - testag-grep/.git: `docs(step3-bis): document D-NEW-2 exit code latent bug`

## Out-of-scope (debito esplicito)

- Fix del bug exit-code (rinviato a Step 10)
- Refactor di `tests/debug_cli.rs` in un vero `#[test]` (rinviato/scartato)
- Modifiche al codice rgrep/src/

## Quando finisci

1. `cargo test` verde
2. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash> outer:<hash>`
3. **Stop**. Aspetta `audita` dal Decider.

---

# ✅ Step 4 — Pattern sources: -e (multi), -f (file)

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `2d78e76` — **Commit outer**: `8eb3b8a`

## Goal

Implementare le sorgenti pattern multiple di GNU grep:
- `-e PATTERN` ripetibile (es. `-e foo -e bar` = OR di entrambi)
- `-f FILE` legge pattern dal FILE (uno per riga)
- Combinazioni: `-e + -f`, `-e + positional`, `-f + positional`

Si presume che `cli.rs` abbia già `pub regexp: Vec<String>` e
`pub file_patterns: Vec<String>` (presenti nella baseline pre-experiment).
Lo step **wires up** la logica: legge i pattern da `-f`, li unisce ai `-e`,
costruisce un Matcher OR-joined.

Include un'estensione del **harness di test** per supportare fixture file
(necessari per testare `-f`).

## Riferimenti contratto

- D-G1 (regex crate)
- D-G6 (thiserror::Error → exit code mapping)
- D-M1 (Pattern Selection group)
- D-M8 (multi-pattern aho-corasick per -F + -e)

## D-decisioni (contratto)

### D 4.1 — `-e` accumula, OR-joined
Multipli `-e PAT` accumulano. La compilazione del matcher fa:
- Per `-E`/`-G`: alternation regex `(?:pat1)|(?:pat2)|...`. Ogni pattern
  parenthesizzato per evitare problemi di precedenza.
- Per `-F`: passa il vettore di pattern direttamente a
  `AhoCorasickBuilder::new().build(patterns)`.
- Per `-P`: stub, errore (Step 15 reale).

### D 4.2 — `-f FILE` legge pattern line-by-line
Implementazione in `runner.rs` (o nuovo helper `pattern_loader.rs` se >50 LOC):
- Apri FILE con `BufRead::read_until(b'\n', ..)` (D-G4 + D-G12)
- Ogni linea (senza il trailing `\n`) è un pattern
- Linee vuote sono ammesse → vedi D 4.3
- Concatena i pattern con quelli da `-e` (ordine: prima `-e`, poi `-f`,
  parity GNU? Verificare con oracolo)

### D 4.3 — Pattern vuoto matcha ogni linea
`-e ""` o linea vuota in `-f FILE` → pattern vuoto. Behavior:
- Per regex: regex `""` matcha posizione zero su ogni linea → match every line
- Per `-F`: stringa literale vuota → matcha ogni linea (semantica empty-substring)

Verifica con testcase 0015. Cita 01-semantic-model.md §6 "Empty Pattern Fallback"
(`pcresearch.c:257`).

### D 4.4 — `-f FILE` inesistente → exit 2
Stesso path errore di "file not found" generale (CLI failure):
- runner ritorna `Err(GrepError::PatternFileError(path, io::Error))`
- main() exit 2

Testcase 0016 verifica.

### D 4.5 — Combinazione `-e` + `-f` + positional
Logica di parsing args (parity GNU):
- Se almeno un `-e` o `-f` è dato → il primo positional NON è pattern,
  è file. Tutti i positional sono file.
- Altrimenti → il primo positional è pattern, resto sono file.

Esempio:
- `rgrep foo file.txt` → pattern="foo", file="file.txt"
- `rgrep -e foo file.txt` → pattern="foo" (da -e), file="file.txt"
- `rgrep -e foo -e bar file.txt file2.txt` → patterns=[foo,bar], files=[file.txt, file2.txt]

Implementazione: in `runner` (o helper), check `if !cli.regexp.is_empty() ||
!cli.file_patterns.is_empty() { use cli.pattern as file too } else { ... }`.

### D 4.6 — Harness extension: fixture_files
Estendere `TestCase` TOML con campo opzionale:
```toml
fixture_files = [
    { name = "patfile.txt", content = "foo\nbar\n" },
    # multiple files supported
]
```

**Comportamento harness**:
- Per ogni testcase, se `fixture_files` presente:
  - Crea directory temp unica: `target/test-fixtures/<testcase_name>/`
  - Scrivi ogni file con il content specificato
  - Sostituisci nei `args` il placeholder `{FIXTURES}` con il path assoluto della temp dir
  - Esegui rgrep e oracle con args sostituiti
  - Cleanup post-test: `fs::remove_dir_all(temp_dir)` (anche se test fallisce)

Esempio `args`:
```toml
args = ["-f", "{FIXTURES}/patfile.txt"]
```

### D 4.7 — Testcase obbligatori (TDD-first)

| ID | Args | Stdin / Fixtures | Expected | Note |
|---|---|---|---|---|
| 0012 | `["-e", "foo", "-e", "bar"]` | `foo\nbaz\nbar\nfoobar\n` | `foo\nbar\nfoobar\n` | OR multi-`-e` |
| 0013 | `["-f", "{FIXTURES}/p.txt"]` | stdin: `foo\nbaz\nbar\n` <br>fixture `p.txt`: `foo\nbar\n` | `foo\nbar\n` | -f base |
| 0014 | `["-e", "foo", "-f", "{FIXTURES}/p.txt"]` | stdin: `foo\nbaz\nbar\nbaz\nqux\n` <br>fixture `p.txt`: `bar\nqux\n` | `foo\nbar\nqux\n` | combinazione -e + -f |
| 0015 | `["-e", ""]` | `foo\nbar\n` | `foo\nbar\n` | empty pattern matcha tutto |
| 0016 | `["-f", "/nonexistent/patfile"]` | irrelevant | `` (empty) exit 2 | -f file non esiste → exit 2 |

Note 0016: ugrep e GNU grep entrambi exit 2 su `-f` con file inesistente.
Probabilmente non serve `skip_if_bsd`. Verificare; se ugrep diverge,
marcare skip + motivo.

### D 4.8 — File modificati attesi
```
rgrep/src/cli.rs                       (eventuale: tweak Vec<String> defaults)
rgrep/src/matcher.rs                   (compile multi-pattern: alternation o aho-corasick array)
rgrep/src/runner.rs                    (read pattern files, merge -e/-f, positional logic D 4.5)
rgrep/tests/diff_runner.rs             (extend TOML format: fixture_files + {FIXTURES} placeholder)
rgrep/tests/cases/0012_multiple_e.toml
rgrep/tests/cases/0013_pattern_file.toml
rgrep/tests/cases/0014_combined_e_f.toml
rgrep/tests/cases/0015_empty_pattern.toml
rgrep/tests/cases/0016_pattern_file_missing.toml
rgrep/tests/testsuite.toml             (+5 entries)
```

Se introduci helper file (es. `src/pattern_loader.rs`), aggiungilo all'elenco
e documenta in commit `IN-SCOPE`.

### D 4.9 — Build precondition (invariata)
- `cargo build` 0 warning di codice
- `cargo test` verde con 16 differential testcase + ≥9 unit (regressione zero)
- I testcase pre-esistenti (0001-0011) **devono continuare a passare**

### D 4.10 — Unit test per pattern loading
Aggiungere unit test in `runner.rs` (o nuovo helper) per:
- Empty pattern file → 0 patterns loaded
- Pattern file con linee vuote miste → linee vuote diventano empty patterns (preservate)
- Pattern file con CRLF → strip `\r\n` correttamente

Almeno **3 unit test** nuovi.

### D 4.11 — Note implementazione
- Non ricostruire il regex se è già compilato (cache nel Matcher)
- `-F + -e + -f` significa: tutti i pattern vanno in AhoCorasick array, non
  c'è alternation regex (D-M8)
- L'ordine pattern (prima `-e`, poi `-f`, o contrario?) di GNU: confermato che
  GNU grep concatena nell'ordine di apparizione in argv. Approssimazione
  accettabile: `-e` prima, `-f` poi (l'output è un set di linee match, non
  importa l'ordine se non per `-o` highlighting). Se i testcase rivelano
  divergenza, aprire DESIGN-Q.

## Acceptance criteria

- ✅ `cargo build` 0 warning, `cargo test` verde (16 differential + ≥12 unit)
- ✅ I 5 nuovi testcase passano (eventualmente con `skip_if_bsd`)
- ✅ Harness supporta `fixture_files` + `{FIXTURES}` placeholder
- ✅ Funzione di pattern loading con ≥3 unit test
- ✅ Aggiunta entry in `diary/03-translation-log.md` per `grep.c` pattern parsing logic (D-M1 espanso con multi-pattern semantics)
- ✅ Nessun nuovo D-NEW-* introdotto (se compaiono divergenze, documentare prima di committare)
- ✅ Commit nel repo `rgrep/.git` + commit nel repo outer per `03-translation-log.md` update
- ✅ **Entrambi** i commit hanno il format obbligatorio integrale (lesson learned Step 3-bis)

## Out-of-scope (debito esplicito)

- `-e` con pattern multi-line (escape `\n`) → rinviato/scartato (corner case)
- Performance: pattern file >10MB → not optimized in this step
- Refactor di `Matcher` enum oltre il minimo necessario per multi-pattern
- Ordering specifico tra `-e` e `-f` → approssimazione "appearance order"

## Commit attesi

**Due commit** (uno per repo, format obbligatorio integrale entrambi):

In `rgrep/.git`:
```
feat(step4): pattern sources -e (multi) and -f (file)

IN-SCOPE:
- Multi -e pattern OR alternation in matcher.rs
- Pattern file loading via -f in runner.rs
- Empty pattern handling (matches every line)
- -f missing file → exit 2 (D-G6 path)
- Positional pattern vs file argument disambiguation (D 4.5)
- Harness extension: fixture_files + {FIXTURES} placeholder
- 5 new differential testcases (0012-0016)
- N unit tests for pattern loading

OUT-OF-SCOPE (debito esplicito):
- Multi-line patterns
- Pattern file >10MB perf optimization
- Strict argv-ordering of -e vs -f patterns
- No rgrep/src/ refactor beyond Step 4 needs

Testcase aggiunti: 5. Totali: 16.
```

In `testag-grep/.git`:
```
docs(step4): translation log entry for pattern sources

IN-SCOPE:
- diary/03-translation-log.md: D-M1 expanded with multi-pattern semantics
  (covers grep.c CLI pattern parsing → cli.rs Vec<String> + runner.rs file loader)

OUT-OF-SCOPE (debito esplicito):
- No code changes
- No new D-decisions

Testcase aggiunti: 0. Totali: 16.
```

## Quando finisci

1. `cargo test --manifest-path rgrep/Cargo.toml` verde con 16 differential + ≥12 unit
2. Verifica con `grep -c '0012\|0013\|0014\|0015\|0016' rgrep/tests/testsuite.toml` ≥ 5
3. Verifica fixture handling: `cargo test test_differential -- --nocapture 2>&1 | grep -i fixture` mostra creazione/cleanup temp dir
4. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash> outer:<hash>`
5. **Stop**. Aspetta `audita` dal Decider.

---

# ✅ Step 5 — Output formatting: -n -b -H -h -c -l -L

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `6a3d6ab` — **Commit outer**: `fcff905`

## Goal

Allineare l'output formatting di rgrep alla parity GNU grep per i 7 flag
del gruppo "Output Control" (01-semantic-model.md §1):
- `-n` line-number prefix (1-based)
- `-b` byte-offset prefix (0-based)
- `-H` force filename prefix (single file)
- `-h` suppress filename prefix (multi-file)
- `-c` count matching lines (already partial — testcase 0005 di Step 0)
- `-l` files-with-matches (filename-only, stops scanning on first match)
- `-L` files-without-match (complementary di -l)

Si presume che i flag siano già dichiarati in `cli.rs` (lo sono, dalla
baseline pre-experiment). Lo step **wires up** la logica di output e
verifica l'ordine dei prefissi.

## Riferimenti contratto

- D-G6 (exit code semantics)
- D-M1 (Output Control group, originariamente in `print_line_head` `grep.c:1163`)
- 01-semantic-model.md §1 "Output Control"

## D-decisioni (contratto)

### D 5.1 — Ordine prefissi (parity GNU)
Quando combinati, l'ordine dei prefissi è:
```
filename:line_number:byte_offset:line_content
```
Separatore = `:` (default; `\0` con `-Z` ma quello è Step 12).

Esempio:
- `rgrep -nb foo file.txt` → `file.txt:1:0:foo line`
- `rgrep -n foo` (stdin) → `1:foo line`
- `rgrep -bn foo` = `rgrep -nb foo` (stessa cosa, prefisso uguale)

### D 5.2 — Filename prefix logic
- **default behavior**: prefisso filename presente se più di 1 file specificato OR `-r/-R` attivo
- `-H` force prefisso anche per single file (es. `rgrep -H foo file.txt` → `file.txt:foo`)
- `-h` sopprime prefisso anche con multi-file (last-wins se entrambi specificati)
- `-h` E `-H` insieme → l'ultimo specificato vince (parity GNU)

Implementazione: post-parsing in runner.rs:
```rust
let show_filename = match (cli.with_filename, cli.no_filename) {
    (true, _)  => true,   // -H force
    (_, true)  => false,  // -h suppress
    _          => files.len() > 1 || cli.recursive,
};
```

### D 5.3 — `-c` count
- Conta **linee matchanti** (non occorrenze totali del pattern)
- Multi-file: una riga per file: `filename:N`
- Senza filename prefix (single file): solo `N`
- `-c` con `-v` conta linee NON matchanti (combinabile)

GNU grep ritorna sempre exit 0 con `-c` se almeno un file ha 0 match e
almeno uno ha 1+ match (vedi test). In caso di **tutti i file 0 match**
exit 1. **NON va in conflitto con D-NEW-2 latente**: rgrep oggi ritorna
sempre 0 (Step 10 sistemerà).

### D 5.4 — `-l` files-with-matches
- Stampa **solo il nome del file** se contiene almeno un match
- **Stop scanning** del file alla prima riga matchante (ottimizzazione)
- Multi-file: una riga per file matchante
- Stdin: stampa `(standard input)` o `--label` se specificato (Step 13)
- `-l` ha precedenza su `-c`/`-n`/`-b`/`-H`: se `-l` attivo, niente prefissi

### D 5.5 — `-L` files-without-match
- Complementare di `-l`: stampa nome file se contiene **zero** match
- Stop scanning del file alla prima riga matchante (early exit per scartarlo)
- `-L` ha precedenza simile a `-l`

### D 5.6 — `-b` byte-offset
- Offset **0-based** del primo byte della linea matchante (non del match
  all'interno della linea)
- Per stdin: byte offset cumulativo del flusso
- Per file: byte offset all'interno del file
- Con `-z` (Step 12) l'offset è dal NUL byte precedente; out of scope qui

### D 5.7 — `-n` line-number
- 1-based, line number all'interno del file (o cumulativo per stdin)
- Reset per ogni file in modalità multi-file/recursive

### D 5.8 — Testcase obbligatori (TDD-first)

| ID | Args | Stdin / Fixtures | Expected | Note |
|---|---|---|---|---|
| 0017 | `["-n", "foo"]` | `bar\nfoo\nbaz\nfoo bar\n` | `2:foo\n4:foo bar\n` | line-number 1-based |
| 0018 | `["-b", "foo"]` | `bar\nfoo\nbaz\n` | `4:foo\n` | byte-offset 0-based, "bar\n" = 4 byte |
| 0019 | `["-H", "foo", "{FIXTURES}/file1.txt"]` | fixture `file1.txt` content `foo\nbar\nfoo\n` | `{FIXTURES}/file1.txt:foo\n{FIXTURES}/file1.txt:foo\n` | -H force filename single file |
| 0020 | `["-h", "foo", "{FIXTURES}/f1.txt", "{FIXTURES}/f2.txt"]` | fixtures: `f1.txt`=`foo\n`, `f2.txt`=`foo\n` | `foo\nfoo\n` | -h suppress filename multi-file |
| 0021 | `["-l", "foo", "{FIXTURES}/match.txt", "{FIXTURES}/nomatch.txt"]` | fixtures: `match.txt`=`foo\n`, `nomatch.txt`=`bar\n` | `{FIXTURES}/match.txt\n` | -l files-with-matches |
| 0022 | `["-L", "foo", "{FIXTURES}/match.txt", "{FIXTURES}/nomatch.txt"]` | fixtures: stesso di 0021 | `{FIXTURES}/nomatch.txt\n` | -L files-without-match |
| 0023 | `["-nb", "foo"]` | `bar\nfoo\n` | `2:4:foo\n` | combinato -n -b, ordine prefissi D 5.1 |

Note: i testcase con `{FIXTURES}` placeholder useranno path assoluti
nell'output expected. **Attenzione**: `expected_stdout` deve usare
`{FIXTURES}` anche nell'output, e il harness deve fare la sostituzione
**anche** sull'expected_stdout per il match. Estensione minima del
diff_runner.

### D 5.9 — Estensione harness: `{FIXTURES}` anche in `expected_stdout`
Modificare `diff_runner.rs` per applicare la sostituzione `{FIXTURES}`
**anche** su `expected_stdout` prima del confronto, oltre che sugli args.
Una linea aggiunta a `processed_args` block per processare anche
`processed_expected`.

### D 5.10 — File modificati attesi
```
rgrep/src/cli.rs                       (eventuale: tweaks per H/h interaction)
rgrep/src/runner.rs                    (output formatting logic)
rgrep/src/matcher.rs                   (eventuale: per -l/-L early exit)
rgrep/tests/diff_runner.rs             (estensione D 5.9)
rgrep/tests/cases/0017_line_number.toml
rgrep/tests/cases/0018_byte_offset.toml
rgrep/tests/cases/0019_with_filename.toml
rgrep/tests/cases/0020_no_filename.toml
rgrep/tests/cases/0021_files_with_matches.toml
rgrep/tests/cases/0022_files_without_match.toml
rgrep/tests/cases/0023_combined_n_b.toml
rgrep/tests/testsuite.toml             (+7 entries)
```

Se introduci `src/output.rs` come helper, aggiungilo all'elenco.

### D 5.11 — Build precondition (invariata)
- `cargo build` 0 warning di codice
- `cargo test` verde con 23 differential testcase + ≥10 unit (regressione zero)

### D 5.12 — Note implementazione
- `-c` esistente da Step 0 testcase 0005 — **non rompere** quel testcase
- `-l`/`-L` early-exit ottimizzazione: opzionale ma consigliato (parity GNU per perf)
- Filename per stdin = `(standard input)` se `--label` non dato (Step 13 lo cambierà)
- Quando in modalità `-c`, niente line-number/byte-offset prefix (solo `[filename:]N`)

## Acceptance criteria

- ✅ `cargo build` 0 warning, `cargo test` verde (23 differential + ≥10 unit)
- ✅ I 7 nuovi testcase passano
- ✅ Ordine prefissi `filename:line:byte:content` verificato in 0023
- ✅ Estensione harness D 5.9 funzionante (sostituzione `{FIXTURES}` anche in expected_stdout)
- ✅ Aggiunta entry in `diary/03-translation-log.md` per `print_line_head` translation
- ✅ Due commit (rgrep + outer), entrambi format integrale

## Out-of-scope (debito esplicito)

- `--color` highlighting (Step 6)
- `-Z` NUL filename suffix (Step 12)
- `-T` initial-tab alignment (Step 13)
- `--label` per stdin (Step 13)
- Performance ottimizzazione su file grandi
- Exit code 1 per "no match" (D-NEW-2, fix in Step 10)

## Commit attesi

In `rgrep/.git`:
```
feat(step5): output formatting flags -n -b -H -h -c -l -L

IN-SCOPE:
- Prefix order filename:line:byte:content per D 5.1
- -H/-h filename precedence logic (D 5.2)
- -l early-exit on first match (D 5.4)
- -L early-exit logic (D 5.5)
- Harness extension: {FIXTURES} substitution in expected_stdout (D 5.9)
- 7 new differential testcases (0017-0023)
- N unit tests for output formatting

OUT-OF-SCOPE (debito esplicito):
- --color (Step 6)
- -Z NUL filename suffix (Step 12)
- --label (Step 13)
- Exit code 1 for no-match (D-NEW-2 → Step 10)

Testcase aggiunti: 7. Totali: 23.
```

In `testag-grep/.git`:
```
docs(step5): translation log entry for output formatting

IN-SCOPE:
- diary/03-translation-log.md: print_line_head translation
  (grep.c:1163 → runner.rs output formatting)

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 23.
```

## Quando finisci

1. `cargo test` verde con 23 differential + ≥10 unit
2. Verifica `grep -c '0017\|0018\|0019\|0020\|0021\|0022\|0023' rgrep/tests/testsuite.toml` ≥ 7
3. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash> outer:<hash>`
4. **Stop**. Aspetta `audita` dal Decider.

---

# 🟡 Step 6 — -o, --color (GREP_COLORS) (PARTIAL)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commits rgrep**: `4ad2fda` + `980a024` (due commit per uno step, anti-pattern)
**Commit outer**: `2698c7b`
**Leftover**: vedi Step 6-bis sotto.

## Goal

Implementare le due feature di output highlighting:
- `-o, --only-matching`: stampa solo la **sottostringa matchante** (una per
  occorrenza, non per linea), una per riga
- `--color[=WHEN]`: highlighting ANSI dei match. Modi: `never`, `always`, `auto`
- Parsing `GREP_COLORS` env var per personalizzare i colori (subset minimo:
  `ms`, `mc`, `fn`, `ln`, `bn`, `se`, `sl`, `cx`)

## Riferimenti contratto

- D-G7 (Color = ANSI manuale + parsing `GREP_COLORS` env, status: da-validare)
- D-M1 (Output Control gruppo)
- 01-semantic-model.md §1 "Output Control" → flag `-o`

## D-decisioni (contratto)

### D 6.1 — `-o` semantica
Stampa **solo la sottostringa matchante**, una per riga. Se una linea
contiene più match, stampa una riga per ogni match (non sovrapposti).

Esempi:
- `printf 'foo bar foo\n' | rgrep -o foo` → `foo\nfoo\n`
- `printf 'aaa\n' | rgrep -o 'a'` → `a\na\na\n`
- `-o` con `-c` → conta **occorrenze**, non linee (parity GNU)

Edge case: pattern che matcha empty string (`-E "a*"`) → comportamento GNU
è "match every position", ma può loop infinito. **Skip** questo edge case
(out of scope, GNU stesso warns con `-P`).

### D 6.2 — `-o` interazione con prefissi (-n, -b, -H, -h)
Quando `-o` è attivo, i prefissi vengono applicati **per ogni match**:
- `-n` = numero di linea (stessa linea, ripetuta per ogni match della linea)
- `-b` = byte offset **del match** (NOT della linea — divergenza da `-b` solo)
- `-H/-h` come prima

Esempio:
- `printf 'foo bar foo\n' | rgrep -on foo` → `1:foo\n1:foo\n`
- `printf 'foo bar foo\n' | rgrep -ob foo` → `0:foo\n8:foo\n`

### D 6.3 — `--color` modes
Tre modi:
- `never` → mai colori (anche se TTY)
- `always` → sempre colori
- `auto` → colori solo se stdout è TTY

**rgrep default = `never`** (per ora; auto-detection è opzionale,
documentare in commit). GNU grep default = `auto`. **Divergenza tracciata
come D-NEW-3** in `04a-divergences.md`.

Razionale per default `never`: TTY auto-detection con `is_terminal()`
richiede dipendenza extra (oppure unsafe libc::isatty), e rende i testcase
differential difficili (l'output cambia tra terminale interattivo e
pipe/test). Lo step 6 si concentra sul comportamento esplicito, l'auto
viene rinviato a polish in Step 17.

### D 6.4 — GREP_COLORS env var (subset)
Parser di env var per i seguenti **specifier minimi**:
- `ms` = matching text in selected line (default: `01;31` = bold red)
- `mc` = matching text in context line (default: `01;31`)
- `fn` = filename (default: `35` = magenta)
- `ln` = line number (default: `32` = green)
- `bn` = byte offset (default: `32`)
- `se` = separator `:` (default: `36` = cyan)
- `sl` = selected line content (default: empty = inherit)
- `cx` = context line content (default: empty)

Format: `key1=val1:key2=val2:...`. Specifier non riconosciuti = ignorati
silenziosamente (parity GNU).

Esempio env: `GREP_COLORS='ms=1;33:fn=34'` → match in giallo grassetto,
filename in blu.

### D 6.5 — ANSI escape generation in-house
Niente crate `colored`/`termcolor`. Scrivere helper:
```rust
fn ansi_wrap(text: &str, code: &str) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, text)
}
```

Reset `\x1b[0m` dopo ogni segmento colorato. Il codice è raw (es. `01;31`),
non interpreted.

### D 6.6 — `--color` with no arg = `always` (parity GNU)
GNU grep: `--color` senza valore = `--color=always`. Implementare con
clap `default_missing_value`.

### D 6.7 — Disabilita color quando stdout non è TTY (per `auto`)
Anche se rgrep default è `never`, supportiamo `auto` correttamente:
- `auto` + stdout TTY → color enabled
- `auto` + stdout pipe/file → color disabled

Use `std::io::IsTerminal` (Rust 1.70+, già nello std). NO crate extra.

### D 6.8 — Testcase obbligatori (TDD-first)

| ID | Args / env | Stdin | Expected | Note |
|---|---|---|---|---|
| 0024 | `["-o", "foo"]` | `foo bar foo\n` | `foo\nfoo\n` | only-matching base |
| 0025 | `["-o", "-n", "foo"]` | `foo bar foo\nbaz\nfoo\n` | `1:foo\n1:foo\n3:foo\n` | -o + -n |
| 0026 | `["-o", "-b", "foo"]` | `foo bar foo\n` | `0:foo\n8:foo\n` | -o + -b (offset del match, non della linea) |
| 0027 | `["--color=never", "foo"]` | `foo\n` | `foo\n` | --color=never (no ANSI) |
| 0028 | `["--color=always", "foo"]` | `foo\n` | `\x1b[01;31mfoo\x1b[0m\n` | --color=always (default GREP_COLORS for ms) |
| 0029 | env `GREP_COLORS='ms=1;33'` + `["--color=always", "foo"]` | `foo\n` | `\x1b[1;33mfoo\x1b[0m\n` | env override |

**Note testcase 0028**: l'expected_stdout contiene escape ANSI literali. Il
TOML deserializzato deve preservarli. Verificare con un piccolo unit test.

**Note testcase 0029**: il harness deve supportare l'override di env vars
per testcase. Estensione: `env = { GREP_COLORS = "ms=1;33" }` opzionale
nel TOML. Vedi D 6.9.

**Note testcase 0028/0029 vs oracolo**: ugrep e GNU grep producono
output ANSI **identico** se GREP_COLORS è uguale. Verificare; se ugrep
diverge, marcare `skip_if_bsd = true` con motivo.

### D 6.9 — Estensione harness: env vars per testcase
Aggiungere campo TOML opzionale:
```toml
env = { GREP_COLORS = "ms=1;33" }
```

In `diff_runner.rs`, prima di spawn:
```rust
if let Some(env) = &case.env {
    for (k, v) in env {
        command.env(k, v);
    }
}
```

Applicare a **entrambi** rgrep e oracle.

### D 6.10 — File modificati attesi
```
rgrep/src/cli.rs                       (eventuale: --color default_missing_value)
rgrep/src/runner.rs                    (output con ANSI codici)
rgrep/src/output.rs                    (NUOVO se >50 LOC: helper ANSI + GREP_COLORS parser)
rgrep/src/matcher.rs                   (eventuale: -o richiede find_iter invece di is_match)
rgrep/tests/diff_runner.rs             (estensione D 6.9: env per testcase)
rgrep/tests/cases/0024..0029_*.toml    (6 nuovi)
rgrep/tests/testsuite.toml             (+6)
```

### D 6.11 — Build precondition (invariata)
- `cargo build` 0 warning di codice
- `cargo test` verde con 29 differential + ≥10 unit (regressione zero)

### D 6.12 — D-NEW-3 in `04a-divergences.md`
Aggiungere entry:
```markdown
### D-NEW-3: --color default = never (vs GNU = auto)

- **Severità**: parity / scelta_design
- **Categoria**: choice_design
- **Causa**: scelta_design (semplificazione testabilità)
- **Discovered in**: spec Step 6
- **Sintomo**:
  GNU grep default --color=auto → colori in TTY interattivo, niente in pipe.
  rgrep default --color=never → mai colori finché esplicitamente richiesto.
- **Razionale**: testcase differential più stabili, niente dipendenza da
  TTY runtime state. `--color=auto` resta supportato esplicitamente.
- **Decisione**:
  - [x] documentato come D-NEW-3
  - [ ] eventualmente cambio default a auto in Step 17 (polish)
- **Status**: aperto (scelta accettata per ora)
```

### D 6.13 — Unit test
Aggiungere ≥3 unit test per:
- Parser GREP_COLORS (default values, override, unknown keys ignored)
- ANSI wrap helper (singolo segmento, reset)
- `-o` byte offset calculation (offset del match, non della linea)

3 funzioni `#[test]` separate (lesson learned Step 4: bundling 3 scenari
in 1 funzione = nota minore — questa volta separa).

## Acceptance criteria

- ✅ `cargo build` 0 warning, `cargo test` verde (29 differential + ≥13 unit)
- ✅ I 6 nuovi testcase passano
- ✅ `--color=always` produce ANSI escape (verificare in 0028)
- ✅ `GREP_COLORS` env override funzionante (verificare in 0029)
- ✅ `-o` byte offset = offset del match (verificare in 0026)
- ✅ Estensione harness `env` field funzionante
- ✅ D-NEW-3 in `04a-divergences.md`
- ✅ Aggiunta entry in `diary/03-translation-log.md` per output color
- ✅ Due commit (rgrep + outer), entrambi format integrale
- ✅ ≥3 funzioni `#[test]` separate (D 6.13)

## Out-of-scope (debito esplicito)

- TTY auto-detection default (rinviato a Step 17 polish)
- GREP_COLORS specifier completi (`mt`, `ne`, etc. — solo i 8 minimi qui)
- Color highlighting di filename in modalità multi-file (semplificato)
- Empty-match infinite loop guard per `-o "a*"` (skip)

## Commit attesi

In `rgrep/.git`:
```
feat(step6): -o only-matching and --color highlighting

IN-SCOPE:
- -o prints only matched substrings, one per occurrence (D 6.1)
- -o interaction with -n/-b: byte offset of match, not line (D 6.2)
- --color modes: never/always/auto with IsTerminal detection (D 6.3, D 6.7)
- GREP_COLORS env parser: 8 specifier subset (D 6.4)
- In-house ANSI escape generation (no termcolor crate) (D 6.5)
- Harness extension: env field per testcase (D 6.9)
- 6 new differential testcases (0024-0029)
- N unit tests for color parsing and ANSI wrap

OUT-OF-SCOPE (debito esplicito):
- TTY auto default (rgrep = never, GNU = auto, divergenza D-NEW-3)
- Full GREP_COLORS specifier set
- Color filename in recursive output

Testcase aggiunti: 6. Totali: 29.
```

In `testag-grep/.git`:
```
docs(step6): translation log entry + D-NEW-3 color default divergence

IN-SCOPE:
- diary/03-translation-log.md: D-G7 + output color translation
- diary/04a-divergences.md: D-NEW-3 (rgrep --color default = never)

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 29.
```

## Quando finisci

1. `cargo test` verde con 29 differential + ≥13 unit
2. Verifica `grep -c '0024\|0025\|0026\|0027\|0028\|0029' rgrep/tests/testsuite.toml` ≥ 6
3. Verifica `04a-divergences.md` contiene `D-NEW-3`
4. Aggiorna l'header di QUESTO step con hash REALI
5. **Stop**. Aspetta `audita`.

---

# ✅ Step 6-bis — Cleanup leftover di Step 6

**Stato**: ✅ APPROVED (warning) — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commits rgrep**: `e1a1d89` + `dce7cb1` (2 commit, 2° offesa consecutiva — escalation in Step 7)

## Goal

Chiudere i leftover dell'audit di Step 6 (verdict 🟡 PARTIAL, vedi
[AUDIT_LOG.md](AUDIT_LOG.md)). Niente nuove feature. Solo:
1. Rimuovere il warning di build
2. Cambiare il flag sbagliato del testcase 0029
3. Lesson learned (no remediation) sui due commit per step

## Contesto

Audit Step 6 ha approvato la sostanza (29/29 testcase + 13 unit verdi,
tutti i 13 D 6.x letteralmente applicati) ma rilevato:
1. `cargo build` ha **1 warning di codice**: `unused import: std::collections::HashMap` in `src/output.rs:1`. Viola D 6.11 / D 0.6.
2. Testcase 0029 è marcato `expected_to_fail = true` ma rgrep funziona correttamente. La divergenza è solo sull'oracle dev-machine (ugrep ignora `GREP_COLORS`). Flag corretto = `skip_if_bsd = true`.
3. Step 6 ha prodotto 2 commit con titolo identico in `rgrep/.git`. Lesson learned, nessuna remediation invasiva (no rebase di history pubblicata).

## D-decisioni (contratto)

### D 6-bis.1 — Rimuovere `unused import HashMap`
In `rgrep/src/output.rs:1`, rimuovere la riga:
```rust
use std::collections::HashMap;
```
(o l'import correlato, qualsiasi sia la forma esatta).

Verificare con `cargo build 2>&1 | grep -v incremental | grep -v "consider moving" | grep ^warning:` → deve essere **vuoto**.

### D 6-bis.2 — Correggere flag testcase 0029
In `rgrep/tests/cases/0029_color_env_override.toml`:
- **Rimuovere** la riga `expected_to_fail = true`
- **Aggiungere**:
  ```toml
  skip_if_bsd = true
  skip_reason = "ugrep does not honor GREP_COLORS env var (rgrep correctly does)"
  ```

Razionale: rgrep onora `GREP_COLORS` correttamente come specificato in D 6.4.
Verificato live (subprocess Python):
- rgrep: `\x1b[1;33m...` (env applied) ✅
- ugrep oracle: `\x1b[01;31m...` (env IGNORED) ⚠️

Questa è una divergenza dell'oracle (ugrep), non di rgrep. Il flag
giusto è `skip_if_bsd` (consistente con D 0.7), non `expected_to_fail`
(che implica "rgrep broken").

### D 6-bis.3 — Lesson learned: un commit per step
Niente da fare per `4ad2fda`+`980a024` (water under the bridge — no rebase).
Promemoria: dal prossimo step, **un commit per step** (CLAUDE.md §3).
Pre-commit checklist: `cargo build && cargo test` PRIMA del primo commit;
se fallisce, fix in working tree e committa una sola volta.

### D 6-bis.4 — File modificati attesi
```
rgrep/src/output.rs                              (rimuovere import unused)
rgrep/tests/cases/0029_color_env_override.toml   (sostituire expected_to_fail con skip_if_bsd)
```

**Niente altro toccato.** Niente nuovi testcase.

### D 6-bis.5 — Verifica post-fix
Dopo le modifiche:
- `cargo build 2>&1 | grep '^warning:' | grep -v incremental | grep -v "consider moving"` → vuoto
- `cargo test` → 29/29 verde, con 0029 mostrato come `[skip]` su dev-machine ugrep
- Live exec di 0029 con env GREP_COLORS=ms=1;33 produce `\x1b[1;33m...` su rgrep

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice (no incrementali filesystem)
- ✅ `cargo test` verde, 29/29 differential + 13 unit
- ✅ 0029 ha `skip_if_bsd = true` + `skip_reason`, NO `expected_to_fail`
- ✅ Un solo commit, format obbligatorio integrale
- ✅ Solo i 2 file di D 6-bis.4 modificati

## Out-of-scope (debito esplicito)

- Nessun cambio al codice di rgrep oltre il warning fix
- Nessun nuovo testcase
- Nessuna modifica al diary

## Commit atteso

Un solo commit (uno solo, in `rgrep/.git`):
```
chore(step6-bis): fix unused import + correct testcase 0029 skip flag

IN-SCOPE:
- Remove unused std::collections::HashMap import in src/output.rs
- Replace expected_to_fail=true with skip_if_bsd=true in
  tests/cases/0029_color_env_override.toml (rgrep correctly honors
  GREP_COLORS; divergence is in ugrep oracle, not rgrep)

OUT-OF-SCOPE (debito esplicito):
- No code logic changes
- No new testcases
- No diary updates

Testcase aggiunti: 0. Totali: 29.
```

Niente commit nel repo outer (nessun file outer cambia).

## Quando finisci

1. `cargo build` 0 warning di codice + `cargo test` verde
2. Verifica `grep "skip_if_bsd" rgrep/tests/cases/0029_color_env_override.toml` → trova
3. Verifica `grep "expected_to_fail" rgrep/tests/cases/0029_color_env_override.toml` → vuoto
4. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash>`
5. **Stop**. Aspetta `audita` dal Decider.

---

# 🟡 Step 7 — Recursion: -r, -R, -d, -D (PARTIAL — BUILD RED)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `b52d887` — **Commit outer**: `1d622d9`
**Leftover**: cargo build FALLISCE (missing `}` in `src/cli.rs:245`). Vedi Step 7-bis sotto.

> 📝 **NOTA RELAY-SHELL**: il Decider fa relay copia-incolla tra Gemini
> e zsh. Quando Gemini fornisce un blocco multi-comando, se `cargo test`
> fallisce nel mezzo, il `git commit` parte comunque (incolla cieca).
> I 2 commit per step in Step 6 e 6-bis non erano scope creep di Gemini
> ma effetto collaterale del relay. Vedi D 7.0 (rilassata) sotto.

## Goal

Implementare la ricorsione di GNU grep nei filesystem:
- `-r, --recursive`: scende ricorsivamente nelle directory (NO follow symlink di default)
- `-R, --dereference-recursive`: come `-r` ma **segue symlink**
- `-d ACTION`: cosa fare con directory passate come argomento (`read`/`recurse`/`skip`)
- `-D ACTION`: cosa fare con device/FIFO/socket (`read`/`skip`)

## Riferimenti contratto

- D-G2 (ricorsione = `walkdir`)
- D-M2 (file traversal in 02-mapping-table.md §2.1)
- 01-semantic-model.md §1 "File Traversal" + §6 "Symlink Recursion Loop"
- D-NEW-2 latente (exit code 1 per no-match) — fix differito a Step 10

## D-decisioni (contratto)

### D 7.0 — Process discipline (rilassata: relay-shell aware)
Preferenza **un commit per step** in `rgrep/.git` + UN commit in
`testag-grep/.git` se ci sono diary updates.

**Fornire all'utente UN UNICO blocco di comandi atomico con `&&` chain**
che fa fail-fast su build/test rossi:

```bash
cd /Volumes/Extreme\ Pro/Claude/testag-grep/rgrep && \
  cargo build 2>&1 | tail -5 && \
  cargo test 2>&1 | tail -10 && \
  git add -A && \
  git commit -m "$(cat <<'EOF'
feat(step7): recursive traversal -r, -R, -d, -D

IN-SCOPE:
- ...
EOF
)"
```

Razionale: con `&&`, se cargo test fallisce, lo shell short-circuita e
non esegue `git add`/`git commit`. Niente commit zombie su working tree
broken. L'utente può copiare-incollare cieco senza rischio.

**Se nonostante `&&` viene committato uno stato broken**: forniamo un
secondo commit di fix come "hot-patch" — accettabile, NON è 🟡 PARTIAL
finché:
- Il 2° commit ha titolo `fix(step7-hotfix): <fix>` (non duplicato)
- Il body del 2° commit referenzia l'hash del 1° (`fixes <hash7>`)
- Il 2° commit fa SOLO il fix necessario (no scope creep)

**Escalation rimane attiva su scope creep**: se il 2° commit introduce
modifiche fuori dallo scope dichiarato del 1°, Step 7 sarà 🟡 PARTIAL
indipendentemente dalla sostanza.

In caso di blocco vero su DESIGN-Q non risolvibile: tagga
`🟡 BLOCKED — DESIGN-Q` nell'header e fermati prima di committare.

### D 7.1 — `-r` e `-R` semantica
- `-r` / `--recursive`: scende ricorsivamente, **NON segue symlink**
  (di default `walkdir::WalkDir::new(...).follow_links(false)`)
- `-R` / `--dereference-recursive`: scende ricorsivamente, **SEGUE
  symlink** (`follow_links(true)`)
- Se entrambi specificati, **last wins** (parity GNU)

Le due flag implicano automaticamente il prefisso filename (D 5.2):
quando `-r` o `-R` è attivo, ogni match ha `filename:` prefisso.

### D 7.2 — `-d ACTION` (directories)
Tre azioni:
- `read` (default): tenta di leggere la directory come fosse un file →
  errore (parity GNU)
- `skip`: ignora silenziosamente le directory passate come arg
- `recurse`: equivalente a `-r` (scende ricorsivamente)

Se `-r` o `-R` è attivo, `-d` è ignorato (parity GNU: ricorsione vince).

### D 7.3 — `-D ACTION` (devices/FIFO/socket)
Due azioni:
- `read` (default): tenta di leggere; per FIFO blocca, per socket fallisce
- `skip`: ignora silenziosamente

Implementazione: prima di aprire un file, check `metadata().file_type()`:
se è device/socket/FIFO e `-D skip`, salta.

**OUT-OF-SCOPE in Step 7**: gestione di `block device`, `char device`,
`socket`. Solo `FIFO` (named pipe) deve essere riconosciuto per skip.
Documenta la limitazione in commit IN-SCOPE.

### D 7.4 — Symlink loop protection
`walkdir` con `follow_links(true)` ha già protezione interna contro loop
(track visited inodes). Verificare che funzioni con un testcase loop.

Out-of-scope: gestione esplicita di hard link "uguali a se stesso".
walkdir gestisce, accettiamo come blackbox.

### D 7.5 — Exit code semantics
- `-r/-R` su directory con almeno un match → exit 0
- `-r/-R` su directory con zero match → exit 1 (atteso, ma D-NEW-2 latente
  → exit 0 di rgrep oggi)
- `-r/-R` su directory inesistente → exit 2

Per i testcase: testare la **prima** condizione (match found, exit 0) e
**la terza** (directory inesistente, exit 2). La seconda condizione è
bloccata da D-NEW-2 → marcare come `expected_to_fail = true` con motivo
"D-NEW-2 latente" finché Step 10 non sistema, OPPURE saltare il caso.

### D 7.6 — Testcase obbligatori (TDD-first)

Tutti i testcase usano `fixture_files` per creare struct di directory.

| ID | Args | Fixtures | Expected | Note |
|---|---|---|---|---|
| 0030 | `["-r", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n`, `dir/sub/b.txt`=`bar\nfoo\n` | `{FIXTURES}/dir/a.txt:foo\n{FIXTURES}/dir/sub/b.txt:foo\n` | -r ricorsivo base |
| 0031 | `["-R", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n`, `dir/link → b.txt`, `b.txt`=`foo\n` | output di entrambi a.txt e (link→)b.txt | -R follow symlink (può richiedere fixture extension D 7.7) |
| 0032 | `["-d", "skip", "foo", "{FIXTURES}/dir"]` | `dir/` vuota | `` (empty stdout) exit 1 (D-NEW-2 → exit 0 oggi) | -d skip |
| 0033 | `["-d", "recurse", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n` | `{FIXTURES}/dir/a.txt:foo\n` | -d recurse = -r |
| 0034 | `["-r", "foo", "/nonexistent/dir"]` | nessuna | `` (empty), stderr contains "No such" | exit 2 |

**Per 0031 (symlink)**: l'estensione harness `fixture_files` attualmente
crea solo file regolari. Per testare symlink serve estensione (D 7.7).
Se troppo invasivo, **scope-out 0031** con DESIGN-Q e marcare come
`expected_to_fail` con motivo "fixture symlinks not supported yet";
spec l'estensione come Step 8-bis o item nel backlog.

### D 7.7 — Estensione harness (opzionale): symlink in fixture_files
Estendere `FixtureFile` TOML con campo opzionale `symlink_to`:
```toml
[[fixture_files]]
name = "link"
symlink_to = "b.txt"   # crea link → b.txt
```

Oppure aggiungere un campo separato `fixture_symlinks`:
```toml
[[fixture_symlinks]]
name = "link"
target = "b.txt"
```

**Decisione**: l'Implementer sceglie la forma. Se troppo invasivo, salta
0031 con DESIGN-Q; non è bloccante per Step 7.

### D 7.8 — File modificati attesi
```
rgrep/src/cli.rs                       (eventuale: tweaks per -d/-D action enum)
rgrep/src/runner.rs                    (logica di traversal con walkdir)
rgrep/src/matcher.rs                   (probabilmente NULL change)
rgrep/tests/diff_runner.rs             (eventuale: D 7.7 estensione symlinks)
rgrep/tests/cases/0030..0034_*.toml    (5 nuovi)
rgrep/tests/testsuite.toml             (+5)
```

### D 7.9 — Build precondition (invariata)
- `cargo build` 0 warning di codice
- `cargo test` verde con 34 differential + ≥13 unit

### D 7.10 — Unit test
Aggiungere ≥2 unit test per:
- `-d` action enum parsing (read/recurse/skip)
- `-D` action enum parsing (read/skip)

Test separati `#[test]`, niente bundling (lesson Step 4).

### D 7.11 — Diary update
Aggiungere entry in `diary/03-translation-log.md` per il modulo
"file traversal" (parte di grep.c § "File Traversal" → walkdir adapter).

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 34 differential (incluso eventuale skip per 0031)
  + ≥15 unit
- ✅ I 5 nuovi testcase passano (eventualmente 0031 con expected_to_fail)
- ✅ `-r` non segue symlink, `-R` segue (D 7.1)
- ✅ `-d skip` ignora directory; `-d recurse` = `-r`
- ✅ Directory inesistente → exit 2 (testcase 0034)
- ✅ Diary 03 aggiornato
- ✅ **UN commit** in `rgrep/.git` + UN commit in outer (per diary)
- ✅ Format obbligatorio integrale entrambi
- ✅ **Process discipline D 7.0**: nessun secondo commit non autorizzato

## Out-of-scope (debito esplicito)

- `--include`, `--exclude`, `--exclude-dir` (Step 8)
- `--exclude-from` file di esclusione (Step 8)
- Esit code 1 per "no match in directory" (D-NEW-2 → Step 10)
- Block/char device skipping con tipo specifico (solo FIFO in Step 7)
- Output ordinato deterministico tra OS — accettiamo ordine walkdir default

## Commit attesi

**UN commit in `rgrep/.git`**:
```
feat(step7): recursive traversal -r, -R, -d, -D

IN-SCOPE:
- -r/-R recursive via walkdir (follow_links false/true) (D 7.1)
- -d ACTION (read/recurse/skip) for directory args (D 7.2)
- -D ACTION (read/skip) for FIFO devices (D 7.3)
- 5 new differential testcases (0030-0034)
- N unit tests for action enum parsing

OUT-OF-SCOPE (debito esplicito):
- --include/--exclude (Step 8)
- Exit code 1 for no-match in directory (D-NEW-2 → Step 10)
- Block/char device specific handling
- Symlink fixture extension (if not implemented)

Testcase aggiunti: 5. Totali: 34.
```

**UN commit in `testag-grep/.git`**:
```
docs(step7): translation log entry for file traversal

IN-SCOPE:
- diary/03-translation-log.md: walkdir adapter for grep.c file traversal

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 34.
```

## Quando finisci

1. `cargo build` 0 warning di codice + `cargo test` verde
2. Verifica `grep -c '0030\|0031\|0032\|0033\|0034' rgrep/tests/testsuite.toml` ≥ 5
3. **Verifica `git log --oneline <last_anchor>..HEAD` mostra ESATTAMENTE 1 commit per repo** (questo è D 7.0)
4. Aggiorna l'header di QUESTO step con hash REALI
5. **Stop**. Aspetta `audita`.

---

# ✅ Step 7-bis — Fix BUILD RED + verify Step 7

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `651f415`

> ⚠️ **STEP CRITICO**: il commit `b52d887` di Step 7 ha lasciato `src/cli.rs`
> con un `}` mancante. **Cargo build FALLISCE**. Niente sostanza è verificabile
> finché non si fixa.

## Goal

Riportare il build a verde aggiungendo il `}` mancante in `src/cli.rs`,
e poi eseguire la verifica completa di Step 7 che era stata bloccata
dal build red.

## Contesto

Audit Step 7 ha rivelato:
```
error: this file contains an unclosed delimiter
   --> src/cli.rs:245:3
208 | impl Config {
    |             - unclosed delimiter
217 |     pub fn get_before_context(&self) -> usize {
218 |         std::cmp::max(self.before_context, self.context)
219 | }                            <- chiude get_before_context
                                   <- MANCA `}` per chiudere impl Config
221 | #[cfg(test)]
222 | mod tests {
...
```

Questo significa che il `&&` chain di D 7.0 NON è stato applicato
correttamente — altrimenti `cargo build` red avrebbe bloccato il commit.

## D-decisioni (contratto)

### D 7-bis.1 — Fix brace
In `rgrep/src/cli.rs`, dopo riga 219 (chiusura `get_before_context`),
aggiungere `}` per chiudere `impl Config`:

```rust
    pub fn get_before_context(&self) -> usize {
        std::cmp::max(self.before_context, self.context)
    }    // <-- questo `}` chiude get_before_context
}        // <-- questo `}` AGGIUNGI: chiude impl Config

#[cfg(test)]
mod tests {
    ...
}
```

Verifica con `cargo build` → 0 errori, 0 warning di codice.

### D 7-bis.2 — Verifica testcase 0030-0034 passano
Dopo il fix:
```bash
cd rgrep && cargo build && cargo test 2>&1 | grep -E "running|test result"
```

Atteso:
- 34 differential testcase + ≥15 unit verdi
- Eventualmente 0031 in skip (se la fixture symlink extension non è ancora
  funzionante su tutti i sistemi)

Se uno dei 5 nuovi testcase **fallisce** runtime (non per build, ma per
match diff vs oracle), aprire **SPEC-Q** o `skip_if_bsd` con motivo,
**NON modificare expected silenziosamente**.

### D 7-bis.3 — Live-exec ≥3 dei 5 testcase
```bash
# 0030 -r base
mkdir -p /tmp/r7/dir/sub && \
  echo "foo" > /tmp/r7/dir/a.txt && \
  printf "bar\nfoo\n" > /tmp/r7/dir/sub/b.txt && \
  ./target/debug/rgrep -r foo /tmp/r7/dir

# 0034 -r dir inesistente
./target/debug/rgrep -r foo /nonexistent/dir; echo "exit=$?"

# 0033 -d recurse
./target/debug/rgrep -d recurse foo /tmp/r7/dir
```

Verificare che producano output coerente con expected_stdout dei TOML.

### D 7-bis.4 — Comando atomico per il commit
Forniscimi il blocco SHELL atomico che fa:
```bash
cd /Volumes/Extreme\ Pro/Claude/testag-grep/rgrep && \
  cargo build 2>&1 | tail -3 && \
  cargo test 2>&1 | tail -10 && \
  git add -A && \
  git commit -m "$(cat <<'EOF'
fix(step7-hotfix): close impl Config block in cli.rs

IN-SCOPE:
- Add missing `}` at src/cli.rs after line 219 to close impl Config
- Build now passes (was red after b52d887)

Fixes b52d887: build was red due to unclosed delimiter.

OUT-OF-SCOPE (debito esplicito):
- No logic changes
- No new testcases

Testcase aggiunti: 0. Totali: 34.
EOF
)"
```

**Importante**: il `&&` tra ogni passo. Se `cargo build` o `cargo test`
falliscono, il `git commit` NON parte.

### D 7-bis.5 — File modificati attesi
```
rgrep/src/cli.rs    (aggiungere `}` mancante)
```

**SOLO** quel file. Niente altro toccato in Step 7-bis.

### D 7-bis.6 — Lesson learned (sequence pattern)
Per evitare che si ripeta:
- Usare `&&` (NON `;` o newline) tra cargo build, cargo test, git add, git commit
- Verificare PRIMA di copiare-incollare nel terminale che il blocco è una
  singola riga semantica (eventualmente con line continuation `\`)
- Se serve un blocco multi-line, racchiuderlo in `bash -c '...'` o usare
  `set -e` all'inizio

## Acceptance criteria

- ✅ `src/cli.rs` ha la `}` di chiusura di `impl Config` corretta
- ✅ `cargo build` 0 errori, 0 warning di codice
- ✅ `cargo test` verde — 34 differential + ≥15 unit
- ✅ Almeno 3 testcase 0030-0034 verificati live-exec
- ✅ UN solo commit con titolo `fix(step7-hotfix): ...`
- ✅ Body referenzia `b52d887`

## Out-of-scope (debito esplicito)

- Refactor di cli.rs oltre il brace fix
- Implementazione dei filtri --include/--exclude (Step 8)
- Diary updates (lo step 7 originale ha già aggiornato 03)

## Quando finisci

1. `cargo build && cargo test` verdi (eseguito tramite il blocco atomico)
2. Verifica `git log --oneline dce7cb1..HEAD` in rgrep/ mostra **2 commit**:
   `b52d887` (originale) + `<hash7-bis>` (questo fix)
3. Aggiorna l'header di QUESTO step da
   `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash>`
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 8 — Filtering: --include, --exclude, --exclude-dir, --exclude-from

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `053a684` — **Commit outer**: `54426a3`

## Goal

Implementare i 4 filtri di traversal di GNU grep, applicati durante la
ricorsione `-r/-R` (Step 7) per decidere quali file/directory considerare:
- `--include=GLOB`: whitelist (solo i file che matchano vengono cercati)
- `--exclude=GLOB`: blacklist (file che matchano sono saltati)
- `--exclude-dir=GLOB`: blacklist directory (intere subtree saltate)
- `--exclude-from=FILE`: legge pattern di esclusione da FILE (uno per riga)

Tutti i flag sono ripetibili (multi-pattern via `globset` `Vec<Glob>`).

## Riferimenti contratto

- D-G3 (glob = `globset` crate)
- D-M3 / D-M-Filtering (file traversal in 02-mapping-table.md §2.1)
- 01-semantic-model.md §1 "File Traversal" + Step 7 walkdir

## D-decisioni (contratto)

### D 8.0 — Pattern relay-shell (continua)
Stesso pattern di Step 7: blocco shell atomico con `&&` chain
(cargo build && cargo test && git add && git commit). Niente commit con
build/test rosso. Hot-fix dello stesso scope è OK con titolo
`fix(step8-hotfix)`; scope creep = 🟡 PARTIAL.

### D 8.1 — `--include=GLOB` whitelist
- Ripetibile: `--include="*.rs" --include="*.toml"` = OR di entrambi
- Si applica solo ai **file**, NON alle directory
- Se nessun `--include` è specificato → nessun whitelist, tutti i file passano
- Se almeno un `--include` è specificato → un file deve matchare almeno un pattern

Implementazione: durante walkdir, per ogni entry `is_file()`, check con
`GlobSet::is_match(file_name)` (basename match, not full path — parity GNU).

### D 8.2 — `--exclude=GLOB` blacklist file
- Ripetibile (OR multiplo)
- Si applica solo ai **file**, NON alle directory
- File matchante un pattern viene **saltato** (no scansione)
- Combinazione con `--include`: prima check include (se richiesto), poi exclude
  (parity GNU: AND di "include OK" e "NOT excluded")

### D 8.3 — `--exclude-dir=GLOB` blacklist directory
- Ripetibile (OR multiplo)
- Si applica solo alle **directory**
- Match sul **basename** della directory (es. `--exclude-dir=node_modules`
  matcha qualsiasi `node_modules` ovunque nel tree)
- Quando una directory matcha → walkdir **skippa l'intera subtree** (early prune)
- Implementazione: in `walkdir` usare `into_iter().filter_entry(|e| ...)` per
  escludere subtree

### D 8.4 — `--exclude-from=FILE`
- Legge un file con un pattern per riga
- Ogni riga è un **pattern di exclude** (additivi a `--exclude`)
- Linee vuote ignorate
- Linee che iniziano con `#` ignorate (comment, parity GNU? Verificare —
  in caso ambiguo, NON gestire come comment, vedi nota sotto)
- File inesistente → exit 2 con errore stderr

**Note su `#` comment**: GNU grep `--exclude-from` storicamente NON tratta
`#` come comment (a differenza di `.gitignore`). **Default**: trattare `#`
come literal pattern. Se ugrep diverge, marcare con DESIGN-Q.

### D 8.5 — Combinazione filtri
Order of evaluation per ogni file/dir:
1. **Directory** durante traversal: scarta se matcha `--exclude-dir`
2. **File** durante traversal:
   - Se `--include` specificato: scarta se NON matcha nessun include
   - Se matcha qualche `--exclude` o `--exclude-from`: scarta
   - Altrimenti: include nel risultato

### D 8.6 — Senza `-r/-R`
Filtri specificati senza `-r/-R` → behavior GNU: ignored o warning?
- GNU grep: filtri ignorati silenziosamente (non rilevanti per file espliciti)
- ugrep: simile

Implementazione rgrep: ignora i filtri se `-r/-R` non attivi.

### D 8.7 — Testcase obbligatori (TDD-first)

| ID | Args | Fixtures | Expected | Note |
|---|---|---|---|---|
| 0035 | `["-r", "--include=*.txt", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n`, `dir/b.log`=`foo\n` | `{FIXTURES}/dir/a.txt:foo\n` | --include base |
| 0036 | `["-r", "--exclude=*.log", "foo", "{FIXTURES}/dir"]` | stessa di 0035 | `{FIXTURES}/dir/a.txt:foo\n` | --exclude base |
| 0037 | `["-r", "--exclude-dir=skipme", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n`, `dir/skipme/b.txt`=`foo\n` | `{FIXTURES}/dir/a.txt:foo\n` | --exclude-dir |
| 0038 | `["-r", "--exclude-from={FIXTURES}/excl.txt", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n`, `dir/b.log`=`foo\n`, `excl.txt`=`*.log\n` | `{FIXTURES}/dir/a.txt:foo\n` | --exclude-from |
| 0039 | `["-r", "--include=*.txt", "--exclude=ignore.txt", "foo", "{FIXTURES}/dir"]` | `dir/a.txt`=`foo\n`, `dir/ignore.txt`=`foo\n` | `{FIXTURES}/dir/a.txt:foo\n` | --include + --exclude (intersection) |

**Note testcase 0037**: serve subdir nel `fixture_files`. La struttura è
piatta (un file per entry); per creare `dir/skipme/b.txt`, il fixture
`name = "skipme/b.txt"` deve essere supportato (path con `/` che il harness
crea ricorsivamente). Verificare se l'harness attuale lo supporta o se serve
estensione minima.

**Note testcase 0038**: il file `excl.txt` è una fixture extra fuori da `dir/`,
quindi va in `{FIXTURES}/excl.txt` (root del temp dir). Verifica che il
harness lo crei correttamente.

### D 8.8 — File modificati attesi
```
rgrep/src/cli.rs            (eventuale: già ha i campi exclude/include/exclude_dir/exclude_from)
rgrep/src/runner.rs         (logica filtraggio durante walkdir)
rgrep/tests/diff_runner.rs  (eventuale: supporto subdir in fixture_files name)
rgrep/tests/cases/0035..0039_*.toml
rgrep/tests/testsuite.toml  (+5)
```

### D 8.9 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 39 differential + ≥15 unit (regressione zero)

### D 8.10 — Unit test
Aggiungere ≥3 unit test separati per:
- `globset` builder con multi-pattern OR
- `--exclude-from` file parser (skip empty lines, no `#` comment)
- Combinazione include + exclude (intersection logic)

### D 8.11 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per file filtering
(walkdir `filter_entry` adapter).

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 39 differential + ≥18 unit
- ✅ I 5 nuovi testcase passano (eventualmente con `skip_if_bsd`)
- ✅ `--include` whitelist file ✅, `--exclude` blacklist file, `--exclude-dir`
  prune subtree, `--exclude-from FILE` carica pattern da file
- ✅ Combinazione filtri: AND di "include OK && NOT excluded"
- ✅ Diary 03 aggiornato
- ✅ UN commit per repo, format obbligatorio integrale

## Out-of-scope (debito esplicito)

- `--exclude-from FILE` con `#` come comment (GNU non lo fa, manteniamo)
- Glob characters ANSI ranges esoterici
- Performance ottimizzazione su tree con migliaia di file
- Exit code 1 per "no match in directory" (D-NEW-2 latente, Step 10)

## Commit attesi

In `rgrep/.git`:
```
feat(step8): file/dir filtering --include/--exclude/--exclude-dir/--exclude-from

IN-SCOPE:
- --include glob whitelist via globset (D 8.1)
- --exclude glob blacklist file via globset (D 8.2)
- --exclude-dir prune subtree via walkdir filter_entry (D 8.3)
- --exclude-from FILE pattern loader (D 8.4)
- Combination: AND of include OK and NOT excluded (D 8.5)
- 5 new differential testcases (0035-0039)
- N unit tests for globset and combination logic

OUT-OF-SCOPE (debito esplicito):
- # comment in --exclude-from (GNU does not, neither do we)
- Performance on 1000+ file trees
- Exit code 1 for no-match (D-NEW-2 → Step 10)

Testcase aggiunti: 5. Totali: 39.
```

In `testag-grep/.git`:
```
docs(step8): translation log entry for file filtering

IN-SCOPE:
- diary/03-translation-log.md: globset filter_entry adapter

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 39.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain per build+test+commit
2. Verifica `git log --oneline 651f415..HEAD` mostra 1 commit per repo
3. Aggiorna l'header con hash REALI
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 9 — Context: -A, -B, -C, --group-separator

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `b994875` — **Commit outer**: `bc8eec3`

## Goal

Implementare le context lines di GNU grep:
- `-A NUM, --after-context=NUM`: stampa NUM righe **dopo** ogni match
- `-B NUM, --before-context=NUM`: stampa NUM righe **prima** ogni match
- `-C NUM, --context=NUM`: shorthand per `-A NUM -B NUM`
- `--group-separator=SEP`: separatore tra blocchi di context (default `--`)
- `--no-group-separator`: nessun separatore

I flag sono già dichiarati in `cli.rs` (baseline pre-experiment) come
`pub after_context: usize`, `pub before_context: usize`, `pub context: usize`,
`pub group_separator: String`, `pub no_group_separator: bool`. Lo step **wires
up** la logica nel runner.

## Riferimenti contratto

- D-M2 (Context Control via `VecDeque` ring buffer)
- 01-semantic-model.md §1 "Context Control"

## D-decisioni (contratto)

### D 9.0 — Pattern relay-shell
Stesso D 8.0: blocco shell atomico con `&&` chain. Nessun commit con build/test
red. Hot-fix dello stesso scope OK con titolo `fix(step9-hotfix)`.

### D 9.1 — Effective context = max
Quando combinati `-A`, `-B`, `-C` o `--*-context`:
```rust
effective_after  = max(after_context, context)
effective_before = max(before_context, context)
```
Già implementato in `cli.rs::get_after_context()` e `get_before_context()`
(baseline pre-experiment). **Verificare** che siano usati correttamente in
runner.rs.

### D 9.2 — Ring buffer per before-context
Usare `VecDeque<String>` di dimensione `effective_before`:
- Push ogni linea letta
- Quando supera la capacità, pop_front
- Su match: flush il VecDeque (stampa context before) + linea matchante

```rust
let mut before_buf: VecDeque<(usize, String)> = VecDeque::with_capacity(B);
// per ogni linea:
if matches { /* flush before_buf, then print line */ }
else if B > 0 {
    before_buf.push_back((line_num, line));
    if before_buf.len() > B { before_buf.pop_front(); }
}
```

### D 9.3 — Counter per after-context
Dopo un match, le successive `effective_after` linee non-matching vanno
stampate. Counter `after_remaining: usize`:
- Su match: `after_remaining = effective_after`
- Su linea non-match con `after_remaining > 0`: stampa la linea, decrementa
- Le linee stampate come "after context" usano separatore `-` (parity GNU)
  invece di `:` per il line/byte prefix

### D 9.4 — Prefix separator: `:` per match, `-` per context
GNU grep usa:
- `filename:line_num:content` per **match lines**
- `filename-line_num-content` per **context lines** (sia before che after)

Implementazione: in `print_line_head` (o equivalente in runner.rs), passare
un parametro `is_match: bool` e scegliere il sep di conseguenza.

### D 9.5 — Group separator tra blocchi non-contigui
Quando 2 blocchi context non si sovrappongono (esempio: linea 5 match con
-A 2, linea 20 match con -A 2 → blocchi 3-7 e 18-22 separati), stampare
`--\n` tra i due blocchi (default).

Se i blocchi si sovrappongono → fonderli senza separatore.

`--group-separator=STRING`: usa `STRING` invece di `--`.
`--no-group-separator`: niente separatore (ma blocchi restano separati da
linea-sep solo).

### D 9.6 — `-C 0` = `-A 0 -B 0` = nessun context
Edge case: `-C 0` esplicito non aggiunge context. Behavior identico a no-flag.
Nessun separatore neanche tra match contigui (parity GNU).

### D 9.7 — Interazione con `-c`/`-l`/`-L`
Quando `-c`, `-l`, `-L` sono attivi, context flag sono **ignorati** (parity GNU:
output è solo nome/count, niente content).

### D 9.8 — Testcase obbligatori (TDD-first)

| ID | Args | Stdin | Expected | Note |
|---|---|---|---|---|
| 0040 | `["-A", "1", "foo"]` | `bar\nfoo\nbaz\nqux\n` | `foo\nbaz\n` | -A 1 base |
| 0041 | `["-B", "1", "foo"]` | `bar\nfoo\nbaz\n` | `bar\nfoo\n` | -B 1 base |
| 0042 | `["-C", "1", "foo"]` | `a\nb\nfoo\nc\nd\n` | `b\nfoo\nc\n` | -C 1 = -A 1 -B 1 |
| 0043 | `["-A", "1", "foo"]` | `foo\na\nfoo\nb\n` | `foo\na\nfoo\nb\n` | overlap, no separator |
| 0044 | `["-A", "1", "foo"]` | `foo\na\nb\nc\nfoo\nd\n` | `foo\na\n--\nfoo\nd\n` | gap, separator `--` |
| 0045 | `["-A", "1", "--no-group-separator", "foo"]` | stessa di 0044 | `foo\na\nfoo\nd\n` | --no-group-separator |
| 0046 | `["-A", "1", "--group-separator=###", "foo"]` | stessa di 0044 | `foo\na\n###\nfoo\nd\n` | custom separator |

### D 9.9 — File modificati attesi
```
rgrep/src/runner.rs                    (logica context con VecDeque + counter)
rgrep/src/cli.rs                       (probabilmente NULL, già pre-existing)
rgrep/tests/cases/0040..0046_*.toml    (7 nuovi)
rgrep/tests/testsuite.toml             (+7)
```

### D 9.10 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 46 differential + ≥18 unit (regressione zero)

### D 9.11 — Unit test
Aggiungere ≥3 unit test separati per:
- VecDeque ring buffer (push beyond capacity → oldest discarded)
- Group separator emission logic (overlap vs gap)
- Effective context calculation (`max(A, C)`)

### D 9.12 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per Context Control
(parte di grep.c "Context Control" → runner.rs VecDeque adapter, citando D-M2).

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 46 differential + ≥21 unit
- ✅ I 7 nuovi testcase passano
- ✅ Prefix `-` per context lines vs `:` per match lines (D 9.4)
- ✅ Overlap → no separator; gap → separator (D 9.5)
- ✅ `--group-separator=X` custom (D 9.5)
- ✅ `--no-group-separator` no separator (D 9.5)
- ✅ Diary 03 aggiornato
- ✅ UN commit per repo, format integrale

## Out-of-scope (debito esplicito)

- Performance ottimizzazione su file enormi
- `-C` come distinct from `-A` + `-B` quando entrambi specificati (max wins, già D 9.1)
- Exit code 1 per "no match" (D-NEW-2 → Step 10)

## Commit attesi

In `rgrep/.git`:
```
feat(step9): context lines -A, -B, -C, --group-separator

IN-SCOPE:
- VecDeque ring buffer for -B before-context (D 9.2)
- Counter for -A after-context (D 9.3)
- Effective context = max(A,C) and max(B,C) (D 9.1)
- Prefix `-` for context vs `:` for match (D 9.4)
- Group separator emission on gap, no sep on overlap (D 9.5)
- --group-separator=STRING custom + --no-group-separator
- 7 new differential testcases (0040-0046)
- N unit tests for buffer and separator logic

OUT-OF-SCOPE (debito esplicito):
- Performance on huge files
- Exit code 1 for no-match (D-NEW-2 → Step 10)

Testcase aggiunti: 7. Totali: 46.
```

In `testag-grep/.git`:
```
docs(step9): translation log entry for context control

IN-SCOPE:
- diary/03-translation-log.md: VecDeque ring buffer for -B/-A/-C

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 46.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain
2. Verifica `git log --oneline 053a684..HEAD` mostra 1 commit per repo
3. Aggiorna l'header con hash REALI
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 10 — Limits: -m, -q, -s + fix D-NEW-2 (exit code semantics)

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `9bb0412` — **Commit outer**: `2f6b1e3`
**STEP STRATEGICO**: chiude D-NEW-2 latente. Exit code canonical 0/1/2.

> 🎯 **STEP STRATEGICO**: questo step chiude finalmente **D-NEW-2** (bug
> latente exit-code da Step 3). Da qui in poi exit code rgrep ≡ exit code
> GNU grep canonical: 0=match, 1=no-match, 2=error. Step 16 (proptest)
> dipende da questo fix.

## Goal

Implementare 3 flag di "limit/quiet/silent" + sistemare la semantica
degli exit code (D-NEW-2 latente da Step 3):
- `-m NUM, --max-count=NUM`: stop dopo NUM matches per file (early exit)
- `-q, --quiet, --silent`: nessun output, exit 0 al primo match
- `-s, --no-messages`: sopprimi error messages stderr per file inesistenti
- **Exit code semantics**: 0=match, 1=no-match, 2=error (D-NEW-2 fix)

## Riferimenti contratto

- D-G6 (thiserror::Error → exit code mapping)
- D-NEW-2 in `04a-divergences.md` (exit code latent bug)
- 01-semantic-model.md §1 "Output Control" + §7 "Exit codes"

## D-decisioni (contratto)

### D 10.0 — Pattern relay-shell
Stesso D 9.0: blocco shell atomico con `&&` chain.

### D 10.1 — `-m NUM` early exit per file
Ogni file conta i matches; quando arriva a NUM, **smette di scansionare**
(parity GNU). Implementazione: counter per-file in runner.rs, break del
loop di lettura quando `match_count >= max_count`.

Note edge:
- `-m 0` è valido in GNU grep ma equivale a "trova 0 match" → output vuoto + exit 1
- `-c` con `-m`: `-c` mostra il count cap'd a NUM (`min(actual, max)`)
- `-v` con `-m`: conta NON-match raggiunti, stessa logica
- Multi-file: il counter è per-file, non globale. Ogni file riparte da 0.

### D 10.2 — `-q` quiet mode
- **Nessun output stdout** (anche se ci sono match)
- Exit 0 al **primo match** trovato (early exit cross-file)
- Exit 1 se nessun match trovato in nessun file
- Exit 2 su errore (regex syntax, file IO, ecc.) — vedi D 10.4

`-q` è essenzialmente `-l` ma silenzioso e con early exit globale (non
per-file).

### D 10.3 — `-s` no-messages
Sopprime stderr per:
- File inesistenti (es. `rgrep foo /nonexistent` con `-s` → no stderr,
  ma still exit 2)
- File non leggibili (permessi)
- Eventuali warning di scansione

**NON** sopprime errori di syntax regex (quelli vanno comunque su stderr).

### D 10.4 — Exit code semantics (D-NEW-2 fix)
**Implementazione canonica**:
- Exit **0**: almeno un match trovato in almeno un file
- Exit **1**: nessun match trovato in nessun file (pattern compilato OK)
- Exit **2**: errore — pattern syntax invalida, file inesistente,
  pattern file `-f` inesistente, errore di IO

Implementazione:
- Tracking globale `any_match: bool` in runner.rs
- Al termine del run, return type del runner cambia da `Result<(), ...>` a
  qualcosa che esprime "completed successfully" + "had matches"
- main.rs decide exit 0/1/2 in base al risultato

Suggerimento idiomatico:
```rust
pub enum RunResult {
    MatchFound,           // exit 0
    NoMatch,              // exit 1
}

pub fn run(...) -> Result<RunResult, GrepError>;
```
main.rs:
```rust
match run(...) {
    Ok(RunResult::MatchFound) => 0,
    Ok(RunResult::NoMatch)    => 1,
    Err(_)                    => 2,  // GrepError già stampato
}
```

**Aggiornare D-NEW-2 in `04a-divergences.md`**: marcare `[x] fix in Step 10`
come completato.

### D 10.5 — Interazione con context (-A/-B/-C)
- `-q` ignora context (output zero)
- `-m NUM` con `-A` non conta le linee di context (solo i match veri)
- `-c` mostra solo il numero di **match lines**, non lines totali con context

### D 10.6 — Testcase obbligatori (TDD-first)

| ID | Args | Stdin / Fixtures | Expected stdout | Exit | Note |
|---|---|---|---|---|---|
| 0047 | `["-m", "2", "foo"]` | `foo\nfoo\nfoo\nbar\n` | `foo\nfoo\n` | 0 | -m 2 stop dopo 2 |
| 0048 | `["-m", "0", "foo"]` | `foo\nbar\n` | `` | 1 | -m 0 = no match output, exit 1 |
| 0049 | `["-q", "foo"]` | `foo\n` | `` | 0 | -q match → exit 0, no output |
| 0050 | `["-q", "foo"]` | `bar\n` | `` | 1 | -q no-match → exit 1 |
| 0051 | `["-s", "foo", "/nonexistent/file"]` | `` (stdin vuoto) | `` (stderr soppresso) | 2 | -s sopprime stderr |
| 0052 | `["foo"]` | `bar\nbaz\n` | `` | **1** | **D-NEW-2 FIX**: no-match → exit 1 |
| 0053 | `["foo", "/nonexistent/file"]` | `` | `` (stderr "No such") | **2** | **D-NEW-2 FIX**: file mancante → exit 2 |

**0052 e 0053 sono i fix di D-NEW-2.** Pre-Step-10 erano exit 0 (bug); ora
diventano exit 1 e 2 rispettivamente. **Verifica**: questi 2 testcase
DEVONO PASSARE solo dopo il fix di runner+main.

### D 10.7 — File modificati attesi
```
rgrep/src/runner.rs      (RunResult enum, max_count counter, quiet early exit)
rgrep/src/main.rs        (mappare RunResult → exit code 0/1/2)
rgrep/src/cli.rs         (probabilmente NULL)
rgrep/tests/cases/0047..0053_*.toml    (7 nuovi)
rgrep/tests/testsuite.toml             (+7)
diary/04a-divergences.md  (aggiornare D-NEW-2 status: fix completato)
```

### D 10.8 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 53 differential + ≥21 unit

### D 10.9 — Unit test
Aggiungere ≥3 unit test separati per:
- `-m` counter early-exit logic
- `-q` early-exit cross-file
- `RunResult` mapping main → exit code

### D 10.10 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per Limit Control +
exit code semantics. Aggiornare `diary/04a-divergences.md` D-NEW-2 status.

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 53 differential + ≥24 unit
- ✅ I 7 nuovi testcase passano
- ✅ **D-NEW-2 fix verificato**: testcase 0052 (no-match → exit 1) e 0053
  (missing file → exit 2) PASSANO
- ✅ `04a-divergences.md` D-NEW-2 status aggiornato a "fix completato in Step 10"
- ✅ `-m`, `-q`, `-s` funzionanti live
- ✅ Diary 03 aggiornato
- ✅ UN commit per repo, format integrale

## Out-of-scope (debito esplicito)

- Performance ottimizzazione su file enormi
- `-q` con `-r/-R`: cross-tree early exit (early-exit globale è sufficient,
  ma può esserci sotto-ottimo: skip restanti file se match trovato — accept se
  testcase passano)
- Atomic flag `--null-data` (Step 12)

## Commit attesi

In `rgrep/.git`:
```
feat(step10): limits -m -q -s + exit code semantics (D-NEW-2 fix)

IN-SCOPE:
- -m NUM early-exit per file (D 10.1)
- -q quiet mode with global early-exit (D 10.2)
- -s suppress stderr for missing/unreadable files (D 10.3)
- Exit code semantics: 0=match, 1=no-match, 2=error (D 10.4)
  - Fixes D-NEW-2 latent bug (was always exit 0)
- RunResult enum in runner.rs, mapped to exit code in main.rs
- 7 new differential testcases (0047-0053)
- N unit tests for limit and quiet logic

OUT-OF-SCOPE (debito esplicito):
- Performance optimization
- Atomic --null-data (Step 12)

Testcase aggiunti: 7. Totali: 53.
```

In `testag-grep/.git`:
```
docs(step10): translation log + D-NEW-2 fix completion

IN-SCOPE:
- diary/03-translation-log.md: Limit Control entry + RunResult mapping
- diary/04a-divergences.md: D-NEW-2 status updated (fix completed in Step 10)

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 53.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain
2. Verifica `git log --oneline b994875..HEAD` mostra 1 commit per repo
3. **Verifica D-NEW-2 fix**: live-exec
   ```
   printf 'foo\n' | ./target/debug/rgrep 'baz'    # → exit 1 (was 0)
   ./target/debug/rgrep foo /nonexistent           # → exit 2 (was 0)
   ```
4. Aggiorna l'header con hash REALI
5. **Stop**. Aspetta `audita`.

---

# 🟡 Step 11 — Binary: -a, -U, --binary-files (PARTIAL — BUILD RED)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `bcbe045` — **Commit outer**: `be03b75`
**Leftover**: `cargo test` non compila (mock Config in matcher.rs manca campo `without_match`). Working tree ha già il fix uncommitted. Vedi Step 11-bis sotto.

## Goal

Implementare la gestione dei file binari di GNU grep:
- `-a, --text`: tratta il file come testo (no detection)
- `-U, --binary`: tratta il file come binary (no CRLF stripping)
- `--binary-files=TYPE`: tipo `binary` (default: "Binary file matches"
  message), `text` (= -a), `without-match` (skip silently)

Heuristic detection: presenza di NUL byte (0x00) nel primo blocco di
lettura (parity GNU `buf_has_encoding_errors`, citato in
01-semantic-model.md §6 e `grep.c:1169`).

## Riferimenti contratto

- D-M3 (binary detection NUL-heuristic in 02-mapping-table.md)
- 01-semantic-model.md §6 "Dati binari mascherati" (`grep.c:1169`)
- 01-semantic-model.md §1 "Binary/Data" group

## D-decisioni (contratto)

### D 11.0 — Pattern relay-shell
Stesso D 10.0: blocco shell atomico con `&&` chain.

### D 11.1 — Binary detection: NUL byte heuristic
Per ogni file (non stdin), prima di processarlo:
- Leggere il primo blocco (es. 32 KB o l'intero file se più piccolo)
- Cercare NUL byte (`memchr::memchr(0, &buf)`)
- Se NUL trovato → file classificato **binary**
- Altrimenti → **text**

Per stdin (no seek): leggere all'inizio comunque, ma non si può "rewind"
→ buffer il primo blocco e scansiona da quello.

### D 11.2 — `-a, --text`
Disabilita la detection: tratta come **text** sempre, anche se contiene
NUL bytes. Modalità "force text".

### D 11.3 — `-U, --binary`
- Tratta come **binary** sempre, indipendentemente dalla heuristic
- **NO CRLF stripping**: su Windows il default GNU strips `\r\n` → `\n`
  per il match. Con `-U` non strippa. Su Linux/macOS è no-op (non c'è
  CRLF stripping di default). Documentare come behavior parity.

### D 11.4 — `--binary-files=TYPE`
Tre tipi:
- `binary` (default se NULL detected): output speciale
  `Binary file <name> matches` invece delle linee, e exit 0 se almeno
  un match
- `text` (alias di `-a`): tratta come text, output normale
- `without-match` (alias `-I`): skip il file se binary, no output, no
  match conta

### D 11.5 — Output `Binary file <name> matches`
Quando un file è binary (default) e c'è almeno un match:
- Stampa `Binary file <filename> matches\n` su stdout (NOT stderr)
- NON stampa le linee matchanti
- Exit code: 0 (match trovato in qualche file)

Esempio:
```
$ rgrep foo binary.bin
Binary file binary.bin matches
$ echo $?
0
```

### D 11.6 — Stdin label per binary
Se stdin è binary: `Binary file (standard input) matches` (parity GNU).

### D 11.7 — Interazione con `-a`/`-U`/`--binary-files`
Precedenza:
1. `--binary-files=TYPE` esplicito vince
2. `-a` = `--binary-files=text`
3. `-I` o `--binary-files=without-match`: skip
4. `-U` = `--binary-files=binary` + no CRLF strip
5. Default (nessun flag): `--binary-files=binary`

Last wins se contraddittori.

### D 11.8 — Testcase obbligatori (TDD-first)

Tutti i testcase usano `fixture_files` per creare file con/senza NUL.

| ID | Args | Fixtures | Expected | Note |
|---|---|---|---|---|
| 0054 | `["foo", "{FIXTURES}/bin.dat"]` | `bin.dat`=`foo\x00bar\n` (NUL byte) | `Binary file {FIXTURES}/bin.dat matches\n` exit 0 | binary detection default |
| 0055 | `["-a", "foo", "{FIXTURES}/bin.dat"]` | stessa di 0054 | `foo\x00bar\n` (linea con NUL) exit 0 | -a force text |
| 0056 | `["-I", "foo", "{FIXTURES}/bin.dat", "{FIXTURES}/text.txt"]` | `bin.dat`=`foo\x00`, `text.txt`=`foo\n` | `{FIXTURES}/text.txt:foo\n` exit 0 | -I = without-match |
| 0057 | `["--binary-files=text", "foo", "{FIXTURES}/bin.dat"]` | stessa di 0054 | linea raw con NUL exit 0 | --binary-files=text |
| 0058 | `["--binary-files=binary", "foo", "{FIXTURES}/text.txt"]` | `text.txt`=`foo\n` | `{FIXTURES}/text.txt:foo\n` exit 0 | binary forced ma no NUL → comporta come text |

### D 11.9 — Estensione harness: fixture binary content
Per scrivere file con NUL byte, il TOML deserializzato deve preservare
sequenze tipo `\x00`. Verifica che il parser TOML gestisca `\x00` in
stringhe quotate. Se non riesce, alternativa: usare un campo opzionale
`content_bytes = [102, 111, 111, 0, 98, 97, 114, 10]` (Vec<u8>) e
discriminare tra `content` (UTF-8 string) e `content_bytes` (raw bytes).

**Decisione**: Implementer sceglie. Se TOML supporta `\x00` in string
literal, usa quello (più leggibile). Altrimenti aggiunge `content_bytes`.

### D 11.10 — File modificati attesi
```
rgrep/src/cli.rs                       (eventuale: BinaryAction enum)
rgrep/src/runner.rs                    (binary detection + branch logic)
rgrep/tests/diff_runner.rs             (eventuale: content_bytes support)
rgrep/tests/cases/0054..0058_*.toml    (5 nuovi)
rgrep/tests/testsuite.toml             (+5)
```

### D 11.11 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 58 differential + ≥24 unit

### D 11.12 — Unit test
Aggiungere ≥3 unit test separati per:
- NUL byte heuristic (text vs binary classification)
- `--binary-files=TYPE` parsing
- "Binary file matches" message generation

### D 11.13 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per Binary handling
(grep.c `buf_has_encoding_errors` → memchr::memchr NUL detection).

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 58 differential + ≥27 unit
- ✅ I 5 nuovi testcase passano
- ✅ Binary detection live: file con NUL → "Binary file X matches"
- ✅ `-a`, `-U`, `--binary-files=TYPE` funzionanti
- ✅ Diary 03 aggiornato
- ✅ UN commit per repo, format integrale

## Out-of-scope (debito esplicito)

- Encoding-aware detection (UTF-8 BOM, latin-1, ...)
- Binary file with valid UTF-8 ma NUL byte (corner case)
- Output binario raw via stdout pipe (linea con NUL passa attraverso, parity GNU OK)
- CRLF stripping su platform Windows (out-of-scope per macOS/Linux dev)

## Commit attesi

In `rgrep/.git`:
```
feat(step11): binary file handling -a -U --binary-files

IN-SCOPE:
- NUL byte heuristic detection (D 11.1)
- -a force text mode (D 11.2)
- -U force binary mode (D 11.3)
- --binary-files={binary|text|without-match} (D 11.4)
- "Binary file X matches" output (D 11.5)
- 5 new differential testcases (0054-0058)
- N unit tests for detection and message logic

OUT-OF-SCOPE (debito esplicito):
- Encoding-aware detection (UTF-8 BOM, locale)
- Windows CRLF stripping
- Raw binary output through pipes

Testcase aggiunti: 5. Totali: 58.
```

In `testag-grep/.git`:
```
docs(step11): translation log entry for binary handling

IN-SCOPE:
- diary/03-translation-log.md: NUL heuristic + binary-files branch

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 58.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain
2. Verifica `git log --oneline 9bb0412..HEAD` mostra 1 commit per repo
3. Aggiorna l'header con hash REALI
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 11-bis — Commit fix mock Config + verify Step 11

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `5b3649b`
**Carried-over debt**: D 11.12 ≥3 unit test binary handling, recovery in step futuri o Step 17.

> ⚠️ **STEP CRITICO**: il commit `bcbe045` ha lasciato `cargo test` rosso
> (mock Config incompleto). Working tree ha già il fix uncommitted.
> Niente sostanza è verificabile finché non si committa.

## Goal

Riportare cargo test a verde committando il fix già presente in working
tree, e poi eseguire la verifica completa di Step 11.

## Contesto

Audit Step 11 ha rivelato:
```
error[E0063]: missing field `without_match` in initializer of `cli::Config`
  --> src/matcher.rs:184  (mock test Config)
```

Il working tree ha già il fix:
```
git diff src/matcher.rs
+            without_match: false,
```

Probabilmente il `&&` chain di Gemini ha incluso `cargo build` (passa)
ma non `cargo test` (fallisce). **Lesson learned**: il chain DEVE
includere `cargo test` esplicitamente.

## D-decisioni (contratto)

### D 11-bis.1 — Commit del fix mock già in working tree
Il working tree ha la modifica `+ without_match: false,` nel mock Config
in matcher.rs. Committarla.

```bash
cd rgrep && \
  cargo build && \
  cargo test && \
  git add src/matcher.rs && \
  git commit -m "..."
```

### D 11-bis.2 — Verifica Step 11 testcase passano
Dopo il fix:
- `cargo test` verde con 58 differential + ≥27 unit
- Live-exec ≥3 testcase 0054-0058 per confermare:
  - File con NUL → "Binary file X matches"
  - `-a` → output linea raw
  - `--binary-files=without-match` → skip silently

### D 11-bis.3 — Investigare/eliminare warning di build
`cargo build` segnala 2 warning a livello `(lib)` + `(bin)`. Vedere se
sono unused imports / dead code o falsi positivi filesystem. Se code
warnings reali → fixare; se filesystem (incremental cache) → tollerare.

```bash
cd rgrep && cargo build 2>&1 | grep -E "warning|-->" | grep -v incremental | grep -v "consider moving"
```

### D 11-bis.4 — Lesson learned: `&&` chain DEVE includere cargo test
Pattern obbligatorio:
```bash
cd rgrep && \
  cargo build 2>&1 | tail -5 && \
  cargo test 2>&1 | tail -10 && \   # <-- fondamentale
  git add -A && \
  git commit -m "..."
```

NON fare `cargo build && git commit` saltando `cargo test`.

### D 11-bis.5 — File modificati attesi
```
rgrep/src/matcher.rs   (commit del fix already in WT: + without_match: false)
```

Eventualmente:
- `rgrep/src/runner.rs` o `cli.rs` se le warning di D 11-bis.3 sono code

**SOLO** quel file (più eventuale fix warning). Niente altro.

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice (eventualmente fixare warning lib/bin)
- ✅ `cargo test` verde — 58 differential + ≥27 unit
- ✅ Live-exec ≥3 testcase 0054-0058 confermano comportamento atteso
- ✅ UN solo commit con titolo `fix(step11-hotfix): ...`
- ✅ Body referenzia `bcbe045`

## Out-of-scope (debito esplicito)

- Refactor di Config beyond fix
- Implementazione `-z`/`-Z` (Step 12)

## Commit atteso

UN commit in `rgrep/.git`:
```
fix(step11-hotfix): add without_match field to mock Config

IN-SCOPE:
- Add `without_match: false` to mock Config in matcher.rs:184 tests
- Build now compiles cargo test (was red after bcbe045)
- [eventuale] Resolve N warnings in src/

Fixes bcbe045: cargo test was red due to mock Config missing
the new without_match field added in Step 11.

OUT-OF-SCOPE (debito esplicito):
- No logic changes to binary detection
- No new testcases
- No diary updates

Testcase aggiunti: 0. Totali: 58.
```

Niente commit nel repo outer (nessun file outer cambia).

## Quando finisci

1. `cargo build && cargo test` verdi (eseguito tramite blocco atomico)
2. Aggiorna l'header da `🚧 PRONTO PER IMPLEMENTER` a
   `🟢 FATTO — AUDIT PENDING — commit rgrep:<hash>`
3. **Stop**. Aspetta `audita`.

---

# 🟡 Step 12 — NUL handling: -z, -Z (PARTIAL — commit incompleto)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `85d6a9c` (titolo `fix(step12-hotfix)`, doveva essere `feat(step12)`)
**Leftover**: 5 testcase TOML + testsuite.toml + diary 03 update tutti uncommitted. Vedi Step 12-bis sotto.

## Goal

Implementare il NUL byte handling di GNU grep:
- `-z, --null-data`: separa **record** con NUL byte invece di `\n`. Permette
  di processare output di `find ... -print0` o file con linee contenenti `\n`.
- `-Z, --null`: mette NUL byte dopo il **filename** nell'output (invece di
  `:`). Per consumer downstream tipo `xargs -0`.

`-z` cambia il delimitatore di linea ovunque: lettura, output, contatore
linee. È il più "deep" dei flag perché tocca il read loop.

## Riferimenti contratto

- D-G12 (eolbyte come `delimiter: u8`)
- D-G4 (`BufRead::read_until` per supportare `-z`)
- 01-semantic-model.md §1 "Binary/Data" + §6 "Empty Line"

## D-decisioni (contratto)

### D 12.0 — Pattern relay-shell
Stesso D 11.0: blocco shell atomico con `cargo build && cargo test && git add &&
git commit`. **Includere `cargo test` (lesson Step 11-bis)**.

### D 12.1 — `-z, --null-data` cambia il delimitatore di lettura
Quando `-z` è attivo:
- `BufRead::read_until(0, &mut buf)` invece di `read_until(b'\n', ...)`
- Pattern di output: usa `\0` come record separator (output finale termina con `\0`)
- Line count: cresce per ogni record `\0`-separato (non per `\n`-separated)

Implementazione: parametro `delimiter: u8` in `runner.rs` propagato in tutto
il read loop. Default = `b'\n'`, `-z` → `b'\0'`.

### D 12.2 — `-z` interazione con flags
- `-z` + `-n`: line number è il record number (records `\0`-separated)
- `-z` + `-b`: byte offset all'inizio del record
- `-z` + `-c`: conta record matchanti (non linee testo)
- `-z` + `-A`/`-B`/`-C`: context = `N` record `\0`-separated

### D 12.3 — `-Z, --null` cambia il separatore filename
Quando `-Z` è attivo, il separatore tra `filename` e il resto dell'output
diventa **NUL byte** (0x00) invece di `:`.

Esempio:
```
$ rgrep -Z foo file.txt
file.txt\x00matching line content
```

Si applica a TUTTI i prefissi filename: per `-l`, `-L`, `-c`, default match
output con filename. NON cambia i separatori line:byte (rimangono `:` di
default, `-` per context).

Edge case: `-Z` + `-z` + `-l` su multi-file:
```
$ rgrep -Zlz foo file.txt
file.txt\x00     # filename + NUL invece di newline
```

Quando `-Z` + `-l` sono insieme, anche il **terminatore** dopo il filename
diventa `\0` (parity GNU per consumo via `xargs -0`).

### D 12.4 — Output stdout binary-safe
Quando `-z` o `-Z` è attivo, l'output può contenere NUL byte → `stdout` deve
essere scritto come bytes raw (`io::Write::write_all(&buf)`) invece di
`println!` o stringhe formattate. Se rgrep usa già `&[u8]` internamente
(D-G5 lossy), no problema.

### D 12.5 — Testcase obbligatori (TDD-first)

**Note**: gli expected_stdout dei testcase con NUL devono usare `\0` in
TOML (lesson Step 11) per encode il NUL byte.

| ID | Args | Stdin / Fixtures | Expected stdout | Note |
|---|---|---|---|---|
| 0059 | `["-z", "foo"]` | `foo\0bar\0foo bar\0` | `foo\0foo bar\0` | -z null-data record sep |
| 0060 | `["-Z", "foo", "{FIXTURES}/file.txt"]` | fixture: `foo\n` | `{FIXTURES}/file.txt\0foo\n` | -Z null filename sep |
| 0061 | `["-zn", "foo"]` | `bar\0foo\0baz\0` | `2:foo\0` | -z + -n record number |
| 0062 | `["-Zl", "foo", "{FIXTURES}/m.txt", "{FIXTURES}/n.txt"]` | fixtures: `m.txt`=`foo\n`, `n.txt`=`bar\n` | `{FIXTURES}/m.txt\0` | -Z + -l filename + NUL terminator |
| 0063 | `["-zc", "foo"]` | `foo\0bar\0foo\0baz\0` | `2\n` | -z + -c count records |

### D 12.6 — File modificati attesi
```
rgrep/src/runner.rs                    (delimiter parameter, read_until, Output binary-safe)
rgrep/src/cli.rs                       (probabilmente NULL, già pre-existing)
rgrep/tests/cases/0059..0063_*.toml    (5 nuovi)
rgrep/tests/testsuite.toml             (+5)
```

### D 12.7 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 63 differential + ≥24 unit (regressione zero)

### D 12.8 — Unit test
Aggiungere ≥3 unit test separati per:
- `read_until` con delimiter `\0` vs `\n` (basic test)
- Line/record counter con delimiter custom
- Output binary-safe con NUL byte (verifica nessun panic UTF-8)

### D 12.9 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per NUL handling
(D-G12 `eolbyte` → `delimiter: u8` parameter).

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 63 differential + ≥27 unit
- ✅ I 5 nuovi testcase passano (eventualmente con `skip_if_bsd`)
- ✅ `-z` cambia delimitatore di record correttamente (live exec)
- ✅ `-Z` mette NUL dopo filename (live exec)
- ✅ Output binary-safe (no panic su NUL byte)
- ✅ Diary 03 aggiornato
- ✅ UN commit per repo, format integrale

## Out-of-scope (debito esplicito)

- Performance optimization su record giganti (>1MB)
- Combinazione esotica `-z` + `-r/-R` su file binari
- BOM (UTF-8 BOM) handling — fuori scope esperimento

## Commit attesi

In `rgrep/.git`:
```
feat(step12): NUL handling -z null-data and -Z null filename separator

IN-SCOPE:
- -z changes record delimiter to NUL via BufRead::read_until(0, ..) (D 12.1)
- -z interacts with -n (record num), -b (record offset), -c (record count) (D 12.2)
- -Z changes filename separator to NUL byte (D 12.3)
- Output binary-safe via io::Write::write_all (D 12.4)
- 5 new differential testcases (0059-0063)
- N unit tests for delimiter and binary-safe output

OUT-OF-SCOPE (debito esplicito):
- Performance on huge records
- BOM handling

Testcase aggiunti: 5. Totali: 63.
```

In `testag-grep/.git`:
```
docs(step12): translation log entry for NUL handling

IN-SCOPE:
- diary/03-translation-log.md: D-G12 eolbyte / delimiter: u8

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 63.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain (incluso `cargo test`)
2. Verifica `git log --oneline 5b3649b..HEAD` mostra 1 commit per repo
3. Aggiorna l'header con hash REALI
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 12-bis — Commit testcase + diary + fix warning

**Stato**: ✅ APPROVED (warning format) — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `2f158f2` — **Commit outer**: `662c1da`
**Note**: format obbligatorio violato in entrambi commit (solo titolo, no body) — recurring soft issue.

## Goal

Chiudere i leftover di Step 12: committare i 5 testcase TOML + testsuite.toml
(tutti uncommitted in WT), aggiungere outer commit per diary 03, fixare il
warning `unused variable: null_sep`.

## Contesto

Step 12 ha implementato il codice `-z`/`-Z` correttamente in `85d6a9c` (live
exec verificato), ma:
1. Titolo commit wrong (`fix(step12-hotfix)` invece di `feat(step12)`)
2. **5 testcase TOML uncommitted** (`tests/cases/0059..0063_*.toml`)
3. **testsuite.toml modificato uncommitted**
4. Niente outer commit per `diary/03-translation-log.md` (D 12.9)
5. 1 warning `unused variable: null_sep` (D 12.7)

## D-decisioni (contratto)

### D 12-bis.1 — Commit dei 5 testcase + manifest
Stage e commit di:
```
tests/cases/0059_null_data.toml
tests/cases/0060_null_filename.toml
tests/cases/0061_null_data_only_matching.toml
tests/cases/0062_null_filename_files_with_matches.toml
tests/cases/0063_null_data_cross_file.toml
tests/testsuite.toml
```

Verifica che `cargo test` esegua i 5 nuovi differential testcase (count
totale 63 differential atteso).

### D 12-bis.2 — Fixare warning `unused variable: null_sep`
Cercare in `src/runner.rs` la variabile `null_sep` e:
- Rimuoverla se davvero unused
- O usarla nel codice se serve (es. probabilmente era dest. al separator
  filename ma poi è stato sostituito da `b"\0"` literal)

Verifica: `cargo build 2>&1 | grep "^warning:" | grep -v incremental | grep -v "consider moving"` → vuoto.

### D 12-bis.3 — Outer commit per diary 03
Aggiornare `diary/03-translation-log.md` con entry per NUL handling
(D-G12 `eolbyte` → `delimiter: u8`, citazione di `grep.c` se possibile).
Commit nel repo outer.

### D 12-bis.4 — File modificati attesi
**rgrep**:
```
src/runner.rs              (rimuovere unused null_sep)
tests/testsuite.toml       (commit modifica WT)
tests/cases/0059..0063_*.toml  (5 nuovi)
```

**outer**:
```
diary/03-translation-log.md  (entry NUL handling)
```

### D 12-bis.5 — Lesson learned: include tutto nel `git add -A`
Pattern shell deve usare `git add -A` o `git add tests/ src/` per includere
i file nuovi (untracked). `git add` selettivo manuale rischia di lasciare
file fuori.

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 63 differential + 28 unit (regressione zero)
- ✅ I 5 testcase 0059-0063 passano
- ✅ Diary 03 ha entry NUL handling
- ✅ DUE commit:
  - rgrep: `chore(step12-bis): commit pending testcases + fix unused warning`
  - outer: `docs(step12-bis): translation log entry for NUL handling`
- ✅ Format integrale entrambi

## Out-of-scope (debito esplicito)

- Cambiare il titolo del commit `85d6a9c` (no rebase, water under the bridge)
- Refactor di runner.rs oltre fix warning
- Step 13 work

## Commit attesi

In `rgrep/.git`:
```
chore(step12-bis): commit pending testcases + fix unused warning

IN-SCOPE:
- Commit 5 testcase TOML (0059-0063) and testsuite.toml entry that were
  left uncommitted in 85d6a9c working tree
- Remove unused `null_sep` variable in src/runner.rs (build warning fix)

Fixes leftover from 85d6a9c (titled fix(step12-hotfix), should have been
feat(step12)): 5 testcases were authored but not staged.

OUT-OF-SCOPE (debito esplicito):
- No logic changes
- Cannot retitle 85d6a9c (no rebase policy)

Testcase aggiunti: 5. Totali: 63.
```

In `testag-grep/.git`:
```
docs(step12-bis): translation log entry for NUL handling

IN-SCOPE:
- diary/03-translation-log.md: D-G12 eolbyte / delimiter: u8 NUL handling

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 63.
```

## Quando finisci

1. `cargo build && cargo test` verdi
2. `git status` mostra working tree pulito
3. Aggiorna l'header di QUESTO step con hash REALI
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 13 — Misc: -T, --label, --line-buffered

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `3aa2e9a` — **Commit outer**: `b57af95`
Format obbligatorio integrale rispettato (recurring violation finalmente corretta).

> 📝 **Promemoria caldo (recurring)**: il commit message **format obbligatorio**
> richiede body con `IN-SCOPE`/`OUT-OF-SCOPE`/`Testcase aggiunti: N. Totali: M.`,
> non solo il titolo. Step 3 e Step 12-bis hanno entrambi violato. Per Step 13
> assicurati di includere il body completo.

## Goal

Implementare 3 flag misc di GNU grep:
- `-T, --initial-tab`: allinea i tabulatori dopo i prefissi (filename/line/byte)
- `--label=LABEL`: usa LABEL come nome del file per stdin (default `(standard input)`)
- `--line-buffered`: flush stdout dopo ogni linea (utile per pipe interattive)

I flag sono già dichiarati in `cli.rs` (baseline pre-experiment): `initial_tab`,
`label` (default `(standard input)`), `line_buffered`. Lo step **wires up** la
logica nel runner.

## Riferimenti contratto

- D-M1 (Output Control gruppo)
- 01-semantic-model.md §1 "Misc"

## D-decisioni (contratto)

### D 13.0 — Pattern relay-shell + format obbligatorio
- Blocco shell atomico con `&&` chain incluso `cargo test`
- **Body commit con IN-SCOPE/OUT-OF-SCOPE/Testcase aggiunti** (recurring violation issue)

### D 13.1 — `-T, --initial-tab` tab alignment
GNU grep semantica: prima di ogni linea di output, prepende un `\t` per
spaziare il prefisso. Implementazione minima: aggiungere `\t` all'inizio
di ogni linea matchante.

Verificare con oracle: GNU grep + ugrep producono `\tfoo\n` con `-T foo`.

Edge case: con prefissi (`-n`, `-b`, `-H`), il `\t` va all'inizio di tutto
o tra prefisso e content? GNU grep lo mette all'**inizio** (prima del prefisso).
Implementazione esistente di Gemini in print_line probabilmente già lo gestisce
— verificare.

### D 13.2 — `--label=LABEL` per stdin
Sostituisce `(standard input)` con LABEL ovunque venga stampato il "filename"
per stdin:
- Output match con prefisso filename
- Output `-l`/`-L`/`-c` con filename
- Output `Binary file LABEL matches`

Si applica SOLO quando rgrep legge da stdin (no file argomenti). Per file
veri il flag è ignorato.

### D 13.3 — `--line-buffered` flush per linea
Quando attivo, dopo ogni linea di output `stdout.flush()`. Default
behavior dipende dal terminale:
- TTY interattivo: flush automatico per linea
- Pipe: flush per blocco (4KB?)

Con `--line-buffered`: forza flush dopo ogni linea ovunque.

Edge case: l'output binary-safe di Step 12 usa `write_all`. Aggiungere
`stdout.flush()` dopo il terminator se `line_buffered`.

### D 13.4 — Testcase obbligatori (TDD-first)

| ID | Args | Stdin | Expected | Note |
|---|---|---|---|---|
| 0064 | `["-T", "foo"]` | `foo\n` | `\tfoo\n` | -T tab alignment |
| 0065 | `["-Tn", "foo"]` | `foo\n` | `\t1:foo\n` (or `1:\tfoo\n`?) | -T + -n verify ordering |
| 0066 | `["--label=mystdin", "-H", "foo"]` | `foo\n` | `mystdin:foo\n` | --label replaces stdin name |
| 0067 | `["--label=BIN", "foo"]` | `foo\x00bar\n` | `Binary file BIN matches\n` | --label in binary message |
| 0068 | `["--line-buffered", "foo"]` | `foo\nbar\nfoo\n` | `foo\nfoo\n` | --line-buffered (no observable diff in static test) |

**Note testcase 0065**: l'esatto ordering tra `\t` e prefisso line number può
variare. **Approccio**: prima eseguire l'oracolo (`grep -Tn foo` su stdin)
per vedere l'expected_stdout reale, poi mirror in rgrep.

**Note testcase 0068**: `--line-buffered` non ha output diverso in test
non-interattivo (il differential test legge tutto stdout alla fine). Quindi
testcase 0068 verifica solo che il flag non rompe niente. Per behaviour vera
serve un test interattivo (out-of-scope).

### D 13.5 — File modificati attesi
```
rgrep/src/runner.rs                    (logica -T tab, --label sub, --line-buffered flush)
rgrep/src/cli.rs                       (probabilmente NULL, già pre-existing)
rgrep/tests/cases/0064..0068_*.toml    (5 nuovi)
rgrep/tests/testsuite.toml             (+5)
```

### D 13.6 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 68 differential + ≥28 unit

### D 13.7 — Unit test
Aggiungere ≥3 unit test separati per:
- `-T` tab prepending
- `--label` substitution per stdin
- `--line-buffered` flush behavior (può essere stub se non testabile)

### D 13.8 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per Misc flags.

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 68 differential + ≥31 unit
- ✅ I 5 nuovi testcase passano
- ✅ Live exec: `rgrep -T foo <<<foo` → `\tfoo`
- ✅ Live exec: `rgrep --label=X foo <<<foo` → no observable diff senza prefix; con `-H` → `X:foo`
- ✅ Diary 03 aggiornato
- ✅ DUE commit (rgrep + outer), entrambi **format integrale**
  (recurring violation: questa volta il body deve esserci)

## Out-of-scope (debito esplicito)

- Test interattivo per `--line-buffered`
- `-T` con tab-stop calcolato (impl minima è solo `\t` literal)

## Commit attesi

In `rgrep/.git`:
```
feat(step13): misc flags -T, --label, --line-buffered

IN-SCOPE:
- -T initial-tab prepending (D 13.1)
- --label= substitution for stdin filename (D 13.2)
- --line-buffered stdout flush per line (D 13.3)
- 5 new differential testcases (0064-0068)
- N unit tests for tab/label/buffered

OUT-OF-SCOPE (debito esplicito):
- Interactive --line-buffered behavior testing
- Tab-stop column-aware alignment

Testcase aggiunti: 5. Totali: 68.
```

In `testag-grep/.git`:
```
docs(step13): translation log entry for misc flags

IN-SCOPE:
- diary/03-translation-log.md: -T, --label, --line-buffered

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 68.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain (incluso cargo test)
2. **Verifica che entrambi i commit abbiano body completo** (lesson Step 12-bis)
3. Verifica `git log --oneline 2f158f2..HEAD` mostra 1 commit per repo
4. Aggiorna l'header con hash REALI
5. **Stop**. Aspetta `audita`.

---

# 🟡 Step 14 — mmap path: --mmap (PARTIAL — commit incompleto)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `b7dc89c` (titolo `fix(step14-hotfix)`, doveva essere `feat(step14)`)
**Commit outer**: ❌ inesistente (`c02d244` era hash fittizio nell'header)
**Leftover**: 3 testcase TOML + testsuite.toml + diary 03 update tutti uncommitted. Vedi Step 14-bis sotto.

## Goal

Implementare `--mmap`: memory-mapped I/O via `memmap2` crate (già in
dependencies dalla baseline pre-experiment) per ottimizzare la lettura
di file regolari grandi. Fallback a BufReader se mmap fallisce o se
l'input è stdin/device/FIFO.

**Behavior**: identico al BufReader path. `--mmap` è solo una
ottimizzazione perf; l'output deve essere identico.

## Riferimenti contratto

- D-G3 placeholder + `memmap2` already in `Cargo.toml`
- Skill `legacy-port`: "limita unsafe, isola" — D 14.3
- 01-semantic-model.md §4 "Strategia memory management"

## D-decisioni (contratto)

### D 14.0 — Pattern relay-shell + format obbligatorio
Stesso D 13.0. **Body integrale** richiesto in entrambi commit.

### D 14.1 — `--mmap` solo per file regolari
Pre-condition checks prima di tentare mmap:
- File deve essere file regolare (`metadata().is_file()`, NOT directory/symlink/device/socket)
- File size > 0 (mmap di file vuoto fallisce su alcune piattaforme)
- Input non da stdin (impossibile mmap-are stdin)

Se anche solo una pre-condition fallisce → **fallback automatico** a
BufReader. Niente errore visibile al utente.

### D 14.2 — Fallback automatico
```rust
let use_mmap = config.mmap && metadata.is_file() && metadata.len() > 0;
let result = if use_mmap {
    match try_mmap_search(&file, ...) {
        Ok(r) => r,
        Err(_) => bufread_search(&file, ...),  // fallback silenzioso
    }
} else {
    bufread_search(&file, ...)
};
```

### D 14.3 — `unsafe` block isolato
`Mmap::map(&File)` richiede `unsafe` (file potrebbe essere modificato
durante la lettura, undefined behavior se altri processi lo modificano).
Skill warning: limita lo scope di `unsafe` al solo mapping:

```rust
fn try_mmap_search(file: &File, ...) -> io::Result<...> {
    let mmap = unsafe { Mmap::map(file)? };  // <-- unsafe SOLO qui
    let bytes: &[u8] = &mmap;                 // <-- da qui in poi safe
    process_buffer(bytes, ...)
}
```

Niente `unsafe` per byte access dopo Mmap::map (già `&[u8]` safe).
Niente `unsafe` per write/cast/transmute fuori dal mapping.

### D 14.4 — Output identico a BufReader
Differential testing tra mmap e BufReader path: l'output deve essere
**byte-per-byte identico**. Non ci devono essere differenze in
- Line count
- Match positions
- Byte offsets
- Newline handling

Implementazione: separare il "search core" (`fn search_buffer(buf: &[u8],
...)`) dalla I/O strategy. Sia BufReader che mmap chiamano lo stesso
search_buffer dopo aver caricato i bytes.

### D 14.5 — Testcase obbligatori (TDD-first)

| ID | Args | Fixtures | Expected | Note |
|---|---|---|---|---|
| 0069 | `["--mmap", "foo", "{FIXTURES}/file.txt"]` | `file.txt`=`foo\nbar\nfoo\n` | `{FIXTURES}/file.txt:foo\n{FIXTURES}/file.txt:foo\n` (con sort_output) o linee semplici se single-file no prefix | --mmap base |
| 0070 | `["--mmap", "-n", "foo", "{FIXTURES}/file.txt"]` | stesso 0069 | linee con line-number | --mmap + -n |
| 0071 | `["--mmap", "foo"]` | stdin: `foo\n` | `foo\n` (fallback automatico a BufReader) | --mmap su stdin → fallback |

Note: `--mmap` su single file probabilmente non mostra prefisso filename
(D 5.2). Verifica live l'expected_stdout reale.

### D 14.6 — File modificati attesi
```
rgrep/src/runner.rs                    (mmap path + fallback logic)
rgrep/src/cli.rs                       (probabilmente NULL, mmap già in cli)
rgrep/tests/cases/0069..0071_*.toml    (3 nuovi)
rgrep/tests/testsuite.toml             (+3)
```

### D 14.7 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde con 71 differential + ≥27 unit

### D 14.8 — Unit test
Aggiungere ≥2 unit test separati per:
- mmap su file regolare → ok
- mmap su stdin (NULL file) → fallback a BufReader

Testare l'**equivalenza** dell'output mmap vs BufReader su un file
regolare è il test più importante:
```rust
#[test]
fn test_mmap_buf_equivalence() {
    // run with mmap, capture output
    // run without mmap, capture output
    // assert outputs equal
}
```

### D 14.9 — Diary update
Aggiornare `diary/03-translation-log.md` con entry per mmap.
Citazione 01-semantic-model.md §4: "Non c'è alcun utilizzo di `mmap`
(historically rimosso per race condition hardware e I/O)" — rgrep
introduce mmap come **opt-in** (`--mmap`), non default. Spiegare
trade-off rispetto a GNU.

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 71 differential + ≥29 unit
- ✅ I 3 nuovi testcase passano
- ✅ Live exec: `rgrep --mmap foo file.txt` produce stesso output di
  `rgrep foo file.txt`
- ✅ Live exec: `rgrep --mmap foo < stdin` (impossibile mmap stdin) → fallback
  silenzioso, output corretto
- ✅ `unsafe` block presente SOLO su `Mmap::map`, nessun unsafe altrove
  in runner.rs (verifica con `grep -n unsafe src/runner.rs`)
- ✅ Diary 03 aggiornato
- ✅ DUE commit (rgrep + outer), entrambi format integrale

## Out-of-scope (debito esplicito)

- File size threshold per attivare mmap automaticamente (always opt-in via flag)
- mmap di simlink (resolved as regular file? → fallback a BufReader)
- Performance benchmark vs BufReader (out of scope esperimento metodologico)

## Commit attesi

In `rgrep/.git`:
```
feat(step14): mmap path via memmap2 with auto-fallback

IN-SCOPE:
- --mmap opt-in via memmap2::Mmap (D 14.1)
- Auto-fallback to BufReader for stdin/device/empty files (D 14.2)
- Single unsafe block isolated to Mmap::map (D 14.3)
- 3 new differential testcases (0069-0071)
- N unit tests for mmap path and fallback

OUT-OF-SCOPE (debito esplicito):
- Size threshold auto-activation
- Symlink mmap edge cases
- Performance benchmark

Testcase aggiunti: 3. Totali: 71.
```

In `testag-grep/.git`:
```
docs(step14): translation log entry for mmap path

IN-SCOPE:
- diary/03-translation-log.md: opt-in mmap with safe fallback;
  trade-off vs GNU (which removed mmap due to race conditions)

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 71.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain (cargo build && cargo test && add && commit)
2. Verifica `git log --oneline 3aa2e9a..HEAD` mostra 1 commit per repo
3. Aggiorna l'header con hash REALI
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 14-bis — Commit testcase + diary mmap

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `eb7db7e` — **Commit outer**: `f9695c2`

## Goal

Chiudere i leftover di Step 14 (déjà vu Step 12-bis): committare i 3
testcase TOML + testsuite.toml in `rgrep/.git`, committare il diary 03
update in `testag-grep/.git`. Niente nuove feature.

## Contesto

Step 14 ha implementato `--mmap` correttamente (`unsafe` isolato, output
identico a BufReader, fallback automatico) e committato il codice in
`b7dc89c` (titolo `fix(step14-hotfix)` invece di `feat(step14)`, ma
sostanza tecnica eccellente). **MA**:

```
# rgrep working tree:
M  tests/testsuite.toml
?? tests/cases/0069_mmap_base.toml
?? tests/cases/0070_mmap_line_number.toml
?? tests/cases/0071_mmap_fallback_stdin.toml

# outer working tree:
M  diary/03-translation-log.md  (entry mmap già scritta)
```

Header NEXT_STEPS Step 14 conteneva un hash outer `c02d244` **fittizio**
(verificato: `git cat-file -t c02d244` → "Not a valid object name"). Il
commit outer non era mai stato fatto.

## D-decisioni (contratto)

### D 14-bis.1 — Commit dei 3 testcase + manifest in rgrep
```bash
cd rgrep && \
  cargo build && \
  cargo test && \
  git add tests/testsuite.toml tests/cases/0069*.toml tests/cases/0070*.toml tests/cases/0071*.toml && \
  git commit -m "..."
```

### D 14-bis.2 — Outer commit per diary 03 mmap entry
La voce su mmap è già scritta in `diary/03-translation-log.md` (vedi WT).
Va solo committata.

```bash
cd .. && \
  git add diary/03-translation-log.md && \
  git commit -m "..."
```

### D 14-bis.3 — File modificati attesi

**rgrep**:
```
tests/testsuite.toml             (commit modifica WT)
tests/cases/0069..0071_*.toml    (3 nuovi)
```

**outer**:
```
diary/03-translation-log.md  (entry mmap già scritta)
```

### D 14-bis.4 — Format obbligatorio integrale (recurring lesson)
Body con IN-SCOPE/OUT-OF-SCOPE/Testcase aggiunti in **entrambi** commit.

### D 14-bis.5 — Lesson learned: pattern shell completo
Il pattern `&&` chain corretto include git add **TUTTI** i file:
```bash
cargo build && cargo test && git add -A && git commit -m "..."
```
oppure `git add` esplicito di:
- `src/` modificati
- `tests/` (nuovi e modificati, entrambi)
- `tests/testsuite.toml` (manifest)

`-A` evita di dimenticare untracked.

### D 14-bis.6 — NIENTE hash fittizi nell'header
**Mai scrivere `outer:cXXXXXX` se il commit non esiste**. Aggiorna
l'header SOLO dopo aver fatto i commit reali (e verifica con
`git rev-parse HEAD`).

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 71 differential + 30 unit (i 3 nuovi testcase ora attivi)
- ✅ DUE commit:
  - rgrep: `chore(step14-bis): commit pending mmap testcases`
  - outer: `docs(step14-bis): translation log entry for mmap path`
- ✅ Format obbligatorio integrale entrambi (lesson recurring)
- ✅ `git status` working tree pulito in entrambi repo
- ✅ Header Step 14-bis con hash REALI (verificare con git rev-parse)

## Out-of-scope (debito esplicito)

- Cambio titolo `b7dc89c` (no rebase, water under the bridge)
- Step 15 work

## Commit attesi

In `rgrep/.git`:
```
chore(step14-bis): commit pending mmap testcases

IN-SCOPE:
- Commit 3 testcase TOML (0069-0071) and testsuite.toml entry that were
  left uncommitted in b7dc89c working tree

Fixes leftover from b7dc89c (titled fix(step14-hotfix), should have been
feat(step14)): testcases were authored but not staged.

OUT-OF-SCOPE (debito esplicito):
- No code logic changes
- Cannot retitle b7dc89c (no rebase policy)

Testcase aggiunti: 3. Totali: 71.
```

In `testag-grep/.git`:
```
docs(step14-bis): translation log entry for mmap path

IN-SCOPE:
- diary/03-translation-log.md: --mmap implementation, single unsafe block,
  search_buffer extracted for output equivalence with BufReader

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 71.
```

## Quando finisci

1. Forniscimi UN UNICO blocco shell `&&` chain con `git add -A` o stage esplicito completo
2. Dopo i commit: `git rev-parse HEAD` in entrambi repo per gli hash REALI
3. Aggiorna l'header di QUESTO step con hash REALI (NON inventati)
4. **Stop**. Aspetta `audita`.

---

# 🟡 Step 15 — -P perl-regexp via pcre2 (feature gate) (PARTIAL — outer uncommitted)

**Stato**: 🟡 PARTIAL — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `43a3f5e` (titolo `fix(step15-hotfix)`, doveva essere `feat(step15)`)
**Commit outer**: ❌ assente (diary 02 + 03 modificati in WT, non committati)
**Leftover**: vedi Step 15-bis sotto. Sostanza tecnica eccellente, solo cleanup outer.

## Goal

Implementare `-P, --perl-regexp` come **feature opzionale** Cargo:
- Default build (no feature): `-P` continua a errore stub di Step 3
  (`rgrep: -P only supported when compiled with --features perl-regexp` exit 2)
- Build con `--features perl-regexp`: `-P` compila pattern PCRE2 reali via
  crate `pcre2`, supporto lookahead/lookbehind/back-reference

Step PIÙ TECNICO che metodologico — il pcre2 crate fa quasi tutto il lavoro,
serve solo il glue con `Matcher` enum.

## Riferimenti contratto

- D-M10 (pcre2 crate, feature-gated bridge — strategia A skill)
- D-M11 (JIT stack limit configurazione)
- D-M12 (invalid UTF-8 skip via pcre2::bytes)
- 02-mapping-table.md §2.4 pcresearch.c

## D-decisioni (contratto)

### D 15.0 — Pattern relay-shell + format integrale
Stesso D 14.0. Body completo nei commit.

### D 15.1 — Feature flag in `Cargo.toml`
```toml
[dependencies]
pcre2 = { version = "0.2", optional = true }

[features]
perl-regexp = ["dep:pcre2"]
```

Senza feature → `pcre2` non incluso, build default rimane snella.

### D 15.2 — `-P` stub (default, no feature) invariato
La logica esistente da Step 3:
```rust
#[cfg(not(feature = "perl-regexp"))]
fn compile_perl(...) -> Result<Matcher, GrepError> {
    eprintln!("rgrep: -P only supported when compiled with --features perl-regexp");
    Err(GrepError::PcreUnsupported)
}
```

### D 15.3 — `-P` real implementation (feature-gated)
```rust
#[cfg(feature = "perl-regexp")]
fn compile_perl(patterns: &[String], opts: &Config) -> Result<Matcher, GrepError> {
    let pat = patterns.join("|");
    let mut builder = pcre2::bytes::RegexBuilder::new();
    builder.caseless(opts.ignore_case);
    builder.utf(true).jit(true);
    let re = builder.build(&pat).map_err(GrepError::Pcre)?;
    Ok(Matcher::Pcre2(re))
}
```

Aggiungere variante `Pcre2(pcre2::bytes::Regex)` all'enum `Engine`/`Matcher`
(o equivalente esistente in matcher.rs), gated da `#[cfg(feature = "perl-regexp")]`.

### D 15.4 — `Matcher::is_match` / `find` per Pcre2
Implementare i metodi del Matcher trait/impl per la variante Pcre2:
- `is_match(&self, line: &[u8]) -> bool`: usa `re.is_match(line)`
- `find(&self, line: &[u8]) -> Option<Range<usize>>`: usa `re.find(line)`

### D 15.5 — JIT stack (D-M11)
PCRE2 JIT può fallire su pattern molto complessi. Gestire `JitStackLimit`
fallback al non-JIT senza errore visibile (parity GNU grep).

In Step 15 minimo: NON gestire fallback esplicito (delegato al pcre2 crate
default). Documentare come scope-out.

### D 15.6 — Testcase obbligatori (TDD-first)
**Tutti gated `skip_if_bsd = true`** perché ugrep oracle ha PCRE2 nativo
ma con sintassi leggermente diversa.

| ID | Args | Stdin | Expected | Note |
|---|---|---|---|---|
| 0072 | `["-P", "foo(?=bar)"]` | `foobar\nfoo\n` | `foobar\n` | -P lookahead |
| 0073 | `["-P", "(?<=foo)bar"]` | `foobar\nbar\n` | `foobar\n` | -P lookbehind |
| 0074 | `["-P", "(\\w+)\\s+\\1"]` | `foo foo\nbar\n` | `foo foo\n` | -P backreference |

I 3 testcase usano feature PCRE2-specifiche che il `regex` crate non supporta.
**SE compilato senza feature** → testcase devono essere SKIPPED via testsuite logic
(non eseguiti). Eventualmente aggiungere campo `requires_feature = "perl-regexp"`
al TOML, gestito dal harness.

### D 15.7 — Estensione harness: `requires_feature`
```toml
requires_feature = "perl-regexp"
```

In `diff_runner.rs`:
```rust
#[cfg(feature = "perl-regexp")]
const PCRE_AVAILABLE: bool = true;
#[cfg(not(feature = "perl-regexp"))]
const PCRE_AVAILABLE: bool = false;

if let Some(feat) = &case.requires_feature {
    if feat == "perl-regexp" && !PCRE_AVAILABLE {
        eprintln!("[skip] {} (requires --features perl-regexp)", case.name);
        continue;
    }
}
```

### D 15.8 — File modificati attesi
```
rgrep/Cargo.toml                       (pcre2 optional + features)
rgrep/Cargo.lock                       (auto-update)
rgrep/src/cli.rs                       (probabilmente NULL)
rgrep/src/matcher.rs                   (Matcher::Pcre2 variant + impl gated)
rgrep/src/runner.rs                    (eventuale: dispatch -P alla branch giusta)
rgrep/tests/diff_runner.rs             (estensione requires_feature)
rgrep/tests/cases/0072..0074_*.toml    (3 nuovi)
rgrep/tests/testsuite.toml             (+3)
```

### D 15.9 — Build precondition: DUE build
- `cargo build` (default, no feature) → 0 warning, exclude testcase 0072-0074
- `cargo build --features perl-regexp` → 0 warning, include testcase 0072-0074
- `cargo test` (default) verde — 71 differential + skip 3 = 71 effective + 29 unit
- `cargo test --features perl-regexp` verde — 74 differential + 29+ unit

L'Implementer deve verificare ENTRAMBI i build green.

### D 15.10 — Unit test gated
Aggiungere ≥2 unit test sotto `#[cfg(feature = "perl-regexp")]`:
- `test_pcre_lookahead`
- `test_pcre_backreference`

Questi test girano SOLO con `cargo test --features perl-regexp`.

### D 15.11 — Diary update
- `diary/03-translation-log.md`: entry pcresearch.c → pcre2 crate, feature-gated
  (citare D-M10..M12)
- `diary/02-mapping-table.md`: aggiornare D-M10 status: confirmed ↔ implemented

## Acceptance criteria

- ✅ `cargo build` (default) 0 warning di codice
- ✅ `cargo build --features perl-regexp` 0 warning di codice
- ✅ `cargo test` (default) verde — 71 effective differential (3 skipped no-feature) + 29 unit
- ✅ `cargo test --features perl-regexp` verde — 74 differential + ≥31 unit
- ✅ Live exec senza feature: `rgrep -P "foo" <<<foo` → exit 2 stub (Step 3 invariato)
- ✅ Live exec CON feature: `rgrep -P "foo(?=bar)" <<<"foobar"` → `foobar\n`
- ✅ Diary 03 + 02 aggiornati
- ✅ DUE commit (rgrep + outer), entrambi format integrale

## Out-of-scope (debito esplicito)

- JIT stack limit fallback automatico (delegated to crate default)
- PCRE2 syntax diff vs ECMAScript regex (out of scope)
- Performance benchmark vs `regex` crate (out of scope)
- Test su pattern PCRE catastrophic backtracking (skill: limita ReDoS surface,
  ma rgrep dipende dal crate)

## Commit attesi

In `rgrep/.git`:
```
feat(step15): -P perl-regexp via pcre2 crate (feature-gated)

IN-SCOPE:
- pcre2 = "0.2" optional dependency, feature "perl-regexp" (D 15.1)
- Matcher::Pcre2 variant gated #[cfg(feature = "perl-regexp")] (D 15.3, D 15.4)
- -P stub error preserved for default build (D 15.2, parity Step 3)
- Harness requires_feature field for conditional skip (D 15.7)
- 3 new differential testcases (0072-0074) gated to feature
- N unit tests gated to feature

OUT-OF-SCOPE (debito esplicito):
- JIT stack fallback (delegated to crate)
- PCRE2 vs ECMAScript syntax diff
- Performance benchmark

Testcase aggiunti: 3. Totali: 74.
```

In `testag-grep/.git`:
```
docs(step15): translation log + mapping table for PCRE2 feature gate

IN-SCOPE:
- diary/03-translation-log.md: pcresearch.c → pcre2 crate, feature-gated bridge
- diary/02-mapping-table.md: D-M10 status updated to "implemented"

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 74.
```

## Quando finisci

1. Forniscimi DUE blocchi shell:
   a. `cd rgrep && cargo build && cargo test && git add ... && git commit`
   b. `cd rgrep && cargo build --features perl-regexp && cargo test --features perl-regexp` (verifica feature ON)
2. Outer commit per diary
3. Aggiorna l'header con hash REALI (NON inventati)
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 15-bis — Commit outer diary entries

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit outer**: `2d22d15`

## Goal

Chiudere i leftover di Step 15: committare le entries di `diary/02-mapping-table.md`
e `diary/03-translation-log.md` già scritte in WT outer.

Niente nuove feature, niente modifiche al codice rgrep.

## Contesto

Step 15 ha implementato `-P` perl-regexp feature-gated **eccellentemente**:
- Stub preservato in default build
- Lookahead/lookbehind/backref live verificati con feature
- Harness `requires_feature` esteso
- Due build (default + feature) entrambi verdi
- 30 test default + 32 test feature

**Però** identico pattern di Step 12/14: outer commit per diary mancante.
`git status` outer mostra:
```
M  diary/02-mapping-table.md
M  diary/03-translation-log.md
```

Le entries sono già scritte (Implementer ha aggiornato i diary in WT) —
serve solo committarle.

## D-decisioni (contratto)

### D 15-bis.1 — Outer commit per diary 02 + 03
```bash
cd /Volumes/Extreme\ Pro/Claude/testag-grep && \
  git add diary/02-mapping-table.md diary/03-translation-log.md && \
  git commit -m "$(cat <<'EOF'
docs(step15-bis): translation log + mapping table for PCRE2 feature gate

IN-SCOPE:
- diary/03-translation-log.md: pcresearch.c → pcre2 crate (feature-gated bridge)
- diary/02-mapping-table.md: D-M10 status updated to "implemented"

OUT-OF-SCOPE (debito esplicito):
- No code changes
- Cannot retitle 43a3f5e (no rebase policy)

Testcase aggiunti: 0. Totali: 74.
EOF
)"
```

### D 15-bis.2 — File modificati attesi
```
diary/02-mapping-table.md   (commit modifica WT — già scritta)
diary/03-translation-log.md (commit modifica WT — già scritta)
```

**SOLO** outer repo. Niente in `rgrep/.git`.

### D 15-bis.3 — Format obbligatorio integrale
Body del commit deve avere IN-SCOPE/OUT-OF-SCOPE/Testcase aggiunti.

### D 15-bis.4 — Lesson learned (recurring): commit outer dopo commit rgrep
Pattern documentato in CLAUDE.md "Pattern relay-shell": forniscimi DUE
blocchi shell, uno per ogni repo, NON un solo blocco che cambia cwd a metà
(rischio di dimenticare il secondo).

```bash
# Block 1 (rgrep)
cd /path/rgrep && cargo build && cargo test && git add -A && git commit -m "feat..."

# Block 2 (outer)
cd /path && git add diary/ && git commit -m "docs..."
```

Step 12, 14, 15 hanno tutti dimenticato il blocco 2.

## Acceptance criteria

- ✅ `git status` outer working tree pulito
- ✅ Commit outer con format integrale
- ✅ `git log -1 testag-grep/.git` mostra il nuovo commit con hash REALE
- ✅ Header Step 15-bis aggiornato con hash REALE

## Out-of-scope (debito esplicito)

- Cambio titolo `43a3f5e` (no rebase, water under the bridge)
- Step 16 work
- Eventuale aggiunta di entries diary aggiuntive non già scritte

## Quando finisci

1. `git status` in entrambi i repo: pulito
2. `git rev-parse HEAD` in outer per l'hash
3. Aggiorna l'header con hash REALE
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 16 — Differential proptest

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `6ccd72a` — **Commit outer**: `29c8ba4`
**Phase 4b skill completata**: 3072 random differential checks, 0 divergenze nuove.

> 🎯 **STEP STRATEGICO** (Phase 4b della skill): property-based differential
> testing cross-implementation. Trova bug latenti che la suite differential
> statica non vede. Skill warning: range progressivo, niente range stretti.

## Goal

Implementare property-based testing comparativo tra rgrep e l'oracolo
(`grep` di sistema, ugrep su dev machine), via `proptest` crate. Tre
strategie minime:
1. **Literal patterns**: pattern senza meta, input arbitrario
2. **Anchored regex**: `^foo$`, `^bar`, `baz$`
3. **Character classes**: `[a-z]`, `[A-Z0-9]`, `[^xyz]`

Property: per ogni `(pattern, input)` random valido, `rgrep_output ==
oracle_output` modulo le divergenze documentate (D-NEW-1 ERE default,
D-NEW-3 color never).

D-NEW-2 (exit code) era prerequisito critico — fixato in Step 10.

## Riferimenti contratto

- Skill `legacy-port` Phase 4b "property-based testing"
- Skill warning: "range stretti perdono bug" (rawk orphan-dot bug)
- D-NEW-1, D-NEW-3 (divergenze documentate da skip nel proptest)
- Anchor differential testing: `tests/diff_runner.rs` (statico già 74 cases)

## D-decisioni (contratto)

### D 16.0 — Pattern relay-shell + format integrale
Stesso D 15.0. Body completo entrambi commit.

### D 16.1 — `proptest` come dev-dependency
```toml
[dev-dependencies]
proptest = "1.4"
```

### D 16.2 — Posizione del proptest
Nuovo file `rgrep/tests/proptest_differential.rs` (integration test
separato dal `diff_runner.rs` statico). Test gated da feature flag
opzionale **OPPURE** sempre on con seed deterministico per CI.

**Decisione**: sempre on, con `proptest!{}` macro standard. Default 256
casi per strategia. Test eseguito da `cargo test --test proptest_differential`.

### D 16.3 — 3 strategie obbligatorie

**Strategia 1: Literal patterns**
```rust
proptest! {
    #[test]
    fn prop_literal_match(
        pattern in "[a-z]{1,8}",        // pattern pure letters 1-8 chars
        input in "[a-z\n]{0,200}",       // input letters + newlines
    ) {
        let rgrep_out = run_rgrep(&[&pattern], input.as_bytes());
        let oracle_out = run_oracle(&[&pattern], input.as_bytes());
        prop_assert_eq!(rgrep_out, oracle_out);
    }
}
```

**Strategia 2: Anchored regex**
```rust
proptest! {
    #[test]
    fn prop_anchored(
        anchor in prop_oneof![Just("^"), Just("$"), Just("^.*$")],
        body in "[a-z]{0,5}",
        input in "[a-zA-Z0-9 \n]{0,150}",
    ) {
        let pat = format!("{}{}", anchor, body);
        let rgrep_out = run_rgrep(&[&pat], input.as_bytes());
        let oracle_out = run_oracle(&[&pat], input.as_bytes());
        prop_assert_eq!(rgrep_out, oracle_out);
    }
}
```

**Strategia 3: Character classes**
```rust
proptest! {
    #[test]
    fn prop_char_class(
        class_body in "[a-zA-Z0-9]{1,5}",
        negate in any::<bool>(),
        input in "[a-zA-Z0-9 \n]{0,150}",
    ) {
        let pat = if negate {
            format!("[^{}]", class_body)
        } else {
            format!("[{}]", class_body)
        };
        let rgrep_out = run_rgrep(&[&pat], input.as_bytes());
        let oracle_out = run_oracle(&[&pat], input.as_bytes());
        prop_assert_eq!(rgrep_out, oracle_out);
    }
}
```

### D 16.4 — Range progressivo (skill warning)
Per ogni strategia, partire con range "narrow" e poi espandere se non
trova bug:
- Iniziale: 256 casi, range come sopra
- Se passa: NON dichiarare vittoria, espandere a 1024 casi e/o range
  più ampio (es. literal pattern fino a 16 char, input fino a 500 char,
  classes con range Unicode `[À-ÿ]`)
- **Documentare** in `04a-divergences.md` ogni bug trovato come `D-NEW-N`

Skill: range troppo stretti perdono bug (rawk orphan-dot).

### D 16.5 — Skip divergenze note
Le divergenze D-NEW-1 (default ERE) e D-NEW-3 (color never) NON dovrebbero
manifestarsi nelle 3 strategie sopra (sono divergenze su CLI flag, non
su pattern matching).

Eventuali divergenze NUOVE → bug latenti da catalogare. NON cambiare
silenziosamente expected: ogni nuova divergenza è il deliverable
principale di Phase 4 (skill: "Divergenze NUOVE non previste sono il
deliverable più valuable").

### D 16.6 — Helper `run_rgrep` / `run_oracle`
Funzioni helper in `proptest_differential.rs`:
```rust
fn run_rgrep(args: &[&str], stdin: &[u8]) -> (i32, Vec<u8>) {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_rgrep"));
    cmd.args(args)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(2), out.stdout)
}

fn run_oracle(args: &[&str], stdin: &[u8]) -> (i32, Vec<u8>) {
    // come sopra ma Command::new("grep")
}
```

### D 16.7 — Skip se oracle non è GNU/ugrep
Se `grep --version` non riconosce GNU/ugrep, skip silenzioso del proptest
(testcase tag `#[cfg(...)]` non possibile dinamicamente, usare runtime
check + early return Ok).

### D 16.8 — Almeno 1 D-NEW-N nuova attesa
Skill: "Property-based testing trova bug latenti nell'originale che
l'analisi statica della Fase 1/2 non vede". Aspetto **almeno 1
divergenza nuova** documentata in `04a-divergences.md` (D-NEW-4 o
successiva).

Se proptest passa senza errori: documenta come "no nuove divergenze
trovate, range esteso a N casi senza failure" (skill `99-conclusions.md`
fa report).

### D 16.9 — File modificati attesi
```
rgrep/Cargo.toml                       (+ dev-dep proptest)
rgrep/Cargo.lock                       (auto-update)
rgrep/tests/proptest_differential.rs   (nuovo, ~150-250 LOC)
diary/04a-divergences.md                (eventuali D-NEW-N nuove)
diary/03-translation-log.md            (entry Step 16)
```

### D 16.10 — Build precondition
- `cargo build` 0 warning di codice (default)
- `cargo test` verde — 74 differential statici + N proptest cases (default 256 × 3 strategie = ~768 cases) + ≥29 unit
- Tempo proptest ragionevole: <30s su default range

### D 16.11 — Diary updates
- `diary/03-translation-log.md`: entry Step 16 — proptest infrastructure
- `diary/04a-divergences.md`: aggiungere D-NEW-N per ogni nuova divergenza
  (formato standard severità/categoria/causa/sintomo). Se nessuna nuova
  trovata, sezione "Step 16 results" con range esplorati.

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — 74 differential + ~768 proptest + ≥29 unit
- ✅ `tests/proptest_differential.rs` con 3 funzioni `proptest!` per le 3 strategie
- ✅ `proptest` come dev-dependency
- ✅ Diary 03 aggiornato con entry Step 16
- ✅ Diary 04 aggiornato (con o senza D-NEW-N nuove)
- ✅ DUE commit (rgrep + outer), entrambi format integrale, hash REALI

## Out-of-scope (debito esplicito)

- cargo-fuzz / libFuzzer (skill: per esperimenti metodologici proptest ha
  miglior costo/findings)
- Property: `parse(print(v)) == v` (non si applica a grep, no roundtrip)
- Range esteso che richiede >5min di esecuzione (Step 17 polish può
  estendere se serve)

## Commit attesi

In `rgrep/.git`:
```
feat(step16): differential proptest cross-implementation

IN-SCOPE:
- proptest dev-dependency (D 16.1)
- tests/proptest_differential.rs with 3 strategies (D 16.3):
  literal, anchored, character classes
- run_rgrep/run_oracle helpers (D 16.6)
- Default 256 cases × 3 = ~768 random differential checks (D 16.10)

OUT-OF-SCOPE (debito esplicito):
- cargo-fuzz (proptest has better cost/findings ratio for grep)
- Roundtrip property (not applicable to grep)
- Extended range > 5min (Step 17 polish)

Testcase aggiunti: 0 testcase TOML (3 proptest strategie). Totali: 74.
```

In `testag-grep/.git`:
```
docs(step16): translation log + Step 16 results in divergences

IN-SCOPE:
- diary/03-translation-log.md: proptest infrastructure entry
- diary/04a-divergences.md: D-NEW-N entries for any new divergences,
  or "Step 16 results: no new divergences found in N×3 cases"

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 74.
```

## Quando finisci

1. Forniscimi DUE blocchi shell:
   - rgrep: `cd rgrep && cargo build && cargo test --test proptest_differential
     && cargo test && git add -A && git commit -m "feat..."`
   - outer: `cd .. && git add diary/ && git commit -m "docs..."`
2. Verifica: tempo proptest <30s
3. Aggiorna l'header con hash REALI in entrambi i blocchi
4. **Stop**. Aspetta `audita`.

---

# ✅ Step 17 — Conclusions + metrics (FINAL) 🏁

**Stato**: ✅ APPROVED — audit chiuso 2026-05-03 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit outer**: `28c9b82`
🏁 **ESPERIMENTO testag-grep CHIUSO** — diary 99 + metrics deliverable finali.

> 🏁 **STEP FINALE** dell'esperimento. Phase 5 della skill `legacy-port`:
> sintesi metodologica + metriche quantitative. Il diario è il deliverable
> principale dell'esperimento (skill: "Se la suite di diary è incompleta,
> l'esperimento non è chiuso anche se il codice funziona").

## Goal

Produrre i due deliverable finali della skill `legacy-port`:
1. `diary/99-conclusions.md`: risposta alle **6 domande standard** della skill
2. `diary/metrics.md`: metriche quantitative dell'esperimento

Niente nuovo codice, niente nuovi testcase. Solo sintesi e numeri.

## Riferimenti contratto

- Skill `legacy-port` "Sintesi finale" — 6 domande standard
- AUDIT_LOG.md — fonte primaria delle metriche di processo
- Diary 01-04 — fonte primaria del contenuto narrativo

## D-decisioni (contratto)

### D 17.0 — Pattern relay-shell + format integrale
Stesso D 16.0. Body completo nei commit.

### D 17.1 — `99-conclusions.md`: 6 domande skill
Il file deve rispondere a queste 6 domande nell'ordine:

```markdown
# 99 — Conclusions

## §1 Dove l'AI ha funzionato meglio del previsto?
[Cite step-specifici: es. "Step 8 filter glob OR-logic — Gemini ha
 implementato globset semantics correctly al primo tentativo, 0 iterazioni"]

## §2 Dove ha fallito sistematicamente?
[Pattern recurring identificati nell'AUDIT_LOG: commit body malformed
 (Step 3, 12-bis), 2-commit-per-step (Step 6, 6-bis), build red committato
 (Step 7, 11), titolo wrong fix(stepN-hotfix) (Step 12, 14, 15), outer commit
 dimenticato (Step 12, 14, 15)]

## §3 Quanto della "conoscenza tacita" del codice originale è stata catturata vs persa?
[Catturata: API surface, dispatch matcher, edge cases §6 di 01.
 Persa: micro-ottimizzazioni page-aligned buffer (D-G4 → BufRead semplice),
 fallback DFA→Posix interno (D-G11 delegato a regex crate)]

## §4 Il workflow TDD inverso ha funzionato? Quale alternativa avresti provato?
[TDD-first applicato Step 0-15: testcase scritti prima di implementazione.
 Funzionato bene per binary handling, exit code, context. Alternative:
 inferring testcases da differential su gnu-grep/tests/ — out of scope]

## §5 Quanto è generalizzabile questo metodo ad altri progetti?
[Pattern documentati: relay-shell `&&` chain, fixture_files harness ext,
 D-NEW-N divergence cataloging, -bis recovery pattern. Applicabili a sed,
 awk (vedi rawk gemello), find]

## §6 Qual è il vero collo di bottiglia: capacità dell'AI, qualità dei prompt, o test suite di partenza?
[Bottleneck primario: **process discipline relay-shell**. La capacità AI
 di tradurre è elevata. La qualità delle D-decisioni dell'Architect ha
 limitato 1-2 casi (Step 7 brace fix, Step 11 mock). Il loop relay-shell
 con incolla cieca è il vero rischio: build red commit. CLAUDE.md
 "Pattern relay-shell" lo mitiga ma non elimina]
```

### D 17.2 — Sintesi quantitativa in 99-conclusions.md
Sezione finale `## §7 Sintesi quantitativa`:
- Step totali: 17 principali + 8 -bis = 25
- Verdetto: N ✅ / M 🟡 / 0 ❌
- Tempo totale interazione: ~Xh (stima)
- LOC tradotte: ~750 → ~1500 Rust (TBD da metrics)
- Differential testcase: 74 statici + 3072 proptest = ~3146 checks
- D-NEW-N: 4 (D-NEW-1 ERE default, D-NEW-2 exit code fixed, D-NEW-3 color never, D-NEW-4 no findings proptest)

### D 17.3 — `metrics.md`: numeri quantitativi
Tabella + lista in formato markdown:

```markdown
# metrics — testag-grep

## LOC

| Source | LOC |
|---|---|
| Pre-experiment Rust (rgrep baseline) | ~741 |
| Post-experiment Rust | ~XXXX |
| Net additions | ~YYYY |
| GNU grep upstream (src/*.c) | ~4500 |
| Compressione C → Rust | ~70% |

## Test

| Metric | Value |
|---|---|
| Differential testcase statici | 74 |
| Proptest cases (3 strategies × 1024) | 3072 |
| Unit test totali | XX |
| Test green rate | 100% |

## Process

| Metric | Value |
|---|---|
| Step totali | 17 main + 8 -bis = 25 |
| ✅ APPROVED | XX |
| 🟡 PARTIAL recovered | XX |
| ❌ REJECTED | 0 |
| Commit rgrep | XX |
| Commit outer | XX |

## Divergenze

| ID | Categoria | Status |
|---|---|---|
| D-NEW-1 | choice_design (ERE default) | open (accettata) |
| D-NEW-2 | bug latent (exit code) | fixed Step 10 |
| D-NEW-3 | choice_design (color never) | open (accettata) |
| D-NEW-4 | proptest no findings | informational |

## Tempo

| Phase | Stima |
|---|---|
| Phase 1 (semantic model, Step 1) | ~1.5h |
| Phase 2 (mapping, Step 2) | ~2h |
| Phase 3 (impl Step 3-15) | ~10-15h |
| Phase 4 (proptest Step 16) | ~2h |
| Phase 5 (this conclusions Step 17) | ~1h |
| **Totale** | **~17-22h** |
```

L'Implementer raccoglie i numeri reali da:
- `git log --oneline --all | wc -l` (commits totali)
- `wc -l rgrep/src/*.rs` (LOC Rust)
- `cargo test 2>&1 | grep "test result"` (test count)
- AUDIT_LOG.md (verdetti per step)
- Stima tempo basata sui timestamp git

### D 17.4 — Polish opzionale (out-of-scope se troppo)
Se rimane tempo:
- Allineare i nomi delle strategie A-F skill (vedi nota Step 2 audit:
  Gemini ha usato custom labels). Refactor 02-mapping-table.md §2 con
  i nomi canonici (Adapter, Shim, Full port, ...).
- Fix D-G11 cross-ref errato in §3 SAME_INODE row (vedi Step 2 audit
  note minore).

Skip se complica. Non bloccante.

### D 17.5 — File modificati attesi
```
diary/99-conclusions.md   (NUOVO)
diary/metrics.md          (NUOVO)
```

Eventualmente:
- `diary/02-mapping-table.md` (polish opzionale D 17.4)

NIENTE codice rgrep toccato. NIENTE nuovi testcase.

### D 17.6 — Build precondition
- `cargo build` 0 warning di codice (regressione zero)
- `cargo test` verde — 33+ test (regressione zero)

### D 17.7 — Commit nel repo outer SOLO
Nessun commit in rgrep (no code changes). UN commit outer per i due
deliverable finali.

## Acceptance criteria

- ✅ `diary/99-conclusions.md` esiste con le 6 sezioni + §7 quantitativa
- ✅ `diary/metrics.md` esiste con LOC, Test, Process, Divergenze, Tempo
- ✅ Numeri reali, non placeholder (es. LOC count effettivo, non `XXXX`)
- ✅ UN commit outer, format integrale, hash reale
- ✅ `cargo test` ancora verde (regressione zero verificata)

## Out-of-scope (debito esplicito)

- Polish §2 mapping table label A-F (rinviato/scartato)
- Performance benchmark vs `grep` (mai in questo esperimento)
- Crate publish su crates.io
- CI integration (.github/workflows)

## Commit atteso

UN commit nel repo outer:
```
docs(step17): final conclusions + experiment metrics

IN-SCOPE:
- diary/99-conclusions.md: 6 standard skill questions answered
- diary/metrics.md: LOC, test count, process verdicts, divergences, time

OUT-OF-SCOPE (debito esplicito):
- Phase 5 polish (label A-F refactor in 02 mapping)
- No code changes
- No new testcases

Testcase aggiunti: 0. Totali: 74 differential + 3 proptest strategies.

🏁 Esperimento testag-grep CHIUSO.
```

## Quando finisci

1. Forniscimi UN blocco shell (solo outer)
2. Verifica `cargo test` ancora verde (regressione zero)
3. Aggiorna l'header con hash REALE
4. **Stop**. Aspetta `audita`. Questo è l'audit di chiusura
   dell'esperimento.

---

# ✅ Step 18 — Import & convert gnu-grep/tests/ to TOML differential 🏁

**Stato**: ✅ APPROVED (note minori) — audit chiuso 2026-05-04 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `78aac88` — **Commit outer**: `9d38bc2`
🏁 **ESPERIMENTO RICHIUSO** — D-NEW-5 catalogata, 54 testcase importati, 12 bug noti (debito Step 19 future).

> 🔄 **ESPERIMENTO RIAPERTO** dopo Step 17. Decisione del Decider:
> chiudere il gap del §4 di `99-conclusions.md` (alternativa inesplorata
> = importare `gnu-grep/tests/`).

## Goal

Convertire (in modo automatizzato + curated) i ~129 shell test scripts di
`gnu-grep/tests/` nel formato TOML differenziale. Espandere la suite da
74 a 200-500+ testcase. Catalogare le **nuove divergenze** scoperte come
`D-NEW-5..N` in `04a-divergences.md`.

L'esperimento riapre Phase 4 (differential testing) con un dataset 5×
maggiore, sfruttando la suite ufficiale GNU come oracolo strutturato.

## Riferimenti contratto

- 99-conclusions.md §4 (alternativa inesplorata, ora esplorata)
- Skill `legacy-port` Phase 4: differential testing strategy
- Step 16 D-NEW-4: "no findings after 3072 random inputs" — Step 18
  espande con input strutturato dalla GNU testsuite

## D-decisioni (contratto)

### D 18.0 — Pattern relay-shell + format integrale
Stesso D 17.0. Body completo nei commit. **DUE blocchi shell** separati
(rgrep + outer) come da pattern Step 15-bis lesson.

### D 18.1 — Strumento di conversione
Implementare uno script di conversione (Python o Rust binario) in
`rgrep/tools/convert_gnu_tests.py` (o `tools/convert_gnu_tests.rs`).

**Decisione lingua**: Python preferito perché:
- Parser shell ad-hoc richiede meno boilerplate
- Non aggiunge crate alla dipendenza prod
- Veloce da iterare
- Non finisce nel binary distribuito

**Logica del parser** (semplificato):
```python
# Per ogni file in gnu-grep/tests/*:
# 1. Skip se file non è shell script (tests, awk, pm, etc)
# 2. Skip se file usa init.sh framework esoterico (path_prepend_, srcdir,
#    LC_ALL=ja_JP) — pattern di esclusione listato in D 18.5
# 3. Per ogni blocco "echo|printf X | grep ARGS [...] [&&|;] check":
#      - Estrai stdin (X), args (ARGS), expected_exit_code
#      - Genera TOML in tests/cases/0NNN_<scriptname>_<seq>.toml
# 4. Auto-skip se richiede file fixture esterni non riproducibili
```

### D 18.2 — Numerazione TOML continuativa
Continuare dalla numerazione corrente (74 testcase ora) → 0075-0XXX.
Se risultano >500 testcase, considerare prefisso semantico
(`gnu_NNNN_<name>.toml` invece di solo `NNNN_<name>.toml`).

**Decisione**: usare prefisso `gnu_` per i testcase importati (per
distinguerli dai 74 curati a mano). Es. `tests/cases/gnu_0001_backref_seq1.toml`.

### D 18.3 — Format TOML invariato
Mantenere lo stesso schema dei testcase esistenti (D 0.3 + extension D 4.6
fixture_files + D 5.9 placeholder + D 6.9 env). Niente nuovo schema.

Aggiungere campo opzionale `source` per tracciare l'origine:
```toml
source = "gnu-grep/tests/backref:line17-21"
```
Per debug: se un testcase fallisce, sai da dove viene.

### D 18.4 — Skip-list automatica (D 18.5)
Pattern di esclusione automatica nello script di conversione:

| Pattern script | Motivo skip |
|---|---|
| Usa `path_prepend_` o `srcdir=` | init.sh framework GNU-only |
| `LC_ALL=` o `LANG=` con valore non-C | locale-specific (skip Step 18; Step 19 future) |
| Usa `get-mb-cur-max` | binary helper specifico |
| Compara file binari output con `cmp` o `od` | richiede setup riproducibile |
| Usa `${SHELL}` interpolation strana | non portabile |
| Manipola `IFS` o env esoteriche | shell-specific |
| Pattern multibyte specifico (`fr_FR.UTF-8`) | locale skip |

Lo script emette per ogni file skippato:
```
[skip] tests/word-multibyte.sh: locale-specific multibyte (fr_FR)
[skip] tests/get-mb-cur-max.sh: requires C helper binary
[ok]   tests/backref.sh: 4 testcase generated → gnu_0075..0078
```

### D 18.5 — Curation manuale leggera
Dopo conversione automatica, l'Implementer:
1. Esegue `cargo test --test diff_runner` con i nuovi testcase
2. Per ogni testcase fallito (DiffOutcome ≠ Match), classifica:
   - **A**: bug reale rgrep → marca `expected_to_fail = true` con motivo
     "rgrep bug, fix in Step 19"
   - **B**: divergenza ugrep oracle (no GNU strict) → marca
     `skip_if_bsd = true` + `skip_reason`
   - **C**: parsing errato del converter → fix manuale TOML o skip
   - **D**: nuova divergenza meritevole di catalogazione → entry
     `D-NEW-N` in `04a-divergences.md` + skip_if_bsd

NON modificare expected_stdout silenziosamente (anti-pattern, vedi
SPEC-Q rule).

### D 18.6 — Numero target di testcase generati
Target: **almeno 200 nuovi testcase TOML accepted**. Se la conversione
automatica produce <200, è OK comunque (acceptance criterion sulla
numerosità è soft — il vero criterio è la qualità + nuove divergenze
trovate).

Se conversione produce >500, ragionevole skippare i casi più esotici
per evitare bloat (max 500 in suite, eccedenza in `_archive/gnu_tests_overflow/`).

### D 18.7 — Almeno 1 D-NEW-N nuova attesa
Skill: importare gnu-grep/tests/ è esattamente il tipo di estensione che
**dovrebbe** rivelare divergenze nuove (parametri esotici, edge case del
parser regex, locale handling che proptest non tocca).

Se 0 nuove divergenze: documentare come "rgrep parity ≥X% confermata su
suite ufficiale GNU di N test" (formato simile a D-NEW-4 di Step 16).

### D 18.8 — File modificati attesi
```
rgrep/tools/convert_gnu_tests.py            (NUOVO, lo script)
rgrep/tools/README.md                       (NUOVO, come usarlo)
rgrep/tests/cases/gnu_0001_*.toml           (N nuovi)
rgrep/tests/testsuite.toml                  (+N entries)

(eventuale)
rgrep/src/runner.rs                         (SOLO se servono fix per testcase
                                             che rivelano bug correggibili
                                             in scope semplice — vedi
                                             classificazione A in D 18.5)
diary/03-translation-log.md                 (entry Step 18)
diary/04a-divergences.md                     (D-NEW-5..N)
```

### D 18.9 — Build precondition
- `cargo build` 0 warning di codice (regressione zero)
- `cargo test` verde — 74 originali + N gnu_ + 3 proptest strategies + ≥32 unit
- Tempo `cargo test` ragionevole — se si gonfia (>60s), scope-out
  gli ultimi 100+ testcase più lenti

### D 18.10 — Diary updates
- `diary/03-translation-log.md`: entry Step 18 con conversion stats
  (N tentati, N skipped, N accepted, N nuove divergenze)
- `diary/04a-divergences.md`: ogni nuova D-NEW-N con severità/categoria/causa
- (opzionale) `diary/99-conclusions.md`: appendice "Step 18 reopen" con
  meta-osservazione sull'estensione

### D 18.11 — Gestione testcase falliti come `expected_to_fail`
Per i testcase che falliscono per **bug reale** (classe A):
- `expected_to_fail = true` + `expected_to_fail_reason = "..."`
- Tracking: lista in `diary/03-translation-log.md` Step 18 entry
- Roadmap: ogni `expected_to_fail = true` è un debito tecnico per Step 19
  o successivi

Per i testcase falliti per divergenza oracle (classe B):
- `skip_if_bsd = true` + `skip_reason`
- Documenta in `04a-divergences.md` se la divergenza è interessante

## Acceptance criteria

- ✅ `tools/convert_gnu_tests.py` esiste e produce TOML
- ✅ Almeno 200 nuovi testcase `gnu_*.toml` accepted (skip_if_bsd o
  expected_to_fail compresi)
- ✅ `cargo test` verde — 74 originali + ≥200 gnu_ + ≥32 unit
- ✅ Almeno 1 D-NEW-N nuova in `04a-divergences.md` (oppure giustificazione
  "≥X% parity confermata, no findings nuovi")
- ✅ `tools/README.md` spiega come (re)eseguire la conversione
- ✅ Diary 03 + 04 aggiornati
- ✅ DUE commit (rgrep + outer), entrambi format integrale

## Out-of-scope (debito esplicito)

- Implementare `init.sh` adapter per i test che lo richiedono (Step 19)
- Locale-aware testing (Step 19 o Step 20)
- Test che richiedono `get-mb-cur-max` C helper
- Fix di bug reali rivelati dai testcase importati (classe A) — entrano
  in step successivi (Step 19+)
- Push del repo GitHub (decisione separata del Decider; questo step
  produce solo locale)

## Commit attesi

In `rgrep/.git`:
```
feat(step18): import gnu-grep/tests/ to TOML differential format

IN-SCOPE:
- tools/convert_gnu_tests.py: shell-to-TOML converter (D 18.1)
- tools/README.md: usage guide
- tests/cases/gnu_*.toml: N converted testcases (D 18.2)
- tests/testsuite.toml: +N manifest entries
- Skip-list heuristics for non-convertible scripts (D 18.4)
- Auto-classification of failures: bug / oracle-divergence / parse-error (D 18.5)

OUT-OF-SCOPE (debito esplicito):
- init.sh framework adapter (Step 19+)
- Locale-aware test handling
- Real bug fixes revealed by imported tests (deferred)

Testcase aggiunti: N. Totali: 74 + N + 3 proptest = ...
```

In `testag-grep/.git`:
```
docs(step18): translation log + new divergences from gnu-grep test import

IN-SCOPE:
- diary/03-translation-log.md: Step 18 conversion stats
- diary/04a-divergences.md: D-NEW-5..N entries (or no-findings note)

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: ...
```

## Quando finisci

1. Forniscimi DUE blocchi shell separati (rgrep + outer)
2. Verifica `cargo test` verde con tempo ragionevole (<60s preferibile)
3. Verifica `find rgrep/tests/cases -name 'gnu_*.toml' | wc -l` ≥ 200
4. Aggiorna l'header con hash REALI in entrambi
5. **Stop**. Aspetta `audita` per il verdetto finale Step 18.

---

# ✅ Step 19 — Fix dei 12 bug noti (backref support + D-NEW-5) 🔧

**Stato**: ✅ APPROVED — audit chiuso 2026-05-04 (vedi [AUDIT_LOG.md](AUDIT_LOG.md))
**Commit rgrep**: `b9e797e` — **Commit outer**: `be7be70`
🔧 **12 bug chiusi**: D-NEW-5 fixed + 9 backref via fancy-regex fallback. 100% testcase verdi (0 expected_to_fail).

> 🔧 **STEP DI FIX**: chiude i 12 `expected_to_fail = true` testcase
> aperti in Step 18. Due classi di bug:
> - **9 bug Backreferences (D-M6)**: regex crate non supporta `\1..\9`
> - **3 bug D-NEW-5**: validation `-e` multi mascherato

## Goal

Trasformare i 12 testcase `expected_to_fail = true` (gnu_0075..0112) in
testcase **passing** rimuovendo i bug rgrep sottostanti. Aggiornare
D-NEW-5 status a "fixed Step 19" + (se applicabile) aggiornare D-M6 con
"backreferences supported via fancy-regex fallback".

## Riferimenti contratto

- D-M6 (regex compile/exec, attualmente "no backref support")
- D-NEW-5 (catalogata in 04a-divergences.md, status "open / debt Step 19")
- 12 testcase `gnu_*.toml` con `expected_to_fail = true`
- 02-mapping-table.md §2.2 (DFA Engine, dfa_comp → Engine::Regex)

## D-decisioni (contratto)

### D 19.0 — Pattern relay-shell + format integrale
DUE blocchi shell separati (rgrep + outer). Body integrale entrambi commit.
Hash reali nell'header.

### D 19.1 — Fix D-NEW-5: validazione `-e` per pattern individuale
**Bug attuale**: in `Matcher::new`, i patterns multipli da `-e` vengono
joined con `|` PRIMA della compilazione. Risultato: `-e "[" -e "]"` →
pattern unico `[|]` (char class valida con un solo carattere `|`).

**Fix**: validare ogni pattern individualmente PRIMA di joining:
```rust
fn compile_patterns(patterns: &[String], opts: &MatcherOpts) -> Result<Engine, GrepError> {
    // 1. Try compile each pattern individually as syntax check
    for pat in patterns {
        if Regex::new(pat).is_err() && fancy_regex::Regex::new(pat).is_err() {
            return Err(GrepError::InvalidRegex(pat.clone()));
        }
    }
    // 2. Now join with | for actual execution (efficiency)
    let joined = patterns.iter().map(|p| format!("(?:{})", p)).collect::<Vec<_>>().join("|");
    // ... compile joined ...
}
```

**Variante**: se preferisci compilare separatamente e iterare sui matcher,
va bene — l'output finale deve essere lo stesso. La cosa cruciale è
che la **validation è per-pattern**.

### D 19.2 — Fix D-M6: support backreferences via fancy-regex
**Bug attuale**: pattern come `\(.\)\(.\).\2\1` o `(.+)\1` falliscono perché
il `regex` crate (RE2-style) non supporta backreferences (per ragioni di
performance lineare).

**Fix**: aggiungere `fancy-regex` come dipendenza (pure Rust, no C deps,
supporta backref + lookaround). Auto-fallback runtime:

```rust
[dependencies]
fancy-regex = "0.13"  # pure Rust, supports backref/lookaround
```

Pattern di compilazione:
```rust
pub enum Engine {
    Regex(regex::Regex),
    Fancy(fancy_regex::Regex),  // NUOVO
    AhoCorasick(aho_corasick::AhoCorasick),
    #[cfg(feature = "perl-regexp")]
    Pcre2(pcre2::bytes::Regex),
}

fn try_compile(pat: &str, opts: ...) -> Result<Engine, GrepError> {
    // First: try regex crate (fast RE2)
    match regex::Regex::new(pat) {
        Ok(r) => Ok(Engine::Regex(r)),
        Err(_) => {
            // Fallback: try fancy-regex (supports backref)
            match fancy_regex::Regex::new(pat) {
                Ok(r) => Ok(Engine::Fancy(r)),
                Err(e) => Err(GrepError::InvalidRegex(e.to_string())),
            }
        }
    }
}
```

**Trade-off**: i pattern con backref saranno più lenti (fancy-regex usa NFA
backtracking, non DFA). Acceptable: GNU grep stesso usa NFA per backref.

### D 19.3 — `is_match` / `find` per Engine::Fancy
Implementare per la nuova variante:
- `is_match(line)` → `fancy_regex::Regex::is_match(line)?`
- `find(line) -> Option<Range<usize>>` → `fancy_regex::Regex::find(line)?.map(|m| m.start()..m.end())`
- Nota: fancy-regex API ritorna `Result<Option<Match>, Error>`, gestire l'errore (raro a runtime se compilation passata)

### D 19.4 — BRE backref support via translation
Per `-G` (BRE), il translator `bre_to_ere` deve preservare i backref:
- BRE: `\(group\) \1` → ERE: `(group) \1` (parentesi sostituite, backref invariato)
- Già fatto? Verificare in `bre_to_ere` esistente. Se backref non passa attraverso correttamente, fix.

### D 19.5 — Rimuovere `expected_to_fail = true` dai 12 testcase
Dopo il fix:
- I 12 testcase passano → rimuovere `expected_to_fail = true` E `expected_to_fail_reason`
- Lasciare `source = "gnu-grep/tests/..."` (utile per tracking)

Alla fine: `grep -c "expected_to_fail = true" tests/cases/gnu_*.toml` → **0**

### D 19.6 — Aggiornare D-NEW-5 in 04a-divergences.md
Aggiungere status:
```markdown
- **Soluzione adottata**: validazione per-pattern in `Matcher::new` prima
  del joining. Compile attempt isolato per ogni `-e` pattern; primo
  fallimento → exit 2 (parity GNU). Fixed in **Step 19**.
- **Status**: chiuso ✅
```

### D 19.7 — Aggiornare D-M6 in 02-mapping-table.md
La decisione D-M6 originale ("regex non valide → process::exit(2)") è
ancora valida, ma va aggiunto:
```markdown
- **D-M6 (revisione Step 19)**: Backreferences `\1..\9` supportate via
  fallback automatico al crate `fancy-regex` quando il `regex` crate
  rifiuta il pattern. Trade-off: pattern con backref usano NFA (slower)
  ma supportati. Cleanup del 12 expected_to_fail in `gnu_*.toml`.
```

### D 19.8 — Testcase obbligatori (TDD-first per il fix di D-NEW-5)
Aggiungere uno smoke test esplicito per D-NEW-5 (separato dai gnu_):
```toml
# tests/cases/0075_multi_e_invalid_validation.toml  (riusa numero libero)
name = "multi -e invalid pattern validation"
args = ["-e", "[", "-e", "]"]
stdin = "anything\n"
expected_stdout = ""
expected_exit_code = 2
source = "regression test for D-NEW-5 fix"
```

Nota: il numero `0075` è già usato per il primo gnu_ — usare il prossimo
libero (`0073` o `0074` se liberi nei 0001-0074).

Verifica con `ls tests/cases/00*.toml | wc -l` quale numero è libero.
**Decisione**: aggiungere come `tests/cases/0075_multi_e_invalid_validation.toml`
solo se la numerazione 0001-0074 è completa; altrimenti usa il primo
libero. Sai la convenzione: zero-padded 4 cifre.

### D 19.9 — File modificati attesi
```
rgrep/Cargo.toml                          (+ fancy-regex = "0.13")
rgrep/Cargo.lock                          (auto-update)
rgrep/src/matcher.rs                      (Engine::Fancy variant + impl, validation)
rgrep/src/runner.rs                       (eventuale: dispatch update)
rgrep/tests/cases/gnu_0075..0112*.toml    (12 file: rimuovere expected_to_fail)
rgrep/tests/cases/00NN_multi_e_*.toml     (1 nuovo testcase di regressione)
rgrep/tests/testsuite.toml                (+1 entry)

diary/02-mapping-table.md                 (revisione D-M6)
diary/04a-divergences.md                   (D-NEW-5 status closed)
diary/03-translation-log.md               (entry Step 19)
```

### D 19.10 — Build precondition
- `cargo build` 0 warning di codice
- `cargo test` verde — i 12 ex-`expected_to_fail` ora **passing** + il
  nuovo testcase regressione D-NEW-5

### D 19.11 — Unit test
Aggiungere ≥3 unit test separati per:
- `test_backref_simple` (es. `(.+)\1` matcha "abab")
- `test_backref_named_in_pattern` (es. `(\d+)-\1` matcha "42-42")
- `test_multi_e_invalid_validation` (es. `compile_patterns(&["[", "]"])` ritorna Err)

### D 19.12 — Diary update
- `03-translation-log.md`: entry Step 19 (12 fix + fancy-regex fallback)
- `04a-divergences.md`: D-NEW-5 status → fixed
- `02-mapping-table.md`: revisione D-M6

## Acceptance criteria

- ✅ `cargo build` 0 warning di codice
- ✅ `cargo test` verde — TUTTI i 12 ex-`expected_to_fail` ora passing
- ✅ `grep -c "expected_to_fail = true" tests/cases/gnu_*.toml` → **0**
- ✅ Nuovo testcase regressione D-NEW-5 passing
- ✅ `fancy-regex` aggiunto a Cargo.toml
- ✅ `Engine::Fancy` variant implementato
- ✅ Validation per-pattern di `-e` multi
- ✅ Diary 02 + 03 + 04 aggiornati
- ✅ DUE commit (rgrep + outer), entrambi format integrale

## Out-of-scope (debito esplicito)

- Performance optimization di fancy-regex (slower del regex DFA, OK)
- Lookaround/lookbehind via fancy-regex per pattern -E (non in scope se
  non richiesto dai 12 testcase)
- Fix di altri bug latenti scoperti durante l'implementazione (open
  ticket per Step 20 se necessario)

## Commit attesi

In `rgrep/.git`:
```
feat(step19): fix 12 known bugs (backref support + D-NEW-5 validation)

IN-SCOPE:
- Add fancy-regex dependency for backref support (D 19.2)
- Engine::Fancy variant with auto-fallback compile order (D 19.3)
- Per-pattern validation in -e multi to fix D-NEW-5 (D 19.1)
- Remove expected_to_fail=true from 12 gnu_ testcases (D 19.5)
- New regression testcase for D-NEW-5
- N unit tests for backref + multi-e validation

OUT-OF-SCOPE (debito esplicito):
- fancy-regex performance vs regex DFA
- General lookaround support outside backref tests
- Newly discovered bugs (Step 20 if needed)

Testcase aggiunti: 1. Totali: 129 (+12 da expected_to_fail → passing).
```

In `testag-grep/.git`:
```
docs(step19): close D-NEW-5, revise D-M6 backref support

IN-SCOPE:
- diary/04a-divergences.md: D-NEW-5 status updated to fixed
- diary/02-mapping-table.md: D-M6 revised (fancy-regex fallback)
- diary/03-translation-log.md: Step 19 entry

OUT-OF-SCOPE (debito esplicito):
- No code changes

Testcase aggiunti: 0. Totali: 129.
```

## Quando finisci

1. Forniscimi DUE blocchi shell (rgrep + outer) con `&&` chain
2. Verifica `grep -c "expected_to_fail = true" rgrep/tests/cases/gnu_*.toml` → 0
3. Aggiorna l'header con hash REALI
4. **Stop**. Aspetta `audita`.

---

## Backlog (post-Step 17, non promossi)

Items che possono diventare step se l'esperimento si estende:

1. CI integration (GitHub Actions cross-platform: macOS BSD grep + Ubuntu GNU grep)
2. Benchmark suite vs `grep` su file >1GB
3. Fuzz integration (`cargo-fuzz`) — solo se motivato da divergenza non risolta
4. Crate publish su crates.io (richiede licenza review)

---

# ✅ Ondata 1 — Clippy auto-fix + cargo fmt — APPROVED

**Stato**: ✅ APPROVED — 2026-05-18 08:55 GMT+2
**Commit rgrep**: `106a138` — `refactor(lint): clippy --fix + cargo fmt`
**Commit outer**: pending (submodule bump + questa sezione + AUDIT_LOG verdict)
**Sessione Implementer**: 2026-05-18 (Claude Opus 4.7 in ruolo Implementer fuori-workflow)
**Sessione Architect-Auditor**: 2026-05-18 (Claude Opus 4.7 — verdict 8/8 punti ✅, vedi `AUDIT_LOG.md` → "Verdict Architect-side")

> ⚠️ **Step fuori dal workflow originario di porting** (Step 0..19 chiusi).
> Non aggiunge feature, non tocca diff_runner né testsuite. Solo lint
> cleanup post-porting.
>
> Trigger: Decider ha richiesto valutazione aderenza a Rust syntax/style/
> pattern. Output: report multi-livello (clippy default + pedantic +
> nursery + meta-cognition L1/L2/L3 routing). Ondata 1 = quick-wins
> auto-applicabili.

## Goal Ondata 1 (eseguito)

Applicare i fix auto-suggeriti da `cargo clippy --fix` (lint level default,
non pedantic) + `cargo fmt` sul workspace `rgrep/`. Mantenere semantica
invariata, verificare zero regressioni nel test suite.

## Contratto applicato

| Vincolo | Stato |
|---|---|
| Build verde pre & post | ✅ `cargo build` 0 errori, 0 warning di codice |
| Test verdi pre & post | ✅ 36/36 (5 suites, 3.97s) — identico al baseline |
| Un singolo commit | ✅ `106a138` |
| Format commit obbligatorio | ✅ IN-SCOPE / OUT-OF-SCOPE / Testcase counters |
| Niente scope creep | ✅ Solo clippy --fix + fmt, niente refactor manuale |

## Diff summary

```
src/cli.rs                     |  49 ++++-
src/lib.rs                     |   2 +-
src/main.rs                    |   2 +-
src/matcher.rs                 | 127 ++++++++-----
src/output.rs                  |  13 +-
src/runner.rs                  | 403 +++++++++++++++++++++++++++++------------
tests/diff_runner.rs           | 102 +++++++----
tests/proptest_differential.rs |  45 ++---
8 files changed, 524 insertions(+), 219 deletions(-)
```

99% del diff = fmt-reflow (long-line break, multi-line struct, import
reorder, trailing whitespace). 1% = un fix semantico: `manual_flatten` in
[rgrep/src/matcher.rs](../../src/matcher.rs):

```rust
// PRIMA
for mat_res in re.find_iter(line) {
    if let Ok(mat) = mat_res {
        result.push_str(&line[last_match..mat.start()]);
        ...
    }
}

// DOPO
for mat in re.find_iter(line).flatten() {
    result.push_str(&line[last_match..mat.start()]);
    ...
}
```

## D-decisioni implicite (zero in questa ondata)

Niente nuove decisioni di design. Tutti i cambi sono auto-rewrites di
clippy default (no `-W clippy::pedantic`, no `-W clippy::nursery`).

## Lint snapshot

| Livello | Pre | Post |
|---|---|---|
| `cargo clippy` (default) | ~28 actionable | 2 strutturali (richiedono Ondata 2) |
| `cargo clippy -W pedantic -W nursery` | 102 | ~50 (richiedono Ondata 2-3) |

I 2 warning default residui:
- `too_many_arguments` su `print_line` in `rgrep/src/runner.rs:466` (9/7) —
  risolvibile con struct `PrintCtx` (Ondata 2)
- `if_same_then_else` su `rgrep/src/runner.rs:336-339` — branch identici
  per `null/not-null`, richiede review semantica (Ondata 2)

## Checklist Audit (per Architect)

- [ ] Verificare che `cargo build` + `cargo test` siano verdi su `106a138`
- [ ] Spot-check diff su matcher.rs (manual_flatten semantic-preserving)
- [ ] Verificare format commit `106a138` integrale (IN-SCOPE/OUT-OF-SCOPE)
- [ ] Differential `tests/diff_runner.rs` regressione zero (deve restare 36/36)
- [ ] Decidere prossimo step:
  - **(a)** Promuovi Ondata 2 (refactor strutturale: `get_*` rename,
    `thiserror`, no `process::exit` in lib, split `bufread_search`)
  - **(b)** Promuovi Ondata 3 (architettura: `Config` sub-struct, `Engine`
    trait, `Cow<str>`)
  - **(c)** Chiudi l'esperimento di lint qui — pedantic warning rimanenti
    sono "noise" accettabile per progetto educational

## OUT-OF-SCOPE (Ondata 2-3, pending decisione Decider)

**Ondata 2 — Refactor strutturale (~1h)**
- Naming: `get_after_context` / `get_before_context` / `get_binary_action`
  → senza prefisso `get_` (richiede gestione collisione con field
  `pub after_context: usize`)
- Error type: `Box<dyn Error>` (7 occorrenze) → `thiserror::Error` enum
  tipizzato in nuovo `rgrep/src/error.rs`
- Anti-pattern: `std::process::exit(2)` in `rgrep/src/matcher.rs:75` →
  `return Err(RgrepError::PcreUnavailable)`
- Splits: `bufread_search` (141 LOC, cognitive 41) → estrarre
  `MatchContext { history, print_after, last_printed_line, match_count }`
- `print_line` (9 args) → introdurre `struct PrintCtx<'a>`
- Pedantic: `#[must_use]` su getter (9 occ), `derive(Eq)` (4 occ),
  `# Errors` doc (4 occ), format!() inline `{var}` (35 occ)

**Ondata 3 — Architettura (~2-3h)**
- `Config` flat-bag (30 bool) → sub-struct con `#[command(flatten)]`
  (PatternOpts, OutputOpts, ContextOpts, FilterOpts, BinaryOpts)
- `Engine` enum → trait sealed o `enum_dispatch` per eliminare dispatch
  duplicato 3× (is_match, highlight, find_match_offsets)
- `highlight()` return → `Cow<'a, str>` per evitare alloc-per-riga in
  caso `--color=never` (default)
- `// SAFETY:` comment su blocco mmap in `rgrep/src/runner.rs:168`

## Note metodologiche

Eseguito in modalità Implementer single-shot (Claude Opus 4.7 in entrambi
i ruoli — fuori dal pattern relay-shell standard). Skill loadate:
`rust-skills:coding-guidelines`, `rust-skills:m15-anti-pattern`,
`rust-skills:domain-cli` (meta-cognition L1+L2+L3 tracing applicato).

L'esecuzione è stata diretta (no Gemini) perché l'Ondata 1 è 100%
clippy/rustfmt-driven, quindi riproducibile e auto-verificabile. Le
Ondate 2-3 richiederanno decisioni di design (D-Ondata-2.k, D-Ondata-3.k)
e dovrebbero passare per il pattern Architect-spec → Implementer-impl
standard.

---

## Note operative per l'Implementer (Gemini)

1. **Leggi solo lo step 🚧 PRONTO**, ignora gli step 🔒 LOCKED.
2. **Aggiungi i testcase PRIMA dell'implementazione** (TDD-first). Verifica
   che i nuovi test falliscano per il motivo atteso prima di implementare.
3. **Un solo commit per step.** Se ti accorgi a metà che lo step è troppo
   grande, ferma, lascia un `// DESIGN-Q:` e tagga `🟡 BLOCKED — DESIGN-Q`.
4. **Mai modificare `expected_*` di un testcase senza SPEC-Q.** Se un test
   sembra avere expected sbagliato, lascia commento `<!-- SPEC-Q: ... -->` e
   tagga lo step `🟡 BLOCKED — SPEC-Q`.
5. **Build green obbligatorio.** Se non riesci a far passare tutto, NON
   committare un parziale rosso. Tagga `🟡 BLOCKED` e descrivi il blocco.
6. **Format commit message obbligatorio.** Niente eccezioni.
7. **Quando finisci uno step**: aggiorna l'header di QUESTO file (`🚧` → `🟢
   FATTO — AUDIT PENDING — commit <hash>`) e fermati. Aspetta `audita` dal
   Decider.

## Note operative per il Decider (utente)

- Trigger principale: **`audita`** → Architect esegue audit dello step in 🟢 status.
- Se l'Implementer apre 🟡 BLOCKED, mandami il tipo (DESIGN-Q o SPEC-Q) e il commit hash.
- Mai copiare il diff dell'Implementer all'Architect: il git log basta.

---

# ✅ Ondata 2 — Refactor strutturale — APPROVED — commit 25e6817

**Stato**: ✅ APPROVED (8/8) — audit chiuso 2026-05-18 10:20 GMT+2 (vedi `AUDIT_LOG.md` → "Audit Ondata 2")
**Promosso**: 2026-05-18 dall'Architect (dopo audit ✅ Ondata 1, vedi `AUDIT_LOG.md` → "Verdict Architect-side")
**Predecessore**: Ondata 1 (`106a138` rgrep, ✅ APPROVED 8/8 punti)
**Implementer**: Claude Opus 4.7 (1M context) single-shot, 2026-05-18 — commit `25e6817`
**Auditor**: Claude Opus 4.7 (1M context) — ruolo Architect-Auditor, 2026-05-18 — commit outer pending
**Effort effettivo**: ~1h (refactor mirato, no algoritmica nuova) — stima rispettata

> ⚠️ **Step fuori dal workflow di porting**. Step 0..19 chiusi (parità GNU grep
> verificata). Ondata 2 = lavoro di **code quality** sul codice già funzionante,
> senza aggiungere feature né cambiare comportamento osservabile (`cargo test`
> deve restare 36/36 IDENTICO al baseline `106a138`).

## Goal Ondata 2

Eliminare i **6 anti-pattern strutturali** rimasti dopo Ondata 1 **senza toccare
l'architettura** (sub-struct `Config`, `Engine` trait, `Cow<str>` rimangono
Ondata 3). Risultato atteso:

- `cargo clippy` (default lints) → **0 warning**
- `cargo clippy -W clippy::pedantic -W clippy::nursery` → **< 20 warning** (era ~50)
- `cargo test` → **36/36 verde IDENTICO** (qualsiasi delta = regressione = bloccante)
- Diff atteso: ~300-500 LOC modificati, +50/-50 netto (refactor in loco)

## Contratto applicato (precondizioni invariabili)

| Vincolo | Verifica |
|---|---|
| Build verde pre & post | `cd rgrep && cargo build` 0 errori, 0 warning di codice |
| Test verdi pre & post | `cd rgrep && cargo test` 36/36 (5 suites) IDENTICO al baseline |
| `cargo clippy` default | Da 2 warning (Ondata 1 residui) → 0 |
| Un singolo commit | `refactor(ondata2): structural cleanup — error type, naming, decomp` |
| Format commit obbligatorio | IN-SCOPE / OUT-OF-SCOPE / Testcase counters |
| Niente scope creep | Solo le D-Ondata-2.k qui sotto. Niente Config sub-struct, niente Engine trait, niente Cow. |
| Niente nuove feature | rgrep deve fare ESATTAMENTE le stesse cose di prima |
| Lingua | Codice + commit + comment inline → inglese |

## D-decisioni Ondata 2

Le D-decisioni sono **contratto**. L'Implementer le applica letteralmente. In
caso di ambiguità non coperta, aprire `// DESIGN-Q:` e taggare lo step
`🟡 BLOCKED — DESIGN-Q`. NON improvvisare.

### D-Ondata-2.1 — Naming: rimuovere prefisso `get_`

**Anti-pattern**: rust-skill `coding-guidelines` (P.NAM.04 — no `get_` prefix on
accessors). 3 occorrenze in [rgrep/src/cli.rs](../../src/cli.rs):

```
get_after_context  → after_context_lines  (per evitare collisione con field `after_context: usize`)
get_before_context → before_context_lines
get_binary_action  → binary_action
```

**Razionale collisione**: `Config` ha già `pub after_context: usize` e
`pub before_context: usize` come field clap. Quindi non si può chiamare
`fn after_context(&self) -> usize` direttamente — serve un nome distinto
sull'accessore che combina `--context N` + `--after-context M` con priorità.

**Strategia**: rinominare anche tutti i call site (sono pochi, in `runner.rs`).
NON cambiare i field `pub after_context`/`pub before_context` (sono Clap-driven,
toccarli rompe CLI parity).

### D-Ondata-2.2 — Error type: introdurre `thiserror::Error` enum

**Anti-pattern**: rust-skill `m06-error-handling` — `Box<dyn Error>` è
acceptable per binary main() ma anti-pattern per library code. 7 occorrenze
sparse in `runner.rs` + `matcher.rs`.

**Strategia**:
1. Aggiungere dipendenza `thiserror = "2"` (o versione coerente con `regex`/`fancy-regex` già presenti) a `rgrep/Cargo.toml`.
2. Creare `rgrep/src/error.rs` con:
   ```rust
   #[derive(thiserror::Error, Debug)]
   pub enum RgrepError {
       #[error("invalid regex: {0}")]
       InvalidRegex(String),
       #[error("io error reading {path}: {source}")]
       Io { path: String, source: std::io::Error },
       #[error("pcre/perl-regexp not available in this build")]
       PcreUnavailable,
       #[error("glob pattern error: {0}")]
       InvalidGlob(String),
   }
   ```
3. Sostituire tutte le 7 `Box<dyn Error>` con `RgrepError`.
4. `From<std::io::Error> for RgrepError::Io` quando il `?` propaga (path-aware: usare `.map_err` se il path è necessario; altrimenti `#[from]` semplice).
5. `main.rs` può mantenere `Box<dyn Error>` al confine main (vedi rust-skill `domain-cli` — main può boxare).

**DESIGN-Q ammessa**: se l'enum ha bisogno di più varianti (es. `OpenFile`, `BinaryDetect`), aggiungerle ma senza esplodere oltre 6 varianti. Se serve più granularità, taggare `🟡 BLOCKED — DESIGN-Q` e fermarsi.

### D-Ondata-2.3 — Anti-pattern `std::process::exit` in library code

**Anti-pattern**: rust-skill `domain-cli` — `std::process::exit()` in library
code rompe la composabilità (test harness, library reuse). 1 occorrenza:
[rgrep/src/matcher.rs:75](../../src/matcher.rs#L75).

**Strategia**:
1. Sostituire `std::process::exit(2)` con `return Err(RgrepError::PcreUnavailable)`.
2. Il call site (probabilmente `Matcher::new`) deve già ritornare `Result`. Se non lo fa, propagare la firma `Result<Self, RgrepError>`.
3. `main.rs` cattura l'errore e fa `process::exit(2)` con messaggio coerente (compatibile con exit code GNU grep convention già documentata).

**Verifica differential**: dopo il fix, il comando che attivava il path PCRE in build no-pcre deve ancora exitare con codice 2 e messaggio "perl-regexp not available" o simile (testcase `tests/diff_runner.rs` se esiste, altrimenti smoke manuale).

### D-Ondata-2.4 — Decomporre `bufread_search` (141 LOC, cognitive 41)

**Anti-pattern**: rust-skill `m15-anti-pattern` — funzioni > 100 LOC con
cognitive complexity > 30 sono untestable e error-prone. Funzione in
[rgrep/src/runner.rs](../../src/runner.rs) (lookup `fn bufread_search`).

**Strategia**: estrarre `MatchContext` struct privato + helper method:

```rust
struct MatchContext {
    history: VecDeque<HistoryEntry>,
    print_after: usize,
    last_printed_line: u64,
    match_count: u64,
}

impl MatchContext {
    fn new(before_context: usize) -> Self { ... }
    fn record_line(&mut self, line_no: u64, line: &str) { ... }
    fn flush_before(&mut self, line_no: u64, ...) { ... }
    fn flush_after(&mut self, line_no: u64, ...) -> bool { ... }
}
```

Poi `bufread_search` diventa il loop principale (~40 LOC) che chiama i metodi di
`MatchContext`. NON cambiare la logica observable — è purely structural extract.

**Verifica anti-regressione**: `tests/diff_runner.rs` ha già copertura context (`-A`, `-B`, `-C`). Se passa post-refactor → ok.

### D-Ondata-2.5 — Decomporre `print_line` (9 args → struct `PrintCtx`)

**Anti-pattern**: clippy `too_many_arguments` (9/7). Funzione in `runner.rs:607`.

**Strategia**:

```rust
struct PrintCtx<'a> {
    filename: &'a str,
    line_no: u64,
    byte_offset: u64,
    matcher: &'a Matcher,
    colors: &'a GrepColors,
    config: &'a Config,
    print_filename: bool,
    print_line_no: bool,
    print_byte_offset: bool,
}

fn print_line(ctx: &PrintCtx, line: &str) { ... }
```

Argomenti: ridurre da 9 a 2 (`&PrintCtx`, `&str`). Il `PrintCtx` viene costruito una volta nel chiamante.

**Razionale ordine campi**: dati immutabili + reference grouped first, flag bool grouped last. Non è strict requirement ma migliora leggibilità.

### D-Ondata-2.6 — `if_same_then_else` a runner.rs:462-467

**Anti-pattern**: clippy `if_same_then_else`. Codice attuale:

```rust
if config.files_without_match && !has_match {
    let term = if config.null { '\0' } else { '\n' };
    print!("{}{}", filename, term);
} else if config.files_with_matches && has_match {
    let term = if config.null { '\0' } else { '\n' };
    print!("{}{}", filename, term);
} else if config.count { ... }
```

**Strategia**: collassare i due rami identici in una guard combinata:

```rust
let should_print_filename =
    (config.files_without_match && !has_match)
    || (config.files_with_matches && has_match);

if should_print_filename {
    let term = if config.null { '\0' } else { '\n' };
    print!("{}{}", filename, term);
} else if config.count { ... }
```

Semantica preservata. Diff netto: -3 righe.

### D-Ondata-2.7 — `// SAFETY:` comment sul blocco mmap

**Anti-pattern**: rust-skill `unsafe-checker` — ogni blocco `unsafe` deve
avere un commento `// SAFETY:` che giustifica le invarianti. 1 occorrenza in
[rgrep/src/runner.rs](../../src/runner.rs) (lookup `MmapOptions::new`).

**Strategia**: aggiungere sopra il blocco unsafe:

```rust
// SAFETY: We assume the file is not modified or truncated by another
// process for the duration of the mmap. This matches the GNU grep
// behavior, which also relies on this assumption (and prints a warning
// in some cases when the file shrinks mid-read). For grep's read-only
// use case, the risk surface is acceptable: undefined behavior occurs
// only if a concurrent writer truncates the file below the mmap'd
// length, in which case the OS may deliver SIGBUS on access.
unsafe { MmapOptions::new().map(file)? }
```

Solo commento, nessun cambio di codice. È documentazione di soundness — invariante che il revisore deve poter verificare.

### D-Ondata-2.8 — Pedantic batch (solo i quick-win, no riscritture)

**Anti-pattern**: clippy `-W pedantic -W nursery` residui dopo Ondata 1.

**Quattro micro-categorie quick-win**:

1. **`#[must_use]` su getter pubblici** (~9 occ): aggiungere `#[must_use]` su
   `pub fn` che ritornano `&T` o owned-`T` senza side effect. Esempio:
   `pub fn after_context_lines(&self) -> usize` → `#[must_use] pub fn ...`.
2. **`derive(Eq)` quando si ha `PartialEq`** (~4 occ): se un type ha già
   `#[derive(PartialEq)]` e tutti i field implementano `Eq`, aggiungere `Eq`.
3. **`# Errors` doc sezione su `Result`-returning pub fn** (~4 occ): aggiungere
   `/// # Errors\n/// Returns RgrepError::X if ...` (rust-skill
   `coding-guidelines` P.DOC.03).
4. **`format!()` inline interpolation `{var}` invece di `{}, var`** (~35 occ):
   `format!("{}", x)` → `format!("{x}")`. **Cautela**: SOLO se il var è già un
   identificatore semplice; se è `obj.field` o espressione, lasciare la forma
   lunga (clippy non lo segnala in quel caso).

**NON in scope (deferiti a Ondata 3)**:
- `inefficient_to_string` (richiede `Cow<str>` refactor)
- `needless_pass_by_value` (richiede signature redesign)
- `must_use_candidate` su `impl` traits (richiede dispatch analysis)

### D-Ondata-2.9 — Build precondition (regola CLAUDE.md)

`cargo build` deve essere verde 0 errori 0 warning di codice PRIMA del commit.
`cargo test` deve essere 36/36 IDENTICO al baseline. Se la regressione è
1 testcase, **fermarsi**, taggare `🟡 BLOCKED`, NON committare. Anti-pattern
critico rawk Step 13 (commit con build red).

### D-Ondata-2.10 — Single commit, format obbligatorio

Un solo commit, format integrale:

```
refactor(ondata2): structural cleanup — error type, decomposition, naming

IN-SCOPE:
- Add thiserror::Error RgrepError enum, replace 7 Box<dyn Error> uses
- Remove std::process::exit(2) from matcher.rs, propagate Result instead
- Decompose bufread_search via MatchContext struct (141 LOC → ~40+helpers)
- Decompose print_line via PrintCtx<'a> struct (9 args → 2)
- Rename get_after_context/get_before_context/get_binary_action (no get_ prefix)
- Collapse if_same_then_else at runner.rs (files-without-match / files-with-matches)
- Add // SAFETY: comment on mmap unsafe block in runner.rs
- Apply pedantic quick-wins: #[must_use] (9), derive(Eq) (4), # Errors doc (4),
  format!() inline interpolation (35)

OUT-OF-SCOPE (Ondata 3 — architecture):
- Config flat-bag (30 bool) → sub-struct with #[command(flatten)]
- Engine enum → trait sealed or enum_dispatch
- highlight() return type → Cow<'a, str>
- Other pedantic/nursery requiring architecture redesign

Testcase aggiunti: 0. Totali: 36. cargo test green pre and post.

Co-Authored-By: <agent identifier> <noreply@anthropic.com>
```

### D-Ondata-2.11 — Pattern relay-shell `&&` chain (se Implementer è Gemini)

Se l'Implementer è Gemini Antigravity, fornire UN UNICO blocco shell atomico fail-fast:

```bash
cd /Volumes/Extreme\ Pro/Claude/testag-grep/rgrep && \
  cargo build 2>&1 | tail -5 && \
  cargo clippy 2>&1 | tail -5 && \
  cargo test 2>&1 | tail -10 && \
  git add -A && \
  git commit -m "$(cat <<'EOF'
<format integrale qui>
EOF
)"
```

Razionale: short-circuit → no commit zombie su working tree broken.

### D-Ondata-2.12 — Aggiornamento file post-commit

Dopo il commit Implementer su rgrep, aggiornare in QUESTO file:

- Header Ondata 2: `🚧 PRONTO` → `🟢 FATTO — AUDIT PENDING — commit <hash>`

Niente altro. AUDIT_LOG.md / outer commit / submodule pointer = lavoro Architect-Auditor della sessione successiva.

## Checklist Implementer (pre-commit)

- [ ] `cd rgrep && cargo build` → 0 errori, 0 warning
- [ ] `cd rgrep && cargo clippy` (default) → 0 warning
- [ ] `cd rgrep && cargo test` → 36/36 IDENTICO al baseline
- [ ] `cd rgrep && cargo fmt --check` → clean
- [ ] Tutte le 12 D-Ondata-2.k applicate (o BLOCKED documentato)
- [ ] Format commit integrale (IN-SCOPE / OUT-OF-SCOPE / Testcase counters)
- [ ] Header Ondata 2 in NEXT_STEPS.md aggiornato a `🟢 FATTO — AUDIT PENDING`

## Checklist Auditor (post-commit, sessione successiva)

Pattern identico ad Ondata 1 (vedi `AUDIT_LOG.md` → "Audit Ondata 1" come
template):

1. Build verde + test 36/36
2. Clippy default 0 warning
3. Format commit integrale
4. 12 D-Ondata-2.k tutte applicate
5. Zero regressione differential
6. Single-commit rule
7. Branch corretto (main subrepo)
8. Zero scope creep (no Config sub-struct, no Engine trait, no Cow)

---

# ✅ Ondata 3 — Architettura — APPROVED (8/8) — commit `87c652f`

**Stato**: ✅ APPROVED (8/8) — audit chiuso 2026-05-18 12:18 GMT+2 (vedi `AUDIT_LOG.md` → "Audit Ondata 3 — Verdict Architect-side")
**Promosso**: 2026-05-18 10:31 GMT+2 dall'Architect-Auditor (dopo audit ✅ Ondata 2 8/8, vedi `AUDIT_LOG.md` → "Audit Ondata 2 — Verdict Architect-side")
**Predecessore**: Ondata 2 (`25e6817` rgrep, ✅ APPROVED 8/8 — error type, decomposition, naming)
**Implementer**: Claude Opus 4.7 (1M context, Anthropic) — single-shot, 2026-05-18 11:12+ GMT+2
**Commit**: `87c652f` su rgrep main, ahead origin/main by 3
**Effort stimato**: ~2-3h (refactor architetturale, no algoritmica nuova)
**Effort effettivo**: ~1h45m (Claude single-shot)

> ⚠️ **Step fuori dal workflow di porting**. Step 0..19 chiusi (parità GNU grep
> verificata). Ondate 1-2 chiuse (lint cleanup + refactor strutturale).
> Ondata 3 = **redesign architetturale mirato** sui residui pedantic/nursery
> di Ondata 2 (~18 warning) e sui debiti documentali. Comportamento observable
> resta IDENTICO (`cargo test` deve restare 36/36).

## Goal Ondata 3

Eliminare i **5 anti-pattern architetturali** rimasti dopo Ondata 2 più il
fix di attribuzione header. Risultato atteso:

- `cargo clippy` (default) → **0 warning** (già 0 in Ondata 2, mantenuto)
- `cargo clippy -W clippy::pedantic -W clippy::nursery` → **< 5 warning** (era ~18)
- `cargo test` → **36/36 verde IDENTICO** (qualsiasi delta = regressione = bloccante)
- Diff atteso: ~400-700 LOC modificati, +50/-100 netto (refactor architetturale in loco)

## Contratto applicato (precondizioni invariabili)

| Vincolo | Verifica |
|---|---|
| Build verde pre & post | `cd rgrep && cargo build` 0 errori, 0 warning di codice |
| Test verdi pre & post | `cd rgrep && cargo test` 36/36 (5 suites) IDENTICO al baseline `25e6817` |
| `cargo clippy` default | 0 warning (mantenuto da Ondata 2) |
| `cargo clippy -W pedantic -W nursery` | Da ~18 → < 5 |
| Un singolo commit | `refactor(ondata3): architectural cleanup — Config split, Engine trait, Cow highlight` |
| Format commit obbligatorio | IN-SCOPE / OUT-OF-SCOPE / Testcase counters |
| Niente scope creep | Solo le D-Ondata-3.k qui sotto. Niente nuove feature, niente algoritmica diversa, niente perf-tuning extra. |
| Niente nuove feature | rgrep deve fare ESATTAMENTE le stesse cose di prima |
| Lingua | Codice + commit + comment inline → inglese |

## D-decisioni Ondata 3

Le D-decisioni sono **contratto**. L'Implementer le applica letteralmente. In
caso di ambiguità non coperta, aprire `// DESIGN-Q:` e taggare lo step
`🟡 BLOCKED — DESIGN-Q`. NON improvvisare.

### D-Ondata-3.1 — `Config` flat-bag → sub-struct con `#[command(flatten)]`

**Anti-pattern**: clippy `more_than_3_bools_in_struct` (Config ha ~30 bool +
~10 stringhe). rust-skill `coding-guidelines` (P.STR — coesione semantica).

**Strategia**: 5 sub-struct semantiche, ognuna composta in `Config` via
`#[command(flatten)]`. Distribuzione consigliata (ma negoziabile in
`// DESIGN-Q:` se serve riequilibrio):

```rust
#[derive(Args, Debug, PartialEq, Eq)]
struct PatternOpts {
    pattern: Option<String>,
    #[arg(short = 'e', long = "regexp")] regexp: Vec<String>,
    #[arg(short = 'f', long = "file")] pattern_file: Option<String>,
    #[arg(short = 'F', long)] fixed_strings: bool,
    #[arg(short = 'E', long = "extended-regexp")] extended_regexp: bool,
    #[arg(short = 'G', long = "basic-regexp")] basic_regexp: bool,
    #[arg(short = 'P', long = "perl-regexp")] perl_regexp: bool,
    #[arg(short = 'i', long = "ignore-case")] ignore_case: bool,
    #[arg(short = 'w', long = "word-regexp")] word_regexp: bool,
    #[arg(short = 'x', long = "line-regexp")] line_regexp: bool,
}

#[derive(Args, Debug, PartialEq, Eq)]
struct OutputOpts {
    #[arg(short = 'c', long = "count")] count: bool,
    #[arg(short = 'l', long = "files-with-matches")] files_with_matches: bool,
    #[arg(short = 'L', long = "files-without-match")] files_without_match: bool,
    #[arg(short = 'q', long = "quiet")] quiet: bool,
    #[arg(short = 'n', long = "line-number")] line_number: bool,
    #[arg(short = 'H', long = "with-filename")] with_filename: bool,
    #[arg(short = 'h', long = "no-filename")] no_filename: bool,
    #[arg(short = 'b', long = "byte-offset")] byte_offset: bool,
    #[arg(short = 'o', long = "only-matching")] only_matching: bool,
    #[arg(long = "label")] label: Option<String>,
    #[arg(long = "color", value_enum, num_args = 0..=1, default_missing_value = "always")] color: Option<ColorWhen>,
    #[arg(short = 'Z', long = "null")] null: bool,
}

#[derive(Args, Debug, PartialEq, Eq)]
struct ContextOpts {
    #[arg(short = 'A', long = "after-context")] after_context: usize,
    #[arg(short = 'B', long = "before-context")] before_context: usize,
    #[arg(short = 'C', long = "context")] context: usize,
}

#[derive(Args, Debug, PartialEq, Eq)]
struct FilterOpts {
    #[arg(short = 'v', long = "invert-match")] invert_match: bool,
    #[arg(short = 'r', long = "recursive")] recursive: bool,
    #[arg(short = 'R', long = "dereference-recursive")] dereference_recursive: bool,
    #[arg(long = "include")] include: Vec<String>,
    #[arg(long = "exclude")] exclude: Vec<String>,
    #[arg(long = "exclude-from")] exclude_from: Option<String>,
    #[arg(long = "exclude-dir")] exclude_dir: Vec<String>,
    #[arg(short = 'a', long = "text")] text_mode: bool,
    #[arg(short = 'I')] ignore_binary: bool,
    #[arg(short = 's', long = "no-messages")] no_messages: bool,
    #[arg(long = "directories", value_enum)] directories: Option<DirectoriesAction>,
    #[arg(long = "devices", value_enum)] devices: Option<DevicesAction>,
}

#[derive(Args, Debug, PartialEq, Eq)]
struct BinaryOpts {
    #[arg(long = "binary-files", value_enum)] binary_files: Option<BinaryAction>,
    #[arg(long = "binary")] binary: bool,
    #[arg(short = 'U', long = "binary")] binary_short: bool,
    #[arg(short = 'm', long = "max-count")] max_count: Option<usize>,
    #[arg(long = "mmap")] mmap: bool,
}

#[derive(Parser, Debug, PartialEq, Eq)]
pub struct Config {
    #[command(flatten)] pub pattern_opts: PatternOpts,
    #[command(flatten)] pub output_opts: OutputOpts,
    #[command(flatten)] pub context_opts: ContextOpts,
    #[command(flatten)] pub filter_opts: FilterOpts,
    #[command(flatten)] pub binary_opts: BinaryOpts,
    pub files: Vec<String>,
}
```

**Rule chiave**: gli accessor `after_context_lines`/`before_context_lines`/
`binary_action` introdotti da D-Ondata-2.1 vanno mantenuti **identici nella
signature pubblica** ma il body legge da `self.context_opts.*` /
`self.binary_opts.*`. Tutti i call site in `runner.rs` non devono cambiare.

**Verifica anti-regressione**: `cargo test` 36/36. Suite differential
(`tests/diff_runner.rs`) controlla CLI parity, basta verificare passino.

### D-Ondata-3.2 — `Engine` enum → sealed trait con `enum_dispatch`

**Anti-pattern**: 3× match-repetition in `matcher.rs` (`is_match`,
`highlight`, `find_match_offsets` ognuno con 4 arm: Regex/Fancy/AhoCorasick/
Pcre2). rust-skill `m04-zero-cost` (trait sealed pattern).

**Strategia**:

```rust
// Aggiungere a Cargo.toml: enum_dispatch = "0.3"

#[enum_dispatch::enum_dispatch]
pub trait MatchEngine {
    fn is_match(&self, line: &str) -> bool;
    fn highlight<'a>(&self, line: &'a str, colors: &GrepColors) -> std::borrow::Cow<'a, str>;
    fn find_match_offsets(&self, line: &str) -> Vec<(usize, usize)>;
}

#[enum_dispatch::enum_dispatch(MatchEngine)]
pub enum Engine {
    Regex(RegexEngine),
    Fancy(FancyEngine),
    AhoCorasick(AhoCorasickEngine),
    Pcre2(Pcre2Engine),
}
```

Le 3 funzioni in `Matcher::{is_match, highlight, find_match_offsets}`
delegano a `self.engine.is_match(line)` etc., senza più match arm.

**DESIGN-Q ammessa**: se `enum_dispatch` introduce regressioni binary-size
non accettabili (improbabile), fallback su trait sealed senza la macro
(boilerplate aumenta ma zero dipendenze nuove).

**Verifica anti-regressione**: `cargo test` 36/36 (in particolare i testcase
PCRE in `tests/cases/`).

### D-Ondata-3.3 — `highlight()` return → `Cow<'a, str>`

**Anti-pattern**: clippy `inefficient_to_string` + alloc-per-riga su
`--color=never` (default). 4 occorrenze in `matcher.rs` (una per engine).

**Strategia**:

```rust
fn highlight<'a>(&self, line: &'a str, colors: &GrepColors) -> Cow<'a, str> {
    if colors.is_disabled() {
        return Cow::Borrowed(line);
    }
    let mut result = String::with_capacity(line.len() + 32);
    // ... existing push_str logic ...
    Cow::Owned(result)
}
```

Il call site in `runner.rs` (`print_line` → `matcher.highlight(line, &ctx.colors)`)
ora riceve `Cow<'a, str>` invece di `String`. `print!("{}", cow)` funziona
identico (entrambe le varianti `Display`). NON serve cambiare il chiamante.

**Razionale GrepColors**: aggiungere helper `pub fn is_disabled(&self) -> bool`
in `output.rs` che ritorna `true` se tutti i campi colore sono vuoti
(`String::new()`). Coerente con D-Ondata-2.8 (`String::new()` per default).

**Verifica anti-regressione**: `cargo test` 36/36 + verifica visiva con
`echo "foo bar foo" | rgrep --color=always foo` (deve ancora emettere ANSI).

### D-Ondata-3.4 — Decomporre `bufread_search` / `resolve_files` / `run()`

**Anti-pattern**: clippy `too_many_lines` (4 occorrenze residue post-Ondata-2):
- `bufread_search` 148/100
- una funzione ~113/100
- `run()` ~105/100
- un'altra ~101/100

**Strategia**: decomposizione mirata (NO scope creep verso refactor totali).

1. **`bufread_search`**: estrarre `process_line(pctx, matcher, ctx, line_no, line)`
   helper che incapsula la logica per-riga (controllo `is_match` + emissione
   `emit_match_block`/aggiornamento `MatchContext`). Risultato atteso: il loop
   principale `bufread_search` scende a ~60-70 LOC.

2. **`resolve_files`**: estrarre `expand_recursive(path, glob_filter, files)`
   per la logica `--recursive`/`--include`/`--exclude`. Il body principale di
   `resolve_files` resta orchestrazione (~50 LOC).

3. **`run()`**: estrarre `dispatch_search(pctx, matcher, source)` che
   discrimina file/stdin/mmap. `run()` resta ~70 LOC di setup (config
   parsing, matcher build, file resolve, loop).

**Vincolo invariante**: tutte le firme delle helper sono **privato modulo**
(`fn helper(...)` senza `pub`). Non esporre nuova API. NON cambiare la firma
pubblica di `run()` (è quella che `main.rs` chiama).

**Cognitive complexity**: i warning 28/25 e 30/25 (se presenti) devono
scendere sotto 25 come effetto collaterale della decomposizione.

### D-Ondata-3.5 — `needless_pass_by_value` su `run()` e simili

**Anti-pattern**: clippy `needless_pass_by_value`. Probabile in `run(config: Config)`
e in altri 2-3 punti.

**Strategia**: se l'argomento è usato solo in lettura (no `mut`, no `move`
in chiusura, no field-move), passare per `&` riferimento:

```rust
pub fn run(config: &Config) -> Result<RunResult, RgrepError> { ... }
```

Aggiornare call site in `main.rs`:
```rust
let config = Config::parse();
match rgrep::runner::run(&config) { ... }
```

**Cautela**: se un branch fa `config.files` (vec owned) e poi consuma il vec,
serve `config.files.clone()` o ristrutturare. Tagga `// DESIGN-Q:` se non
risolvibile senza scope creep.

### D-Ondata-3.6 — Fix attribuzione header in `src/error.rs`

**Anti-pattern documentale**: il file `src/error.rs` creato da Ondata 2
riporta:

```rust
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Gemini Antigravity (Google)
```

Ma l'Implementer effettivo di Ondata 2 è stato **Claude Opus 4.7 single-shot**
(confermato dal trailer `Co-Authored-By` di `25e6817`). Boilerplate
carry-over.

**Strategia**: in `src/error.rs` (e ogni altro file dove l'header attribuisce
il lavoro Ondata 2 a Gemini per errore), aggiornare a:

```rust
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Claude Opus 4.7 (1M context, Anthropic)  -- Ondata 2 single-shot
```

**Verifica**: `rtk proxy grep -rn "Implementer: Gemini" src/` deve ritornare
zero occorrenze post-fix (oppure restare solo dove l'autore effettivo è
Gemini, se applicabile).

### D-Ondata-3.7 — Pedantic batch finale (cleanup residui)

**Anti-pattern**: clippy `-W pedantic -W nursery` residui dopo D-3.1..3.5
(`binding_name_too_similar` in runner.rs, `single_match_else`,
`unnecessary_wraps`, `could_be_const_fn`, `let_else`).

**Strategia**: applicare i quick-win **solo se non introducono scope creep**:
- `binding_name_too_similar`: rinominare `ctx` ↔ `match_ctx` / `pctx`
  dove serve disambiguazione.
- `single_match_else`: convertire in `if let ... else { ... }`.
- `unnecessary_wraps`: cambiare `fn foo() -> Result<T, E>` in `fn foo() -> T`
  se non c'è path di errore (cautela: solo se chiamante sopporta).
- `could_be_const_fn`: aggiungere `const`.
- `let_else`: rewrite `match x { Some(v) => v, None => return Err(...) }`
  in `let Some(v) = x else { return Err(...); };`.

**NON in scope**: lint che richiedono restructuring più ampio (es. cambio
di trait gerarchia, eliminazione `Box`, etc.).

### D-Ondata-3.8 — Build precondition (regola CLAUDE.md)

`cargo build` deve essere verde 0 errori 0 warning di codice PRIMA del commit.
`cargo test` deve essere 36/36 IDENTICO al baseline `25e6817`. Se la regressione
è 1 testcase, **fermarsi**, taggare `🟡 BLOCKED`, NON committare. Anti-pattern
critico rawk Step 13 (commit con build red).

### D-Ondata-3.9 — Single commit, format obbligatorio

Un solo commit, format integrale:

```
refactor(ondata3): architectural cleanup — Config split, Engine trait, Cow highlight

IN-SCOPE:
- Split Config flat-bag into 5 sub-structs (PatternOpts/OutputOpts/
  ContextOpts/FilterOpts/BinaryOpts) via #[command(flatten)]
- Add sealed trait MatchEngine + enum_dispatch on Engine enum; replace 3x
  match-repetition in Matcher::{is_match,highlight,find_match_offsets}
- highlight() return type → Cow<'a, str>; skip alloc when --color=never
- Decompose bufread_search (148→~65 LOC) via process_line helper
- Decompose resolve_files (~113→~50 LOC) via expand_recursive helper
- Decompose run() (~105→~70 LOC) via dispatch_search helper
- Change run() signature: fn run(config: Config) → fn run(config: &Config)
  to fix needless_pass_by_value; update main.rs call site
- Fix Implementer attribution header in src/error.rs (Gemini → Claude)
- Apply pedantic quick-wins: binding rename (2), single_match_else (1),
  unnecessary_wraps (1), could_be_const_fn (1), let_else (1)

OUT-OF-SCOPE (final residual, accepted technical debt):
- Remaining pedantic/nursery items requiring algorithmic restructuring
- Performance tuning (allocator changes, SIMD, etc.)
- Refactor of testcase manifest format

Testcase aggiunti: 0. Totali: 36. cargo test green pre and post.
cargo clippy (default): 0 warning. cargo clippy -W pedantic -W nursery:
<5 warning (down from ~18, target was <5).

Co-Authored-By: <agent identifier> <noreply@anthropic.com>
```

### D-Ondata-3.10 — Pattern relay-shell `&&` chain (se Implementer è Gemini)

Se l'Implementer è Gemini Antigravity, fornire UN UNICO blocco shell atomico
fail-fast:

```bash
cd /Volumes/Extreme\ Pro/Claude/testag-grep/rgrep && \
  cargo build 2>&1 | tail -5 && \
  cargo clippy --tests -- -D warnings 2>&1 | tail -5 && \
  cargo test 2>&1 | tail -10 && \
  git add -A && \
  git commit -m "$(cat <<'EOF'
<format integrale qui>
EOF
)"
```

Razionale: short-circuit → no commit zombie su working tree broken.

### D-Ondata-3.11 — Aggiornamento file post-commit

Dopo il commit Implementer su rgrep, aggiornare in QUESTO file:

- Header Ondata 3: `🚧 PRONTO` → `🟢 FATTO — AUDIT PENDING — commit <hash>`

Niente altro. AUDIT_LOG.md / outer commit / submodule pointer = lavoro
Architect-Auditor della sessione successiva.

## Checklist Implementer (pre-commit)

- [ ] `cd rgrep && cargo build` → 0 errori, 0 warning
- [ ] `cd rgrep && cargo clippy --tests -- -D warnings` → exit 0
- [ ] `cd rgrep && cargo clippy --tests -- -W clippy::pedantic -W clippy::nursery` → < 5 warning
- [ ] `cd rgrep && cargo test` → 36/36 IDENTICO al baseline `25e6817`
- [ ] `cd rgrep && cargo fmt --check` → clean
- [ ] Tutte le 11 D-Ondata-3.k applicate (o BLOCKED documentato)
- [ ] Format commit integrale (IN-SCOPE / OUT-OF-SCOPE / Testcase counters)
- [ ] Header Ondata 3 in NEXT_STEPS.md aggiornato a `🟢 FATTO — AUDIT PENDING`

## Checklist Auditor (post-commit, sessione successiva)

Pattern identico ad Ondata 1-2 (vedi `AUDIT_LOG.md` → "Audit Ondata 2" come
template più recente):

1. Build verde + test 36/36
2. Clippy default `-D warnings` exit 0
3. Format commit integrale
4. 11 D-Ondata-3.k tutte applicate
5. Zero regressione differential (36/36 identico al baseline `25e6817`)
6. Single-commit rule (`git log 25e6817..<hash> --oneline` = 1 commit)
7. Branch corretto (main subrepo)
8. Zero scope creep (no nuove feature, no algorithmic changes, no perf tuning)

## OUT-OF-SCOPE (final residual debt, accepted)

- Lint pedantic/nursery che richiedono restructuring più ampio dell'enum
  `Engine` o del lookup pattern compiler.
- Performance tuning (allocator changes, SIMD search, regex JIT, mmap
  alignment) — out-of-scope per progetto educational.
- Refactor del formato manifest testcase TOML (`tests/testsuite.toml`).
- Migrazione a `bstr` o `BString` per pure-byte matching (era già
  scope-creep Ondata 2).

## Skill di riferimento (per la sessione Implementer)

L'Implementer dovrebbe caricare le skill rust-skills `coding-guidelines`,
`m04-zero-cost`, `m05-type-driven`, `m15-anti-pattern`, `domain-cli`
(meta-cognition L1+L2+L3 routing applicato).

---

# ✅ Ondata 4 — Performance — APPROVED — commit `a1243e2` (rgrep)

**Stato**: ✅ APPROVED 10/10 — audit chiuso 2026-05-18 ~14:30 GMT+2 dall'Architect-Auditor (Claude Opus 4.7, 1M context). Verdict completo in `AUDIT_LOG.md` → "Audit Ondata 4 — Verdict Architect-side". rgrep `a1243e2` (sub) + outer commit `docs(audit): close Ondata 4 verdict ✅ APPROVED (sub→a1243e2)` di questa sessione.
**Stato precedente**: 🟢 FATTO — AUDIT PENDING — implementato 2026-05-18 ~13:50 GMT+2 dall'Implementer (Claude Opus 4.7 single-shot, per spec D-Ondata-4.7); rgrep commit `a1243e2` (sub) — outer commit + audit pendono per la prossima sessione Architect-Auditor.
**Stato precedente**: 🚧 PRONTO — promosso 2026-05-18 12:40 GMT+2 dall'Architect-Auditor (dopo audit ✅ Ondata 3 8/8, vedi `AUDIT_LOG.md` → "Audit Ondata 3 — Verdict Architect-side")
**Predecessore**: Ondata 3 (`87c652f` rgrep, ✅ APPROVED 8/8 — Config split, Engine trait, Cow highlight)
**Decider sign-off**: 2026-05-18 12:38 GMT+2 (scelta scope tra 4 candidate: Performance / Test robustness / API+docs / runner.rs decomp Wave 2)
**Implementer**: TBD (Claude single-shot consigliato — niente codice critico di produzione)
**Effort stimato**: ~2-3h (benchmark scaffolding + run + writeup; nessun refactor produttivo salvo perf fix evidenti)

> ⚠️ **Step fuori dal workflow di porting**. Ondata 4 = **misurazione perf
> e validazione retroattiva di Ondata 3**. L'obiettivo è chiudere il loop
> sull'ottimizzazione `Cow<'a, str>` introdotta in Ondata 3 (mai misurata)
> e produrre baseline pubblicabili. Il comportamento observable resta
> IDENTICO (`cargo test` deve restare 36/36, 129 differential testcase).

## Goal Ondata 4

Produrre un'analisi perf misurabile, con dati riproducibili, su rgrep vs
oracolo (`/usr/bin/grep`) e — se disponibile — `ripgrep`. Risultato atteso:

- `cargo bench` runnable, suite criterion strutturata (≥5 scenari rappresentativi)
- Confronto **rgrep vs /usr/bin/grep** su tutti gli scenari (BSD grep su macOS)
- Confronto opzionale **rgrep vs ripgrep** (skip-if-missing, non bloccante)
- Misurazione **prima/dopo Cow** (D-Ondata-3.3): isolare l'impatto su
  scenari con e senza `--color`
- Misurazione **mmap path** (`--mmap`) vs default bufread
- Documento `rgrep/PERF_REPORT.md` con tabelle, percentili, interpretazione
- Diario `diary/04-performance.md` (sostituisce o affianca il placeholder
  `04a-divergences.md`? — vedi D-4.7)
- `cargo test` resta 36/36 IDENTICO al baseline `87c652f`
- `cargo clippy` resta clean (default 0 + pedantic <5)

## Contratto applicato (precondizioni invariabili)

| Vincolo | Verifica |
|---|---|
| Build verde pre & post | `cd rgrep && cargo build` 0 errori, 0 warning di codice |
| Test verdi pre & post | `cd rgrep && cargo test` 36/36 IDENTICO al baseline `87c652f` |
| `cargo clippy` default | 0 warning (mantenuto da Ondata 3) |
| `cargo clippy -W pedantic -W nursery` | < 5 warning reali (mantenuto da Ondata 3) |
| `cargo bench --no-run` | exit 0 (compila la suite benchmark) |
| Un singolo commit | `perf(ondata4): benchmark suite + perf report — Cow/mmap validation` |
| Format commit obbligatorio | IN-SCOPE / OUT-OF-SCOPE / Testcase counters |
| Niente scope creep | Solo le D-Ondata-4.k qui sotto. Niente nuove feature, niente refactor produttivo non motivato da numero misurato. |
| Lingua | Codice + commit + comment inline → inglese; PERF_REPORT.md → inglese; diary → italiano |

## D-decisioni Ondata 4

Le D-decisioni sono **contratto**. L'Implementer le applica letteralmente. In
caso di ambiguità non coperta, aprire `// DESIGN-Q:` per codice e
`<!-- SPEC-Q: -->` per documenti, e taggare lo step `🟡 BLOCKED`.
NON improvvisare numeri perf né alterare scenari senza autorizzazione.

### D-Ondata-4.1 — Aggiungere `criterion` come dev-dependency

**Strategia**: aggiungere a `rgrep/Cargo.toml`:

```toml
[dev-dependencies]
# ... existing ...
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "search"
harness = false
```

`harness = false` è obbligatorio per criterion (replace di libtest harness).

**Versione**: usare l'ultima stabile `criterion 0.5.x`. NO `0.4.x` legacy.
Caricare prima la skill `rust-skills:rust-crate-finder` o `rust-learner` per
verificare l'ultima versione corrente al momento dell'esecuzione.

**Verifica**: `cargo build --benches` exit 0.

### D-Ondata-4.2 — Layout `benches/` e corpus di test

**Strategia**: creare la dir `rgrep/benches/` con un singolo file `search.rs`
che ospita TUTTI gli scenari (più semplice di multi-file per progetto di
queste dimensioni).

**Corpus**: usare il subtree `gnu-grep/` già presente nel repo come corpus
realistico (~4.5K LOC C, mix testo + sorgente). Vantaggi: già nel repo,
riproducibile, rappresenta input "vero" (non sintetico). Path da bench:
`../gnu-grep/src/` o `../gnu-grep/` (decidere in `// DESIGN-Q:` se include
o no anche `tests/`).

**Fixture single-file**: per micro-bench su `bufread_search` isolato, generare
in-memoria una `String` di ~1MB con pattern noti (~1000 righe match + ~9000
non-match). NO file temporanei su disco per micro-bench.

**Vincolo riproducibilità**: niente dipendenza da contenuti generati
runtime non deterministici (no `rand` senza seed fisso).

### D-Ondata-4.3 — Scenari di benchmark obbligatori (≥5)

L'Implementer deve includere **almeno questi 5 scenari**, ciascuno come
`c.bench_function(...)` o `c.bench_group(...)`:

1. **literal_match_count** — `rgrep -c "include" gnu-grep/src/*.c`
   Misura: throughput contatore su input medio (~150 file C).
2. **regex_simple** — `rgrep -E "^[a-z]+\(" gnu-grep/src/grep.c`
   Misura: regex semplice ERE su file singolo grande (~3000 righe).
3. **fixed_string_F** — `rgrep -F "MB_LEN_MAX" gnu-grep/src/`
   Misura: fast-path Aho-Corasick / fixed string.
4. **recursive_walk** — `rgrep -r "TODO" gnu-grep/`
   Misura: walk recursive + filter + match (stress su `runner::walk_recursive`).
5. **invert_match** — `rgrep -v "^$" gnu-grep/src/grep.c`
   Misura: inverted match (path `LineOutcome::NoMatch` su molte righe).

**Scenari raccomandati ma non bloccanti** (skip-OK):
6. `pcre_lookbehind` (richiede feature `perl-regexp`)
7. `huge_file_mmap` (richiede `--mmap` + file ≥10MB)

**Anti-pattern critico**: NON usare scenari single-line (es. `grep foo
<<< "bar"`) come unico benchmark — il timing è dominato dallo startup
cost, non dalla search. Ogni scenario deve processare ≥100KB di input.

### D-Ondata-4.4 — Comparison harness (rgrep vs /usr/bin/grep, ripgrep opzionale)

**Strategia**: per ogni scenario D-4.3, eseguire il **medesimo comando**
contro:

- **rgrep** (binario built da `cargo build --release`)
- **/usr/bin/grep** (oracolo di sistema, BSD su macOS, GNU su Linux —
  documentare in PERF_REPORT.md quale)
- **ripgrep** (`rg`) se disponibile in `$PATH`; **skip silenzioso** se assente

Pattern per criterion:

```rust
use std::process::Command;

fn bench_literal_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("literal_count");
    group.bench_function("rgrep", |b| b.iter(|| {
        Command::new(env!("CARGO_BIN_EXE_rgrep"))
            .args(["-c", "include", "../gnu-grep/src/grep.c"])
            .output()
            .unwrap();
    }));
    group.bench_function("system_grep", |b| b.iter(|| {
        Command::new("/usr/bin/grep")
            .args(["-c", "include", "../gnu-grep/src/grep.c"])
            .output()
            .unwrap();
    }));
    if which::which("rg").is_ok() {
        group.bench_function("ripgrep", |b| b.iter(|| {
            Command::new("rg")
                .args(["-c", "include", "../gnu-grep/src/grep.c"])
                .output()
                .unwrap();
        }));
    }
    group.finish();
}
```

**Trade-off accettato**: process-spawn overhead (~1-3ms su macOS) inquina
le misure micro. Per scenari ≥100KB di input l'errore relativo è <5%
(documentare in PERF_REPORT.md). Per micro-bench puro su funzioni Rust
interne, usare bench separati che chiamano `rgrep::runner::*` direttamente
(opzionale, vedi D-4.5).

**Crate `which`**: aggiungere a `[dev-dependencies]` per detection robusta
di `rg` cross-platform.

### D-Ondata-4.5 — Misurare l'impatto Cow di Ondata 3

**Domanda di ricerca**: D-Ondata-3.3 ha introdotto `highlight() -> Cow<'a, str>`
per evitare alloc su `--color=never`. È stato un'ottimizzazione reale o
zero-impact?

**Strategia**: aggiungere uno scenario `cow_impact` che esegue lo stesso
comando con `--color=never` (default) e `--color=always`, e confronta:

```rust
fn bench_cow_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("cow_impact");
    group.bench_function("color_never", |b| b.iter(|| {
        Command::new(env!("CARGO_BIN_EXE_rgrep"))
            .args(["--color=never", "-c", "include", "../gnu-grep/src/"])
            .output().unwrap();
    }));
    group.bench_function("color_always", |b| b.iter(|| {
        Command::new(env!("CARGO_BIN_EXE_rgrep"))
            .args(["--color=always", "-c", "include", "../gnu-grep/src/"])
            .output().unwrap();
    }));
    group.finish();
}
```

**Nota**: con `-c` (count) il path non emette colori, quindi il delta
misura solo l'overhead della check `is_disabled()`. Per misurare l'alloc
saving reale, usare uno scenario senza `-c` che stampa effettivamente
righe colorate vs non colorate.

**Risultato atteso** (predizione Architect, da verificare):
- `--color=never` vs `--color=always`: delta < 20% (allocator moderno gestisce
  bene String corte)
- Se il delta è > 50%, Ondata 3 ha pagato dividendi misurabili → documentare
- Se il delta è < 5%, Cow è essenzialmente cosmetico → documentare onestamente

**Anti-pattern**: NON nascondere risultati "negativi". Se Cow non ha avuto
impatto, scriverlo in PERF_REPORT.md. È esattamente il tipo di lesson
learned che giustifica l'esperimento educational.

### D-Ondata-4.6 — Misurare mmap path

**Domanda di ricerca**: il path `--mmap` (memmap2-backed) batte bufread
su file grandi?

**Strategia**: scenario `mmap_vs_bufread`:

```rust
fn bench_mmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmap_vs_bufread");
    let target = "../gnu-grep/src/grep.c";  // ~120KB
    group.bench_function("bufread_default", |b| b.iter(|| {
        Command::new(env!("CARGO_BIN_EXE_rgrep"))
            .args(["-c", "include", target]).output().unwrap();
    }));
    group.bench_function("mmap_explicit", |b| b.iter(|| {
        Command::new(env!("CARGO_BIN_EXE_rgrep"))
            .args(["--mmap", "-c", "include", target]).output().unwrap();
    }));
    group.finish();
}
```

**Risultato atteso**: per file <1MB, bufread vince (page-fault cost).
Per file ≥10MB, mmap dovrebbe vincere. Se non c'è un file ≥10MB nel
repo, l'Implementer può generare un fixture deterministico (script
shell con `seq`/`cat`) **fuori dal git tree** (es. `target/bench_fixtures/`)
e referenziarlo dal bench. Documentare in `// DESIGN-Q:` come fixture
generation è gestita (lazy in build.rs vs setup esplicito).

### D-Ondata-4.7 — Output: `rgrep/PERF_REPORT.md` + `diary/04-performance.md`

**File 1: `rgrep/PERF_REPORT.md`** (inglese, tecnico, dentro al subrepo).
Struttura obbligatoria:

```markdown
# rgrep Performance Report — Ondata 4

**Date**: 2026-MM-DD
**Commit**: <hash>
**Platform**: <uname -a>, Rust <version>
**Comparison oracle**: /usr/bin/grep (<BSD|GNU> grep <version>)
**ripgrep version** (if available): <version> or "not installed"

## Methodology
- criterion 0.5.x, default 100 samples per scenario
- Process-spawn measurement (overhead ~1-3ms documented)
- Corpus: gnu-grep/ subtree (~4.5K LOC C)
- Median + 95th percentile reported

## Results

### Literal match count (-c)
| Tool | Median | p95 | Δ vs grep |
| rgrep | ... | ... | ... |
| /usr/bin/grep | baseline | baseline | — |
| ripgrep | ... | ... | ... |

[... one section per scenario ...]

## Interpretation
- [scenario]: rgrep is X% [slower|faster] than grep because...
- Cow impact (Ondata 3): [measured Y% on color_never vs color_always]
- mmap impact: [break-even point at ~Z MB]

## Limitations
- Process-spawn overhead inquinates micro-benchmarks
- macOS BSD grep is older / less optimized than GNU grep
- ripgrep uses SIMD literal search, fair comparison only on regex scenarios
```

**File 2: `diary/04-performance.md`** (italiano, narrativo, deliverable della
skill `legacy-port`). Riusa il template degli altri capitoli diary (01-03).
Risponde alle 5 domande della skill applicate a "performance":

1. Cosa abbiamo misurato e cosa no
2. Cosa ha funzionato meglio del previsto (es. ripgrep era 5× più veloce attesi,
   in realtà solo 2× — o viceversa)
3. Cosa ha fallito le aspettative (Cow inutile? mmap deludente?)
4. Trade-off process-spawn vs FFI: perché abbiamo scelto questo metodo
5. Generalizzabilità: cosa replicherebbe rawk/altri porting

**Vincolo numerazione diary**: il file attuale `04a-divergences.md` collide
con `04-performance.md`. **Decisione Architect**: rinominare il vecchio in
`04a-divergences.md` e il nuovo in `04b-performance.md`. Aggiornare ogni
riferimento incrociato (cerca `04a-divergences` in tutto il repo con
`rtk proxy grep -rn "04a-divergences" .`). Se l'Implementer trova
referenze in `99-conclusions.md` o `CLAUDE.md`, aggiornarle.

**Alternativa accettabile** (in `<!-- SPEC-Q: -->` se l'Implementer preferisce):
mantenere `04a-divergences.md` invariato e creare `05-performance.md`. NON
è il pattern stretto della skill, ma evita rename in cascata.

### D-Ondata-4.8 — Zero modifiche al codice produttivo (salvo perf-fix evidenti)

**Vincolo invariante**: Ondata 4 è **misurazione**, non refactor. Modifiche
ammesse a `src/*.rs`:

- ✅ `pub` aggiunto a funzione interna SOLO se serve al bench `criterion`
  per chiamarla diretta (preferire process-spawn quando possibile)
- ✅ Hot-spot fix evidente: se il bench rivela una `.clone()` palese o
  un `.collect::<Vec<_>>()` superfluo nel loop interno, fix consentito.
  **Vincolo**: il fix deve essere ≤10 LOC, documentato in `// PERF-FIX:`
  inline, e citato in `IN-SCOPE` del commit. NO refactor architetturali.
- ❌ Cambio algoritmico (es. da regex → DFA custom)
- ❌ Aggiunta SIMD / unsafe optimization
- ❌ Cambio scheduler / threading (`rayon`, `tokio`)
- ❌ Riorganizzazione moduli

Se l'Implementer trova un'ottimizzazione che richiederebbe più di 10 LOC,
documentarla in `OUT-OF-SCOPE` del commit + `PERF_REPORT.md` → "Future work"
e lasciarla per Ondata 5 o altro.

### D-Ondata-4.9 — Build precondition (regola CLAUDE.md)

`cargo build` deve essere verde 0/0 PRIMA del commit. `cargo build --release`
(usato dai bench) deve essere verde 0/0. `cargo test` deve essere 36/36
IDENTICO al baseline `87c652f`. `cargo bench --no-run` deve compilare exit 0.
**NON serve eseguire l'intero `cargo bench` prima del commit** (lungo);
basta `--no-run`. L'esecuzione completa avviene per generare i numeri di
PERF_REPORT.md, con il commit fatto solo dopo.

Se la regressione è anche 1 testcase, fermarsi, taggare `🟡 BLOCKED`, NON
committare.

### D-Ondata-4.10 — Single commit, format obbligatorio

Un solo commit, format integrale:

```
perf(ondata4): benchmark suite + perf report — Cow/mmap validation

IN-SCOPE:
- Add criterion 0.5 + which to dev-dependencies
- Add [[bench]] entry in Cargo.toml (harness=false)
- Add benches/search.rs with N scenarios (literal_count, regex_simple,
  fixed_string_F, recursive_walk, invert_match [, pcre_lookbehind,
  huge_file_mmap])
- Add cow_impact bench (color=never vs color=always)
- Add mmap_vs_bufread bench
- Add rgrep/PERF_REPORT.md with measured numbers + interpretation
- Add diary/04b-performance.md (or 05-performance.md per D-4.7 fallback)
- [Rename 04a-divergences → 04a-divergences if D-4.7 primary path chosen]
- [Optional PERF-FIX: <≤10 LOC fix> if benchmark revealed an evident
  hot-spot; documented inline with // PERF-FIX: comment]

OUT-OF-SCOPE (accepted, documented in PERF_REPORT.md "Future work"):
- SIMD literal search
- DFA-based engine replacement of `regex` crate
- Threading (rayon parallel walk)
- FFI-based benchmark (vs process-spawn)
- Fixture generation script for huge_file_mmap (>10MB on disk)

Testcase aggiunti: 0. Totali: 36. cargo test green pre and post.
cargo build green 0/0. cargo bench --no-run exit 0.
cargo clippy (default): 0 warning. cargo clippy -W pedantic -W nursery:
<5 warning (maintained from Ondata 3).

Co-Authored-By: <agent identifier> <noreply@anthropic.com>
```

### D-Ondata-4.11 — Pattern relay-shell `&&` chain (se Implementer è Gemini)

Se l'Implementer è Gemini Antigravity, fornire UN UNICO blocco shell atomico
fail-fast (`cargo bench --no-run` invece di `cargo bench` per non esplodere
il tempo di commit):

```bash
cd /Volumes/Extreme\ Pro/Claude/testag-grep/rgrep && \
  cargo build 2>&1 | tail -5 && \
  cargo clippy --tests -- -D warnings 2>&1 | tail -5 && \
  cargo test 2>&1 | tail -10 && \
  cargo bench --no-run 2>&1 | tail -5 && \
  git add -A && \
  git commit -m "$(cat <<'EOF'
<format integrale qui>
EOF
)"
```

I numeri di PERF_REPORT.md vanno generati con `cargo bench` (full run)
**prima** della catena commit, salvati nel file, poi il commit chiude
tutto. Se i numeri cambiano lievemente in un secondo run, è normale
(rumore criterion < 5%); NON rifare il commit.

### D-Ondata-4.12 — Aggiornamento file post-commit

Dopo il commit Implementer su rgrep, aggiornare in QUESTO file:

- Header Ondata 4: `🚧 PRONTO` → `🟢 FATTO — AUDIT PENDING — commit <hash>`

Niente altro. AUDIT_LOG.md / outer commit / submodule pointer = lavoro
Architect-Auditor della sessione successiva.

## Checklist Implementer (pre-commit)

- [ ] `cd rgrep && cargo build` → 0 errori, 0 warning
- [ ] `cd rgrep && cargo clippy --tests -- -D warnings` → exit 0
- [ ] `cd rgrep && cargo clippy --tests -- -W clippy::pedantic -W clippy::nursery` → < 5 warning
- [ ] `cd rgrep && cargo test` → 36/36 IDENTICO al baseline `87c652f`
- [ ] `cd rgrep && cargo bench --no-run` → exit 0
- [ ] `cd rgrep && cargo bench` eseguito almeno una volta per popolare PERF_REPORT.md
- [ ] `cd rgrep && cargo fmt --check` → clean
- [ ] Tutte le 12 D-Ondata-4.k applicate (o BLOCKED documentato)
- [ ] Format commit integrale (IN-SCOPE / OUT-OF-SCOPE / Testcase counters)
- [ ] PERF_REPORT.md completo (≥5 scenari + cow_impact + mmap section)
- [ ] diary/04b-performance.md (o 05-performance.md) presente
- [ ] Header Ondata 4 in NEXT_STEPS.md aggiornato a `🟢 FATTO — AUDIT PENDING`

## Checklist Auditor (post-commit, sessione successiva)

Pattern identico ad Ondate 1-3 (vedi `AUDIT_LOG.md` → "Audit Ondata 3" come
template più recente):

1. Build verde + test 36/36 IDENTICO baseline `87c652f`
2. Clippy default `-D warnings` exit 0
3. Format commit integrale
4. 12 D-Ondata-4.k tutte applicate (o BLOCKED con motivazione)
5. `cargo bench --no-run` exit 0 (suite compila)
6. PERF_REPORT.md presente, ≥5 scenari, numeri presenti (non placeholder TODO)
7. diary/04b-performance.md (o 05-performance.md) presente, risposte alle 5
   domande della skill
8. Single-commit rule (`git log 87c652f..<hash> --oneline | wc -l` = 1)
9. Branch corretto (main subrepo)
10. Zero scope creep oltre eventuali PERF-FIX ≤10 LOC documentati inline

## OUT-OF-SCOPE (accepted, "Future work")

- **SIMD literal search**: richiede `memchr` aggressivo o `aho-corasick`
  con feature `prefilter`. Out-of-scope per progetto educational.
- **DFA custom engine**: il crate `regex` già usa DFA internamente; rimpiazzarlo
  sarebbe scope-creep enorme.
- **Parallel walk** (rayon): possibile Ondata 5 dedicata.
- **FFI benchmark** vs process-spawn: alternativa più precisa ma rende
  fragile la build (rgrep diventa lib oltre che bin).
- **Fixture generation script** per file >10MB: lasciato come step manuale
  documentato in PERF_REPORT.md "Reproducibility".
- **Profilo flamegraph** / `cargo-flamegraph`: utile ma out-of-scope per
  Ondata 4 (richiede `perf` su Linux o `dtrace` su macOS).

## Skill di riferimento (per la sessione Implementer)

L'Implementer dovrebbe caricare le skill rust-skills `m10-performance`
(CRITICAL per benchmark/criterion/profiling) come Layer 1, `domain-cli`
(Layer 3 — process-spawn pattern per CLI tools), `coding-guidelines`
(P.PERF naming). Meta-cognition L1+L3 routing applicato.

Riferimenti documentali:
- criterion docs: <https://bheisler.github.io/criterion.rs/book/>
- ripgrep performance writeups (Andrew Gallant): per metodologia confronto
  fair tra grep-like tools
- `cargo bench` standard Rust docs

---

# ✅ Ondata 5 — Profile-driven targeted fix — APPROVED (12/12) — commit `6ab7be1`

**Stato**: ✅ APPROVED — audit chiuso 2026-05-18 15:55 GMT+2, verdict 12/12
in AUDIT_LOG.md "Audit Ondata 5". Executor ha consegnato rgrep commit
`6ab7be1` 2026-05-18 ~15:30 GMT+2.
**Predecessore**: Ondata 4 (`a1243e2` rgrep, ✅ APPROVED 10/10).
**Implementer hash**: rgrep `6ab7be1` (single commit, perf(ondata5))
**Deliverables**:
- `rgrep/PROFILE_REPORT.md` (samply profiling, 4 scenari, methodology +
  comparative + selected fixes + future work; ~400 righe)
- `rgrep/src/runner.rs` — PERF-FIX-O5-1 (UTF-8 fast path, ~10 LOC) +
  PERF-FIX-O5-2 (slurp-and-cursor per file <16 MB, ~19 LOC)
- `rgrep/PERF_REPORT.md` — sezione "Ondata 5 post-fix" con tabella
  pre/post/Δ per tutti i 7 scenari benchmark
- `diary/05-profiling.md` — 5 domande risposte (metodologia, top
  findings, surprises, fix validati, generalizability)

**Risultati validati** (median, vs Ondata 4 baseline `a1243e2`):
| Scenario | Δ | Status |
|---|---|---|
| `literal_match_count` | −9.5% | ✅ ≥5% |
| `mmap_vs_bufread/bufread_default` | −13.6% | ✅ ≥5% |
| `cow_impact/color_never_count_only` | −15.8% | ✅ ≥5% |
| `mmap_vs_bufread/mmap_explicit` | −4.7% | sub-5% |
| Altri scenari | between −2.4% e +2.7% | within noise |

D-Ondata-5.6 satisfied: 3 scenari ≥5%, max regressione +2.7% (sotto cap
5%). I 3 SPEC-defined worst (regex_simple, invert_match, fixed_string_F)
hanno cambi nel rumore (bottleneck = memchr_aligned, blocked da
D-Ondata-5.10 no-new-deps). Vedi PROFILE_REPORT.md §5 e diary §5 per la
SPEC-vs-profile divergence documentata come generalisable lesson.

**Audit chiuso**: ✅ APPROVED (12/12) — vedi AUDIT_LOG.md "Audit Ondata 5"
per il verdict completo. Tutti i 13 D-Ondata-5.k applicati o BLOCKED-with-
rationale (D-5.6 fix policy con caveat SPEC-vs-profile divergence
onestamente documentata). Anchor rgrep di chiusura ciclo Performance =
`6ab7be1`.

---

## Storico SPEC Ondata 5 (per audit reference)

# ~~🚧~~ 🟢 Ondata 5 — Profile-driven targeted fix (flamegraph + glue cleanup) — ~~PRONTO~~ FATTO

**Stato originale**: 🚧 PRONTO — promosso da Architect-Auditor 2026-05-18 ~14:42 GMT+2
**Predecessore**: Ondata 4 (`a1243e2` rgrep, ✅ APPROVED 10/10).
**Baseline**: `a1243e2` (rgrep) — 36 testcase verdi, criterion suite installata,
PERF_REPORT.md presente con i numeri di confronto vs `/usr/bin/grep`.

> ⚠️ **Step fuori dal workflow di porting** (come Ondata 4). Ondata 5 =
> **misurazione + fix mirato**, NON nuova feature, NON refactor architetturale.
> Il porting C→Rust funzionale è chiuso a Step 19. Le Ondate 4-5 sono lavoro
> di **performance engineering** sul codice già funzionante e già stabilizzato.

## Goal Ondata 5

Profilare i tre scenari con il gap più ampio vs BSD grep (misurati in Ondata 4):

| Scenario | Δ vs grep (Ondata 4) | Hypothesis pre-profile |
|---|---|---|
| `regex_simple` | +30.8 % | regex crate glue overhead, per-line allocation |
| `invert_match` | +28.9 % | regex crate negation + count machinery |
| `fixed_string_F` | +27.2 % | aho-corasick glue + WalkDir/IO loop overhead |

Identificare 1-3 hot-spot **strutturali** (non micro-ottimizzazioni di basso
livello: lasciamo unsafe/SIMD a future ondate) e applicare fix mirati che:
- Sono ≤50 LOC ciascuno
- Sono giustificati da evidenza nel profile (non hunch)
- Non rompono i 36 testcase (precondizione rigida)
- Producono un Δ misurabile ≥5 % nello scenario target alla re-bench

Risultato atteso:
- `rgrep/PROFILE_REPORT.md` (markdown summary, ~200-400 righe)
- 1-3 fix in `src/*.rs` con `// PERF-FIX-O5:` inline + bullet `IN-SCOPE`
- `PERF_REPORT.md` aggiornato con sezione "Ondata 5 post-fix" e tabella
  pre/post/Δ
- `diary/05-profiling.md` con metodologia + findings + lezioni
- Bench re-run mostra miglioramento misurabile in almeno 1 scenario, nessuna
  regressione >5 % in altri scenari

| Metrica | Pre-Ondata-5 (`a1243e2`) | Atteso Post-Ondata-5 |
|---|---|---|
| `cargo build` warning | 0 | 0 |
| `cargo test` | 36/36 | 36/36 IDENTICO |
| `cargo clippy --tests -- -D warnings` | 0 | 0 |
| `cargo clippy -W pedantic -W nursery` | <5 (filesystem noise) | <5 |
| `cargo bench --no-run` | exit 0 | exit 0 |
| Almeno 1 scenario in PERF_REPORT.md | baseline numbers | ≥5 % miglior. mediana |
| Nessuna regressione >5 % | n/a | invariante |

## D-decisioni Ondata 5

### D-Ondata-5.1 — Tool choice: `samply`

**Decisione**: usare `samply` (Markus Stange, Mozilla) come profiler primario.

Razionale:
- **macOS arm64 friendly**: usa `task_for_pid` con entitlement userspace,
  niente `sudo` per profilare il proprio binario (a differenza di
  `cargo-flamegraph` che richiede `dtrace` + SIP relaxed).
- Output: profile JSON che si apre con il **Firefox Profiler** (web-based,
  no install di GUI extra).
- Genera anche flamegraph SVG opzionale tramite `--save-only` + post-process.
- Installazione: `cargo install samply` (~30s).

**Fallback documentato** (`<!-- SPEC-Q: tool fallback -->`):
- Se samply ha issue su questa macchina, fallback a `cargo-instruments`
  (Apple Instruments wrapper). NON usare `cargo-flamegraph` su macOS (richiede
  sudo + SIP).
- Su Linux (re-run futuro): preferire `cargo-flamegraph` (perf-based, più
  preciso).

### D-Ondata-5.2 — Profile target: 3 worst scenarios + 1 control

**Profilare 4 invocazioni**:

1. `regex_simple`: `rgrep -E '^[a-z]+\(' gnu-grep/src/grep.c`
2. `invert_match`: `rgrep -v -c '^$' gnu-grep/src/grep.c`
3. `fixed_string_F`: `rgrep -rF 'MB_LEN_MAX' gnu-grep/src/`
4. **Control**: `literal_match_count` (il best-case +5.5 %, per avere un
   profile di riferimento "in salute" da contrastare con i 3 worst).

Le invocazioni devono essere amplificate (vedi D-5.3) per battere il process-spawn
overhead.

### D-Ondata-5.3 — Signal amplification: loop interno o corpus gigante

Process-spawn aggiunge 1-3 ms su ~3-5 ms totali (vedi PERF_REPORT.md
"Limitations"). Per ottenere un profile dove il **search domina sul startup**,
amplificare il segnale con UNA delle seguenti tecniche (Implementer sceglie
quella che funziona meglio):

**Opzione A — Loop esterno (preferito, zero modifiche al binario)**:
```bash
samply record -o profile_regex_simple.json -- \
  bash -c 'for i in $(seq 1 50); do ./target/release/rgrep -E "^[a-z]+\(" gnu-grep/src/grep.c >/dev/null; done'
```
50 invocazioni → 250 ms totali, di cui ~50-100 ms startup, ~150-200 ms
search reale → search rappresenta ~70 % del profile.

**Opzione B — Corpus gigante (alternativa)**:
```bash
cat gnu-grep/src/grep.c{,,,,,,,,,,} > target/bench_fixtures/big.c   # 1 MB
samply record -o profile_regex_simple.json -- \
  ./target/release/rgrep -E '^[a-z]+\(' target/bench_fixtures/big.c
```
Una sola invocazione, file grande → startup è solo ~1 % del totale.

**Opzione C — Bench in-process** (NON usare): richiederebbe esporre matcher/
runner come `pub`. Conflitta con il principio modulo-privato del progetto.
Bocciata.

Preferire A se Implementer è familiare con `bash for`, B se la fixture non
è un problema (1 MB on disk, regenerabile).

### D-Ondata-5.4 — Fixture management

Generare fixture in `target/bench_fixtures/` (NON committare):
- `target/bench_fixtures/big.c` (~1 MB, concat di `grep.c` × 11)
- `target/bench_fixtures/empty_lines.c` (~500 KB con molte righe vuote per
  amplificare `invert_match`)

Aggiungere a `.gitignore` se non già coperto. Script di generazione
documentato in PROFILE_REPORT.md → "Reproducibility".

### D-Ondata-5.5 — `PROFILE_REPORT.md` — struttura obbligatoria

Creare `rgrep/PROFILE_REPORT.md` con sezioni (in quest'ordine):

1. **Header** — date, baseline commit `a1243e2`, platform, samply version,
   amplification technique (A o B).
2. **Methodology** — comando esatto di profiling, signal amplification,
   come leggere il Firefox Profiler / flamegraph SVG, dove sono salvati i
   profile JSON (`target/profiles/*.json`, .gitignored).
3. **Per-scenario findings** — UNA sezione per ognuno dei 4 scenari (3 worst
   + 1 control). Ogni sezione include:
   - Top 10 hot frames (function name, % self, % total)
   - Allocation hotspots (se rilevabili — opzionale, accettato "N/A — samply
     non traccia allocazioni di default")
   - 1-3 fix candidate ranked: alta/media/bassa confidenza nel beneficio
4. **Comparative analysis** — cosa hanno IN COMUNE i 3 worst che il control
   NON ha? Questo è il cuore del report: identificare il pattern strutturale
   (es. "tutti e 3 i worst passano attraverso `Matcher::find_all_overlapping`
   mentre il control passa per `aho_corasick::find_iter`").
5. **Selected fix(es)** — dichiarare quali fix verranno applicati (max 3),
   con hypothesis esplicita: "Fix N: rimuovere `to_string()` a runner.rs:212
   → atteso -5 % su `regex_simple` perché ogni matched line allocava una
   String nuova".
6. **Future work** — fix considerati ma deferred (motivazione esplicita,
   no scope creep).

Lunghezza attesa: 200-400 righe. Niente filler.

### D-Ondata-5.6 — Fix policy: 1-3 fix, criteri rigidi

**Criteri di selezione** (ogni fix DEVE rispettarli TUTTI):

- ✅ **Hypothesis esplicita dal profile** (citata nel commento `// PERF-FIX-O5:`)
- ✅ **≤50 LOC** (diff netto, conta `+` lines)
- ✅ **Localizzato** (max 2 file modificati per fix)
- ✅ **Bench delta ≥5 %** sulla mediana del scenario target alla re-bench
- ✅ **Zero regressioni >5 %** in altri scenari (regression guard)
- ✅ **Zero modifiche al test suite** (36/36 IDENTICO)
- ✅ **Zero nuovi `unsafe`**
- ✅ **Nessuna nuova dipendenza** (no `cargo add`)

Se un fix candidato fallisce **uno qualsiasi** di questi criteri al test,
**revertirlo subito** e documentarlo in PROFILE_REPORT.md → "Rejected fixes"
con la motivazione (es. "Tentato: cambiare highlight() per usare write_str
invece di format!. Risultato: +0.3 % miglioramento, dentro la noise. Rejected.").

**Cap massimo**: 3 fix in una Ondata. Se ne emergono di più, deferred a
Ondata 6.

### D-Ondata-5.7 — Diary `diary/05-profiling.md` — questions

Creare `diary/05-profiling.md` rispondendo alle **5 domande della skill
legacy-port (variante perf)**:

1. **Methodology**: che tool, perché, come è stato amplificato il segnale.
2. **Top findings**: 3 hot frame emersi, e quanto inaspettati erano (vs la
   hypothesis pre-profile in tabella D-5.0).
3. **Surprises**: cosa pensavamo fosse hot e NON lo era (es. "ci aspettavamo
   `highlight()` Cow allocation, era invece `to_string()` in runner.rs").
4. **Fix(es) applied & validated**: per ogni fix, hypothesis → patch → bench delta.
5. **Generalizability**: cosa replicherebbe rawk/altri porting? Pattern
   trasferibili (es. "profile-driven > spec-driven per perf in porting di
   librerie mature").

Lunghezza: ~150-250 righe.

**Decisione numerazione diary** (replica D-Ondata-4.7):
- Primary path: `diary/05-profiling.md` (numerazione lineare).
- Se Implementer trova conflitto con esistente `05-*.md` → `06-profiling.md`
  con `<!-- SPEC-Q: -->`.

### D-Ondata-5.8 — Regression guard: criterion re-bench dopo OGNI fix

Workflow obbligatorio:

1. Profile generato → PROFILE_REPORT.md sezioni 1-4 scritte
2. Identificato fix candidate #1 → applicato → **`cargo test`** verde →
   **`cargo bench`** full run (~6 min) → confronto numeri pre/post
3. Se Δ ≥+5 % sul target E Δ <5 % regressione su tutti gli altri scenari →
   commit (incrementale solo se necessario; altrimenti tieni in stash e procedi)
4. Ripeti per fix #2 e #3 (se ne hai)
5. **Re-bench finale** dopo tutti i fix applicati → numeri finali in
   PERF_REPORT.md sezione nuova "Ondata 5 post-fix"

**Aggiornamento `PERF_REPORT.md`**: aggiungere sezione "Ondata 5 post-fix" CON
tabella pre/post/Δ per ognuno dei 7 scenari originali, NON sostituire la
sezione "Ondata 4" — è storia.

### D-Ondata-5.9 — Build precondition (regola CLAUDE.md, invariante)

PRIMA del commit:
- `cargo build` → 0 errori, 0 warning
- `cargo build --release` → 0 errori, 0 warning (usato per generare il binario
  di profiling)
- `cargo test` → 36/36 IDENTICO al baseline `a1243e2`
- `cargo clippy --tests -- -D warnings` → exit 0
- `cargo clippy --tests -- -W clippy::pedantic -W clippy::nursery` → <5 warning
  (mantenuto dall'Ondata 3/4)
- `cargo bench --no-run` → exit 0
- `cargo fmt --check` → clean

Se anche 1 testcase diverge, **fermarsi, taggare `🟡 BLOCKED`, NON committare**.

### D-Ondata-5.10 — Out-of-scope hard limits

Le seguenti modifiche sono **vietate** in questa Ondata anche se il profile
le suggerisce; vanno deferred a future Ondate dedicate:

- ❌ Sostituire `regex` crate (Ondata futura "engine swap")
- ❌ Aggiungere SIMD / `unsafe` / `memchr` aggressivo
- ❌ Threading (`rayon`, walker parallelo) → era Direzione A scartata
- ❌ FFI bench harness (rgrep come lib) → era Direzione C scartata
- ❌ Nuove dipendenze runtime (no `cargo add` per dependency, OK per
  dev-dependency se serve solo al profiling, es. `dhat` opzionale)
- ❌ Riorganizzazione moduli
- ❌ Più di 3 fix in un singolo commit (split in Ondata 6 se serve)
- ❌ Modifiche a `tests/testsuite.toml` o `tests/cases/*` (tests are oracle)

### D-Ondata-5.11 — Single commit format

Un solo commit (più hot-fix di scope identico OK, vedi pattern CLAUDE.md):

```
perf(ondata5): profile-driven targeted fix(es) — <short summary>

IN-SCOPE:
- Add rgrep/PROFILE_REPORT.md (samply-based profiling of 3 worst scenarios
  + 1 control, methodology + findings + selected fixes)
- PERF-FIX-O5-N: <description> in src/<file>.rs — atteso ~X% su scenario Y,
  validato a Z%
- [Optional PERF-FIX-O5-2: ...]
- [Optional PERF-FIX-O5-3: ...]
- Update rgrep/PERF_REPORT.md with "Ondata 5 post-fix" section (pre/post/Δ
  table for 7 scenarios)
- Add diary/05-profiling.md (methodology, top findings, surprises, fixes,
  generalizability — 5 skill questions)

OUT-OF-SCOPE (deferred to future waves):
- regex crate swap
- SIMD / unsafe / memchr aggressive
- Threading (rayon)
- FFI bench harness
- More than 3 fixes (cap by D-Ondata-5.6)

Testcase aggiunti: 0. Totali: 36. cargo test IDENTICO baseline a1243e2.
cargo build green 0/0. cargo bench --no-run exit 0. cargo clippy default
0 warning.

Co-Authored-By: <agent identifier> <noreply@anthropic.com>
```

### D-Ondata-5.12 — Executor: Architect-Auditor direct (no Gemini relay)

**Decisione**: come Ondata 4, questa è una sessione **meta** (profiling +
report), non porting di codice. L'Architect-Auditor (Claude Opus) esegue
direttamente senza relay Gemini Antigravity.

Razionale:
- Il profiling richiede iterazione interattiva (look at flamegraph → form
  hypothesis → write fix → re-bench). Pattern relay-shell `&&`-chain non si
  adatta bene a workflow esplorativo.
- Le D-decisioni qui sono molto guidate ma la **fase di interpretazione del
  profile** richiede giudizio che è meglio fare nel modello principale.
- Ondata 4 ha già stabilito il precedente (executor = Claude direct,
  approvato dall'Auditor a posteriori).

**Eccezione**: se questa sessione finisce in OOM/context-limit prima del
commit, ripartire dalla SESSION_HANDOFF.md con Gemini come Implementer
limitato (solo per applicare i fix già identificati in PROFILE_REPORT.md).

### D-Ondata-5.13 — File output policy (gitignore)

I file SEGUENTI **NON** vanno committati (aggiungere a `.gitignore` se non
già coperti):
- `target/profiles/*.json` (output samply, grandi e binary-ish)
- `target/profiles/*.svg` (flamegraph SVG eventuali)
- `target/bench_fixtures/*.c` (fixture generate, ricostruibili)

I file SEGUENTI **SÌ** vanno committati:
- `rgrep/PROFILE_REPORT.md` (~300 righe markdown)
- Modifiche a `src/*.rs` (i fix con `// PERF-FIX-O5:` inline)
- `rgrep/PERF_REPORT.md` aggiornato (sezione nuova "Ondata 5 post-fix")
- `diary/05-profiling.md`

## Checklist Implementer/Executor (pre-commit)

- [ ] `samply` installato (`cargo install samply` o fallback documentato)
- [ ] Fixture generate in `target/bench_fixtures/` (D-5.4)
- [ ] 4 profile generati (`target/profiles/profile_*.json`) — i 3 worst + control
- [ ] PROFILE_REPORT.md completo (sezioni 1-6 di D-5.5)
- [ ] 1-3 fix selezionati, applicati, ognuno con `// PERF-FIX-O5:` inline
- [ ] Bench re-run completo eseguito
- [ ] PERF_REPORT.md aggiornato con sezione "Ondata 5 post-fix" (tabella pre/post/Δ)
- [ ] diary/05-profiling.md presente (5 domande risposte)
- [ ] `cargo build` 0 warning + `cargo test` 36/36 + clippy `-D warnings` exit 0
- [ ] `cargo bench --no-run` exit 0
- [ ] Format commit integrale (IN-SCOPE / OUT-OF-SCOPE / counters)
- [ ] Almeno 1 fix mostra Δ ≥+5 % sul scenario target alla re-bench
- [ ] Nessuna regressione >5 % in altri scenari
- [ ] Header Ondata 5 in NEXT_STEPS.md aggiornato a `🟢 FATTO — AUDIT PENDING`

## Checklist Auditor (post-commit, sessione successiva)

Pattern come Ondate 1-4 (vedi `AUDIT_LOG.md` → "Audit Ondata 4" come
template più recente):

1. Build verde + test 36/36 IDENTICO baseline `a1243e2`
2. Clippy default `-D warnings` exit 0
3. Format commit integrale (IN-SCOPE / OUT-OF-SCOPE)
4. Tutte le 13 D-Ondata-5.k applicate (o BLOCKED con motivazione)
5. PROFILE_REPORT.md completo, 4 sezioni per-scenario, comparative analysis
   presente, selected fixes giustificati da evidenza
6. Ogni fix ha commento `// PERF-FIX-O5:` con hypothesis citata
7. PERF_REPORT.md sezione "Ondata 5 post-fix" presente, tabella pre/post/Δ
   per ≥5 scenari, almeno 1 scenario mostra Δ ≥+5 % miglioramento mediana
8. Nessuna regressione >5 % vs Ondata 4 baseline in nessuno scenario
9. diary/05-profiling.md presente, 5 domande risposte (no placeholder)
10. Single-commit rule (`git log a1243e2..<hash> --oneline | wc -l` = 1
    o ≤2 se hot-fix dello stesso scope)
11. Branch corretto (main subrepo)
12. Zero scope creep oltre i 3 fix dichiarati; nessun cambio out-of-scope D-5.10

## OUT-OF-SCOPE (accepted, "Future work" per Ondata 6+)

- **regex crate swap** (es. `regex-lite`, `pcre2`, DFA custom): Ondata
  dedicata "engine swap" se i numeri lo giustificano.
- **SIMD literal search**: Ondata dedicata "SIMD" se PROFILE_REPORT mostra
  che il literal scan è hot.
- **Threading / parallel walk**: Ondata "parallelism" (Direzione A scartata
  oggi, ripescabile).
- **FFI bench harness**: Ondata "bench infrastructure" — fa rgrep
  diventare lib oltre che bin.
- **Allocation tracking via dhat**: utile per profile alloc-focused, lasciato
  a future iterazione (richiede dev-dep nuova e una nuova metodologia).
- **Profile su Linux con `perf`** (oracle GNU grep): ricalibra i delta vs
  l'oracle "duro", out-of-scope per oggi (macchina è macOS arm64).

## Skill di riferimento (per la sessione executor)

- `m10-performance` (Layer 1 CRITICAL — profiling, criterion, hot path
  analysis)
- `domain-cli` (Layer 3 — process-spawn, CLI binary perf)
- `m11-ecosystem` (Layer 2 — choosing dev-dependencies for profiling tools)
- `coding-guidelines` (P.PERF naming, // PERF-FIX comment convention)
- `legacy-port` skill — Phase 4d (post-port performance work)

Meta-cognition L1+L3 routing applicato.

Riferimenti documentali:
- samply: <https://github.com/mstange/samply>
- Firefox Profiler: <https://profiler.firefox.com> (apre i .json di samply)
- cargo-flamegraph (fallback): <https://github.com/flamegraph-rs/flamegraph>
- ripgrep performance writeups (Andrew Gallant) — pattern per honesty
  reporting + future work labeling
- `cargo bench` standard Rust docs

---

# ✅ Ondata 6 — `is_match_bytes` API for count/quiet fast path — APPROVED (12/12) — commit `759524b`

**Stato**: ✅ APPROVED — audit chiuso 2026-05-18 ~16:50 GMT+2 (vedi
[AUDIT_LOG.md](AUDIT_LOG.md) → "Audit Ondata 6 — Verdict Architect-side").
**Stato precedente**: 🟢 FATTO — AUDIT PENDING — implementato 2026-05-18
~16:30 GMT+2 dall'Architect-Auditor direct executor (per D-Ondata-6.11
amended); rgrep commit `759524b`.
**Stato pre-precedente**: 🚧 PRONTO — promosso dal Decider 2026-05-18 ~16:12
GMT+2 (con amendment SPEC dall'Architect-Auditor — vedi sezione "⚠️ AMENDMENT"
sotto).
**Stato pre-pre-precedente**: 🔒 LOCKED — autorizzata dall'Architect-Auditor
2026-05-18 ~16:00 GMT+2.
**Predecessore**: Ondata 5 (`6ab7be1` rgrep, ✅ APPROVED 12/12).
**Baseline**: `6ab7be1` (rgrep) — 36 testcase verdi, PERF-FIX-O5-1 + O5-2
applicati, PROFILE_REPORT.md disponibile con findings per-scenario.

## ⚠️ AMENDMENT 2026-05-18 ~16:12 GMT+2 (prevale su sezioni originali in caso di conflitto)

Re-read della SPEC originale contro il codice `6ab7be1` ha rivelato 4 drift
significativi (premessa perf parzialmente già coperta da PERF-FIX-O5-1).
L'Architect-Auditor amenda la SPEC come segue. **Tutto sotto questa sezione
prevale sull'originale.**

### Drift identificati

1. **`find_first_match` non esiste**. Il codice usa `Matcher::is_match(&str) → bool`
   come fast path (già) e `Matcher::find_match_offsets` (slow) è già gated
   da `only_matching || color_enabled` in `bufread_search` linee 444-455.
   → Premessa "ridurre lavoro in count-only" parzialmente già coperta.
2. **Trait name**: il trait reale è `MatchEngine` (non `Engine`); l'enum è
   `Engine`. Metodi: `engine_is_match` / `engine_highlight` / `engine_find_offsets`.
3. **PrintCtx non ha count_mode/quiet_mode**. I flag stanno su
   `config.output_opts.count` e `config.output_opts.quiet`.
4. **Wiring corretto in `bufread_search`, NON `process_line`**. `process_line`
   riceve già un `line_str: &str` validato UTF-8. Il vero risparmio è skippare
   la conversione UTF-8 (linee 432-433) per modalità puramente boolean.

### D-decisioni amendate

**D-Ondata-6.1 (AMENDED) — Scope**: matcher API extension + `bufread_search`
fast-path loop only. Niente modifiche a `process_line`. Resto invariato.

**D-Ondata-6.2 (AMENDED) — Signature `MatchEngine::engine_is_match_bytes`**:
```rust
trait MatchEngine {
    // existing methods unchanged
    fn engine_is_match_bytes(&self, buf: &[u8]) -> bool {
        // default: lossy str + engine_is_match
        self.engine_is_match(&String::from_utf8_lossy(buf))
    }
}
```
Override per backend:
- `Regex` → fast UTF-8 path (mirror PERF-FIX-O5-1):
  `std::str::from_utf8(buf).map_or_else(|_| self.is_match(&String::from_utf8_lossy(buf)), |s| self.is_match(s))`
- `AhoCorasick` → diretto byte path: `self.is_match(buf)` (REAL win — skippa la conversione UTF-8)
- `fancy_regex::Regex` → fallback lossy: `self.is_match(&String::from_utf8_lossy(buf)).unwrap_or(false)` (fancy è str-only)
- `pcre2::bytes::Regex` (feature `perl-regexp`) → nativo byte: `self.is_match(buf).unwrap_or(false)`

**D-Ondata-6.3 (AMENDED) — `Matcher::is_match_bytes`**: espone trait method
con gestione `invert_match`:
```rust
pub fn is_match_bytes(&self, buf: &[u8]) -> bool {
    let m = self.engine.as_match_engine().engine_is_match_bytes(buf);
    if self.config.filter_opts.invert_match { !m } else { m }
}
```

**D-Ondata-6.4 (AMENDED) — Wiring fast-loop in `bufread_search`**: dopo il
binary-detection pre-check, computare precondizione `pure_count_quiet`:
```rust
let pure_count_quiet = (output.count || output.quiet)
    && !output.only_matching
    && !output.files_with_matches
    && !output.files_without_match
    && !pctx.color_enabled
    && before_ctx == 0
    && after_ctx == 0
    && config.binary_opts.max_count.is_none()
    && !is_binary;
```
Se `true`, eseguire un loop dedicato: `read_until` → strip delimiter (+ `\r\n`
trailing se delimiter è `\n` e non `binary`) → `matcher.is_match_bytes(line_bytes)`
→ contatori inline (`state.match_count++`, `has_match = true`); se
`output.quiet` → `return Ok(true)` (early exit). **Skippa**: UTF-8 conversion
(linee 432-433), line_str trim, `find_match_offsets`, `process_line` call.
`emit_trailing_summary` resta in fondo (gestisce il print del count finale
per modalità `-c`). Se `pure_count_quiet` è `false`, scendere nel loop attuale
invariato.

### Target perf ricalibrato (AMENDED D-Ondata-6.6)

| Metrica | Pre-O6 (`6ab7be1`) | Atteso Post-O6 |
|---|---|---|
| `cargo build` warning | 0 | 0 |
| `cargo test` | 36/36 | 36/36 IDENTICO |
| `cargo clippy --tests -- -D warnings` | 0 | 0 |
| `cargo bench --no-run` | exit 0 | exit 0 |
| Almeno 1 scenario count/quiet | post-O5 numbers | Δ ≥+1% mediana (PERF-FIX-O5-1 ha già coperto parte del win) |
| Nessuna regressione >5% in altri scenari | n/a | invariante |

Soglie: Δ ≥+3% → full success; 1% ≤ Δ < 3% → marginal success (retain);
Δ < 1% no regressioni → retain l'API (deliverable architetturale +
nuovo bench `quiet_mode`) e riportare onestamente in PERF_REPORT.md come
"Ondata 6 — neutral perf, API addition only".

### LOC budget esteso (AMENDED D-Ondata-6.7)

≤65 LOC netti totali (+15 vs originale 50, per fast-loop esplicito e
`Matcher::is_match_bytes` exposure):
- ~22-28 LOC `src/matcher.rs` (trait default + 4 override + Matcher exposure)
- ~18-22 LOC `src/runner.rs::bufread_search` (precondizione + fast-loop branch)
- ~25-30 LOC `benches/search.rs` (`quiet_mode` group)

### Resto invariato

D-Ondata-6.5 (test parity 36/36 IDENTICO), D-6.8 (build precondition),
D-6.9 (single commit format con titolo `perf(ondata6)`), D-6.10 (out-of-scope
hard limits), D-6.11 (executor diretto Architect-Auditor, no Gemini relay),
D-6.12 (file output policy: matcher.rs + runner.rs + benches/search.rs +
PERF_REPORT.md) restano validi.

---


> ⚠️ **Step fuori dal workflow di porting** (come Ondata 4 e 5). Ondata 6 =
> **fast-path API extension** per modi `count`/`quiet`, NON nuova feature
> visibile all'utente, NON refactor architetturale.

## Goal Ondata 6

Estendere il `Matcher` trait con un metodo `is_match_bytes(&self, buf: &[u8]) -> bool`
che restituisce true/false senza calcolare posizioni di match né allocare
strutture di capture. Wiring opzionale da `runner::process_line` quando la
modalità è puramente boolean (`-c` count_mode o `-q` quiet_mode).

Motivazione dal PROFILE_REPORT.md Ondata 5 §2.4 (control scenario
literal_match_count): nel modo count-only, `Matcher::find_first_match`
restituisce comunque `Some(Match { start, end })` e ogni call site costruisce
ranges che vengono poi scartati. `is_match_bytes` evita il computo del range,
e in particolare permette al backend `regex::Regex` di usare l'API
`is_match` interna (più rapida di `find`).

Win atteso: −5..−10 % mediana sui 4 scenari count-only di PERF_REPORT.md
sezione 6 "cow_impact":
- `cow_impact/color_never_count_only` (oggi post-O5 = 3.515 ms)
- `cow_impact/color_always_count_only` (oggi post-O5 = 3.736 ms)
- `literal_match_count` (oggi post-O5 = 4.023 ms)
- bench dedicato `quiet_mode` (NEW — vedi D-6.4)

| Metrica | Pre-Ondata-6 (`6ab7be1`) | Atteso Post-Ondata-6 |
|---|---|---|
| `cargo build` warning | 0 | 0 |
| `cargo test` | 36/36 | 36/36 IDENTICO |
| `cargo clippy --tests -- -D warnings` | 0 | 0 |
| `cargo bench --no-run` | exit 0 | exit 0 |
| Almeno 1 count/quiet scenario | post-O5 numbers | ≥+3 % miglioramento mediana |
| Nessuna regressione >5 % | n/a | invariante |
| LOC budget netti | n/a | ≤50 (matcher trait + 4 engine impls + runner wiring + bench) |

## D-decisioni Ondata 6

### D-Ondata-6.1 — Scope: matcher API extension only

L'estensione vive interamente in `src/matcher.rs` (signature trait + 4
implementazioni concrete: Regex, FixedString, FixedStringSet, Empty). Wiring
1-line in `src/runner.rs::process_line`. **Nessun engine swap**, **nessuna
modifica al modulo `output`**, **nessuna modifica a `cli.rs`**.

### D-Ondata-6.2 — Signature trait `is_match_bytes`

```rust
trait Engine {
    // existing methods unchanged
    fn is_match_bytes(&self, buf: &[u8]) -> bool;
}
```

Default implementation provided in the trait body: forwards to
`self.find_first_match(buf).is_some()`. Engine implementations override
per win:
- `Regex` backend → `self.regex.is_match(line_str)` (bytes via
  `str::from_utf8` fast path già introdotto in PERF-FIX-O5-1)
- `FixedString` backend → `memchr_aware aho_corasick::AhoCorasick::is_match(buf)`
- `FixedStringSet` → analogo
- `Empty` (no-op) → `true` short-circuit

### D-Ondata-6.3 — Wiring da `runner::process_line`

In `process_line`, dopo il `matcher` dispatch, aggiungere check sul
`PrintCtx::count_mode || PrintCtx::quiet_mode`. Se uno dei due è `true`,
usare `matcher.is_match_bytes(line_bytes)` e fare branch:
- match true + count_mode → `count += 1; return Ok(...)` (skip output)
- match true + quiet_mode → `return Ok(QuietExit::Found)` (early exit)
- match false → `return Ok(NoMatch)`

Nel caso normale (né count né quiet), il path attuale `find_first_match` è
preservato 1:1.

### D-Ondata-6.4 — Bench nuovo: `quiet_mode`

`benches/search.rs` aggiunge un settimo gruppo:
```rust
fn bench_quiet_mode(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiet_mode");
    // rgrep -q "include" gnu-grep/src/grep.c → exit 0 on first match
    // /usr/bin/grep -q ditto
    // ripgrep -q ditto (se installato)
}
```

Win atteso: −15..−30 % vs count_only perché early-exit on first match
elimina lo scan del resto del file. Bench richiama scenario reale: file
~92 KB grep.c, pattern letterale comune (`include` matcha decine di volte
nelle prime righe).

### D-Ondata-6.5 — Test parity: 36/36 IDENTICO

Vincolo rigido. Se anche 1 testcase diverge, fermarsi, taggare
`🟡 BLOCKED — SPEC-Q`, NON committare. La modifica è puramente
performance, semantica deve essere bit-identica.

### D-Ondata-6.6 — Bench gate: ≥+3 % miglioramento

**Bar ridotta da 5 % a 3 %** perché gli scenari count-only sono
notoriamente noisy (varianza 95 % CI ~±3-5 % osservata in Ondata 5
PERF_REPORT.md). 3 % rappresenta una soglia distinguibile dal rumore ma
realistica per un fix di scope ridotto. Se Δ ≥+5 % su almeno 1 scenario:
considerare full success. Se 3 % ≤ Δ < 5 %: marginal success, retain
con honest reporting.

### D-Ondata-6.7 — LOC budget netti

≤50 LOC netti totali, distribuiti:
- ~15-20 LOC in `src/matcher.rs` (trait method + 4 impl override)
- ~5-8 LOC in `src/runner.rs::process_line` (wiring count/quiet branch)
- ~25-30 LOC in `benches/search.rs` (nuovo bench `quiet_mode` group)

Esclusi dal budget: commenti `// PERF-FIX-O6:` inline attribution.

### D-Ondata-6.8 — Build precondition (regola CLAUDE.md, invariante)

PRIMA del commit:
- `cargo build` → 0 errori, 0 warning
- `cargo build --release` → 0 errori, 0 warning
- `cargo test` → 36/36 IDENTICO al baseline `6ab7be1`
- `cargo clippy --tests -- -D warnings` → exit 0
- `cargo bench --no-run` → exit 0 (include il nuovo bench `quiet_mode`)
- `cargo fmt --check` → clean

### D-Ondata-6.9 — Single commit format

```
perf(ondata6): is_match_bytes fast path for count/quiet modes

IN-SCOPE:
- Add Engine::is_match_bytes trait method (default impl via find_first_match,
  overrides for Regex/FixedString/FixedStringSet/Empty backends)
- Wire count_mode/quiet_mode branch in runner::process_line to call
  is_match_bytes instead of find_first_match (skip range computation)
- Add benches/search.rs `quiet_mode` group (-q with early exit, vs
  /usr/bin/grep -q and ripgrep -q if available)
- Update PERF_REPORT.md with "Ondata 6 post-fix" section (pre/post/Δ on
  the 3 count-only scenarios + new quiet_mode bench)

OUT-OF-SCOPE (deferred to future waves):
- regex crate swap (engine swap wave)
- SIMD / unsafe / memchr aggressive (no new runtime deps — see Ondata 7)
- Threading (rayon)
- FFI bench harness
- More than 1 fix (this wave is scoped to is_match_bytes API only)

Testcase aggiunti: 0. Totali: 36. cargo test IDENTICO baseline 6ab7be1.
cargo build green 0/0. cargo bench --no-run exit 0 (incl. quiet_mode).
cargo clippy default 0 warning.

Co-Authored-By: <agent identifier> <noreply@anthropic.com>
```

### D-Ondata-6.10 — Out-of-scope hard limits

Vietato in questa ondata anche se utile:
- ❌ Sostituire `regex` crate (reserved Ondata "engine swap")
- ❌ SIMD / `unsafe` / `memchr` come direct dep (reserved Ondata 7)
- ❌ Threading (`rayon`) — reserved Ondata "parallelism"
- ❌ Modifiche a `output` module
- ❌ Modifiche a `cli.rs`
- ❌ Modifiche a `tests/testsuite.toml` o `tests/cases/*`
- ❌ Più di 1 fix (scope limitato — eventuali secondari → Ondata 6-bis)

### D-Ondata-6.11 — Executor: Architect-Auditor direct (no Gemini relay)

Come Ondata 4 e 5. Razionale: scope ben-definito, no esplorazione richiesta
oltre alla scelta dell'override regex (`is_match` vs `is_match_at`). Pattern
relay-shell `&&`-chain è ridondante per ~50 LOC localizzati su 2 file.

### D-Ondata-6.12 — File output policy

Committati:
- `src/matcher.rs` (trait + 4 impl override)
- `src/runner.rs` (wiring count/quiet branch)
- `benches/search.rs` (nuovo `quiet_mode` group)
- `PERF_REPORT.md` (sezione "Ondata 6 post-fix")

Non committati: `target/` (gitignore già copre).

## Checklist Implementer/Executor (pre-commit)

- [ ] Trait `Engine` esteso con `is_match_bytes` + default impl
- [ ] 4 engine impls override (Regex, FixedString, FixedStringSet, Empty)
- [ ] Wiring `process_line` count/quiet branch (max 8 LOC)
- [ ] Nuovo bench `quiet_mode` group in `benches/search.rs`
- [ ] `cargo build` 0/0 + `cargo test` 36/36 IDENTICO + clippy `-D warnings` exit 0
- [ ] `cargo bench --no-run` exit 0 (include nuovo bench)
- [ ] Full `cargo bench` eseguito, numeri raccolti per "Ondata 6 post-fix"
- [ ] PERF_REPORT.md aggiornato con sezione "Ondata 6 post-fix"
- [ ] Format commit integrale (IN-SCOPE / OUT-OF-SCOPE / counters)
- [ ] Almeno 1 scenario count/quiet mostra Δ ≥+3 % alla re-bench
- [ ] Header Ondata 6 in NEXT_STEPS.md aggiornato a `🟢 FATTO — AUDIT PENDING`

## Checklist Auditor (post-commit, sessione successiva)

Pattern come Ondate 4-5 (vedi `AUDIT_LOG.md` → "Audit Ondata 5" come
template più recente):

1. Build verde + test 36/36 IDENTICO baseline `6ab7be1`
2. Clippy default `-D warnings` exit 0
3. Format commit integrale (IN-SCOPE / OUT-OF-SCOPE)
4. Tutte le 12 D-Ondata-6.k applicate (o BLOCKED con motivazione)
5. `is_match_bytes` presente in trait + 4 override engine impl
6. Wiring `process_line` correttamente gating su `count_mode || quiet_mode`
7. Nuovo bench `quiet_mode` group presente in `benches/search.rs`
8. PERF_REPORT.md sezione "Ondata 6 post-fix" con tabella pre/post/Δ per
   ≥3 scenari count/quiet, almeno 1 mostra Δ ≥+3 % miglioramento mediana
9. Nessuna regressione >5 % vs Ondata 5 baseline in nessuno scenario
10. Single-commit rule (`git log 6ab7be1..<hash> --oneline | wc -l` = 1 o ≤2 hot-fix stesso scope)
11. Branch corretto (main subrepo)
12. Zero scope creep oltre il fix dichiarato; nessun cambio out-of-scope D-6.10

## OUT-OF-SCOPE (accepted, "Future work" per Ondata 7+)

- **`memchr` crate direct dep**: Ondata 7 dedicata (sblocca i 3 SPEC-defined
  worst di Ondata 5).
- **regex engine swap**: Ondata futura "engine swap" se i numeri lo
  giustificano.
- **SIMD literal search aggressivo**: Ondata futura "SIMD".
- **Threading / parallel walk**: Ondata "parallelism".
- **FFI bench harness**: Ondata "bench infrastructure".
- **`dhat` allocation tracking**: dev-dep, opzionale, eventuale Ondata 6-bis
  o post-Ondata-7.

## Skill di riferimento (per la sessione executor)

- `m10-performance` (Layer 1 CRITICAL — criterion API extension, regression
  guard)
- `m05-type-driven` (Layer 1 — trait extension con default impl + override)
- `domain-cli` (Layer 3 — process-spawn pattern bench)
- `m11-ecosystem` (Layer 2 — `regex::Regex::is_match` API, `aho_corasick::AhoCorasick::is_match`)
- `coding-guidelines` (P.PERF naming, // PERF-FIX comment convention)

Meta-cognition L1+L3 routing applicato.

Riferimenti documentali:
- `regex` crate API: <https://docs.rs/regex/latest/regex/struct.Regex.html#method.is_match>
- `aho-corasick` crate API: <https://docs.rs/aho-corasick/latest/aho_corasick/struct.AhoCorasick.html#method.is_match>
- criterion sample_size + warm_up_time tuning per scenari noisy

---

# ✅ Ondata 7 — `memchr` crate direct dep + manual `fill_buf` loop — APPROVED — `3de67b0`

## Emendamento del 2026-09-30 — prevale sulle D-7 storiche sotto

Autorizzazione: il Decider ha approvato la sequenza proposta il 2026-09-30.
Promozione solo dopo la chiusura di 6-bis. Le seguenti correzioni rendono la
SPEC coerente con O6 e con le nuove evidenze; le parti non emendate restano valide.

1. Baseline = commit 6-bis con benchmark corretto, toolchain rustc 1.98.1.
   `regex_simple` storico misurava exit 2; non confrontarlo con il nuovo caso.
   I delta O6 storici non condividono sempre la stessa baseline numerica.
2. Sostituire entrambe le chiamate `read_until` in `bufread_search` (loop
   generale e `pure_count_quiet`) tramite un helper condiviso e privato.
   Preservare append, EOF senza terminatore, delimitatore NUL, retry di
   `Interrupted`, consumo esatto e propagazione degli altri errori al caller.
   Il caller conserva la politica errori preesistente; nessun refactor matcher.
3. TDD: test nuovi autorizzati in runner e un integration test dedicato;
   confini buffer con capacità piccole, righe lunghe, ultima riga, CRLF,
   NUL, byte invalidi, offset e count/quiet. Prima test rossi (helper mancante),
   poi implementazione. Non cambiare expected dei casi esistenti.
4. Budget: ≤60 righe production nette esclusi test/commenti; niente zero-copy
   aggiuntivo finché il fix semplice non è misurato. `memchr = "2"` diretto;
   nessun aggiornamento volontario delle altre dipendenze.
5. Benchmark con baseline nominata `o6bis-20260930`: scenari corretti piccoli
   più `line_scan` (grep.c ×60, count regex/invert/fixed). Gate: riduzione ≥5%
   su almeno uno scenario line-bound, nessuna regressione confermata >5%.
   Se la varianza rende dubbio il confronto, confermare con run alternate
   degli stessi binari pre/post; registrare anche i risultati sfavorevoli.
6. Profilare pre/post sullo stesso corpus e toolchain. I 32–38% sono dati
   storici di O5, mentre `fixed_string_F` aveva 8,4% e costi filesystem
   dominanti. Il vincolo fisso <15% di un simbolo non è un gate portabile
   fra compilatori: riportare campioni, attribuzione e tempi, senza dedurre
   uno speedup dalla sola scomparsa del nome di una funzione.
7. Se non emerge un beneficio dimostrabile, non mantenere un'ottimizzazione
   inefficace per soddisfare la roadmap: registrare l'esito, conservare
   benchmark/test utili e chiudere l'esperimento con motivazione esplicita.
8. Test default e PCRE2, Clippy e fmt verdi; conteggi aggiornati, non il
   vincolo obsoleto «36/36 identico». Un commit implementazione e audit
   documentale separato; nessuna attestazione di audit indipendente.
9. Artefatti di misura autorizzati: `reports/o7-*.json` e helper riproducibile
   `tools/compare_binaries.py` (31 coppie alternate, warmup, bootstrap dei
   rapporti appaiati, verifica exit/stdout/stderr pre/post). I controlli BSD
   invariati mostrano deriva fra run sequenziali: le misure alternate sono
   necessarie per decidere il gate, non un ampliamento funzionale del prodotto.

**Stato corrente**: ✅ APPROVED — 2026-09-30, commit `3de67b0`, baseline `723aab3`.

**Stato storico**: 🔒 LOCKED — autorizzata dall'Architect-Auditor 2026-05-18 ~16:05 GMT+2,
da promuovere a `🚧 PRONTO` dal Decider quando il workflow lo richiede.
**Predecessore**: Ondata 6 (preferibilmente già APPROVED, altrimenti baseline =
ultimo Ondata APPROVED = `6ab7be1`).
**Baseline atteso**: post-Ondata-6 commit (rgrep), o `6ab7be1` se Ondata 6
saltata. L'Auditor decide all'inizio della sessione quale è il baseline
"vigente".

> ⚠️ **Step fuori dal workflow di porting** (come Ondate 4-6). Ondata 7 =
> **engine I/O upgrade** per sbloccare il caveat onesto di Ondata 5: i 3
> SPEC-defined worst targets (regex_simple, invert_match, fixed_string_F)
> hanno bottleneck `memchr_aligned` 32-38 % self (PROFILE_REPORT.md
> Ondata 5 §2.1-2.3), e l'unico fix strutturale richiede `memchr` come
> direct runtime dep — esplicitamente vietato da D-Ondata-5.10 e qui
> **riaperto** come D-Ondata-7.1.

## Goal Ondata 7

Sostituire il `BufRead::read_until(b'\n', ...)` in `runner::bufread_search`
con un loop manuale `fill_buf` + `memchr::memchr(b'\n', ...)` che usa
NEON/SSE2 aware vector scanning della crate `memchr 2`. Aspettativa: il
self time del per-line `memchr_aligned` (oggi 32-38 %) deve scendere a
<15 %, sbloccando un Δ misurabile sui 3 SPEC-defined worst targets di
Ondata 5.

Win atteso: ≥+5 % mediana su almeno 1 dei 3 scenari worst Ondata 5
(`regex_simple`, `invert_match`, `fixed_string_F`). Win atteso bonus:
≥+3 % sui restanti scenari `bufread`-bound (`literal_match_count`,
`bufread_default`).

| Metrica | Pre-Ondata-7 | Atteso Post-Ondata-7 |
|---|---|---|
| `cargo build` warning | 0 | 0 |
| `cargo test` | 36/36 | 36/36 IDENTICO |
| `cargo clippy --tests -- -D warnings` | 0 | 0 |
| `cargo bench --no-run` | exit 0 | exit 0 |
| `Cargo.toml` deps runtime | N | N+1 (`memchr`) |
| `Cargo.lock` deps transitive | ~~grafo attuale~~ | identico (memchr era già transitive via regex) |
| ≥1 scenario worst-Ondata-5 con Δ ≥+5 % | baseline | atteso ≥1 ≥+5 % |
| Profile samply re-run: `memchr_aligned` self % | 32-38 % | <15 % |
| LOC budget netti | n/a | ≤40 in src/runner.rs + 1 line Cargo.toml |

## D-decisioni Ondata 7

### D-Ondata-7.1 — Riapertura esplicita di D-Ondata-5.10 "no new runtime deps"

D-Ondata-5.10 vietava `memchr` come direct runtime dep. Quel veto era
**condizionale al contesto di Ondata 5** (fix ≤50 LOC localizzato, no
ridiscussione architetturale). Per Ondata 7 il veto è **esplicitamente
sollevato** sulla base di:

1. PROFILE_REPORT.md Ondata 5 §2.1-2.3 ha individuato `memchr_aligned`
   come bottleneck strutturale, non micro-ottimizzazione.
2. Diary 05-profiling.md §5 lo identifica come "blocker da sbloccare in
   Ondata 7 dedicata".
3. AUDIT_LOG.md Audit Ondata 5 §"Anti-pattern noti — backlog post-Ondata 5"
   schedula esplicitamente "memchr crate direct dep + manual fill_buf loop"
   come "Ondata 7 candidate".
4. `memchr` è già transitive dep via `regex` (no impatto su `Cargo.lock`
   transitive graph size, solo aggiunta del riferimento diretto in
   `Cargo.toml [dependencies]`).

### D-Ondata-7.2 — `Cargo.toml`: aggiunta `memchr = "2"`

Singola riga in `[dependencies]`:
```toml
memchr = "2"
```

Versione `"2"` (cargo SemVer caret) — al 2026-05-18 la latest stable
è `memchr 2.7.x`. **Non pinnare a `2.x.y` specifico**: `memchr` è una
crate ultra-stabile (Andrew Gallant, ripgrep author) e il `^2` range
è quello standardmente raccomandato dall'autore.

**Cargo.lock**: dovrebbe non cambiare (memchr già presente come transitive
via `regex`). Se il diff `Cargo.lock` mostra >5 LOC: investigare prima
di committare (potrebbe indicare version bump non-banale).

### D-Ondata-7.3 — Replace `read_until` con manual `fill_buf` + `memchr::memchr` loop

Posizione: `src/runner.rs::bufread_search`, dentro il loop `loop { ... }`.
L'attuale chiamata `r.read_until(delimiter, &mut buffer)` viene
sostituita da un loop manuale equivalente:

```rust
// PERF-FIX-O7-1: manual fill_buf + memchr loop replaces read_until.
// memchr::memchr uses NEON on Apple Silicon (and SSE2/AVX2 on x86_64),
// vs std's memchr_aligned which is portable byte-scan. Profile attribution:
// PROFILE_REPORT.md Ondata 5 §2.1 (regex_simple), §2.2 (invert_match),
// §2.3 (fixed_string_F) — memchr_aligned 32-38% self.
loop {
    let chunk = match r.fill_buf() {
        Ok(c) if c.is_empty() => break,
        Ok(c) => c,
        Err(_) => break,
    };
    match memchr::memchr(delimiter, chunk) {
        Some(pos) => {
            buffer.extend_from_slice(&chunk[..=pos]);
            let consumed = pos + 1;
            r.consume(consumed);
            /* process line — same as before */
            buffer.clear();
        }
        None => {
            buffer.extend_from_slice(chunk);
            let consumed = chunk.len();
            r.consume(consumed);
            // continue loop — line crosses buffer boundary
        }
    }
}
```

Punti critici:
- `buffer.clear()` dopo ogni line completata, mai dopo extension parziale
  (continuation case);
- `consume()` chiamato esattamente con il numero di byte estratti dal
  chunk corrente;
- Il delimiter è preservato in `buffer` (semantica identica a `read_until`).

### D-Ondata-7.4 — Lazy line buffer optimization (zero-copy quando possibile)

**Opzionale** — se il fix base (D-7.3) supera il bench gate, fermarsi.
Se invece la win è marginale, considerare il path zero-copy:

Quando una line è interamente contenuta in un singolo `fill_buf` chunk
(no continuation), si può evitare l'`extend_from_slice` e passare
`&chunk[..=pos]` direttamente a `process_line`. Richiede refactor di
`process_line` per accettare `&[u8]` invece di posseduto `Vec<u8>`.

LOC budget: +10 LOC se attivato. **Default**: disattivato. Attivazione
richiede `// PERF-FIX-O7-2:` inline comment + bench validation.

### D-Ondata-7.5 — Profile validation con samply re-run

Re-eseguire il samply profiling con la stessa metodologia di Ondata 5
(Option B, fixture 5.4 MB grep.c × 60, 30 iterazioni helper standalone):

```bash
samply record -o target/profiles/profile_regex_simple_o7.json -- \
  ./target/release/profile_run regex_simple
```

**Aspettativa**: nel profile post-fix, `memchr::memchr` (libcore_simd) o
`memchr::arch::aarch64::neon::memchr` deve dominare al posto di
`memchr_aligned`. Il self time della funzione memchr-related deve scendere
da 32-38 % a <15 % (perché ogni invocation processa più byte per ciclo
grazie a NEON 16-byte SIMD).

Se la metric NON migliora come atteso → fermarsi, taggare
`🟡 BLOCKED — PROFILE-Q`, indagare prima di committare.

### D-Ondata-7.6 — Bench gate

≥+5 % mediana su **almeno 1** dei 3 scenari worst di Ondata 5:
- `regex_simple` (oggi post-O5 = 3.720 ms)
- `invert_match` (oggi post-O5 = 3.586 ms)
- `fixed_string_F` (oggi post-O5 = 4.586 ms)

**Bonus** atteso: ≥+3 % su `literal_match_count`, `bufread_default`
(che hanno già beneficiato del slurp-and-cursor di O5-2 ma scansionano
comunque buffer in-memory con memchr).

**Regression cap**: nessuna regressione >5 % in nessuno degli 11 scenari
benchmark. Cap rigido.

### D-Ondata-7.7 — LOC budget netti

≤40 LOC netti totali:
- ~25-30 LOC in `src/runner.rs::bufread_search` (replacement loop + comments)
- 1 LOC in `Cargo.toml` (`memchr = "2"`)
- ~5-10 LOC `Cargo.lock` (transitive bump se memchr version aggiornata da regex graph)

Esclusi: bench eventuali ri-scrittura (nessuna richiesta in questa
ondata).

### D-Ondata-7.8 — Test parity: 36/36 IDENTICO

Vincolo rigido. La semantica del read-until + line-by-line process è
preservata bit-identica. Se anche 1 testcase diverge, fermarsi, taggare
`🟡 BLOCKED — SPEC-Q`.

### D-Ondata-7.9 — Build precondition (regola CLAUDE.md, invariante)

PRIMA del commit:
- `cargo build` → 0 errori, 0 warning
- `cargo build --release` → 0 errori, 0 warning
- `cargo test` → 36/36 IDENTICO al baseline
- `cargo clippy --tests -- -D warnings` → exit 0
- `cargo bench --no-run` → exit 0
- `cargo fmt --check` → clean

### D-Ondata-7.10 — Single commit format

```
perf(ondata7): memchr direct dep + manual fill_buf loop in bufread_search

IN-SCOPE:
- Add memchr = "2" to Cargo.toml [dependencies] (D-Ondata-7.1 reapre
  D-Ondata-5.10 esplicitamente; motivazione in PROFILE_REPORT.md Ondata 5
  §2.1-2.3 + AUDIT_LOG.md Audit Ondata 5)
- PERF-FIX-O7-1: replace BufRead::read_until with manual fill_buf +
  memchr::memchr loop in src/runner.rs::bufread_search
  (NEON/SSE2/AVX2 aware vector scanning)
- [Optional PERF-FIX-O7-2: lazy line buffer zero-copy fast path]
- Update PERF_REPORT.md with "Ondata 7 post-fix" section (pre/post/Δ
  on 7+ scenarios, focus on 3 SPEC-defined worst from Ondata 5)
- Update PROFILE_REPORT.md (optional) with "Ondata 7 re-profile" appendix
  showing memchr_aligned self% drop from 32-38% to <15%

OUT-OF-SCOPE (deferred to future waves):
- regex crate swap (engine swap wave)
- Threading (rayon)
- FFI bench harness
- is_match_bytes API (Ondata 6 — if not already merged)
- walkdir replacement

Testcase aggiunti: 0. Totali: 36. cargo test IDENTICO baseline <hash>.
cargo build green 0/0. cargo bench --no-run exit 0. cargo clippy default
0 warning.

Co-Authored-By: <agent identifier> <noreply@anthropic.com>
```

### D-Ondata-7.11 — Out-of-scope hard limits

Vietato in questa ondata anche se utile:
- ❌ Sostituire `regex` crate (reserved Ondata "engine swap")
- ❌ Threading (`rayon`) — reserved Ondata "parallelism"
- ❌ FFI bench harness — reserved Ondata "bench infrastructure"
- ❌ Refactor di `process_line` signature oltre il minimo richiesto da D-7.4
- ❌ Modifiche a `output` module, `cli.rs`, `matcher.rs` (Ondata 7 = solo
  runner I/O loop)
- ❌ Modifiche a `tests/testsuite.toml` o `tests/cases/*`

### D-Ondata-7.12 — Executor: Architect-Auditor direct (no Gemini relay)

Come Ondate 4-6. Razionale: scope ben-definito, ma richiede iterazione
interattiva sul profile samply per validare la metric drop. Pattern
relay-shell non si adatta.

### D-Ondata-7.13 — File output policy + PERF_REPORT.md

Committati:
- `Cargo.toml` (+1 LOC: `memchr = "2"`)
- `Cargo.lock` (auto-updated, atteso ≤5 LOC diff)
- `src/runner.rs` (replacement loop con `// PERF-FIX-O7-1:` inline)
- `PERF_REPORT.md` (sezione "Ondata 7 post-fix")
- `PROFILE_REPORT.md` (opzionale, appendix Ondata 7 re-profile)

Non committati: `target/profiles/*`, `target/bench_fixtures/*` (gitignore
già copre via `target/`).

## Checklist Implementer/Executor (pre-commit)

- [ ] `Cargo.toml` aggiornato con `memchr = "2"`
- [ ] `Cargo.lock` rigenerato (diff ≤5 LOC atteso)
- [ ] `src/runner.rs::bufread_search` ha il replacement loop con
      `// PERF-FIX-O7-1:` inline + hypothesis citata
- [ ] (Opzionale) PERF-FIX-O7-2 lazy buffer attivato se win marginale
- [ ] samply re-run su `regex_simple` mostra `memchr` self% <15%
- [ ] `cargo build` 0/0 + `cargo test` 36/36 IDENTICO + clippy `-D warnings` exit 0
- [ ] `cargo bench --no-run` exit 0
- [ ] Full `cargo bench` eseguito, numeri raccolti per "Ondata 7 post-fix"
- [ ] PERF_REPORT.md aggiornato con sezione "Ondata 7 post-fix"
- [ ] (Opzionale) PROFILE_REPORT.md aggiornato con appendix
- [ ] Format commit integrale (IN-SCOPE / OUT-OF-SCOPE / counters)
- [ ] Almeno 1 di {regex_simple, invert_match, fixed_string_F} mostra Δ ≥+5 %
- [ ] Nessuna regressione >5 % in altri scenari
- [ ] Header Ondata 7 in NEXT_STEPS.md aggiornato a `🟢 FATTO — AUDIT PENDING`

## Checklist Auditor (post-commit, sessione successiva)

Pattern come Ondate 4-6:

1. Build verde + test 36/36 IDENTICO baseline (post-O6 commit o `6ab7be1`)
2. Clippy default `-D warnings` exit 0
3. Format commit integrale (IN-SCOPE / OUT-OF-SCOPE)
4. Tutte le 13 D-Ondata-7.k applicate (o BLOCKED con motivazione)
5. `Cargo.toml` ha `memchr = "2"` (direct dep esplicita)
6. `Cargo.lock` diff ragionevole (≤10 LOC, no version bump invasivo)
7. `src/runner.rs::bufread_search` ha replacement loop con
   `// PERF-FIX-O7-1:` inline + hypothesis citata
8. PERF_REPORT.md sezione "Ondata 7 post-fix" con tabella pre/post/Δ,
   almeno 1 dei 3 SPEC-defined worst di Ondata 5 mostra Δ ≥+5 %
9. Nessuna regressione >5 % vs baseline in nessuno scenario
10. (Opzionale) PROFILE_REPORT.md appendix Ondata 7 mostra `memchr` self <15 %
11. Single-commit rule (≤2 hot-fix stesso scope)
12. Branch corretto (main subrepo)
13. Zero scope creep oltre il fix dichiarato; nessun cambio out-of-scope D-7.11

## OUT-OF-SCOPE (accepted, "Future work" per Ondata 8+)

- **regex engine swap** (`regex-lite`, `pcre2`, DFA custom): Ondata
  "engine swap" dedicata se Ondata 7 mostra che il gap residuo
  vs BSD grep è dominato dal regex engine (DFA backtracking).
- **walkdir replacement con hand-rolled `read_dir`**: Ondata "fs walker"
  per chiudere `fixed_string_F` se Ondata 7 non lo risolve.
- **Threading / parallel walk via `rayon`**: Ondata "parallelism".
- **FFI bench harness**: Ondata "bench infrastructure" — rimuove
  process-spawn overhead.
- **`dhat` allocation tracking**: dev-dep opzionale, eventuale Ondata 7-bis
  o post.

## Skill di riferimento (per la sessione executor)

- `m10-performance` (Layer 1 CRITICAL — profiling, criterion, hot path
  analysis)
- `m11-ecosystem` (Layer 2 — `memchr` crate API, `BufRead::fill_buf`/`consume`
  contract)
- `unsafe-checker` (Layer 2 — verificare che `memchr::memchr` è safe wrapper
  sopra unsafe SIMD intrinsics, no nuovo `unsafe` user-space introdotto)
- `domain-cli` (Layer 3 — process-spawn pattern bench)
- `coding-guidelines` (P.PERF naming, // PERF-FIX comment convention)
- `legacy-port` skill — Phase 4d (post-port performance work iteration 2)

Meta-cognition L1+L3 routing applicato.

Riferimenti documentali:
- `memchr` crate: <https://docs.rs/memchr/latest/memchr/fn.memchr.html>
- ripgrep architecture writeup (Andrew Gallant): explains why memchr direct
  use is foundational to grep-like tools
- `BufRead::fill_buf` / `consume` contract: <https://doc.rust-lang.org/std/io/trait.BufRead.html>
- samply per re-profile validation
