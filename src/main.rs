use clap::Parser;
use rgrep::cli::Config;

fn main() {
    let config = Config::parse();
    println!("Parsed config: {:?}", config);
    // Il resto della logica verrà implementato nelle prossime fasi.
}
