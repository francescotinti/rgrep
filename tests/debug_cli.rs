use rgrep::cli::Config;
use std::ffi::OsString;
use clap::Parser;

fn main() {
    let args: Vec<OsString> = vec![
        "rgrep".into(),
        "-E".into(),
        "foo\\|bar".into()
    ];
    let config = Config::parse_from(args);
    println!("extended: {}", config.extended_regexp);
    println!("basic: {}", config.basic_regexp);
}
