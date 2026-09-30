# 02 Mapping Table (C to Rust)

Questo documento formalizza come i costrutti del codice sorgente C originario di GNU grep (definiti in `diary/01-semantic-model.md`) vengono tradotti idiomaticamente in Rust all'interno di `rgrep`. Queste D-decisioni formano il contratto architetturale del porting.

## §1 Decisioni globali (D-G1..D-Gn)

| ID | Costrutto C | Scelta Rust | Razionale | Status |
|---|---|---|---|---|
| D-G1 | Regex engine custom POSIX | `regex` crate | Le performance e la robustezza del crate `regex` giustificano l'abbandono del regex engine POSIX custom di grep. (da 01-semantic-model.md §2 "Architettura del matching engine") | confermato |
| D-G2 | `fts_read` (C library) | `walkdir` crate | Per preservare la parità di GNU `--exclude`, non usiamo `ignore` (che onora implicitamente i `.gitignore`). `walkdir` gestisce il simlink recursion loop out-of-the-box. (da 01-semantic-model.md §6 "Symlink Recursion Loop") | confermato |
| D-G3 | Glob matching C array | `globset` crate | Ottimizzato per filtrare liste di glob (es. `--exclude-dir`) con alta performance. | confermato |
| D-G4 | I/O page-aligned custom | `BufRead::read_until` | Usiamo le primitive standard al posto del `fillbuf` (da 01-semantic-model.md §4 "Strategia memory management") per gestire i null-bytes delimitatori di `-z`. Non usiamo `lines()`. | confermato |
| D-G5 | UTF-8 validation byte-per-byte | `String::from_utf8_lossy` / `&[u8]` | Evitiamo i crash gestendo byte array `&[u8]` ove possibile e stringhe lossy per l'output se necessario. (da 01-semantic-model.md §5 "Encoding e locale") | confermato |
| D-G6 | Exit codes ad-hoc | `thiserror::Error` | Type unificato per gli errori interni tradotto poi in exit codes tramite match nel main. (da 01-semantic-model.md §7 "Exit codes") | confermato |
| D-G7 | `parse_grep_colors` | ANSI manuale / `GREP_COLORS` env | Manterremo compatibilità facendo parsing custom della stringa env. | da-validare |
| D-G8 | BRE (Basic Regex) engine | ERE via pre-processor | Trasliamo il subset di BRE in sintassi ERE compatibile con `regex` crate (escaping automatico dei metacaratteri). | confermato |
| D-G9 | Feature storiche / OS-specific | Scope-out (Debito esplicito) | `--rpm-*` e `GREP_OPTIONS` deprecata non verranno mantenute nel porting Rust. | confermato |
| D-G10 | `getopt_long` manual auto-help | `disable_help_flag = true` su `Config` | In GNU grep `-h` indica `--no-filename`, mentre `clap` lo mappa di default a `--help`. (da 01-semantic-model.md §1 "API surface CLI") | confermato |
| D-G11 | Fallback DFA → NFA/Posix | Delega a `regex` JIT | Non replichiamo la pipeline del fallback interno tra DFA e Posix regex. (da 01-semantic-model.md §2) | confermato |
| D-G12 | `eolbyte` (newline vs null) | `delimiter: u8` struct field | Il separatore di linea è dinamico (`\n` o `\0`) impostato dal flag `-z`. (da 01-semantic-model.md §1 "Binary/Data") | confermato |
| D-G13 | Case folding / mb_clen | `(?i)` flag inline | Per gestire in modo robusto le stringhe multilingua, affidiamo a `regex` (con Unicode attivato) e ai suoi flag inline il case-insensitivity. (da 01-semantic-model.md §5 "Encoding e locale") | confermato |

## §2 Decisioni per modulo (D-Mxx)

### §2.1 grep.c (Orchestration & CLI)
- **Strategia**: C (Replace & Restructure) → map a `src/cli.rs` e `src/runner.rs`.
- **Target LOC**: ~3000 LOC C → ~500 LOC Rust (grazie a `clap`).
- **D-M1**: Le flag del gruppo "Pattern Selection" (`-E`, `-F`, ecc.) configurano la variante logica del Matcher.
- **D-M2**: Le flag del gruppo "Context Control" usano un `VecDeque` come ring buffer in Rust al posto della logica di pending lines.
- **D-M3**: La detection dei file binari (`buf_has_encoding_errors`) si basa sulla ricerca di byte NUL (0x00) nel primo blocco di lettura.

