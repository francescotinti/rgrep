# Skill: Migrazione Iterativa da C a Rust per CLI Utilities

Questa "Skill" è un framework procedurale estrapolato dal progetto `rgrep`. È progettata per guidare un Agente IA o uno sviluppatore nella traduzione e modernizzazione di storici tool a riga di comando scritti in C o C++ verso codice Rust sicuro, idiomatico e altamente performante.

## 🎯 Obiettivo della Skill
Prendere una utility di sistema legacy in C (es. `grep`, `cat`, `ls`, `find`) e riscriverla in Rust garantendo la parità di funzionalità al 100%, ma adottando i moderni standard di sicurezza della memoria, concorrenza e design del software.

---

## 🛠 Fase 1: Mappatura dell'Ecosistema (C -> Rust)
Non tradurre mai il C riga per riga. Identifica le primitive C e mappale alle librerie (crate) dello standard Rust.

*   **Parsing Argomenti**: 
    *   *C*: `getopt`, `getopt_long`, parsing manuale di `argv`.
    *   *Rust*: Usa la crate `clap` (preferibilmente con la macro `#[derive(Parser)]`). È dichiarativa, sicura sui tipi e genera l'help automaticamente.
*   **Navigazione File System**:
    *   *C*: `opendir`, `readdir`, `stat`.
    *   *Rust*: Usa la crate `walkdir` o `ignore`. Gestiscono i symlink, le esclusioni e le ricorsioni in modo iterativo e memory-safe.
*   **Motori di Ricerca (Regex / Stringhe)**:
    *   *C*: `regcomp`, `strstr`, `PCRE`.
    *   *Rust*: Usa la crate `regex` o `aho-corasick` per fixed-strings.
*   **Memory Mapping**:
    *   *C*: `mmap()`, `munmap()`.
    *   *Rust*: Usa la crate `memmap2`. Richiede blocchi `unsafe`, ma limitati alla sola apertura del mapping.

---

## 🔄 Fase 2: Architettura e Struttura Iterativa
Suddividi sempre il progetto in moduli chiari. In C spesso si trova tutto in un enorme file `main.c` da 10.000 righe. In Rust imponi questa struttura:

1.  `src/cli.rs`: Contiene solo la `struct Config` e il parsing dei flag tramite `clap`.
2.  `src/runner.rs` o `src/engine.rs`: Gestisce l'apertura dei file (es. `BufReader`, I/O).
3.  `src/logic.rs` (o `matcher.rs`): Contiene la logica pura (es. algoritmi di match, formattazione matematica). Completamente agnostica rispetto ai file.
4.  `src/main.rs`: File minuscolo. Collega il CLI al Runner.

---

## 🚀 Fase 3: Esecuzione in Multi-Step (La regola d'oro)
Procedi per "Fasi" sequenziali, ognuna accompagnata dai propri Unit Test:
*   **Step 1**: MVP di base. Leggi lo standard input o un singolo file, applica la logica nuda e cruda (es. string match semplice) e stampa.
*   **Step 2**: Sostituisci l'algoritmo base con l'engine definitivo (es. Crate Regex).
*   **Step 3**: Supporto per file multipli e ricorsione.
*   **Step 4**: Implementazione dei Flag "Boolean" (es. `-i` ignore case, `-v` invert).
*   **Step 5**: Flag di Formattazione e Colorazione.
*   **Step 6**: Gestione avanzata del Buffer (Context lines, Code circolari `VecDeque`).
*   **Step 7**: Ottimizzazione di basso livello e Raw Bytes.

---

## ⚠️ Fase 4: Gestione del Basso Livello (I/O)
I tool in C leggono spesso stream grezzi di byte e usano i terminatori `\0`.
In Rust, evita `reader.lines()` se hai bisogno di manipolare i terminatori (es. opzione `-z` null-data) o di calcolare gli offset dei byte.
*   **Pattern Migliore**: Usa `BufRead::read_until(delimiter, &mut buffer)`. Permette di riciclare il vettore di memoria, tracciare esplicitamente i byte letti e trattare sia i file di testo che quelli binari in modo flessibile.
*   **Stringhe Lossy**: Per tool che operano su file sconosciuti, non usare `String::from_utf8` (che panica su binari corrotti). Usa sempre `String::from_utf8_lossy`.

---

## ✅ Fase 5: Unit Testing Continuo
Copia le logiche limite dal codice sorgente C e trasformale in test all'interno del modulo logico:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // Mock della configurazione
    // Asserzioni sui bordi (edge-cases)
}
```
Usa il comando `cargo test` prima di ogni singolo `git commit`. Se un refactoring del Buffer fallisce, te ne accorgi immediatamente e non inquini l'implementazione CLI.

---
> **Suggerimento per l'Intelligenza Artificiale**: Quando ti viene richiesto di migrare un codice C/C++, richiama questa Skill. Leggi sempre il sorgente C per estrapolare la lista delle "Opzioni lunghe" (long_options), raggruppale in macro-categorie (I/O, Filtri, Formattazione) e proponi all'utente un piano d'attacco in Fasi separate per bypassare i limiti di contesto.
