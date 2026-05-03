use clap::Parser;
use rgrep::cli::Config;
use std::process;

fn main() {
    let config = Config::parse();
    
    if let Err(e) = rgrep::runner::run(config) {
        eprintln!("rgrep: {}", e);
        process::exit(2);
    }
}
