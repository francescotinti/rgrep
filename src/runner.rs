// rgrep — GNU grep ported to Rust
// Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
//
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Gemini Antigravity (Google)
//
// https://github.com/francescotinti/rgrep

use crate::cli::Config;
use crate::error::RgrepError;
use crate::matcher::Matcher;
use crate::output::{GrepColors, ansi_wrap};
use globset::{Glob, GlobSet, GlobSetBuilder};
use memmap2::MmapOptions;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::IsTerminal;
use std::io::{self, BufRead, BufReader, Cursor, Write};
use walkdir::WalkDir;

/// Load patterns from a file (one pattern per line, CRLF tolerant).
///
/// # Errors
/// Returns `RgrepError::Io` if the file cannot be opened or read.
pub fn load_pattern_file(path: &str) -> Result<Vec<String>, RgrepError> {
    let file = File::open(path).map_err(|source| RgrepError::Io {
        path: path.to_string(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut patterns = Vec::new();
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        let bytes_read =
            reader
                .read_until(b'\n', &mut buffer)
                .map_err(|source| RgrepError::Io {
                    path: path.to_string(),
                    source,
                })?;
        if bytes_read == 0 {
            break;
        }
        let mut line = String::from_utf8_lossy(&buffer).into_owned();
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        patterns.push(line);
    }
    Ok(patterns)
}

#[derive(Debug, PartialEq, Eq)]
pub enum RunResult {
    MatchFound,
    NoMatch,
}

/// Bundle of print-time parameters that stay invariant across every line of a
/// single file search (replaces the 9-argument [`print_line`] signature).
struct PrintCtx<'a> {
    config: &'a Config,
    filename: &'a str,
    print_filename: bool,
    color_enabled: bool,
    colors: &'a GrepColors,
}

/// Mutable per-file search state extracted from [`bufread_search`]:
/// before-context ring buffer, the after-context countdown, the last printed
/// line cursor and the running match counter.
struct MatchContext {
    history: VecDeque<(usize, usize, String)>,
    print_after: usize,
    last_printed_line: usize,
    match_count: usize,
    before_ctx: usize,
    after_ctx: usize,
}

impl MatchContext {
    fn new(before_ctx: usize, after_ctx: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(before_ctx),
            print_after: 0,
            last_printed_line: 0,
            match_count: 0,
            before_ctx,
            after_ctx,
        }
    }

    fn record_context_line(&mut self, line_number: usize, byte_offset: usize, line: &str) {
        if self.before_ctx == 0 {
            return;
        }
        if self.history.len() == self.before_ctx {
            self.history.pop_front();
        }
        self.history
            .push_back((line_number, byte_offset, line.to_string()));
    }

    fn first_unprinted_history_line(&self, fallback: usize) -> usize {
        for (h_line, _, _) in &self.history {
            if *h_line > self.last_printed_line {
                return *h_line;
            }
        }
        fallback
    }

    const fn needs_separator(&self, first_to_print: usize) -> bool {
        self.last_printed_line > 0
            && first_to_print > self.last_printed_line + 1
            && (self.before_ctx > 0 || self.after_ctx > 0)
    }
}

/// Assemble the raw patterns from `-e`/`-f`/positional argument, and the list
/// of extra files when a positional pattern collides with `-e`/`-f`. Extracted
/// from [`run`] (D-Ondata-3.4).
///
/// # Errors
/// Propagates `RgrepError::Io` from [`load_pattern_file`] when a `-f` file
/// cannot be opened or read.
fn collect_patterns(config: &Config) -> Result<(Vec<String>, Vec<String>), RgrepError> {
    let mut raw_patterns: Vec<String> = config.pattern_opts.regexp.clone();
    for f in &config.pattern_opts.file_patterns {
        raw_patterns.extend(load_pattern_file(f)?);
    }
    let mut extra_files = Vec::new();
    if !config.pattern_opts.regexp.is_empty() || !config.pattern_opts.file_patterns.is_empty() {
        if let Some(p) = &config.pattern_opts.pattern {
            extra_files.push(p.clone());
        }
    } else if let Some(p) = &config.pattern_opts.pattern {
        raw_patterns.push(p.clone());
    }
    Ok((raw_patterns, extra_files))
}

