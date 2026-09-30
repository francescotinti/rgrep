# Session Handoff — Ondata 7 e verifica GNU locale concluse

> Aggiornato: 2026-09-30, Codex. Anchor rgrep: **`3e9d38f`** (`main`).
> Nessuno step implementativo attivo. CI Linux predisposta, **non eseguita**.
> I commit sono locali; nessun push effettuato.

## Stato consegnato

Il Decider ha autorizzato la sequenza baseline → benchmark 6-bis → Ondata 7
emendata → verifica GNU e aggiornamento documentale. La parte locale è conclusa.

| Commit rgrep | Deliverable |
|---|---|
| `759524b` | Baseline iniziale post-Ondata 6 |
| `723aab3` | 6-bis: benchmark corretto, errori bloccanti, count/quiet fixed, corpus amplificato, baseline nominata |
| `3de67b0` | O7: helper memchr condiviso, cinque nuovi test, misure alternate e profiling |
| `3e9d38f` | Harness GNU/BSD esplicito, confronto byte esatto, differenze note, CI e report |

## Risultati verificati

- Build debug/release, Clippy default/all-features e fmt verdi.
- Suite completa eseguita con GNU grep 3.12 e BSD grep 2.6.0-FreeBSD su
  macOS arm64, rustc 1.98.1, sia default sia PCRE2.
- 42 test Rust default / 44 PCRE2; 129 casi nel manifest.
- GNU default: 121 parità, 1 solo rgrep, 3 skip, 4 differenze note.
- GNU PCRE2: 124 parità, 1 skip, 4 differenze note.
- BSD default: 112 parità, 1 solo rgrep, 16 skip; BSD PCRE2: 112 parità, 17 skip.
- Proprietà realmente eseguite: 3072 per build GNU, 6144 totali; zero su BSD.
- `RGREP_STRICT_GNU=1` provato: fallisce esattamente sui quattro casi noti.
- O7: circa −9%, −12%, −10% sui count amplificati regex/invert/fixed,
  misurati con 31 coppie alternate pre/post. Fixed quiet piccolo: +3,52%
  [1,41;5,22]%, regressione e incertezza dichiarate. Nessun guadagno universale.

## Correzioni metodologiche importanti

Il vecchio `regex_simple` passava una regex invalida e misurava exit 2.
È stato corretto. Le tabelle O6 usavano baseline incoerenti per alcune
percentuali: i dati storici sono qualificati, non usati come nuovi gate.
I vecchi proptest erano verdi su BSD ma saltavano il confronto; ora l'oracolo
è esplicito e gli skip sono visibili. L'xfail obsoleto di directory skip è
stato rimosso. Non sono stati cambiati gli expected rgrep per nascondere errori.

## Prossimi passi, ancora aperti

1. **Eseguire la CI su Linux** dopo la pubblicazione dei commit nel repository
   rgrep. Workflow `.github/workflows/verify.yml`, matrice Linux/macOS ×
   default/PCRE2. YAML e shell validati staticamente; nessuna run remota osservata.
2. **SPEC dedicata per diagnostica binaria GNU**: casi 0054 e 0067, stderr
   GNU contro stdout rgrep. Risolvere comportamento e testcase con evidenza.
3. **SPEC dedicata per allineamento `-T`**: casi 0064 e 0065; padding e
   posizione tabulazione differiscono. Le divergenze sono validate separatamente,
   ma restano difetti, non parità riuscita.
4. Rimuovere le relative eccezioni soltanto dopo correzione e test; la modalità
   strict deve infine diventare verde. Rinviare engine swap e parallelismo.

## File da leggere

- `rgrep/GNU_VERIFICATION.md`: copertura eseguita, differenze e riproduzione.
- `rgrep/PERF_REPORT.md`: appendici 6-bis/O7, tabelle e limiti delle misure.
- `rgrep/PROFILE_REPORT.md`: profilo aggiornato; il nuovo helper assorbe parte
  dei simboli inline, la sola sparizione di memchr_aligned non prova speedup.
- `rgrep/reports/`: JSON versionati con stime/raw timing/hash binari e riepiloghi.
- `NEXT_STEPS.md`: emendamento O7 e specifica verifica GNU; cronologia conservata.
- `AUDIT_LOG.md`: ultimi audit esplicitamente eseguiti dallo stesso Codex,
  senza attribuire indipendenza alla review.

## Comandi utili (da rgrep/)

```sh
cargo build --locked
RGREP_ORACLE=/opt/homebrew/bin/ggrep cargo test --locked -- --nocapture
RGREP_ORACLE=/opt/homebrew/bin/ggrep cargo test --locked --features perl-regexp -- --nocapture
RGREP_ORACLE=/usr/bin/grep cargo test --locked -- --nocapture
cargo clippy --locked --tests --benches --all-features -- -D warnings
cargo fmt --check
```

I percorsi sopra si riferiscono al Mac attuale. Su Linux `/usr/bin/grep` è
l'oracolo GNU. Il benchmark richiede il sibling `gnu-grep/`; i test e la CI no.
Artefatti locali non versionati: `rgrep/target/verification/` e
`rgrep/target/profiles/o7-{before,after}.json`. Gli avvisi hard-link ExFAT
non sono lint di codice. La root resta un placeholder Cargo: lavorare in rgrep.

Le directory preesistenti non tracciate `.serena/`, `.vexp/`, `_archive/` e
lo stato non pulito di `gnu-grep` non fanno parte di questi commit.
