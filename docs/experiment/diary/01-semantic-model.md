# 01 Semantic Model of GNU Grep

Questo documento descrive il modello semantico mentale di GNU grep basato sull'analisi del codice sorgente C. L'obiettivo è comprendere il funzionamento interno, il flusso di esecuzione e le strategie architetturali prima di effettuare il porting idiomatico in Rust.

## §1 API surface CLI

L'interfaccia a riga di comando di GNU grep è complessa e definisce le modalità operative del programma. Le flag sono processate in `grep.c:2139` (tramite `getopt_long`) e categorizzate logicamente in `usage` (`grep.c:1989`).

| Gruppo | Flag (Short / Long) | Riferimento Interno |
|---|---|---|
| **Pattern Selection** | `-E` (extended), `-F` (fixed), `-G` (basic), `-P` (perl) | Impostano il matcher nell'array `matchers` (`grep.c:2085`). |
| **Pattern Source** | `-e PATTERN`, `-f FILE` | Aggiungono pattern alla lista da compilare. |
| **Matching Control** | `-i` (ignore-case), `-v` (invert-match), `-w` (word-regexp), `-x` (line-regexp) | Alterano la valutazione dei match engine. |
| **Output Control** | `-m NUM`, `-b`, `-n`, `-H`, `-h`, `-o`, `-q` | Regolano i metadati della riga (`print_line_head`, `grep.c:1163`). |
| **Context Control** | `-A NUM`, `-B NUM`, `-C NUM` | Stampano righe circostanti (`prtext`, buffering interno). |
| **File Traversal** | `-r` / `-R` (recursive), `-d ACTION`, `--include`, `--exclude` | Gestiscono la discesa nelle directory (`grepfile`, `grep.c:1756`). |
| **Binary/Data** | `-a`, `-U`, `-I`, `--binary-files=TYPE`, `-z` (null-data) | Alterano la delimitazione e la tolleranza UTF-8. |

## §2 Architettura del matching engine

Grep non possiede un singolo motore di espressioni regolari, ma effettua un **dispatch** al runtime in base alle flag fornite. L'elenco è centralizzato nell'array `matchers` in `grep.c:2085`.

1. **DFA + Regex (`dfasearch.c`)**: Utilizzato per `grep -G` e `grep -E`.
   - Struttura: `struct dfa_comp` (`dfasearch.c:25`).
   - Logica: Compila prima un automa a stati finiti deterministico veloce (DFA). `EGexecute` (`dfasearch.c:352`) cerca potenziali match veloci.
   - Fallback: Se il DFA rileva un potenziale match o back-references (es. `\1`), delega la conferma al motore Regex Posix più lento (`re_search`).

2. **Aho-Corasick / Boyer-Moore (`kwsearch.c`)**: Utilizzato per stringhe fisse (`grep -F`).
   - Struttura: `struct kwsearch` (`kwsearch.c:24`).
   - Logica: Utilizza `kwsexec` (internamente Aho-Corasick o similare) per match esatti e ultra-veloci multi-stringa in `Fexecute` (`kwsearch.c:105`).

3. **PCRE2 (`pcresearch.c`)**: Utilizzato per espressioni regolari Perl (`grep -P`).
   - Struttura: `struct pcre_comp` (`pcresearch.c:49`).
   - Logica: Delega quasi totalmente la logica alla libreria esterna JIT-compiled PCRE2 in `Pexecute` (`pcresearch.c:264`).

Se il pattern per `grep -F` contiene caratteri "particolari" o si chiedono match complessi (`-w`), `Fexecute` può fare fallback dinamico compilando il pattern literal in Regex (`grep.c:2344` `fgrep_to_grep_pattern`).

## §3 Modello dati interno

Le strutture dati principali definiscono i layer di traduzione e buffering:

- `struct dfa_comp` (`dfasearch.c:25`): Contiene la regex compilata (DFA), una keyword cache per i literal obbligatori (`kwset`), e un fallback regex buffer.
- `struct kwsearch` (`kwsearch.c:24`): Incapsula un set di stringhe literal e un possibile pattern fallback compilato al volo.
- `struct pcre_comp` (`pcresearch.c:49`): Incapsula il contesto e lo stack JIT per esecuzioni ad alta performance (`pcre2_jit_stack`).

