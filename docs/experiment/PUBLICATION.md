# Pubblicazione della documentazione dell’esperimento

Snapshot esportato il 2026-09-30 dal workspace locale `testag-grep`.
Il repository esterno non ha un remoto: l’utente ha scelto di pubblicare
la documentazione dentro `francescotinti/rgrep`, nella presente cartella.

- Anchor della documentazione outer: `3cdf1a3` (hash locale, non presente
  nella cronologia del repository rgrep).
- Anchor codice pubblicato prima di questo snapshot: `3e9d38f`.
- Inclusi: README del workspace, istruzioni storiche CLAUDE, piano,
  specifiche, audit, handoff, tutti gli otto diari e 19 script storici di commit.
- Gli script in `_archive/commit_scripts/` sono conservati come documentazione:
  contengono percorsi locali originali e non sono comandi di setup del progetto.
- Cache, log e configurazioni locali di Serena/Vexp, file macOS, target di
  compilazione e checkout upstream GNU non fanno parte dello snapshot.
- La frase «nessun push effettuato» nei report precedenti descrive la chiusura
  locale antecedente alla richiesta di pubblicazione; il codice è ora su GitHub.
- La prima run CI è partita dopo il push del codice:
  [Differential verification](https://github.com/francescotinti/rgrep/actions/runs/36766671569).
  Entrambi i job Linux e macOS con PCRE2 sono passati. macOS default è fallito
  prima della build per un array vuoto sotto Bash 3 con nounset; il workflow
  è stato corretto usando parametri posizionali. L’esito della nuova run va
  verificato nella pagina Actions.

I link Markdown verso rgrep sono adattati; comandi e percorsi citati nelle
specifiche storiche restano quelli originali. Per build e test usare il
[README del codice](../../README.md) e il [report GNU](../../GNU_VERIFICATION.md).
Per future modifiche alla documentazione esterna aggiornare anche lo snapshot.