### §2.2 dfasearch.c (DFA Engine)
- **Strategia**: E (Externalize) → `regex` crate.
- **Target LOC**: ~600 LOC C → ~50 LOC Rust (wrapper struct).
- **D-M4**: La struct `dfa_comp` viene rimpiazzata concettualmente da un costrutto contenente `regex::Regex` o `regex::RegexSet`.
- **D-M5**: La logica di fast-forward e skip del DFA è delegata alle ottimizzazioni interne (`AhoCorasick` o memchr fallback) di `regex` crate.
- **D-M6 (revisione Step 19)**: Backreferences `\1..\9` supportate via fallback automatico al crate `fancy-regex` quando il `regex` crate rifiuta il pattern. Trade-off: pattern con backref usano NFA (slower) ma sono supportati pienamente in rgrep. Le regex non valide in entrambi i parser portano a `process::exit(2)`.

### §2.3 kwsearch.c (Exact Match)
- **Strategia**: E (Externalize) → `aho-corasick` crate.
- **Target LOC**: ~200 LOC C → ~40 LOC Rust.
- **D-M7**: `Fcompile` mappa direttamente alla costruzione della `AhoCorasick` struct.
- **D-M8**: L'esecuzione di pattern simultanei (-e in cascata con -F) usa le feature out-of-the-box multi-pattern search.
- **D-M9**: Fallback a regex quando `-w` (word match) complica i bound e la validazione multibyte.

### §2.4 pcresearch.c (PCRE Engine)
- **Strategia**: F (Feature-gated bridge) → `pcre2` crate.
- **Target LOC**: ~400 LOC C → ~100 LOC Rust.
- **D-M10**: (IMPLEMENTED) Compilazione sotto `#[cfg(feature = "perl-regexp")]` o Cargo feature option. Se disabilitato, emette panic identico a GNU grep.
- **D-M11**: Lo stack limit JIT in `Pcompile` si mappa alle configurazioni limit del crate Rust.
- **D-M12**: Gestione invalid UTF-8 via skip dei codepoint (da 01-semantic-model.md §6 "Invalid UTF-8 in PCRE") si traduce in un fallback di read offset su `pcre2::bytes`.

### §2.5 searchutils.c (Helper Utils)
- **Strategia**: B (Reimplement idioms)
- **Target LOC**: ~200 LOC C → ~50 LOC Rust.
- **D-M13**: `wordchar_next` e lookbehind vengono delegati alle regular expressions (`\b` engine based).
- **D-M14**: `mb_goback` logic viene scartata; il `regex` crate opera sui slice byte raw con lookaround nativo o si usa match regex con boundaries espliciti.
- **D-M15**: In assenza di supporti word boundary su stringhe fisse (come in Aho-Corasick nativo), si wrappano i keyword boundaries nel pattern Regex `\b(word)\b`.

## §3 Tabella di traduzione costrutti

| Costrutto C (grep source) | Mappatura Idiomatica Rust | ID Decisione |
|---|---|---|
| `getopt_long` | `clap::Parser` struct con `#[command]` macro | D-G10 |
| `fillbuf` buffer I/O loop | `BufReader::read_until(delimiter, &mut vec)` | D-G4 |
| `grepbuf` (core match loop) | Iterazione sui match successivi del `Matcher` instanziato | D-M2 |
| `struct dfa_comp` | `regex::Regex` | D-M4 |
| `struct kwsearch` | `aho_corasick::AhoCorasick` | D-M7 |
| `struct pcre_comp` | `pcre2::bytes::Regex` | D-M10 |
| `EXIT_TROUBLE` (2) | `std::process::ExitCode::from(2)` o `exit(2)` | D-G6 |
| `fts_read` | `walkdir::WalkDir::new()` | D-G2 |
| `mb_clen` / `localeinfo` | Gestito implicitamente in `regex` crate unicode | D-M14 |
| `mb_goback` / UTF-8 tracking | Sostituito dalla boundary analysis del crate regex | D-M13 |
| `buf_has_encoding_errors` | `memchr::memchr(0, buf).is_some()` (Euristic NUL) | D-M3 |
| `eolbyte` (`\n` o `\0`) | Proprietà `delimiter: u8` letta in run-loop | D-G12 |
| `match_icase` flag | Aggiunta `(?i)` al pattern, o `case_insensitive(true)` | D-G13 |
| `match_words` flag | Aggiunta `\b(?:pattern)\b` al wrapper regexp | D-M15 |
| `out_invert` flag (`-v`) | `if match_found ^ invert_match { print }` | D-M1 |
| `out_quiet` flag (`-q`) | Bypass the print cycle, flag `exit_on_match` | D-M1 |
| `SAME_INODE(st, out_stat)` | `fs::metadata(in).ino() == fs::metadata(out).ino()` | D-G11 (vedi §6 01) |
| `count_matches` | Accumulatore integer (`usize`) per file | D-M2 |
| `context_length_arg` | Parsing `usize` integrato in `clap` via `value_parser` | D-G10 |
| `print_line_head` | Macro `write!(stdout, "{}:", filename)` condizionale | D-M1 |
| `prtext` | Svuotamento buffer contestuale `VecDeque` leading/trailing | D-M2 |
| `EGexecute` | `Regex::find(text)` / `Regex::find_iter(text)` | D-M5 |
| `Fexecute` | `AhoCorasick::find(text)` / `AhoCorasick::find_iter` | D-M7 |
| `Pexecute` | `pcre2::bytes::Regex::find` | D-M12 |
| `bad_utf8_from_pcre2` skip | Configurazione regex byte modes per saltare invalid utf8 | D-M12 |