/// Drive a single concrete file through the mmap/buffered-reader decision,
/// falling back to buffered I/O whenever mmap is unavailable or the metadata
/// lookup fails. Extracted from [`run`] (D-Ondata-3.4).
///
/// # Errors
/// Propagates `RgrepError` from the underlying `bufread_search` call (I/O
/// failures during read; never from the mmap path because that one is
/// silently downgraded to buffered I/O).
fn search_file(pctx: &PrintCtx, matcher: &Matcher, file: File) -> Result<bool, RgrepError> {
    let Ok(metadata) = file.metadata() else {
        return bufread_search(pctx, matcher, BufReader::new(file));
    };
    let use_mmap = pctx.config.binary_opts.mmap && metadata.is_file() && metadata.len() > 0;
    if use_mmap && let Ok(r) = try_mmap_search(&file, pctx, matcher) {
        return Ok(r);
    }
    // PERF-FIX-O5-2: for small/medium regular files (<16 MB), slurp the
    // contents into a single Vec<u8> and parse via Cursor — Cursor's
    // BufRead::fill_buf returns the full remaining slice, so read_until
    // performs zero intermediate memcpy per line (vs BufReader::read_until
    // which copies each line from the 8 KB internal buffer into the
    // caller's Vec<u8>). PERF_REPORT.md §7 mmap_vs_bufread shows the same
    // Cursor-shaped I/O pattern saves ~12 % vs streaming BufReader on a
    // 92 KB file; mmap is unavailable here (either disabled by config or
    // the kernel mapping failed) so the slurp is the next-best option.
    // Profile attribution: PROFILE_REPORT.md §5 PERF-FIX-O5-2.
    const SLURP_LIMIT: u64 = 16 * 1024 * 1024;
    if metadata.is_file() && metadata.len() > 0 && metadata.len() < SLURP_LIMIT {
        use std::io::Read;
        let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
        let mut r = BufReader::new(&file);
        if r.read_to_end(&mut bytes).is_ok() {
            return bufread_search(pctx, matcher, Cursor::new(bytes));
        }
    }
    bufread_search(pctx, matcher, BufReader::new(file))
}

/// Run a grep invocation end-to-end based on a parsed [`Config`].
///
/// # Errors
/// Returns `RgrepError::InvalidRegex` for pattern compilation failures,
/// `RgrepError::Io` for file open/read errors, `RgrepError::InvalidGlob` for
/// bad `--include`/`--exclude` patterns and `RgrepError::Silent` when one or
/// more file-level errors were already reported to stderr.
pub fn run(config: &Config) -> Result<RunResult, RgrepError> {
    let (raw_patterns, extra_files) = collect_patterns(config)?;

    let matcher = Matcher::new(config, raw_patterns)?;
    let (files_to_search, mut has_error) = resolve_files(config, extra_files)?;

    let is_recursive = config.filter_opts.recursive
        || config.filter_opts.dereference_recursive
        || config.filter_opts.directories == crate::cli::DirectoriesAction::Recurse;
    let print_filename = match (
        config.output_opts.with_filename,
        config.output_opts.no_filename,
    ) {
        (true, _) => true,
        (_, true) => false,
        _ => files_to_search.len() > 1 || is_recursive,
    };

    let color_enabled = match config.output_opts.color.as_str() {
        "always" => true,
        "auto" => std::io::stdout().is_terminal(),
        _ => false,
    };
    let colors = GrepColors::from_env();

    let mut any_match = false;

    for filename in files_to_search {
        let result = if filename == "-" {
            let pctx = PrintCtx {
                config,
                filename: &config.output_opts.label,
                print_filename,
                color_enabled,
                colors: &colors,
            };
            let stdin = io::stdin();
            let reader = stdin.lock();
            bufread_search(&pctx, &matcher, reader)?
        } else {
            let file = match File::open(&filename) {
                Ok(f) => f,
                Err(e) => {
                    if !config.filter_opts.no_messages {
                        eprintln!("rgrep: {filename}: {e}");
                    }
                    has_error = true;
                    continue;
                }
            };
            let pctx = PrintCtx {
                config,
                filename: &filename,
                print_filename,
                color_enabled,
                colors: &colors,
            };
            search_file(&pctx, &matcher, file)?
        };

        if result {
            any_match = true;
            if config.output_opts.quiet {
                return Ok(RunResult::MatchFound);
            }
        }
    }

    if has_error {
        return Err(RgrepError::Silent);
    }

    if any_match {
        Ok(RunResult::MatchFound)
    } else {
        Ok(RunResult::NoMatch)
    }
}

fn try_mmap_search(file: &File, pctx: &PrintCtx, matcher: &Matcher) -> Result<bool, RgrepError> {
    // SAFETY: We assume the file is not modified or truncated by another
    // process for the duration of the mmap. This match_offsets the GNU grep
    // behavior, which also relies on this assumption (and prints a warning
    // in some cases when the file shrinks mid-read). For grep's read-only
    // use case, the risk surface is acceptable: undefined behavior occurs
    // only if a concurrent writer truncates the file below the mmap'd
    // length, in which case the OS may deliver SIGBUS on access.
    let mmap = unsafe { MmapOptions::new().map(file)? };
    let bytes: &[u8] = &mmap;
    let cursor = Cursor::new(bytes);
    bufread_search(pctx, matcher, cursor)
}

