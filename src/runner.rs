use crate::cli::Config;
use crate::matcher::Matcher;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn run(config: Config) -> Result<(), io::Error> {
    let matcher = Matcher::new(&config);
    let multiple_files = config.files.len() > 1;

    for filename in &config.files {
        let reader: Box<dyn BufRead> = if filename == "-" {
            Box::new(BufReader::new(io::stdin()))
        } else {
            let file = File::open(filename)?;
            Box::new(BufReader::new(file))
        };

        let mut line_number = 1;
        let mut match_count = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            
            if matcher.is_match(&line) {
                match_count += 1;

                if !config.count {
                    print_match(&config, filename, multiple_files, line_number, &line);
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

fn print_match(config: &Config, filename: &str, multiple_files: bool, line_number: usize, line: &str) {
    if multiple_files {
        print!("{}:", filename);
    }
    if config.line_number {
        print!("{}:", line_number);
    }
    println!("{}", line);
}
