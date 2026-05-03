use crate::cli::Config;
use crate::matcher::Matcher;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Cursor, Write};
use std::error::Error;
use std::collections::VecDeque;
use walkdir::WalkDir;
use memmap2::MmapOptions;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::io::IsTerminal;
use crate::output::{GrepColors, ansi_wrap};

pub fn load_pattern_file(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path).map_err(|e| format!("{}: {}", path, e))?;
    let mut reader = BufReader::new(file);
    let mut patterns = Vec::new();
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        let bytes_read = reader.read_until(b'\n', &mut buffer)?;
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut raw_patterns = Vec::new();

    for p in &config.regexp {
        raw_patterns.push(p.clone());
    }

    for f in &config.file_patterns {
        raw_patterns.extend(load_pattern_file(f)?);
    }

    let mut extra_files = Vec::new();

    if !config.regexp.is_empty() || !config.file_patterns.is_empty() {
        if let Some(p) = &config.pattern {
            extra_files.push(p.clone());
        }
    } else {
        if let Some(p) = &config.pattern {
            raw_patterns.push(p.clone());
        }
    }

    let matcher = Matcher::new(&config, raw_patterns)?;
    let (files_to_search, mut has_error) = resolve_files(&config, extra_files)?;
    
    let is_recursive = config.recursive || config.dereference_recursive || config.directories == crate::cli::DirectoriesAction::Recurse;
    let print_filename = match (config.with_filename, config.no_filename) {
        (true, _) => true,
        (_, true) => false,
        _ => files_to_search.len() > 1 || is_recursive,
    };

    let color_enabled = match config.color.as_str() {
        "always" => true,
        "auto" => std::io::stdout().is_terminal(),
        _ => false,
    };
    let colors = GrepColors::from_env();

    for filename in files_to_search {
        if filename == "-" {
            let reader = BufReader::new(io::stdin());
            process_file(&config, &matcher, reader, &config.label, print_filename, color_enabled, &colors)?;
        } else {
            let file = match File::open(&filename) {
                Ok(f) => f,
                Err(e) => {
                    if !config.no_messages {
                        eprintln!("rgrep: {}: {}", filename, e);
                    }
                    has_error = true;
                    continue;
                }
            };

            if config.mmap {
                if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                    let cursor = Cursor::new(&mmap[..]);
                    process_file(&config, &matcher, cursor, &filename, print_filename, color_enabled, &colors)?;
                    continue;
                }
            }

            let reader = BufReader::new(file);
            process_file(&config, &matcher, reader, &filename, print_filename, color_enabled, &colors)?;
        }
    }

    if has_error {
        std::process::exit(2);
    }

    Ok(())
}

fn process_file<R: BufRead>(
    config: &Config,
    matcher: &Matcher,
    mut reader: R,
    filename: &str,
    print_filename: bool,
    color_enabled: bool,
    colors: &GrepColors,
) -> Result<(), Box<dyn Error>> {
    let before_ctx = config.get_before_context();
    let after_ctx = config.get_after_context();
    let delimiter = if config.null_data { 0 } else { b'\n' };

    let mut line_number = 1;
    let mut byte_offset = 0;
    let mut match_count = 0;
    let mut has_match = false;
    
    let mut history: VecDeque<(usize, usize, String)> = VecDeque::with_capacity(before_ctx);
    let mut print_after = 0;
    let mut last_printed_line = 0;

    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        let bytes_read = match reader.read_until(delimiter, &mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                if !config.no_messages {
                    eprintln!("rgrep: {}: {}", filename, e);
                }
                break;
            }
        };
        
        let line_cow = String::from_utf8_lossy(&buffer);
        let mut line_str = line_cow.as_ref();
        if line_str.ends_with(delimiter as char) {
            line_str = &line_str[..line_str.len() - 1];
        }
        
        if delimiter == b'\n' && line_str.ends_with('\r') {
            line_str = &line_str[..line_str.len() - 1];
        }

        let mut is_match = false;
        let mut matches = vec![];
        if config.max_count.map_or(true, |m| match_count < m) {
            is_match = matcher.is_match(line_str);
            if is_match && (config.only_matching || color_enabled) && !config.invert_match {
                matches = matcher.find_match_offsets(line_str);
            }
        }

        if is_match {
            has_match = true;
            if config.only_matching {
                match_count += if config.count && matches.len() > 0 { matches.len() } else { 1 };
            } else {
                match_count += 1;
            }

            if config.quiet {
                std::process::exit(0);
            }

            if config.files_with_matches || config.files_without_match {
                break;
            } else if config.count {
                // Do nothing here
            } else {
                let mut first_to_print = line_number;
                for (h_line_num, _, _) in &history {
                    if *h_line_num > last_printed_line {
                        first_to_print = *h_line_num;
                        break;
                    }
                }

                if last_printed_line > 0 && first_to_print > last_printed_line + 1 && (before_ctx > 0 || after_ctx > 0) {
                    if !config.no_group_separator {
                        println!("{}", config.group_separator);
                    }
                }

                for (h_line_num, h_byte_offset, h_line) in &history {
                    if *h_line_num > last_printed_line {
                        print_line(config, filename, print_filename, *h_line_num, *h_byte_offset, h_line, false, color_enabled, colors);
                        last_printed_line = *h_line_num;
                    }
                }
                history.clear();

                if config.only_matching {
                    for (m_offset, m_str) in matches {
                        let output = if color_enabled { ansi_wrap(&m_str, &colors.ms) } else { m_str };
                        print_line(config, filename, print_filename, line_number, byte_offset + m_offset, &output, true, color_enabled, colors);
                    }
                } else {
                    let output_line = if color_enabled { matcher.highlight(line_str, colors) } else { line_str.to_string() };
                    print_line(config, filename, print_filename, line_number, byte_offset, &output_line, true, color_enabled, colors);
                }
                last_printed_line = line_number;
                print_after = after_ctx;
            }
        } else {
            if print_after > 0 {
                if !(config.files_with_matches || config.files_without_match || config.count || config.only_matching) {
                    print_line(config, filename, print_filename, line_number, byte_offset, line_str, false, color_enabled, colors);
                }
                last_printed_line = line_number;
                print_after -= 1;
            } else if before_ctx > 0 {
                if history.len() == before_ctx && before_ctx > 0 {
                    history.pop_front();
                }
                history.push_back((line_number, byte_offset, line_str.to_string()));
            }
        }

        if let Some(max) = config.max_count {
            if match_count >= max && print_after == 0 {
                break;
            }
        }

        byte_offset += bytes_read;
        line_number += 1;
    }

    if config.files_without_match && !has_match {
        let term = if config.null { '\0' } else { '\n' };
        print!("{}{}", filename, term);
    } else if config.files_with_matches && has_match {
        let term = if config.null { '\0' } else { '\n' };
        print!("{}{}", filename, term);
    } else if config.count {
        if print_filename {
            let sep = if config.null { '\0' } else { ':' };
            println!("{}{}{}", filename, sep, match_count);
        } else {
            println!("{}", match_count);
        }
    }

    Ok(())
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, Box<dyn Error>> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(builder.build()?)
}

