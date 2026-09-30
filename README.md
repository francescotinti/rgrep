# rgrep 🦀

> Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
> AI-assisted port — Architect: Claude Opus 4.7 (Anthropic) · Implementer: Gemini Antigravity (Google)

`rgrep` is a modern, modular, and blazingly fast clone of the original **GNU grep**, completely rewritten in **Rust**. 

This project started as an educational experiment to translate a minimal 30-line C implementation of grep into Rust, and iteratively evolved into a 100% compliant, feature-rich text search engine capable of matching almost all major GNU grep functionalities.

## 🚀 Features

`rgrep` uses Rust's heavily optimized `regex` and `globset` crates to provide lightning-fast text processing, alongside modern features like memory-mapping and robust file-system traversal.

### Supported GNU Grep Flags:

**Pattern & Regex Control**
* `-e, --regexp`: Specify multiple patterns (OR logic).
* `-f, --file`: Read patterns line by line from a file.
* `-F, --fixed-strings`: Interpret patterns as literal strings instead of Regex.
* `-i, --ignore-case`: Ignore case distinctions.
* `--no-ignore-case`: Do not ignore case distinctions.
* `-w, --word-regexp`: Match whole words only.
* `-x, --line-regexp`: Match exactly the whole line.
* `-E, -G, -P`: Compatibility stubs for Extended, Basic, and Perl regex.

**Context Control**
* `-A, --after-context=NUM`: Print NUM lines of trailing context.
* `-B, --before-context=NUM`: Print NUM lines of leading context.
* `-C, --context=NUM`: Print NUM lines of output context.
* `--group-separator=SEP`: Use custom separator between context blocks (default: `--`).
* `--no-group-separator`: Do not use a group separator.

**File & Directory Traversal**
* `-r, --recursive`: Read all files under each directory, recursively.
* `-R, --dereference-recursive`: Recursive search, following symbolic links.
* `-d, --directories=ACTION`: Handle directories (read, skip).
* `-D, --devices=ACTION`: Handle devices (read, skip).
* `--include=GLOB`: Search only files matching GLOB.
* `--exclude=GLOB`: Skip files matching GLOB.
* `--exclude-dir=GLOB`: Exclude entire directories matching GLOB.
* `--exclude-from=FILE`: Read exclude globs from a file.

**Output Formatting**
* `-n, --line-number`: Prefix each line with its 1-based line number.
* `-b, --byte-offset`: Print the 0-based byte offset of each line.
* `-H, --with-filename`: Print the file name for each match.
* `-h, --no-filename`: Suppress the prefixing of file names.
* `-l, --files-with-matches`: Print only names of FILEs containing matches.
* `-L, --files-without-match`: Print only names of FILEs containing no matches.
* `-o, --only-matching`: Print only the matched parts of a line.
* `-c, --count`: Print a count of matching lines for each file.
* `-T, --initial-tab`: Line up tabs for easier reading.
* `--color`: Highlight matches and file names using ANSI colors.

**Low Level & Binary**
* `-z, --null-data`: Read lines separated by a zero byte (NULL) instead of newline.
* `-Z, --null`: Output a zero byte after the file name instead of a colon.
* `-a, --text`: Process a binary file as if it were text.
* `-U, --binary`: Treat the files as binary.
* `--binary-files=TYPE`: Treat binary files as TYPE.

**Optimizations & Limits**
* `-m, --max-count=NUM`: Stop reading a file after NUM matching lines.
* `--mmap`: Use low-level OS memory-mapped I/O to read input files (can improve performance on large files).
* `--line-buffered`: Flush output on every line.
* `-q, --quiet, --silent`: Exit immediately with zero status if any match is found.
* `-s, --no-messages`: Suppress error messages about nonexistent or unreadable files.
* `--label=LABEL`: Use LABEL as the standard input file name prefix.

## 📦 Installation

To build `rgrep` from source, ensure you have the Rust toolchain installed.

```bash
# Clone the repository
git clone https://github.com/francescotinti/rgrep.git
cd rgrep

# Build for release
cargo build --release

# The compiled binary will be at:
# target/release/rgrep
```

You can install it directly into your local cargo path:
```bash
cargo install --path .
```

## 🛠️ Usage Example

Search recursively in the current directory for the exact word "TODO", ignoring case, printing line numbers, showing 2 lines of context, and highlighting matches in color:

```bash
rgrep -r -w -i -n -C 2 --color "TODO" .
```

Search using memory-mapping for maximum speed and stop after 5 matches:
```bash
rgrep --mmap -m 5 "foo" large_file.log
```

## Verification status (2026-09-30)

The current suite has 137 manifest cases and 46 default / 48 PCRE2 Rust tests.
GNU grep 3.12 and BSD grep were exercised locally on macOS. GNU runs perform
3072 generated comparisons each; BSD runs explicitly skip those properties.
Two known GNU differences remain, both in `-T` alignment; the binary-match
diagnostic follows GNU >= 3.5 (stderr) since Step BD.
See [GNU_VERIFICATION.md](GNU_VERIFICATION.md) for pass/skip counts and commands.
The first CI run passed both Linux builds and macOS with PCRE2. The default
macOS job exposed a Bash 3 argument-handling issue, corrected in the workflow;
the second run (`36766955016`) passed all four jobs. See
[publication status](docs/experiment/PUBLICATION.md).

Ondata 7's delimiter optimization reduced elapsed time by roughly 9–12% on
three amplified count workloads in alternating pre/post measurements; small
CLI workloads were mostly flat. See [PERF_REPORT.md](PERF_REPORT.md) for raw
evidence, uncertainty and the small fixed-quiet regression.

## Experiment history

The [experiment documentation](docs/experiment/README.md) is split into
what remains to do (`TODO.md`), what was delivered (`DONE.md`) and the
architecture with its evolution (`ARCHITECTURE.md`), plus the full
historical specifications, audit log, diaries and archived commit scripts
under `history/`. See its [publication notes](docs/experiment/PUBLICATION.md)
for provenance.

## 📜 License
This project is open-source and built for educational and practical usage.