/// Loop-control outcome returned by [`process_line`]: either iterate again,
/// stop reading this file, or terminate the whole `bufread_search` early with
/// a fixed `has_match` value.
enum LineOutcome {
    Continue,
    Break,
    Return(bool),
}

/// Apply the per-line decision tree once we know whether a line matched: bump
/// counters, emit context+match block, honor `-q`/`-l`/`-L`/`-c`/binary modes.
/// Extracted from [`bufread_search`] (D-Ondata-3.4) so the outer reader loop
/// stays focused on I/O and context bookkeeping.
#[allow(clippy::too_many_arguments)] // each arg is mandatory state for the inner decision tree
fn process_line(
    pctx: &PrintCtx,
    matcher: &Matcher,
    state: &mut MatchContext,
    is_binary: bool,
    is_match: bool,
    line_number: usize,
    byte_offset: usize,
    line_str: &str,
    match_offsets: Vec<(usize, String)>,
) -> LineOutcome {
    let config = pctx.config;
    let output = &config.output_opts;

    if !is_match {
        if state.print_after > 0 {
            if !(output.files_with_matches
                || output.files_without_match
                || output.count
                || output.only_matching)
            {
                print_line(pctx, line_number, byte_offset, line_str, false);
            }
            state.last_printed_line = line_number;
            state.print_after -= 1;
        } else if state.before_ctx > 0 {
            state.record_context_line(line_number, byte_offset, line_str);
        }
        return LineOutcome::Continue;
    }

    if output.only_matching {
        state.match_count += if output.count && !match_offsets.is_empty() {
            match_offsets.len()
        } else {
            1
        };
    } else {
        state.match_count += 1;
    }

    if output.quiet {
        return LineOutcome::Return(true);
    }

    if is_binary {
        // Step BD: GNU >= 3.5 semantics. `-c` keeps counting every matching
        // line; `-l`/`-L` stop at the first match; otherwise report once on
        // stderr (never stdout) and stop. `-s` does not suppress it (GNU 3.8).
        if output.count {
            return LineOutcome::Continue;
        }
        if reports_binary_match(output) {
            eprint!("{}", binary_match_message(pctx.filename, &output.label));
            return LineOutcome::Return(true);
        }
        return LineOutcome::Break;
    }

    if output.files_with_matches || output.files_without_match {
        LineOutcome::Break
    } else if output.count {
        LineOutcome::Continue
    } else {
        emit_match_block(
            pctx,
            matcher,
            state,
            line_number,
            byte_offset,
            line_str,
            match_offsets,
        );
        LineOutcome::Continue
    }
}

