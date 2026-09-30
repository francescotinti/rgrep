# metrics — testag-grep

## Stato corrente — 2026-09-30, anchor `3e9d38f`

| Metrica | Valore verificato |
|---|---|
| Rust `src/*.rs`, inclusi test inline | 2379 righe |
| Casi nel manifest / file TOML | 129 / 129 |
| Test Rust default / PCRE2 | 42 / 44 |
| GNU default: parity / only-rgrep / skip / known | 121 / 1 / 3 / 4 |
| GNU PCRE2: parity / only-rgrep / skip / known | 124 / 0 / 1 / 4 |
| BSD default: parity / only-rgrep / skip | 112 / 1 / 16 |
| BSD PCRE2: parity / only-rgrep / skip | 112 / 0 / 17 |
| Confronti generativi GNU in sessione | 6144 (3072 per build) |
| Confronti generativi BSD | 0, skip esplicito |
| Gruppi benchmark | 10 |
| Miglioramento O7 su corpus count amplificato | circa 9–12%, coppie alternate |
| Build / Clippy / fmt | verdi |
| Linux CI | predisposta; non eseguita |

Dati numerici e limiti in `rgrep/reports/` e `rgrep/GNU_VERIFICATION.md`.
I quattro known GNU sono debito funzionale, non test di parità riusciti.

## Baseline verificata il 2026-09-30 (post-Ondata 6)

- Commit rgrep `759524b`; 2277 righe in `src/*.rs`.
- 129 casi TOML registrati (129 file), di cui 17 marcati skip BSD e uno
  `expected_to_fail`. Il conteggio dei file non equivale ai casi verificati.
- Default: 32 unit test + 1 harness + 3 proprietà; PCRE2: 34 + 1 + 3.
- Su BSD le proprietà fanno early return: **nessun confronto generativo**.
- Build, test default/PCRE2, Clippy all-features e fmt verificati verdi.
- GNU grep 3.12 locale: prima esecuzione del solo harness default con
  **8 differenze**, da classificare nella fase di verifica GNU.
- Le metriche sotto sono una fotografia dello Step 17, non dello stato corrente.

## LOC

| Source | LOC |
|---|---|
| Pre-experiment Rust (rgrep baseline) | ~741 |
| Post-experiment Rust (src/*.rs) | ~1443 |
| Net additions | ~702 |
| GNU grep upstream (src/*.c) | ~4500 |
| Compressione C -> Rust | ~68% |

## Test

| Metric | Value |
|---|---|
| Differential testcase statici | 74 |
| Proptest cases (3 strategies × 1024) | 3072 |
| Unit test totali | 32 |
| Test green rate | 100% |

## Process

| Metric | Value |
|---|---|
| Step totali | 17 main + 8 -bis = 25 |
| ✅ APPROVED | 25 |
| 🟡 PARTIAL recovered | 6 |
| ❌ REJECTED | 0 |
| Commit rgrep | 24 |
| Commit outer | 26 |

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
| Phase 2 (mapping, Step 2) | ~2.0h |
| Phase 3 (impl Step 3-15) | ~14.0h |
| Phase 4 (proptest Step 16) | ~2.0h |
| Phase 5 (this conclusions Step 17) | ~1.0h |
| **Totale** | **~20.5h** |
