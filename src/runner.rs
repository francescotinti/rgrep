use crate::cli::Config;
use crate::matcher::Matcher;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::error::Error;
use walkdir::WalkDir;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let matcher = Matcher::new(&config)?;
    let files_to_search = resolve_files(&config)?;
    let multiple_files = files_to_search.len() > 1;

    for filename in files_to_search {
        let reader: Box<dyn BufRead> = if filename == "-" {
            Box::new(BufReader::new(io::stdin()))
        } else {
            let file = File::open(&filename)?;
            Box::new(BufReader::new(file))
        };

        let mut line_number = 1;
        let mut match_count = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            
            if matcher.is_match(&line) {
                match_count += 1;

                if !config.count {
                    let output_line = if config.color {
                        matcher.highlight(&line)
                    } else {
                        line.to_string()
                    };
                    print_match(&config, &filename, multiple_files, line_number, &output_line);
                }
            }
            line_number += 1;
        }

        if config.count {
            if multiple_files {
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

fn print_match(config: &Config, filename: &str, multiple_files: bool, line_number: usize, line: &str) {
    if multiple_files {
        print!("{}:", filename);
    }
    if config.line_number {
        print!("{}:", line_number);
    }
    println!("{}", line);
}