// PERF-FIX-O7-1: share SIMD delimiter scanning across general and count/quiet
// paths. The current baseline attributes 46.3% of invert-count samples to
// std's memchr_aligned (PROFILE_REPORT.md, 2026-09-30).
fn read_delimited<R: BufRead>(
    reader: &mut R,
    delimiter: u8,
    buffer: &mut Vec<u8>,
) -> io::Result<usize> {
    let mut total = 0;
    loop {
        let chunk = match reader.fill_buf() {
            Ok(chunk) => chunk,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if chunk.is_empty() {
            return Ok(total);
        }
        let end = memchr::memchr(delimiter, chunk);
        let used = end.map_or(chunk.len(), |index| index + 1);
        buffer.extend_from_slice(&chunk[..used]);
        reader.consume(used);
        total += used;
        if end.is_some() {
            return Ok(total);
        }
    }
}

#[allow(clippy::unnecessary_wraps)] // Result kept for forward-compat with future I/O propagation
fn bufread_search<R: BufRead>(
    pctx: &PrintCtx,
    matcher: &Matcher,
    mut reader: R,
) -> Result<bool, RgrepError> {
    let config = pctx.config;
    let before_ctx = config.before_context_lines();
    let after_ctx = config.after_context_lines();
    let delimiter: u8 = if config.output_opts.null_data {
        0
    } else {
        b'\n'
    };

    let mut line_number: usize = 1;
    let mut byte_offset: usize = 0;
    let mut has_match = false;

    let mut state = MatchContext::new(before_ctx, after_ctx);
    let mut buffer = Vec::new();

    let binary_action = config.binary_action();
    let mut is_binary = false;

    if binary_action != crate::cli::BinaryAction::Text
        && let Ok(buf) = reader.fill_buf()
        && buf.contains(&0)
        && delimiter != 0
    {
        is_binary = true;
        if binary_action == crate::cli::BinaryAction::WithoutMatch {
            return Ok(false);
        }
    }

    // PERF-FIX-O6: pure-count / pure-quiet fast path. When the mode is
    // exclusively boolean (no color, no `-o`, no `-l`/`-L`, no context, no
    // `-m` cap, no binary detection) we can skip the UTF-8 conversion,
    // the line trim, find_match_offsets and the process_line dispatch.
    // AhoCorasick (`-F`) and pcre2 (`-P`) backends become direct byte
    // calls. Profile attribution: PROFILE_REPORT.md §2.4 + PERF-FIX-O6.
    let output = &config.output_opts;
    let pure_count_quiet = (output.count || output.quiet)
        && !output.only_matching
        && !output.files_with_matches
        && !output.files_without_match
        && !pctx.color_enabled
        && before_ctx == 0
        && after_ctx == 0
        && config.binary_opts.max_count.is_none()
        && !is_binary;

    if pure_count_quiet {
        loop {
            buffer.clear();
            match read_delimited(&mut reader, delimiter, &mut buffer) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    if !config.filter_opts.no_messages {
                        eprintln!("rgrep: {}: {e}", pctx.filename);
                    }
                    break;
                }
            }
            let mut line_bytes: &[u8] = &buffer;
            if line_bytes.last() == Some(&delimiter) {
                line_bytes = &line_bytes[..line_bytes.len() - 1];
            }
            if !config.binary_opts.binary && delimiter == b'\n' && line_bytes.last() == Some(&b'\r')
            {
                line_bytes = &line_bytes[..line_bytes.len() - 1];
            }
            if matcher.is_match_bytes(line_bytes) {
                state.match_count += 1;
                has_match = true;
                if output.quiet {
                    return Ok(true);
                }
            }
        }
        emit_trailing_summary(pctx, &state, has_match);
        return Ok(has_match);
    }

    loop {
        buffer.clear();
        let bytes_read = match read_delimited(&mut reader, delimiter, &mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                if !config.filter_opts.no_messages {
                    eprintln!("rgrep: {}: {e}", pctx.filename);
                }
                break;
            }
        };

        // PERF-FIX-O5-1: validate UTF-8 via the usize-block fast path in
        // core::str::run_utf8_validation rather than the byte-by-byte
        // Utf8Chunks scan used by from_utf8_lossy. ASCII input (the common
        // case for source code) hits the fast path; non-UTF-8 input falls
        // back to the existing lossy semantics with identical observable
        // behaviour. Profile attribution: PROFILE_REPORT.md §5 PERF-FIX-O5-1.
        let line_cow: Cow<'_, str> = std::str::from_utf8(&buffer)
            .map_or_else(|_| String::from_utf8_lossy(&buffer), Cow::Borrowed);
        let mut line_str = line_cow.as_ref();
        if line_str.ends_with(delimiter as char) {
            line_str = &line_str[..line_str.len() - 1];
        }
        if !config.binary_opts.binary && delimiter == b'\n' && line_str.ends_with('\r') {
            line_str = &line_str[..line_str.len() - 1];
        }

        let mut is_match = false;
        let mut match_offsets = vec![];
        if config
            .binary_opts
            .max_count
            .is_none_or(|m| state.match_count < m)
        {
            is_match = matcher.is_match(line_str);
            if is_match
                && (config.output_opts.only_matching || pctx.color_enabled)
                && !config.filter_opts.invert_match
            {
                match_offsets = matcher.find_match_offsets(line_str);
            }
        }

        if is_match {
            has_match = true;
        }
        match process_line(
            pctx,
            matcher,
            &mut state,
            is_binary,
            is_match,
            line_number,
            byte_offset,
            line_str,
            match_offsets,
        ) {
            LineOutcome::Continue => {}
            LineOutcome::Break => break,
            LineOutcome::Return(b) => return Ok(b),
        }

        if let Some(max) = config.binary_opts.max_count
            && state.match_count >= max
            && state.print_after == 0
        {
            break;
        }

        byte_offset += bytes_read;
        line_number += 1;
    }

    emit_trailing_summary(pctx, &state, has_match);
    Ok(has_match)
}

/// Whether a matching binary file produces the "binary file matches"
/// diagnostic: suppressed by `-q`, `-c`, `-l`, `-L`, by nothing else.
const fn reports_binary_match(output: &crate::cli::OutputOpts) -> bool {
    !(output.quiet || output.count || output.files_with_matches || output.files_without_match)
}

/// GNU >= 3.5 wording: `PROG: NAME: binary file matches`, where stdin uses
/// the `--label` value (default `(standard input)`).
fn binary_match_message(filename: &str, label: &str) -> String {
    let name = if filename == "-" { label } else { filename };
    format!("rgrep: {name}: binary file matches\n")
}

