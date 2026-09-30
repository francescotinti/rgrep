# 04b Performance

Questo capitolo chiude il ciclo dell'esperimento sul lato **misura**: dopo
le tre ondate di pulizia (Ondata 1 lint, Ondata 2 strutturale, Ondata 3
architetturale), Ondata 4 ha trasformato in numeri ciò che fino a quel
momento erano ipotesi. Il deliverable tecnico vive in
[`rgrep/PERF_REPORT.md`](../rgrep/PERF_REPORT.md); qui raccontiamo *cosa*
abbiamo deciso di misurare, *come* lo abbiamo misurato e *cosa* ne
abbiamo imparato.

## 1. Cosa abbiamo misurato — e cosa no

Abbiamo strutturato sette scenari criterion (cinque obbligatori dalla
spec D-Ondata-4.3 più due di validazione retroattiva):

1. **`literal_match_count`** — `-c "include"` su cinque `.c` del subtree
   `gnu-grep/src/`. Misura il path letterale "veloce" (aho-corasick lato
   rgrep, fast-path lato BSD grep).
2. **`regex_simple`** — `-E "^[a-z]+\("` su `grep.c` (3036 righe). Misura il
   path regex semplice ERE.
3. **`fixed_string_F`** — `-rF "MB_LEN_MAX"` ricorsivo. Forza il fast-path
   Aho-Corasick e contemporaneamente il walker `walkdir`.
4. **`recursive_walk`** — `-r "TODO"` ricorsivo. Stress su `runner::walk_recursive`,
   con sample_size ridotto a 20 (scenario più costoso, evitiamo che
   criterion spari oltre i 5 secondi/sample standard).
