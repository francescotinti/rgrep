# 04 Divergences

Questo documento elenca le divergenze introdotte nel porting rispetto all'originale GNU grep.

## Divergenze intenzionali

### D-NEW-1: Default ERE (Extended Regular Expressions)
- **Severità**: parity / scelta_design
- **Descrizione**: Mentre GNU grep storico utilizzava `-G` (BRE, Basic Regular Expressions) come default se non venivano forniti flag specifici di engine, `rgrep` utilizza `-E` (ERE, Extended Regular Expressions) come default implicito.
- **Razionale**: L'ecosistema Rust e in particolare il crate `regex` operano primariamente in logica ERE compatibile. Scegliere ERE come default allinea `rgrep` ai tool CLI moderni (ripgrep, ag, ecc.), pur supportando `-G` per retro-compatibilità esplicita (che attiva il translation layer BRE -> ERE prima della compilazione).

## Divergenze latenti / bug pre-esistenti

### D-NEW-2: Exit Code Canonici
- **Tipo**: Bug Latente
- **Descrizione**: Pre-Step-10, `rgrep` non ritornava mai exit 1 per "nessun match trovato", contrariamente a POSIX grep. Ritornava exit 0 o 2.
- **Soluzione adottata**: Implementato l'enum `RunResult { MatchFound, NoMatch }` in `runner.rs`. `main.rs` mappa esplicitamente 0, 1 e 2. Fixato in **Step 10**.

### D-NEW-3: --color default = never (vs GNU = auto)

- **Severità**: parity / scelta_design
- **Categoria**: choice_design
- **Causa**: scelta_design (semplificazione testabilità)
- **Discovered in**: spec Step 6
- **Sintomo**:
  GNU grep default --color=auto → colori in TTY interattivo, niente in pipe.
  rgrep default --color=never → mai colori finché esplicitamente richiesto.
- **Razionale**: testcase differential più stabili, niente dipendenza da
  TTY runtime state. `--color=auto` resta supportato esplicitamente.
- **Decisione**:
  - [x] documentato come D-NEW-3
  - [ ] eventualmente cambio default a auto in Step 17 (polish)
- **Status**: aperto (scelta accettata per ora)

### D-NEW-4: Nessuna nuova divergenza trovata (finora)
- **Tipo**: Esito Proptest (Step 16)
- **Descrizione**: Il differential proptest eseguito su `rgrep` vs `grep` con pattern literal, anchored e character classes non ha individuato divergenze impreviste.

### D-NEW-5: Molteplici -e con sintassi invalida mascherati da unione OR
- **Tipo**: Bug Reale (Regex Parsing)
- **Descrizione**: Se si passano più espressioni `-e` che individualmente non sono valide (es. `-e "["` e `-e "]"`), `rgrep` le unisce precocemente in un'unica stringa `[|]`. Questa stringa risulta un'espressione regolare valida (classe di caratteri contenente il pipe), mentre l'oracolo `grep` le processa individualmente ed emette un fatal error (exit 2) alla prima fallace.
- **Soluzione adottata**: Validazione per-pattern in `Matcher::new` prima del joining. Compile attempt isolato per ogni `-e` pattern; primo fallimento → exit 2 (parity GNU). Fixed in **Step 19**.
- **Status**: chiuso ✅