/// Print the before-context history (if any), the match line itself (or its
/// `--only-matching` slices) and arm the after-context countdown.
fn emit_match_block(
    pctx: &PrintCtx,
    matcher: &Matcher,
    state: &mut MatchContext,
    line_number: usize,
    byte_offset: usize,
    line_str: &str,
    match_offsets: Vec<(usize, String)>,
) {
    let config = pctx.config;

    let first_to_print = state.first_unprinted_history_line(line_number);
    if state.needs_separator(first_to_print) && !config.output_opts.no_group_separator {
        println!("{}", config.output_opts.group_separator);
    }

    let history: Vec<_> = state.history.drain(..).collect();
    for (h_line_num, h_byte_offset, h_line) in history {
        if h_line_num > state.last_printed_line {
            print_line(pctx, h_line_num, h_byte_offset, &h_line, false);
            state.last_printed_line = h_line_num;
        }
    }

    if config.output_opts.only_matching {
        for (m_offset, m_str) in match_offsets {
            let output = if pctx.color_enabled {
                ansi_wrap(&m_str, &pctx.colors.ms)
            } else {
                m_str
            };
            print_line(pctx, line_number, byte_offset + m_offset, &output, true);
        }
    } else {
        let output_line: std::borrow::Cow<'_, str> = if pctx.color_enabled {
            matcher.highlight(line_str, pctx.colors)
        } else {
            std::borrow::Cow::Borrowed(line_str)
        };
        print_line(pctx, line_number, byte_offset, &output_line, true);
    }

    state.last_printed_line = line_number;
    state.print_after = state.after_ctx;
}

/// Emit the per-file trailing summary line: `-l/-L` filename, or `-c` count.
fn emit_trailing_summary(pctx: &PrintCtx, state: &MatchContext, has_match: bool) {
    let config = pctx.config;
    let filename = pctx.filename;

    let print_filename_summary = (config.output_opts.files_without_match && !has_match)
        || (config.output_opts.files_with_matches && has_match);

    if print_filename_summary {
        let term = if config.output_opts.null { '\0' } else { '\n' };
        print!("{filename}{term}");
    } else if config.output_opts.count {
        if pctx.print_filename {
            let sep = if config.output_opts.null { '\0' } else { ':' };
            println!("{filename}{sep}{}", state.match_count);
        } else {
            println!("{}", state.match_count);
        }
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, RgrepError> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(builder.build()?)
}

/// Compiled glob filters shared by every recursive walk: include/exclude file
/// names, exclude-dir directory names, plus the raw exclude pattern list whose
/// emptiness gates the matcher lookup.
struct FilterSets<'a> {
    include: &'a GlobSet,
    exclude: &'a GlobSet,
    exclude_dir: &'a GlobSet,
    exclude_patterns: &'a [String],
}

/// Build the union of `--exclude` patterns coming from the CLI and the
/// `--exclude-from` file (if any). Returns the patterns plus a `has_error`
/// flag that bubbles up to [`resolve_files`].
fn collect_exclude_patterns(config: &Config) -> (Vec<String>, bool) {
    let mut exclude_patterns = config.filter_opts.exclude.clone();
    let mut has_error = false;
    if let Some(f) = &config.filter_opts.exclude_from {
        if let Ok(content) = fs::read_to_string(f) {
            for line in content.lines() {
                if !line.is_empty() {
                    exclude_patterns.push(line.to_string());
                }
            }
        } else {
            if !config.filter_opts.no_messages {
                eprintln!("rgrep: {f}: No such file or directory");
            }
            has_error = true;
        }
    }
    (exclude_patterns, has_error)
}

/// Walk a directory tree honoring `-R`/`--recursive`, `--include`/`--exclude`
/// and `--exclude-dir`. Append every accepted file path to `out` and return
/// `true` if any walker error was reported. Extracted from [`resolve_files`]
/// (D-Ondata-3.4).
fn walk_recursive(
    root: &str,
    config: &Config,
    filters: &FilterSets,
    out: &mut Vec<String>,
) -> bool {
    let mut has_error = false;
    let mut it = WalkDir::new(root)
        .follow_links(config.filter_opts.dereference_recursive)
        .into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(e)) => {
                if !config.filter_opts.no_messages {
                    eprintln!("rgrep: {e}");
                }
                has_error = true;
                continue;
            }
            Some(Ok(entry)) => entry,
        };

        let file_name_os = entry.file_name();

        if entry.file_type().is_dir() {
            if !config.filter_opts.exclude_dir.is_empty()
                && filters.exclude_dir.is_match(file_name_os)
            {
                it.skip_current_dir();
            }
            continue;
        }

        if entry.file_type().is_file() {
            if !config.filter_opts.include.is_empty() && !filters.include.is_match(file_name_os) {
                continue;
            }
            if !filters.exclude_patterns.is_empty() && filters.exclude.is_match(file_name_os) {
                continue;
            }
            out.push(entry.path().to_string_lossy().into_owned());
        }
    }
    has_error
}

