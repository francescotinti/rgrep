# 99 — Conclusions

> Le sezioni originali sotto descrivono Step 17–19. La chiusura aggiornata
> dopo Ondata 7 e verifica GNU è nell’appendice 2026-09-30.

## §1 Dove l'AI ha funzionato meglio del previsto?
La capacità di traduzione dei costrutti semantici e la padronanza di macro-feature di Rust (in primis l'integrazione di crates standard per rimpiazzare funzioni custom in C) si sono rivelati superiori alle aspettative. In particolare, il setup differenziale iniziale con `tests/diff_runner.rs` e la transizione architetturale del Matcher in `src/matcher.rs` hanno mostrato come l'AI possa astrarre con precisione il modello semantico di GNU grep. Non solo, anche l'adozione di `walkdir` (Step 7), la fallback routine `unsafe` su `mmap` isolata impeccabilmente a un solo blocco (Step 14) e la perfetta padronanza delle opzioni di `clap` dimostrano che a livello di coding "core", l'agent è ampiamente autonomo e corretto al primo tentativo, riducendo un codice monolitico in C (~4500 LOC) ad un idiomatico porting Rust di ~1443 LOC.

## §2 Dove ha fallito sistematicamente?
Il fallimento sistematico si è concentrato quasi esclusivamente sul "process discipline". Nonostante le chiare direttive della documentazione `CLAUDE.md`, l'agent ha inciampato ripetutamente nei medesimi pattern:
- **Outer Commit Dimenticato:** In 3 step consecutivi (Step 12, 14, 15), la disciplina di committare i documenti del workspace `outer` è stata violata (generando l'innesco di continui Step N-bis di recupero).
- **Build Red Committato:** Negli Step 7 e 11 l'agent ha fornito script per effettuare il commit senza prima aver superato validazioni essenziali o senza considerare warning non fatali (gestiti poi).
- **Titoli dei Commit Errati:** Titoli "fix(stepN-hotfix)" usati impropriamente invece di rispettare la tassonomia decisa per gli scope `feat(stepN)`.

Tutto ciò evidenzia la necessità di costringere il modello a script bash del tutto separati (Pattern relay-shell esplicito in due blocchi separati `rgrep` e `outer`) al fine di forzarlo a validare le precondizioni separatamente.

## §3 Quanto della "conoscenza tacita" del codice originale è stata catturata vs persa?
**Catturata:** La superficie delle API in input (gestione di directory, file speciali, stdin fallback, group separator, limitazioni per directory e device). L'edge-case management, specialmente la distinzione tra TTY interattivi e fallback su file (D-G12, binary files), e i fallback PCRE2.
**Persa:** Micro-ottimizzazioni estreme (come l'utilizzo della struct custom `dfa_comp` e l'elaborato NFA→Posix fallback interno al C originale di grep, interamente appaltate al crate `regex` che generalizza la maggior parte di queste logiche), oltre alle estreme gestioni di locale multi-byte (C-style mb_clen) che Rust normalizza gestendo stringhe o slice nativi (`&[u8]`).

## §4 Il workflow TDD inverso ha funzionato? Quale alternativa avresti provato?
Il TDD inverso ("TDD-first") - in cui testcase specifici (da oracolo GNU/ugrep) venivano generati PRIMA dell'implementazione - ha funzionato egregiamente per isolare la logica delle feature in scope. L'oracolo ha garantito che il porting rimanesse pragmatico, bypassando dubbi di design teorici per abbracciare l'output crudo (vedi il setup del diff runner).
**Alternativa inesplorata:** Importare as-is un dump selettivo della cartella `tests/` upstream di GNU grep e lanciare uno script che "inferisce" expected-outputs iterando le test suite per deduzione, riducendo l'onere manuale dell'Architect nella specifica dei casi (seppur introducendo rumore inutile derivante da flag deprecati o OS-specifici di GNU test suite).

## §5 Quanto è generalizzabile questo metodo ad altri progetti?
Il porting legacy metodologico applicato qui è generalizzabile all'intero panorama dei tool Unix storici. I pattern documentati come il `relay-shell && chain`, le estensioni harness flessibili (`fixture_files`), il tracciamento formale delle divergenze in `D-NEW-N` e la progressione chirurgica in "Step", possono essere un blueprint ufficiale per porting paralleli quali `sed`, `find`, o coreutils affini (lo conferma anche il "gemello" `rawk`). Il property-based testing comparativo ("Phase 4b") si conferma una tecnica essenziale che ogni port legacy dovrebbe prevedere per trovare asimmetrie latenti invisibili al TDD.

## §6 Qual è il vero collo di bottiglia: capacità dell'AI, qualità dei prompt, o test suite di partenza?
Il collo di bottiglia indiscutibile è **la process discipline relay-shell** (disciplina d'esecuzione). Le capacità deduttive e traduttive dell'AI applicate su astrazioni C vs Rust sono eccellenti e scalano bene, tuttavia, il processo va a sbattere contro la tendenza del modello a "precipitarsi al traguardo", bypassando pipeline di test e commit multi-repository. La qualità dei prompt (Decision-D dell'Architect) previene allucinazioni logiche ma non argina la mancanza di feedback loop real-time a meno che l'AI non sia costretta a script atomici o framework in cui fallire la pre-condizione blocca l'intero esito della run. L'Agent è un brillante coder ma un pessimo manager dei propri workflow a meno che non venga tenuto strettamente ai binari del "relè".

## Appendice: Step 18 Reopen
L'esperimento è stato riaperto allo Step 18 per esplorare concretamente l'alternativa inesplorata citata nel §4: l'importazione automatizzata della test-suite shell originaria di GNU grep. Attraverso uno script Python custom (`convert_gnu_tests.py`), siamo riusciti ad analizzare decine di file shell, parsare i blocchi `echo | grep` bypassando costrutti non portabili, e generare in tempo reale testcase TOML interrogando il `grep` di sistema per ricavare gli *expected output* dinamici. 
Questa estensione ha amplificato a dismisura il volume di diff-testing (spingendo a galla edge case incredibili, come il mascheramento di sintassi invalida con molteplici flag `-e` scoperto nel test `backref`, ora rubricato come `D-NEW-5`). Si conferma la regola d'oro del re-engineering legacy: **non c'è TDD progettato a tavolino che possa superare l'implacabile copertura dell'oracolo originale lanciato in brute-force sulle sue stesse suite.**


## Appendice 2026-09-30 — misure e copertura verificabili

La revisione ha individuato limiti non riconducibili alla sola disciplina
di commit: un benchmark passava una regex invalida e ignorava exit 2; alcune
percentuali storiche usavano baseline diverse dalle mediane tabellate; i tre
proptest risultavano verdi su BSD pur saltando tutti i confronti. Quindi
«test green» e «audit APPROVED» non bastano da soli a dimostrare copertura
o miglioramento prestazionale.

Correzioni consegnate: benchmark che fallisce sugli errori, baseline nominate
e artefatti numerici, conteggi reali del harness, oracolo esplicito e
6144 confronti generativi eseguiti contro GNU. Quattro difetti GNU rimangono
visibili e rifiutati dalla modalità strict.

Ondata 7 ha ridotto di circa 9–12% tre workload count amplificati. La deriva
fra run sequenziali era enorme anche per BSD invariato: solo l'esperimento
con binari pre/post alternati ha isolato un beneficio credibile. I casi piccoli
restano perlopiù piatti e fixed quiet mostra una regressione ~3,5%.

La parte locale dell'iterazione è chiusa (`3e9d38f`). Non si dichiara parità
GNU completa né portabilità Linux verificata: CI Linux predisposta ma non
eseguita, output binario e -T ancora da allineare.
