# Documentazione dell'esperimento testag-grep

Porting di **GNU grep** (C, ~4,5K LOC) verso Rust (`rgrep`, ~2,4K LOC)
con workflow multi-agent Architect/Implementer/Decider, seguendo la skill
`legacy-port`. Gemello metodologico di `rawk` (One True Awk → Rust).

La documentazione è divisa in tre file, per tre domande:

| Domanda | File |
|---|---|
| Cosa resta da fare? | [TODO.md](TODO.md) — backlog ordinato, SPEC degli step attivi, scope-out |
| Cosa è stato fatto? | [DONE.md](DONE.md) — metriche verificate, cronologia step e ondate, lezioni, registro audit con anchor |
| Com'è fatto e come ci è arrivato? | [ARCHITECTURE.md](ARCHITECTURE.md) — moduli C e strategia per modulo, struttura Rust, flusso, testing, divergenze, evoluzione |

Le regole operative (ruoli, comandi, gate, anti-pattern) sono nel
`CLAUDE.md` alla radice del workspace outer, unico file rimasto lì.

## Materiale storico integrale

In [history/](history/), congelato al 2026-09-30. Percorsi e hash citati
al suo interno si riferiscono al workspace outer originale.

- [NEXT_STEPS.md](history/NEXT_STEPS.md) — tutte le SPEC (D-decisioni) di Step 0–19 e Ondate 1–7, ~7 800 righe. Navigare con `rg -n '^# ' history/NEXT_STEPS.md`.
- [AUDIT_LOG.md](history/AUDIT_LOG.md) — verdetti integrali con checklist e anchor.
- [EXPERIMENT_PLAN.md](history/EXPERIMENT_PLAN.md) — piano iniziale (maggio 2026) con stime e predizioni.
- [SESSION_HANDOFF.md](history/SESSION_HANDOFF.md) — ultimo handoff prima della riorganizzazione.
- [README-workspace.md](history/README-workspace.md) — README del workspace outer.
- [diary/](history/diary/) — 01 semantic model, 02 mapping table, 03 translation log, 04a divergences, 04b performance, 05 profiling, 99 conclusions, metrics.
- [commit_scripts/](history/commit_scripts/) — 19 script del pattern relay-shell (Step 10–19), conservati come documentazione: contengono percorsi locali.

## Documentazione del codice (fuori da questa cartella)

- [README](../../README.md) — feature e uso della CLI
- [PORTING.md](../../PORTING.md) — cosa è stato consegnato
- [GNU_VERIFICATION.md](../../GNU_VERIFICATION.md) — copertura reale, divergenze, riproduzione
- [PERF_REPORT.md](../../PERF_REPORT.md) e [PROFILE_REPORT.md](../../PROFILE_REPORT.md) — benchmark e profili, con appendici 6-bis/O7
- [tests/README.md](../../tests/README.md) — schema dei testcase e oracolo
- [reports/](../../reports/) — JSON versionati di bench, profili, verifica e CI

Provenienza e pubblicazione: [PUBLICATION.md](PUBLICATION.md).