fn resolve_files(config: &Config, extra_files: Vec<String>) -> Result<(Vec<String>, bool), Box<dyn Error>> {
    use crate::cli::{DirectoriesAction, DevicesAction};
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

    let include_set = build_globset(&config.include)?;
    
    let mut exclude_patterns = config.exclude.clone();
    if let Some(f) = &config.exclude_from {
        if let Ok(content) = fs::read_to_string(f) {
            for line in content.lines() {
                if !line.is_empty() {
                    exclude_patterns.push(line.to_string());
                }
            }
        } else {
            if !config.no_messages {
                eprintln!("rgrep: {}: No such file or directory", f);
            }
            has_error = true;
        }
    }
    let exclude_set = build_globset(&exclude_patterns)?;
    
    let exclude_dir_set = build_globset(&config.exclude_dir)?;

    for path in &files {
        if path == "-" {
            resolved_files.push(path.clone());
            continue;
        }

        let metadata = fs::metadata(path);
        if let Ok(meta) = metadata {
            let is_recursive = config.recursive || config.dereference_recursive || config.directories == DirectoriesAction::Recurse;
            if meta.is_dir() {
                if is_recursive {
                    let mut it = WalkDir::new(path).follow_links(config.dereference_recursive).into_iter();
                    loop {
                        let entry = match it.next() {
                            None => break,
                            Some(Err(e)) => {
                                if !config.no_messages {
                                    eprintln!("rgrep: {}", e);
                                }
                                has_error = true;
                                continue;
                            },
                            Some(Ok(entry)) => entry,
                        };

                        let file_name_os = entry.file_name();

                        if entry.file_type().is_dir() {
                            if !config.exclude_dir.is_empty() && exclude_dir_set.is_match(file_name_os) {
                                it.skip_current_dir();
                            }
                            continue;
                        }

                        if entry.file_type().is_file() {
                            if !config.include.is_empty() && !include_set.is_match(file_name_os) {
                                continue;
                            }
                            if !exclude_patterns.is_empty() && exclude_set.is_match(file_name_os) {
                                continue;
                            }
                            resolved_files.push(entry.path().to_string_lossy().into_owned());
                        }
                    }
                } else if config.directories == DirectoriesAction::Read {
                    if !config.no_messages {
                        eprintln!("rgrep: {}: Is a directory", path);
                    }
                    has_error = true;
                }
            } else {
                let file_type = meta.file_type();
                if file_type.is_fifo() || file_type.is_socket() || file_type.is_block_device() || file_type.is_char_device() {
                    if config.devices == DevicesAction::Skip {
                        continue;
                    }
                }
                resolved_files.push(path.clone());
            }
        } else {
            if !config.no_messages {
                eprintln!("rgrep: {}: No such file or directory", path);
            }
            has_error = true;
        }
    }

    Ok((resolved_files, has_error))
}

fn print_line(config: &Config, filename: &str, print_filename: bool, line_number: usize, byte_offset: usize, line: &str, is_match: bool, color_enabled: bool, colors: &GrepColors) {
    let sep = if is_match { ":" } else { "-" };
    let sep_col = if color_enabled { ansi_wrap(sep, &colors.se) } else { sep.to_string() };
    let null_sep = "\0";

    if config.initial_tab {
        print!("\t");
    }

    if print_filename {
        let fname_col = if color_enabled { ansi_wrap(filename, &colors.fn_color) } else { filename.to_string() };
        if config.null {
            print!("{}{}", fname_col, null_sep);
        } else {
            print!("{}{}", fname_col, sep_col);
        }
    }
    if config.line_number {
        let lnum_col = if color_enabled { ansi_wrap(&line_number.to_string(), &colors.ln) } else { line_number.to_string() };
        print!("{}{}", lnum_col, sep_col);
    }
    if config.byte_offset {
        let boff_col = if color_enabled { ansi_wrap(&byte_offset.to_string(), &colors.bn) } else { byte_offset.to_string() };
        print!("{}{}", boff_col, sep_col);
    }
    println!("{}", line);
    
    if config.line_buffered {
        let _ = io::stdout().flush();
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
}
