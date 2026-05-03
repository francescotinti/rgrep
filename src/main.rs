use clap::Parser;
use rgrep::cli::Config;
use std::process;

fn main() {
    let config = Config::parse();
    
    match rgrep::runner::run(config) {
        Ok(rgrep::runner::RunResult::MatchFound) => process::exit(0),
        Ok(rgrep::runner::RunResult::NoMatch) => process::exit(1),
        Err(e) => {
            let msg = e.to_string();
            if !msg.is_empty() {
                eprintln!("rgrep: {}", msg);
            }
            process::exit(2);
        }
    }
}
