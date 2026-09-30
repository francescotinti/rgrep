# 03 Translation Log

Questo documento tiene traccia delle traduzioni effettuate per ciascun modulo, mappandole alle decisioni prese in Phase 2.

## grep.c (Orchestration & CLI)
- **Implementazione**: tradotto in `rgrep/src/cli.rs` e `rgrep/src/runner.rs` in Step 3, 4, 5.
- **Decisioni applicate**:
  - **D-M1**: le flag del gruppo "Pattern Selection" configurano la variante del Matcher. Supporto esteso (D 4.1..D 4.5) per multi-pattern via array in OR-logic o Aho-Corasick. L'ordine di processamento per `-e` multipli e file `-f` accomuna i pattern in un'unica instanza. La logica di pattern posizionale (`args[0]`) è sovrascritta (diventa un input file) se `-e` o `-f` sono passati esplicitamente, conformemente alla disambiguazione parity di GNU.
  - **D-M1 (Output Control)**: implementato in `runner.rs` (precedentemente `print_line_head` in `grep.c:1163`). Supporta i prefissi formattati `filename:line_number:byte_offset:line_content`. Introdotte early-exit optimizations per `-l` e `-L`. La logica di precedenza per la stampa del filename in `-H` e `-h` risolve correttamente la concorrenza con `-r`/`-R` o output su molteplici file (D 5.1..D 5.7). Supporto per `GREP_COLORS` e highlighting aggiunto in `output.rs` (D 6.1..6.13). Logica `-o` implementata in `matcher.rs` (`find_match_offsets`).
- **Stato**: Completato.

