# testag-grep — GNU grep → Rust (esperimento multi-agent)

Esperimento di porting di **GNU grep** (C, ~4.5K LOC) verso Rust (`rgrep/`,
~2.4K LOC) seguendo la skill `legacy-port` con workflow a tre ruoli:
**Architect/Auditor** (scrive SPEC e audita), **Implementer** (implementa uno
step alla volta, un commit per step) e **Decider** (umano, relay e decisioni
finali). Gemello metodologico di `rawk` (One True Awk → Rust).

## Stato (2026-09)

- Step 0–19: porting funzionale completato (flag principali di GNU grep,
  `-P` via feature opzionale `perl-regexp`/pcre2, mmap, contesto, ricorsione,
  filtri, binary/NUL handling).
- Ondate 1–7 + 6-bis: ✅ chiuse. Benchmark corretti, baseline esplicite,
  scansione delimitatori con `memchr`; benefici misurati circa 9–12% sui
  tre scenari count amplificati (nessuna promessa universale).
- Verifica GNU/BSD locale completata, default e PCRE2: 42/44 test Rust;
  129 casi manifest con conteggi parità/skip/divergenze separati, 6144
  confronti generativi GNU nella sessione. Quattro divergenze GNU restano aperte.
- CI Linux/macOS predisposta e verificata staticamente; **Linux non eseguito**.
- Anchor corrente rgrep: `3e9d38f`. Prossime priorità: run Linux e correzione
  output binario / allineamento `-T`, non nuove ottimizzazioni speculative.

Dettagli operativi e handoff in [SESSION_HANDOFF.md](SESSION_HANDOFF.md).

## Layout

| Path | Contenuto |
|---|---|
| `rgrep/` | Il port Rust. **Repo git annidato** (branch `main`), gitlink senza `.gitmodules`. |
| `gnu-grep/` | Snapshot upstream GNU grep (C + 149 testcase shell), riferimento read-only. |
| `minigrep.c`, `Cargo.toml`, `src/main.rs` | Placeholder didattico originale (~30 LOC). **Non è il progetto**: ignorare. |
| `CLAUDE.md` | Ruoli, comandi, regole invariabili, gotcha ambiente. |
| `NEXT_STEPS.md` | Backlog e SPEC contrattuali (D-decisioni) per ogni Step/Ondata. |
| `AUDIT_LOG.md` | Verdetti di audit con anchor hash. |
| `EXPERIMENT_PLAN.md` | Mappatura sulle fasi della skill `legacy-port`. |
| `diary/` | Semantic model, mapping table, translation log, divergenze, performance, profiling, conclusioni, metriche. |

## Build e test

Tutto avviene in `rgrep/`, mai nella root:

```bash
cd rgrep
cargo build
cargo test                                  # differential vs grep di sistema + proptest + unit
cargo clippy --tests -- -D warnings
cargo fmt --check
cargo test --features perl-regexp           # abilita -P (richiede libpcre2)
cargo bench                                 # criterion, vedi rgrep/PERF_REPORT.md
```

I testcase differenziali sono file TOML in `rgrep/tests/cases/` registrati in
`rgrep/tests/testsuite.toml`. L'oracolo è il `grep` di sistema: su macOS è
BSD grep, quindi le divergenze GNU-only sono marcate `skip_if_bsd`.

## Documentazione del port

- [rgrep/README.md](rgrep/README.md) — feature supportate e uso della CLI
- [rgrep/PORTING.md](rgrep/PORTING.md) — cosa è stato consegnato
- [rgrep/PERF_REPORT.md](rgrep/PERF_REPORT.md) — benchmark vs grep di sistema
- [rgrep/PROFILE_REPORT.md](rgrep/PROFILE_REPORT.md) — profiling samply e fix mirati
- [rgrep/GNU_VERIFICATION.md](rgrep/GNU_VERIFICATION.md) — copertura reale, divergenze e CI
- [diary/99-conclusions.md](diary/99-conclusions.md) — lezioni apprese sul workflow multi-agent

## Riferimento C originale

Il placeholder `minigrep.c` resta compilabile per confronto storico:

```bash
gcc minigrep.c -o minigrep_c
./minigrep_c <pattern> <filename>
```