L'elaborazione delle directory avviene in `grepfile` (`grep.c:1756`) usando descrittori file fisici per aggirare race condition e loop simlink.

## §4 Strategia memory management

Grep non carica mai tutto il file in memoria. Gestisce un buffer ad alta performance per minimizzare il numero di syscalls. Non c'è alcun utilizzo di `mmap` (historically rimosso per race condition hardware e I/O).

- **Page-aligned Buffer**: `fillbuf` (`grep.c:955`) cerca di mantenere le letture allineate ai limiti della memoria paginata e processare i blocchi di file.
- **Good Read Size**: La lettura target `good_readsize` garantisce blocchi di 96 KiB per la massima resa cache (`grep.c:894`).
- **Residui**: I byte non analizzati dal blocco precedente vengono tenuti e i buffer scalano in dimensione se non si incontra un delimitatore (`eolbyte`) in tempo. La ricerca viene fatta da `grepbuf` (`grep.c:1465`).

## §5 Encoding e locale

La gestione multibyte e UTF-8 aggiunge un grado di complessità a GNU grep.
- L'ispezione della lunghezza di un carattere viene fatta tramite `mb_clen` (`search.h:93`).
- La ricerca backward necessaria per controllare i boundary delle parole (`-w`) è implementata in modo robusto in `mb_goback` (`searchutils.c:90`), capace di resistere ad encoding corrotti e fallback in single-byte recovery.
- Quando si fa ricerca esatta case-insensitive in un locale multibyte, in `fgrep_icase_available` (`grep.c:2326`), se il mapping multibyte è troppo ambiguo, il grep fa downgrade dal matcher `kwsearch` ad un matcher `dfasearch` meno performante ma in grado di gestire i pesi locale-aware.

## §6 Edge cases (casi limite gestiti)

| Edge Case | Risoluzione nel Modello Semantico | Riferimento |
|---|---|---|
| **File input = file output** | Grep blocca il loop infinito rifiutando l'azione se i file condividono l'`inode`. | `grep.c:1909` |
| **Dati binari mascherati** | Se `buf_has_encoding_errors` è positivo, smette di stampare e dice "binary file matches". | `grep.c:1169` |
| **Empty Pattern Fallback** | Un pattern vuoto matcherebbe ogni carattere. Viene pre-calcolato ed evitato PCRE eval. | `pcresearch.c:257` |
| **Invalid UTF-8 in PCRE** | PCRE2 andrebbe in errore. I byte corrotti vengono isolati e saltati (`bad_utf8_from_pcre2`). | `pcresearch.c:295` |
| **File Devices/Named Pipes** | Grep riconosce `st_mode` per non restare appeso in block read, saltando i device speciali. | `grep.c:1841` |
| **Symlink Recursion Loop** | `FTS_COMFOLLOW` previene l'infinito traversal sui file system in caso di `grep -R`. | `grep.c:1862` |
| **Word Boundary & UTF-8** | Un limite di parola può trovarsi tra multibyte. `wordchar_next` fa check codepoint reali. | `searchutils.c:199` |
| **Righe più grandi della RAM** | Rialloca il buffer raddoppiandolo dinamicamente ma fail-safe a OOM error esplicito. | `grep.c:997` |

## §7 Exit codes

La CLI in `usage` (`grep.c:2069`) sancisce il contratto finale:
- **0**: Almeno una riga o file è stata selezionata.
- **1**: Nessun match trovato su alcun file.
- **2**: Si è verificato un errore sintattico o hardware (`exit_failure = EXIT_TROUBLE`, vedi `grep.c:2476`). In presenza del flag `-q` l'errore non viene sempre stampato.

---
## Note per Phase 2 (Idee Mappatura Rust)
- Non useremo regex custom POSIX. Il fallback DFA->NFA->POSIX C verrà schiacciato dal singolo JIT di Rust `regex` crate per `-E`.
- La lettura file va traslata in `BufReader::read_until` perché iterare i char o usare string `lines()` fallirebbe miseramente nel gestire binary content (`-a` / `-z`).
- Il buffer ad-hoc C page-aligned (`fillbuf`) non andrà re-implementato byte-by-byte: ci fideremo del caching OS unito ai primitive I/O del Rust.
- Il loop su symlink e directories (`fts_read`) si mappa 1:1 su `walkdir`.