## grep.c § "File Traversal"
- **Obiettivo**: Ricorsione nelle directory e gestione di device/FIFO.
- **Implementazione**: In `runner.rs` (`resolve_files`).
- **Decisioni applicate**:
  - Implementato tramite crate `walkdir`. `-r` scorre senza symlink (`follow_links(false)`), `-R` li segue (D 7.1).
  - Aggiunto il filtraggio via `globset` (Step 8): `--include` agisce come whitelist sui file, `--exclude` e `--exclude-from` come blacklist (i commenti `#` sono letterali, parity GNU). `--exclude-dir` pota attivamente interi rami usando `skip_current_dir()`.
  - Gestione context lines `-A`, `-B`, `-C` (Step 9) tramite ring-buffer `VecDeque` e counter `print_after`. Le context line emettono `.` o `-` nei prefissi (es. `file-10-line`).
  - Implementazione Limits (`-m`, `-q`, `-s`) (Step 10): `-m` fa early break sul counter per-file, `-q` fa early return globale su `RunResult::MatchFound` senza emettere testo, `-s` sopprime stderr su open failures.
  - Implementazione semantica `RunResult` (Step 10): 0 = `MatchFound`, 1 = `NoMatch`, 2 = `Error` (fix D-NEW-2).
  - Rilevazione Binaria (Step 11): heuristica NUL-byte (parity GNU `buf_has_encoding_errors`) al primo chunk di read. Supportati i fallback `-a` (text), `-U` (binary senza CRLF stripping) e `--binary-files=TYPE` (binary, text, without-match).
  - NUL handling `-z` e `-Z` (Step 12): `BufRead::read_until` programmabile al byte di fine riga. Output binario nativo garantito da `io::Write::write_all` bypassando `println!` per integrità dei null bytes interni.
  - Misc flags (Step 13): `-T` inserisce `\t` come padding prima della riga, `--label` sovrascrive `(standard input)`, `--line-buffered` forza `io::stdout().flush()` su ogni riga.
  - Memory Mapped I/O (Step 14): `--mmap` implementato via `memmap2`. Mappa l'intero file in memoria usando l'unica chiamata `unsafe` di rgrep (`MmapOptions::new().map(file)`). L'algoritmo di search (`search_buffer`) è estratto ed agnostico, avvolto su `Cursor<&[u8]>` per garantire che l'output sia byte-per-byte identico al `bufread_search`. Fallback automatico su `BufReader` per file inesistenti/vuoti/stdin o metadati mancanti.
  - `-P perl-regexp` (Step 15): Aggiunta la crate `pcre2` come dipendenza opzionale feature-gated. Aggiunta feature `perl-regexp` in `Cargo.toml`. La build di default rimane snella e priva della dipendenza, stampando un fatal-error identico a GNU grep se si usa `-P` senza supporto. Con la feature abilitata, il parsing abbraccia lookahead, lookbehind e backreferences (come stabilito nei punti D-M10, D-M11 e D-M12 del Mapping Table) agganciandosi alla nuova variante `Pcre2` di `Engine`.
  - Differential Proptest (Step 16): Aggiunto framework di property-based testing comparativo usando la crate `proptest`. Implementate 3 strategie di generazione random (pattern letterali, regex ancorate e classi di caratteri) estese a 1024 iterazioni per ciascuna, in modo da sondare sistematicamente edge case non previsti dall'analisi differenziale statica (D 16.3, D 16.4).
  - `-d ACTION` per directory supporta `read`/`recurse`/`skip` (D 7.2).
  - `-D ACTION` supporta `read`/`skip` per device speciali (verificato solo su FIFO in Step 7) usando `std::os::unix::fs::FileTypeExt` (D 7.3).
  - L'ordine dei file tra OS può variare a causa dell'implementazione di `readdir` sottostante.
  - Phase 4b Extension (Step 18): Re-ingegnerizzato framework per convertire le test-suite ufficiali `gnu-grep/tests/*.sh` in casi TOML isolati e deterministici (`gnu_NNNN_*.toml`). Oltre 100 script scansionati. Skippati automaticamente test basati su charset strani o comandi posix complessi non replicabili banalmente. Ottenuti dozzine di test statici extra, e scoperto tramite curation **1** grave bug mascherato dal pipe unione precoce (`D-NEW-5`).
  - Fix 12 bug noti (Step 19): Inserita validazione per-pattern dei multipli `-e` prima del merge logico con `|`, fixando `D-NEW-5`. Introdotto il supporto automatico per le backreferences (`\1..\9`) mappando un auto-fallback in compilazione sul crate pure-Rust `fancy-regex` se il RE2-style standard (`regex`) sbatte fuori `is_err()` a compile-time. Questo ha salvato i restanti 9 testcase GNU originali su syntax capture storiche.

- **Stato**: Completato.

## dfasearch.c (DFA Engine)
- **Implementazione**: tradotto in `rgrep/src/matcher.rs` in Step 3.
- **Decisioni applicate**:
  - **D-M4**: `dfa_comp` è stato rimpiazzato concettualmente dall'astrazione `Engine::Regex` in `matcher.rs`, sfruttando le performance native del crate `regex`.
  - **D-M5**: l'esecuzione veloce e il fallback sono delegati al `regex::RegexBuilder` interno, implementando `is_match`, `find_matches`, e `highlight`.
  - **D-M6**: in caso di regex non valide (come il parsing errato), viene sollevato un errore propagato al `main()` che esegue `process::exit(2)` (D-G6).
- **Stato**: Completato.

## kwsearch.c (Exact Match)
- **Implementazione**: tradotto in `rgrep/src/matcher.rs` in Step 3.
- **Decisioni applicate**:
  - **D-M7**: `Fcompile` è stato mappato sull'inizializzazione del crate `aho-corasick` usando `AhoCorasickBuilder::new().build(...)` all'interno della variante `Engine::AhoCorasick`.
  - **D-M8**: implementata l'esecuzione multi-pattern per `-F` come array di stringhe literal.
  - **D-M9**: fallback dinamico a Regex via bounding esplicito (`\b`) se la ricerca fissa richiede `-w` (word match).
- **Stato**: Completato.
