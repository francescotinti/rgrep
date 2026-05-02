use crate::cli::Config;
use crate::matcher::Matcher;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::error::Error;
use walkdir::WalkDir;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let matcher = Matcher::new(&config)?;
    let files_to_search = resolve_files(&config)?;
    
    // By default, grep prints filename if there's more than one file, unless overriden.
    let mut print_filename = files_to_search.len() > 1;
    if config.no_filename {
        print_filename = false;
    }
    if config.with_filename {
        print_filename = true;
    }

    for filename in files_to_search {
        let reader: Box<dyn BufRead> = if filename == "-" {
            Box::new(BufReader::new(io::stdin()))
        } else {
            let file = match File::open(&filename) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("rgrep: {}: {}", filename, e);
                    continue;
                }
            };
            Box::new(BufReader::new(file))
        };

        let mut line_number = 1;
        let mut match_count = 0;
        let mut has_match = false;

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue, // Ignore read errors for simplicity like grep binary mode
            };
            
            if matcher.is_match(&line) {
                has_match = true;
                match_count += 1;

                if config.quiet {
                    std::process::exit(0);
                }

                if config.files_with_matches || config.files_without_match || config.count {
                    // Don't print lines if we only want file names or counts
                } else if config.only_matching {
                    let matches = matcher.find_matches(&line);
                    for m in matches {
                        let output = if config.color {
                            format!("\x1b[31;1m{}\x1b[0m", m)
                        } else {
                            m
                        };
                        print_match(&config, &filename, print_filename, line_number, &output);
                    }
                } else {
                    let output_line = if config.color {
                        matcher.highlight(&line)
                    } else {
                        line.to_string()
                    };
                    print_match(&config, &filename, print_filename, line_number, &output_line);
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
    }

    Ok(())
}

fn resolve_files(config: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    let mut resolved_files = Vec::new();

    for path in &config.files {
        if path == "-" {
            resolved_files.push(path.clone());
            continue;
        }

        let metadata = fs::metadata(path);
        if let Ok(meta) = metadata {
            if meta.is_dir() {
                if config.recursive {
                    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                        if entry.file_type().is_file() {
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

fn print_match(config: &Config, filename: &str, print_filename: bool, line_number: usize, line: &str) {
    if print_filename {
        let fname_col = if config.color { format!("\x1b[35m{}\x1b[36m:\x1b[0m", filename) } else { format!("{}:", filename) };
        print!("{}", fname_col);
    }
    if config.line_number {
        let lnum_col = if config.color { format!("\x1b[32m{}\x1b[36m:\x1b[0m", line_number) } else { format!("{}:", line_number) };
        print!("{}", lnum_col);
    }
    println!("{}", line);
}