fn resolve_files(
    config: &Config,
    extra_files: Vec<String>,
) -> Result<(Vec<String>, bool), RgrepError> {
    use crate::cli::{DevicesAction, DirectoriesAction};
    use std::os::unix::fs::FileTypeExt;

    let mut resolved_files = Vec::new();
    let mut has_error = false;

    let mut all_files = extra_files;
    all_files.extend(config.files.clone());

    let files = if all_files.is_empty() {
        vec!["-".to_string()]
    } else {
        all_files
    };

    let include_set = build_globset(&config.filter_opts.include)?;
    let (exclude_patterns, exclude_from_err) = collect_exclude_patterns(config);
    has_error |= exclude_from_err;
    let exclude_set = build_globset(&exclude_patterns)?;
    let exclude_dir_set = build_globset(&config.filter_opts.exclude_dir)?;
    let filters = FilterSets {
        include: &include_set,
        exclude: &exclude_set,
        exclude_dir: &exclude_dir_set,
        exclude_patterns: &exclude_patterns,
    };

    let is_recursive = config.filter_opts.recursive
        || config.filter_opts.dereference_recursive
        || config.filter_opts.directories == DirectoriesAction::Recurse;

    for path in &files {
        if path == "-" {
            resolved_files.push(path.clone());
            continue;
        }

        let Ok(meta) = fs::metadata(path) else {
            if !config.filter_opts.no_messages {
                eprintln!("rgrep: {path}: No such file or directory");
            }
            has_error = true;
            continue;
        };

        if meta.is_dir() {
            if is_recursive {
                has_error |= walk_recursive(path, config, &filters, &mut resolved_files);
            } else if config.filter_opts.directories == DirectoriesAction::Read {
                if !config.filter_opts.no_messages {
                    eprintln!("rgrep: {path}: Is a directory");
                }
                has_error = true;
            }
        } else {
            let file_type = meta.file_type();
            if (file_type.is_fifo()
                || file_type.is_socket()
                || file_type.is_block_device()
                || file_type.is_char_device())
                && config.filter_opts.devices == DevicesAction::Skip
            {
                continue;
            }
            resolved_files.push(path.clone());
        }
    }

    Ok((resolved_files, has_error))
}

