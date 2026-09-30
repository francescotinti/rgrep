# CLAUDE.md — testag-grep (rgrep experiment)

> Esperimento di porting **GNU grep (C) → Rust** in modalità multi-agent
> seguendo la skill `legacy-port` (Anthropic Claude). Riferimento metodologico:
> esperimento gemello `rawk` (One True Awk → Rust, mag 2026).

## Ruoli (workflow multi-agent)

| Ruolo | Responsabilità | Chi |
|---|---|---|
| **Architect + Auditor** | Scrive le SPEC (D-decisioni) in `NEXT_STEPS.md`, audita ogni commit, mantiene il backlog. **NON scrive codice.** | Claude Opus 4.7 (1M context) |
| **Implementer** | Legge UNO step `🚧 PRONTO`, aggiunge testcase TDD-first, implementa, fa **un singolo commit**. **NON decide architettura, non riapre D-decisioni.** | Step 0–19: Gemini Antigravity (via relay-shell). Ondate 1+: Claude single-shot (nessun relay-shell) |
| **Decider** | Relay minimale tra i due. Decisore finale su trade-off ambigui. | Utente (Francesco) |

## File chiave

- [NEXT_STEPS.md](NEXT_STEPS.md) — workflow attivo. Solo UN step alla volta è 🚧 PRONTO; gli altri sono 🔒 LOCKED.
- [AUDIT_LOG.md](AUDIT_LOG.md) — log con anchor hash per ripartire da dove eravamo (no "ricorda dove eravamo": è scritto).
- [EXPERIMENT_PLAN.md](EXPERIMENT_PLAN.md) — sintesi metodologica e mapping rispetto alle 4 fasi della skill `legacy-port`.
- [diary/](diary/) — semantic model, mapping table, divergence log, conclusions.

## Fase attuale (2026-09)

- Step 0–19 (porting funzionale) chiusi. Fase corrente: **Ondate** (hardening
  post-port: lint, refactor, architettura, performance). Ondate 1–7 e 6-bis ✅ APPROVED.
  Verifica GNU locale conclusa, CI Linux predisposta ma non eseguita.
  Anchor `3e9d38f`; nessuno step implementativo attivo. Priorità residue:
  run CI Linux, quattro differenze GNU su diagnostica binaria e `-T`.
  Nessuna nuova promozione senza richiesta esplicita del Decider.
- `NEXT_STEPS.md` è ~7800 righe: navigare con `rg -n '^# ' NEXT_STEPS.md`
  e leggere solo la sezione dell'Ondata/Step corrente.
- Ciclo outer per ogni Ondata (3 commit): `docs(ondataN): promote 🔒→🚧` →
  `docs(handoff): … (sub→<hash>)` → `docs(audit): close … ✅ APPROVED (sub→<hash>)`.
- Audit = checklist 12/12 in `AUDIT_LOG.md` (template: ultima sezione
  "Audit Ondata N"). L'Auditor è read-only sul codice `rgrep`.

## Comandi (eseguire in `rgrep/`, NON nella root)

```bash
cd rgrep
cargo build
cargo test                                        # differential + proptest + unit
cargo clippy --tests -- -D warnings                 # gate: exit 0
cargo fmt --check                                   # gate: exit 0
cargo test --features perl-regexp                   # abilita -P (pcre2 opzionale)
CARGO_TERM_QUIET=false cargo test --test diff_runner -- --nocapture  # report per-case
cargo bench                                         # criterion, benches/search.rs
python3 tools/convert_gnu_tests.py                  # importa gnu-grep/tests → TOML
```

La root `testag-grep/` ha un `Cargo.toml` + `src/main.rs` placeholder
(minigrep originale): ignorarli, il progetto vero è `rgrep/`.

## Layout repo