## §4 Sketch API pubblica Rust

```rust
pub enum EngineType {
    Regex(regex::Regex),
    AhoCorasick(aho_corasick::AhoCorasick),
    // Pcre2(pcre2::bytes::Regex) // behind #[cfg]
}

pub struct Matcher {
    engine: EngineType,
}

impl Matcher {
    pub fn compile(patterns: &[String], opts: &MatchOptions) -> Result<Self, GrepError>;
    pub fn is_match(&self, line: &[u8]) -> bool;
    pub fn find(&self, line: &[u8]) -> Option<std::ops::Range<usize>>;
}

pub fn search_reader<R: std::io::BufRead, W: std::io::Write>(
    reader: R,
    writer: &mut W,
    matcher: &Matcher,
    opts: &OutputOptions
) -> Result<MatchStats, GrepError>;
```

## §5 Cose esplicitamente NON portate (scope-out)

1. **GREP_OPTIONS environment variable**: È considerata deprecata in upstream GNU grep ed è attivamente ignorata in favore dei configuratori alias shell.
2. **Flag OS-specifici/Obsoleti**: `--rpm-*` e altre varianti altamente specializzate o specifiche di vendor Linux storiche non vengono portate.
3. **gnulib portability layer**: GNU Grep include m4 e polyfills per compilazione C su OS legacy (es. vecchi Solaris, HP-UX). Rust astrae nativamente l'OS tramite l'`std` standard library, per cui scartiamo in blocco questi header.
4. **Locale handling C-style multi-byte**: Al posto della complessità delle codifiche legacy (`mb_clen`), ci concentriamo sulla gestione unibyte vs UTF-8. Non replicheremo la tavolozza Shift-JIS o locali fallback custom.
5. **Fallback chain interno DFA → Posix Regex**: Riprodurre la delega interna architetturale tra un NFA personalizzato ed un Posix Regex lento per le backreferences comporterebbe creare debito tecnico. Affidiamo a `regex` crate la gestione e il dispatch interno.

## §6 Punti di review per il Decider (umano)

- **Usare `walkdir` o `ignore`?** Abbiamo selezionato `walkdir` per garantire aderenza alla parità con GNU `--exclude` (ignorando il `.gitignore`). C'è interesse a mantenere questo vincolo stretto anziché favorire la comodità utente standard Rust (`ignore`)?
- **Scope di `-P` (PCRE2)**: Lo sviluppo PCRE2 richiede compilazioni C bindgen complesse o disabilitarlo in toto. È corretto gestire un fallback in cui `-P` lancia panic a runtime se non compilato (come fa l'originale GNU) o vogliamo nascondere il flag in fase CLI build time?
- **Color env `GREP_COLORS` parsing**: Il parsing nativo di GNU usa stringhe ANSI non elaborate per le configurazioni multi-parametro. Lo implementiamo in-house o introduciamo un crate tipo `termcolor` rinunciando al formato stringato originale?
- **Type alias std::process::ExitCode**: Rust favorisce i type Enum o trait objects. Mappare `EXIT_TROUBLE` (2) su exit status bypassando lo stack unrolling (con `std::process::exit`) potrebbe spezzare eventuali test unitari futuri. Validare se il `main()` può semplicemente fare un match su un `Result` e ritornare una enum standard `ExitCode`.