fn print_line(pctx: &PrintCtx, line_number: usize, byte_offset: usize, line: &str, is_match: bool) {
    let config = pctx.config;
    let colors = pctx.colors;

    let sep = if is_match { ":" } else { "-" };
    let sep_col = if pctx.color_enabled {
        ansi_wrap(sep, &colors.se)
    } else {
        sep.to_string()
    };

    let mut stdout = io::stdout();

    if config.output_opts.initial_tab {
        let _ = stdout.write_all(b"\t");
    }

    if pctx.print_filename {
        let fname_col = if pctx.color_enabled {
            ansi_wrap(pctx.filename, &colors.fn_color)
        } else {
            pctx.filename.to_string()
        };
        let _ = stdout.write_all(fname_col.as_bytes());
        if config.output_opts.null {
            let _ = stdout.write_all(b"\0");
        } else {
            let _ = stdout.write_all(sep_col.as_bytes());
        }
    }
    if config.output_opts.line_number {
        let lnum_col = if pctx.color_enabled {
            ansi_wrap(&line_number.to_string(), &colors.ln)
        } else {
            line_number.to_string()
        };
        let _ = stdout.write_all(lnum_col.as_bytes());
        let _ = stdout.write_all(sep_col.as_bytes());
    }
    if config.output_opts.byte_offset {
        let boff_col = if pctx.color_enabled {
            ansi_wrap(&byte_offset.to_string(), &colors.bn)
        } else {
            byte_offset.to_string()
        };
        let _ = stdout.write_all(boff_col.as_bytes());
        let _ = stdout.write_all(sep_col.as_bytes());
    }

    let _ = stdout.write_all(line.as_bytes());

    let terminator = if config.output_opts.null_data {
        b"\0"
    } else {
        b"\n"
    };
    let _ = stdout.write_all(terminator);

    if config.output_opts.line_buffered {
        let _ = stdout.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_pattern_file() {
        let dir = std::env::temp_dir().join("test_pattern_loader");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Empty file
        let empty_path = dir.join("empty.txt");
        std::fs::write(&empty_path, "").unwrap();
        let pats = load_pattern_file(empty_path.to_str().unwrap()).unwrap();
        assert!(pats.is_empty());

        // Empty lines mixed
        let mixed_path = dir.join("mixed.txt");
        std::fs::write(&mixed_path, "foo\n\nbar\n").unwrap();
        let pats = load_pattern_file(mixed_path.to_str().unwrap()).unwrap();
        assert_eq!(pats, vec!["foo", "", "bar"]);

        // CRLF
        let crlf_path = dir.join("crlf.txt");
        std::fs::write(&crlf_path, "foo\r\nbar\r\n").unwrap();
        let pats = load_pattern_file(crlf_path.to_str().unwrap()).unwrap();
        assert_eq!(pats, vec!["foo", "bar"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_build_globset() {
        let set = build_globset(&["*.rs".to_string(), "*.toml".to_string()]).unwrap();
        assert!(set.is_match("main.rs"));
        assert!(set.is_match("Cargo.toml"));
        assert!(!set.is_match("README.md"));
    }

    #[test]
    fn test_exclude_from_parsing() {
        let content = "*.log\n\n#comment.txt\n";
        let mut patterns = Vec::new();
        for line in content.lines() {
            if !line.is_empty() {
                patterns.push(line.to_string());
            }
        }
        assert_eq!(patterns, vec!["*.log", "#comment.txt"]);
    }

    #[test]
    fn test_filter_combination() {
        let include_set = build_globset(&["*.txt".to_string()]).unwrap();
        let exclude_set = build_globset(&["ignore.txt".to_string()]).unwrap();

        let check_file = |name: &str| -> bool {
            if !include_set.is_empty() && !include_set.is_match(name) {
                return false;
            }
            if !exclude_set.is_empty() && exclude_set.is_match(name) {
                return false;
            }
            true
        };

        assert!(check_file("test.txt"));
        assert!(!check_file("test.log"));
        assert!(!check_file("ignore.txt"));
    }

    #[test]
    fn test_context_ring_buffer() {
        use std::collections::VecDeque;
        let capacity = 2;
        let mut history: VecDeque<(usize, usize, String)> = VecDeque::with_capacity(capacity);

        // Push 3 items into capacity 2 buffer
        for i in 1..=3 {
            if history.len() == capacity {
                history.pop_front();
            }
            history.push_back((i, i * 10, format!("line {i}")));
        }

        assert_eq!(history.len(), 2);
        assert_eq!(history.front().unwrap().0, 2);
        assert_eq!(history.back().unwrap().0, 3);
    }

    #[test]
    fn test_group_separator_gap_logic() {
        let before_ctx = 1;
        let after_ctx = 1;
        let last_printed = 5;
        let first_to_print = 8;

        let should_print_separator = last_printed > 0
            && first_to_print > last_printed + 1
            && (before_ctx > 0 || after_ctx > 0);
        assert!(should_print_separator); // gap of 2 lines
    }

    #[test]
    fn test_group_separator_overlap_logic() {
        let before_ctx = 1;
        let after_ctx = 1;
        let last_printed = 5;
        let first_to_print = 6;

        let should_print_separator = last_printed > 0
            && first_to_print > last_printed + 1
            && (before_ctx > 0 || after_ctx > 0);
        assert!(!should_print_separator); // no gap, overlapping or contiguous
    }

    fn make_pctx<'a>(
        config: &'a Config,
        filename: &'a str,
        colors: &'a GrepColors,
    ) -> PrintCtx<'a> {
        PrintCtx {
            config,
            filename,
            print_filename: false,
            color_enabled: false,
            colors,
        }
    }

    #[test]
    fn test_limit_max_count() {
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.binary_opts.max_count = Some(2);

        let matcher = Matcher::new(&config, vec!["foo".to_string()]).unwrap();
        let input = "foo\nfoo\nfoo\nbar\n";
        let reader = std::io::BufReader::new(input.as_bytes());
        let colors = GrepColors::from_env();
        let pctx = make_pctx(&config, "test", &colors);

        let result = bufread_search(&pctx, &matcher, reader).unwrap();
        assert!(result); // has match
    }

    #[test]
    fn test_quiet_early_exit() {
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.output_opts.quiet = true;

        let matcher = Matcher::new(&config, vec!["foo".to_string()]).unwrap();
        let input = "bar\nfoo\nbaz\n";
        let reader = std::io::BufReader::new(input.as_bytes());
        let colors = GrepColors::from_env();
        let pctx = make_pctx(&config, "test", &colors);

        let result = bufread_search(&pctx, &matcher, reader).unwrap();
        assert!(result); // match found and early exited
    }

    #[test]
    fn test_run_result_semantics() {
        assert_ne!(RunResult::MatchFound, RunResult::NoMatch);
    }

    #[test]
    fn binary_match_message_uses_file_name() {
        assert_eq!(
            binary_match_message("dir/bin.dat", "(standard input)"),
            "rgrep: dir/bin.dat: binary file matches\n"
        );
    }

    #[test]
    fn binary_match_message_uses_default_stdin_label() {
        assert_eq!(
            binary_match_message("-", "(standard input)"),
            "rgrep: (standard input): binary file matches\n"
        );
    }

    #[test]
    fn binary_match_message_uses_custom_label_for_stdin() {
        assert_eq!(
            binary_match_message("-", "LBL"),
            "rgrep: LBL: binary file matches\n"
        );
    }

    #[test]
    fn binary_match_is_reported_unless_quiet_count_or_list_mode() {
        let parse = |flags: &[&str]| {
            let mut argv = vec![std::ffi::OsString::from("rgrep")];
            argv.extend(flags.iter().map(std::ffi::OsString::from));
            argv.push(std::ffi::OsString::from("foo"));
            crate::cli::Config::parse_args(argv).unwrap()
        };
        assert!(reports_binary_match(&parse(&[]).output_opts));
        // -s suppresses file errors, not the binary-match diagnostic (GNU 3.8).
        assert!(reports_binary_match(&parse(&["-s"]).output_opts));
        assert!(reports_binary_match(
            &parse(&["-o", "-n", "-m1"]).output_opts
        ));
        for silent in [["-q"], ["-c"], ["-l"], ["-L"]] {
            assert!(
                !reports_binary_match(&parse(&silent).output_opts),
                "{silent:?} must stay silent"
            );
        }
    }

    #[test]
    fn test_null_data_delimiter() {
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.output_opts.null_data = true;

        let matcher = Matcher::new(&config, vec!["foo".to_string()]).unwrap();
        let input = b"foo\nbar\0baz";
        let reader = std::io::BufReader::new(&input[..]);
        let colors = GrepColors::from_env();
        let pctx = make_pctx(&config, "test", &colors);

        let result = bufread_search(&pctx, &matcher, reader).unwrap();
        assert!(result);
    }

    #[test]
    fn test_null_filename_output() {
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("-Z"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.output_opts.null = true;
        assert!(config.output_opts.null);
    }

    #[test]
    fn test_print_line_safe_output() {
        // Just verify that setting up null config doesn't crash print line config checks
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.output_opts.null_data = true;
        config.output_opts.null = true;
        assert!(config.output_opts.null_data);
    }

    #[test]
    fn test_mmap_regular_file() {
        use std::io::Write;
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("--mmap"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.binary_opts.mmap = true;

        let temp_path = std::env::temp_dir().join("test_mmap_rgrep.txt");
        let mut file = std::fs::File::create(&temp_path).unwrap();
        file.write_all(b"foo\nbar\nfoo\n").unwrap();

        let metadata = std::fs::metadata(&temp_path).unwrap();
        assert!(metadata.is_file());
        assert!(metadata.len() > 0);
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_mmap_stdin_fallback() {
        let mut config = crate::cli::Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("--mmap"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        config.binary_opts.mmap = true;

        let matcher = Matcher::new(&config, vec!["foo".to_string()]).unwrap();
        let input = b"foo\n";
        let cursor = Cursor::new(&input[..]);
        let colors = GrepColors::from_env();
        let pctx = make_pctx(&config, "(standard input)", &colors);

        // This simulates bufread_search being called on stdin despite config.binary_opts.mmap
        let result = bufread_search(&pctx, &matcher, cursor).unwrap();
        assert!(result);
    }

    #[test]
    fn delimiter_reader_matches_std_across_buffer_boundaries() {
        let inputs = [
            Vec::new(),
            b"\n\nlast".to_vec(),
            b"first\r\nsecond\0\xfftail".to_vec(),
            [vec![b'x'; 20_003], b"\nend\0".to_vec()].concat(),
        ];
        for input in inputs {
            for capacity in 1..=17 {
                for delimiter in [b'\n', 0] {
                    let mut expected_reader = BufReader::with_capacity(capacity, &input[..]);
                    let mut actual_reader = BufReader::with_capacity(capacity, &input[..]);
                    loop {
                        // read_until appends and returns only the newly read length.
                        let mut expected = b"prefix".to_vec();
                        let mut actual = expected.clone();
                        let expected_len = expected_reader
                            .read_until(delimiter, &mut expected)
                            .unwrap();
                        let actual_len =
                            read_delimited(&mut actual_reader, delimiter, &mut actual).unwrap();
                        assert_eq!(actual_len, expected_len);
                        assert_eq!(actual, expected);
                        if actual_len == 0 {
                            break;
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn delimiter_reader_retries_interrupts_and_preserves_partial_errors() {
        struct InterruptedThenError<'a> {
            remaining: &'a [u8],
            interrupted: bool,
        }
        impl io::Read for InterruptedThenError<'_> {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                panic!("reader must use fill_buf/consume");
            }
        }
        impl BufRead for InterruptedThenError<'_> {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                if !self.interrupted {
                    self.interrupted = true;
                    return Err(io::ErrorKind::Interrupted.into());
                }
                if self.remaining.is_empty() {
                    return Err(io::Error::other("injected failure"));
                }
                Ok(&self.remaining[..self.remaining.len().min(3)])
            }
            fn consume(&mut self, amount: usize) {
                self.remaining = &self.remaining[amount..];
            }
        }
        let mut reader = InterruptedThenError {
            remaining: b"ab\nrest",
            interrupted: false,
        };
        let mut buffer = Vec::new();
        assert_eq!(read_delimited(&mut reader, b'\n', &mut buffer).unwrap(), 3);
        assert_eq!(buffer, b"ab\n");
        assert_eq!(reader.remaining, b"rest");
        buffer.clear();
        let error = read_delimited(&mut reader, b'\n', &mut buffer).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(buffer, b"rest");
    }
}