```
testag-grep/
├── CLAUDE.md                # questo file
├── EXPERIMENT_PLAN.md       # methodology summary
├── NEXT_STEPS.md            # workflow attivo (Architect → Implementer)
├── AUDIT_LOG.md             # audit anchor log (Architect)
├── minigrep.c               # C reference minimale (~30 LOC)
├── gnu-grep/                # snapshot upstream GNU grep (~4.5K LOC C, 149 testcase)
├── SESSION_HANDOFF.md       # handoff Implementer→Auditor (può essere stale)
├── rgrep/                   # workspace Rust (repo git ANNIDATO, branch `main`)
│   ├── Cargo.toml           # feature opzionale `perl-regexp` (pcre2)
│   ├── src/
│   │   ├── cli.rs           # clap-derive Config → 5 sub-struct
│   │   ├── runner.rs        # I/O orchestration, process_line, walk_recursive
│   │   ├── matcher.rs       # MatchEngine trait + Engine enum (regex/aho/fancy/pcre2)
│   │   ├── output.rs        # GrepColors, highlight() Cow
│   │   ├── error.rs         # thiserror
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── tests/
│   │   ├── diff_runner.rs   # harness differenziale process-spawn vs `grep` di sistema
│   │   ├── proptest_differential.rs
│   │   ├── testsuite.toml   # manifest autoritativo
│   │   └── cases/NNNN_<name>.toml   # 129 case
│   ├── benches/search.rs    # criterion
│   ├── tools/               # convert_gnu_tests.py, curate.py
│   ├── PERF_REPORT.md · PROFILE_REPORT.md · PORTING.md
│   └── README.md
└── diary/                   # 01-03, 04a-divergences, 04b-performance, 05-profiling, 99, metrics
```

## Regole operative invariabili

1. **Lingua**: codice + commenti tecnici inline + commit message → **inglese**.
   Diario, NEXT_STEPS, AUDIT_LOG, discussioni con il Decider → **italiano**.
2. **Branch**: outer `master`, subrepo `rgrep` `main`; commit conventional-style.
3. **Commit per step**: ogni step → **un solo commit**. Mai più step in un commit.
4. **Build green = precondizione**: `cargo build` 0 warning + `cargo test` verde
   PRIMA di ogni commit. Mai committare con build red (anti-pattern critico
   rawk Step 13).
5. **TDD-first**: l'Implementer aggiunge **prima** i testcase falliti, poi
   l'implementazione che li fa passare.
6. **D-decisioni sono contratto**: l'Implementer le applica letteralmente, non
   "in spirito". In caso di ambiguità non coperta:
   - `// DESIGN-Q: <domanda>` per ambiguità di codice → tagga step `🟡 BLOCKED — DESIGN-Q`
   - `<!-- SPEC-Q: <motivo> -->` per testcase apparentemente errati nello SPEC
     → tagga `🟡 BLOCKED — SPEC-Q`. **NON cambiare expected silenziosamente.**
7. **Commit message format obbligatorio** (vedi NEXT_STEPS.md, ripetuto in
   ogni step):
   ```
   feat(stepN): <titolo breve>

   IN-SCOPE:
   - <bullet list di cosa è stato implementato>

   OUT-OF-SCOPE (debito esplicito):
   - <bullet list di cosa è stato deliberatamente lasciato fuori>

   Testcase aggiunti: N. Totali: M.
   ```
8. **Recovery via -bis**: se un audit rivela uno step `🟡 PARTIAL`, l'Architect
   apre `Step N-bis` con i leftover formalizzati come `D N-bis.k`. Niente debito
   tecnico nascosto.
9. **Step lockati**: l'Implementer non avvia uno step `🔒 LOCKED` finché
   l'Architect non lo promuove a `🚧 PRONTO`.

## Pattern relay-shell (solo se l'Implementer è Gemini)

Applicato negli Step 0–19. N/A nelle Ondate (Implementer Claude con terminale).

L'Implementer (Gemini Antigravity) **non esegue direttamente comandi sul
terminale**. Il Decider fa relay copia-incolla tra Gemini e zsh. Quando
Gemini fornisce un blocco multi-comando, il Decider lo incolla tutto in
una volta. Se `cargo test` fallisce nel mezzo, il `git commit` parte
comunque (incolla cieca).

**Pattern obbligatorio per Gemini**: fornire **UN UNICO blocco shell
atomico con `&&` chain** che fa fail-fast:

```bash
set -o pipefail && \
cd /Volumes/Extreme\ Pro/Claude/testag-grep/rgrep && \
  cargo build 2>&1 | tail -5 && \
  cargo test 2>&1 | tail -10 && \
  git add -A && \
  git commit -m "$(cat <<'EOF'
feat(stepN): <titolo>

IN-SCOPE:
- ...

OUT-OF-SCOPE (debito esplicito):
- ...

Testcase aggiunti: X. Totali: Y.
EOF
)"
```

Razionale: `pipefail` preserva l'exit code di Cargo anche attraverso `tail`;
con `&&` short-circuit, se uno qualunque dei test/build
fallisce, lo `git commit` **non** viene eseguito. Niente commit zombie
su working tree broken.

Trade-off accettato: se nonostante il `&&` chain serve un secondo commit
hot-fix (es. perché il 1° passa cargo test ma rivela un bug solo a live
exec), è **accettabile** purché:
- Titolo `fix(stepN-hotfix): <fix>` (NON duplicato del 1°)
- Body referenzia hash del 1° (`fixes <hashN>`)
- Solo il fix necessario, no scope creep

## Differential testing strategy (D-globale)

- **Process-spawn** (non FFI): l'oracolo è il binario `grep` di sistema
  (`/usr/bin/grep` su macOS = BSD grep; su Linux = GNU grep). Trade-off
  documentato negli audit dei test che divergono per BSD vs GNU.
- Testcase: `tests/cases/NNNN_<name>.toml` (campi: `name`, `args`, `stdin`,
  `expected_stdout`, `expected_exit_code`; opzionali `skip_if_bsd`+`skip_reason`,
  `requires_feature = "perl-regexp"`, `fixture_files`, `env`,
  `sort_output`). Ogni case va registrato in `tests/testsuite.toml`.
  Il campo storico `expected_to_fail` è stato rimosso: schema rigoroso,
  `oracle_args` per equivalenza `--mmap`, `rgrep_only`/`forbids_feature` per
  capability PCRE, output GNU separato per quattro divergenze note. Vedi
  `rgrep/tests/README.md` e `rgrep/GNU_VERIFICATION.md`.
- Oracolo su macOS = BSD grep 2.6.0 (`is_bsd_grep()`): divergenze GNU-only vanno
  marcate `skip_if_bsd`, mai adattando l'expected.
- `RGREP_ORACLE` seleziona il binario; identità GNU/BSD rilevata una volta.
  Tre proprietà da 1024 casi eseguite solo con GNU; su BSD skip esplicito.
  `RGREP_STRICT_GNU=1` rifiuta anche le quattro divergenze note.
- Proptest differenziale in `tests/proptest_differential.rs` (skill
  warning: range stretti perdono bug — vedi rawk orphan-dot bug).

## Gotcha ambiente

- **Subrepo**: `rgrep` è un gitlink senza `.gitmodules`. Commit prima in `rgrep/`
  (branch `main`), poi `git add rgrep` nell'outer (`master`) per bumpare il puntatore.
- **Volume ExFAT**: `cargo build`/`clippy` stampano `warning: rgrep (…) generated N
  warning` per la hard-link cache del filesystem. Non sono lint: il gate è
  `cargo clippy --tests -- -D warnings` con exit 0.
- Output cargo salvato su file contiene escape ANSI: filtrare con `awk`, non `grep`.

## Anti-pattern (lista vincolante per l'Implementer)

- ❌ Committare con `cargo test` rosso o `cargo build` con warning
- ❌ Modificare `expected` di un testcase per "farlo passare" senza SPEC-Q
- ❌ Aggiungere feature non richieste dallo step (scope creep)
- ❌ Refactor cosmetici fuori-scope nello stesso commit
- ❌ Più di un commit per step **con scope creep nel 2° commit**
  (vedi pattern relay-shell sopra: hot-fix dello stesso scope = OK con
  titolo `fix(stepN-hotfix)`)
- ❌ Avviare uno step `🔒 LOCKED` senza promozione

## Riferimenti

- Skill: `~/.claude/skills/legacy-port/SKILL.md`
- Esperimento gemello: `rawk` (One True Awk → Rust)
- Upstream GNU grep: <https://git.savannah.gnu.org/cgit/grep.git>
