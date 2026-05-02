use crate::cli::Config;
use crate::matcher::Matcher;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Cursor};
use std::error::Error;
use std::collections::VecDeque;
use walkdir::WalkDir;
use memmap2::MmapOptions;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let matcher = Matcher::new(&config)?;
    let files_to_search = resolve_files(&config)?;
    
    let mut print_filename = files_to_search.len() > 1;
    if config.no_filename {
        print_filename = false;
    }
    if config.with_filename {
        print_filename = true;
    }

    for filename in files_to_search {
        if filename == "-" {
            let reader = BufReader::new(io::stdin());
            process_file(&config, &matcher, reader, &filename, print_filename)?;
        } else {
            let file = match File::open(&filename) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("rgrep: {}: {}", filename, e);
                    continue;
                }
            };

            if config.mmap {
                if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                    let cursor = Cursor::new(&mmap[..]);
                    process_file(&config, &matcher, cursor, &filename, print_filename)?;
                    continue;
                }
            }

            let reader = BufReader::new(file);
            process_file(&config, &matcher, reader, &filename, print_filename)?;
        }
    }

    Ok(())
}

fn process_file<R: BufRead>(
    config: &Config,
    matcher: &Matcher,
    reader: R,
    filename: &str,
    print_filename: bool,
) -> Result<(), Box<dyn Error>> {
    let before_ctx = config.get_before_context();
    let after_ctx = config.get_after_context();

    let mut line_number = 1;
    let mut match_count = 0;
    let mut has_match = false;
    
    let mut history: VecDeque<(usize, String)> = VecDeque::with_capacity(before_ctx);
    let mut print_after = 0;
    let mut last_printed_line = 0;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        
        let mut is_match = false;
        if config.max_count.map_or(true, |m| match_count < m) {
            is_match = matcher.is_match(&line);
        }

        if is_match {
            has_match = true;
            match_count += 1;

            if config.quiet {
                std::process::exit(0);
            }

            if config.files_with_matches || config.files_without_match || config.count {
                // Do nothing here
            } else {
                // Context Separator
                let mut first_to_print = line_number;
                for (h_line_num, _) in &history {
                    if *h_line_num > last_printed_line {
                        first_to_print = *h_line_num;
                        break;
                    }
                }

                if last_printed_line > 0 && first_to_print > last_printed_line + 1 && (before_ctx > 0 || after_ctx > 0) {
                    println!("--");
                }

                // Print history
                for (h_line_num, h_line) in &history {
                    if *h_line_num > last_printed_line {
                        print_line(config, filename, print_filename, *h_line_num, h_line, false);
                        last_printed_line = *h_line_num;
                    }
                }
                history.clear();

                // Print matched line
                if config.only_matching {
                    let matches = matcher.find_matches(&line);
                    for m in matches {
                        let output = if config.color { format!("\x1b[31;1m{}\x1b[0m", m) } else { m };
                        print_line(config, filename, print_filename, line_number, &output, true);
                    }
                } else {
                    let output_line = if config.color { matcher.highlight(&line) } else { line.to_string() };
                    print_line(config, filename, print_filename, line_number, &output_line, true);
                }
                last_printed_line = line_number;
                print_after = after_ctx;
            }
        } else {
            if print_after > 0 {
                if !(config.files_with_matches || config.files_without_match || config.count || config.only_matching) {
                    print_line(config, filename, print_filename, line_number, &line, false);
                }
                last_printed_line = line_number;
                print_after -= 1;
            } else if before_ctx > 0 {
                if history.len() == before_ctx && before_ctx > 0 {
                    history.pop_front();
                }
                history.push_back((line_number, line.clone()));
            }
        }

        if let Some(max) = config.max_count {
            if match_count >= max && print_after == 0 {
                break;
            }
        }

        line_number += 1;
    }

    if config.files_without_match && !has_match {
        println!("{}", filename);
    } else if config.files_with_matches && has_match {
        println!("{}", filename);
    } else if config.count {
        if print_filename {
            println!("{}:{}", filename, match_count);
        } else {
            println!("{}", match_count);
        }
    }

    Ok(())
}

use globset::{Glob, GlobSet, GlobSetBuilder};

fn build_globset(patterns: &[String]) -> Result<GlobSet, Box<dyn Error>> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(builder.build()?)
}

fn resolve_files(config: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    let mut resolved_files = Vec::new();

    let files = if config.files.is_empty() {
        vec!["-".to_string()]
    } else {
        config.files.clone()
    };

    let include_set = build_globset(&config.include)?;
    let exclude_set = build_globset(&config.exclude)?;
    let exclude_dir_set = build_globset(&config.exclude_dir)?;

    for path in &files {
        if path == "-" {
            resolved_files.push(path.clone());
            continue;
        }

        let metadata = fs::metadata(path);
        if let Ok(meta) = metadata {
            if meta.is_dir() {
                if config.recursive {
                    let mut it = WalkDir::new(path).into_iter();
                    loop {
                        let entry = match it.next() {
                            None => break,
                            Some(Err(_)) => continue,
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
                            if !config.exclude.is_empty() && exclude_set.is_match(file_name_os) {
                                continue;
                            }
                            resolved_files.push(entry.path().to_string_lossy().into_owned());
                        }
                    }
                } else {
                    eprintln!("rgrep: {}: Is a directory", path);
                }
            } else {
                resolved_files.push(path.clone());
            }
        } else {
            eprintln!("rgrep: {}: No such file or directory", path);
        }
    }

    Ok(resolved_files)
}

fn print_line(config: &Config, filename: &str, print_filename: bool, line_number: usize, line: &str, is_match: bool) {
    let sep = if is_match { ":" } else { "-" };
    let sep_col = if config.color { format!("\x1b[36m{}\x1b[0m", sep) } else { sep.to_string() };

    if print_filename {
        let fname_col = if config.color { format!("\x1b[35m{}\x1b[0m", filename) } else { filename.to_string() };
        print!("{}{}", fname_col, sep_col);
    }
    if config.line_number {
        let lnum_col = if config.color { format!("\x1b[32m{}\x1b[0m", line_number) } else { line_number.to_string() };
        print!("{}{}", lnum_col, sep_col);
    }
    println!("{}", line);
}