5. **`invert_match`** — `-v -c "^$"` su `grep.c`. Misura il path
   `LineOutcome::NoMatch` su molte righe (esercita la negazione, non
   solo l'accettazione).
6. **`cow_impact`** — quattro varianti `--color=never|always` × `with output|count only`.
   È la **validazione retroattiva di D-Ondata-3.3**, l'unica modifica
   misurabile introdotta in Ondata 3 (highlight passato da `String` a
   `Cow<'a, str>`).
7. **`mmap_vs_bufread`** — `--mmap` esplicito contro default su un file
   da 92 KB. Sonda se il path mmap (l'unica `unsafe` di rgrep) tiene
   testa al bufread default.

Quello che **abbiamo deliberatamente non misurato**:

- **PCRE / `-P`**. Feature-gated dietro `perl-regexp`; aggiungere la
  feature al bench complicava la matrice senza informarci di niente
  che il differential test non sappia già.
- **File >10 MB**. Lo spec accettava una fixture generata in
  `target/bench_fixtures/` ma scegliemmo di non farlo: vincolo
  "single commit, niente artefatti off-tree" + il fatto che la
  conclusione interessante (mmap vince anche su file piccoli) non
  richiedeva conferma su file grandi per essere riportata
  onestamente.
- **ripgrep**. Non installato sulla macchina di benchmark. Lo skip
  silenzioso (D-Ondata-4.4) ha tenuto le righe vuote nel report e ha
  consentito al bench di girare comunque. Una re-run su un host
  con `rg` nel `$PATH` popolerà la colonna senza modifiche al codice.
- **GNU grep su Linux**. L'oracolo è il BSD grep 2.6.0-FreeBSD bundlato
  con macOS, molto più anziano di GNU grep moderno. Anche questa è una
  re-run differita.

## 2. Cosa ha funzionato meglio del previsto

**Il path mmap**. L'Architect aveva esplicitamente predetto in
D-Ondata-4.6 che mmap avrebbe perso su file <1 MB per via del costo
page-fault: «per file <1MB, bufread vince». Il risultato misurato è
opposto — su un file da 92 KB, `--mmap` chiude in 3,65 ms mediani
contro i 4,14 ms di bufread, **~12 % più veloce con varianza più
stretta**. Due ipotesi compatibili: (a) su APFS + Apple Silicon il
buffer cache unificato rende il page-fault essenzialmente gratuito,
quindi la storia "page-fault costoso su file piccolo" è semplicemente
sbagliata su questa piattaforma; (b) il path `--mmap` in `runner.rs`
salta alcuni layer di astrazione del bufread default, e quella
semplicità strutturale si traduce in microsecondi misurabili. È il
risultato più azionabile del report: su macOS arm64 mmap conviene
quasi sempre, almeno per la fascia dei file che ci interessa.

**`literal_match_count` quasi in parità con BSD grep** (+5,5 %). Per
uno scenario fast-path letterale puro questo è in zona statistical
noise più che gap reale di implementazione. Aho-Corasick del crate
`aho-corasick` gioca alla pari del fast-path C scritto a mano di BSD
grep — almeno fino a che il process-spawn overhead domina le
misure, il che è esattamente questo scenario.

## 3. Cosa ha fallito le aspettative

**L'ottimizzazione `Cow<'a, str>` di Ondata 3 (D-Ondata-3.3) è
essenzialmente zero-impact su questo corpus**: −1,8 % di mediana fra
`--color=never` e `--color=always` sullo scenario "with output",
dentro il margine di rumore di criterion. È un **honest negative
result** e va riportato come tale, in linea con l'antipattern
esplicito D-Ondata-4.5 («NON nascondere risultati negativi»).

Questo *non* invalida Ondata 3: il `Cow` continua a essere l'API più
pulita (incapsula esattamente l'invariante "nessuna allocazione se
non aggiungo escape ANSI") ed era un refactor di chiarezza, non un
hot-path optimization. Ma una lesson learned secca emerge:
**senza profilare il workload reale, l'intuizione "evitare un'alloc
deve essere un guadagno" è quasi sempre falsa**. L'allocator moderno
gestisce String corte praticamente gratis; il costo dominante era
altrove (process spawn + aho-corasick scan), e finché quello non si
sposta, micro-ottimizzare il return type è un esercizio di stile.
Questa è la stessa lezione di tutto il filone perf ripgrep/grep di
Andrew Gallant: misura prima, ottimizza dopo, e accetta che ottimizzare
prima di misurare è quasi sempre tempo sprecato.

**Il distacco regex/invert** (+30 % entrambi). rgrep usa il crate
`regex`, lo stesso che usa ripgrep. Il fatto che siamo 30 % dietro
BSD grep su `regex_simple` *e su un piccolo file* è quasi
certamente un costo di glue code, non del motore regex. La
re-run con ripgrep installato chiuderà la domanda — se ripgrep
fa lo stesso scenario in 3 ms come BSD grep, il gap è glue code
nostra; se anche ripgrep si avvicina a noi, il gap è il
process-spawn che divora i risultati.

## 4. Trade-off process-spawn vs FFI: perché abbiamo scelto process-spawn

La spec ammetteva esplicitamente entrambe le strade. Abbiamo scelto
process-spawn per quattro ragioni concrete:

1. **Fair comparison**. Confrontare rgrep (binario) con `/usr/bin/grep`
   (binario) richiede di pagare a entrambi il costo `fork/exec`,
   altrimenti partiamo in svantaggio strutturale. Questo è lo stesso
   modello che Andrew Gallant usa nei famosi writeup ripgrep vs grep:
   è il modello "fair fight" canonico.

2. **Zero modifica al codice di produzione**. D-Ondata-4.8 vincola
   Ondata 4 a misurazione, non refactor. Un harness FFI avrebbe
   richiesto di rendere pubblici `runner::*` e `matcher::*`, o di
   estrarre `rgrep` come libreria oltre che binario. Refactor
   architetturale che D-Ondata-4.8 esplicitamente vieta in questa
   ondata.

3. **Honestà metodologica**. Il costo che paghiamo *in produzione*
   include il process spawn (l'utente lancia `rgrep ...`, non
   `rgrep::run(...)`). Misurare a livello FFI nasconderebbe quel
   costo e produrrebbe numeri "puliti" ma irrilevanti
   per l'esperienza utente reale.

4. **Cost diagnostico documentato**. Il process-spawn ci costa ~1–3 ms
   su macOS arm64. Su scenari che girano in 3–5 ms, è il 25–60 % del
   tempo misurato. PERF_REPORT.md lo dichiara esplicitamente in
   "Limitations" — chi legge sa che le differenze relative sono
   compresse rispetto alle differenze "reali" sul codice di search.
   È un trade-off accettato, non nascosto.

L'alternativa FFI resta legittima — sta in "Future work" del report
— ma non era il modello giusto per *questa* ondata.

## 5. Generalizzabilità — cosa replicherebbe rawk e altri porting

Questa è la sezione "skill" della skill `legacy-port`. Cinque pattern
che il prossimo porting C→Rust dovrebbe ereditare da Ondata 4 di
testag-grep:

1. **Ondata di misura separata dalle ondate di refactor**. Mescolare
   "pulisco" e "misuro l'effetto della pulizia" nello stesso commit
   è una ricetta per truccare i numeri inconsciamente. Tenerle in
   ondate distinte (Ondata 3 = refactor; Ondata 4 = misura di
   Ondata 3) costringe a riportare risultati negativi onestamente,
   perché chi misura non ha incentivo a difendere chi ha rifattorizzato.

2. **Validare retroattivamente le ottimizzazioni "intuitive"**. Ogni
   refactor giustificato con "evita un'alloc / un clone / un
   collect" dovrebbe avere uno scenario di bench dedicato. La
   maggior parte di questi refactor sono cosmetici sui numeri,
   anche quando sono genuini miglioramenti di API. Il `cow_impact`
   bench di Ondata 4 è esattamente questo pattern: validazione
   *ex-post* di un refactor *ex-ante* non misurato.

3. **Process-spawn come default per il diff-test del CLI**. È lo
   stesso modello del differential testing (process-spawn vs
   `/usr/bin/grep`); ri-usarlo per i bench evita di dover
   mantenere due infrastrutture parallele. Trade-off: i numeri
   sono "sporcati" dallo startup, ma sono numeri *onesti* su
   ciò che l'utente sperimenta lanciando il binario.

4. **`which::which(rg)` + skip silenzioso**. Il bench non deve
   richiedere che ripgrep sia installato per girare. Il pattern
   `if which::which("rg").is_ok() { bench... }` rende il report
   incrementale: chi ha `rg` ottiene tre righe, chi non l'ha
   ottiene due, e nessuno è bloccato. Questo è ripetibile per
   qualsiasi tool di confronto opzionale (`grep`, `egrep`,
   `awk`, `mawk`, `gawk`, ecc.).

5. **Reportare la metodologia *prima* dei numeri**. PERF_REPORT.md
   spende le prime tre sezioni sul *come*, poi i numeri. Senza
   contesto su "process-spawn comprime i delta" e "BSD grep è
   anziano", un lettore che vede "+30 % più lento di grep"
   conclude la cosa sbagliata. La metodologia *è* il numero;
   senza, hai solo cifre.

Bonus, valido per qualsiasi porting con bench criterion: **non
configurare la dipendenza per `cargo bench` finché il porting non è
ragionevolmente assestato**. Aggiungere criterion all'Ondata 1
sarebbe stato premature optimization: avremmo misurato del codice
ancora in evoluzione. Aggiungerlo dopo tre ondate di assestamento
è il momento giusto — è l'unico momento in cui i numeri restano
significativi per più di un commit.
