# Pubblicazione e provenienza della documentazione

## Situazione corrente (dal 2026-09-30, sera)

Questa cartella è la **sede nativa** della documentazione dell'esperimento,
non più uno snapshot. Il workspace outer `testag-grep` (senza remoto)
conserva soltanto `CLAUDE.md` con le regole operative; tutto il resto è
stato spostato qui e riorganizzato in [TODO.md](TODO.md), [DONE.md](DONE.md)
e [ARCHITECTURE.md](ARCHITECTURE.md), con il materiale storico integrale
in [history/](history/).

Per modificare la documentazione si lavora direttamente in questo repository.

## Storico della pubblicazione

- Codice pubblicato su `https://github.com/francescotinti/rgrep` il
  2026-09-30 a partire da `3e9d38f`.
- Prima run CI `36766671569` (su `3e9d38f`): Linux default e PCRE2 verdi,
  macOS PCRE2 verde; macOS default fallito prima della build perché Bash 3
  rifiuta un array vuoto sotto `set -u`. Corretto in `ab26dc4` con parametri
  posizionali. Dettagli in `reports/ci-first-run-20260930.json`.
- Seconda run CI `36766955016` (su `92285c7`): verde su tutti e quattro i
  job. È la prima esecuzione della suite con oracolo GNU su Linux.
- `92285c7` aveva importato uno snapshot dei documenti outer (anchor outer
  `3cdf1a3`, hash non presente in questa cronologia); lo snapshot è stato
  sostituito dalla riorganizzazione corrente.

## Note di provenienza

- Gli script in `history/commit_scripts/` contengono percorsi locali
  originali e non sono comandi di setup del progetto.
- Percorsi shell e hash citati dentro `history/` descrivono l'ambiente
  outer originale (`/Volumes/Extreme Pro/Claude/testag-grep`).
- Cache e configurazioni locali (Serena, Vexp), file macOS `._*`, target
  di compilazione e il checkout upstream `gnu-grep/` non fanno parte del
  repository; i benchmark richiedono `gnu-grep/` come sibling, i test e la
  CI no.
