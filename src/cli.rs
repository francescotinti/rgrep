use clap::Parser;

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about = "A Rust implementation of GNU grep")]
pub struct Config {
    /// Ignore case distinctions in patterns and input data
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Invert the sense of matching, to select non-matching lines
    #[arg(short = 'v', long = "invert-match")]
    pub invert_match: bool,

    /// Prefix each line of output with the 1-based line number within its input file
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Suppress normal output; instead print a count of matching lines for each input file
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Select only those lines containing matches that form whole words
    #[arg(short = 'w', long = "word-regexp")]
    pub word_regexp: bool,

    /// Read all files under each directory, recursively
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Highlight matches in output
    #[arg(long = "color")]
    pub color: bool,

    /// Suppress normal output; instead print the name of each input file from which output would normally have been printed.
    #[arg(short = 'l', long = "files-with-matches")]
    pub files_with_matches: bool,

    /// Suppress normal output; instead print the name of each input file from which no output would normally have been printed.
    #[arg(short = 'L', long = "files-without-match")]
    pub files_without_match: bool,

    /// Quiet; do not write anything to standard output. Exit immediately with zero status if any match is found.
    #[arg(short = 'q', long = "quiet", visible_alias = "silent")]
    pub quiet: bool,

    /// Suppress the prefixing of file names on output.
    #[arg(short = 'h', long = "no-filename")]
    pub no_filename: bool,

    /// Print the file name for each match.
    #[arg(short = 'H', long = "with-filename")]
    pub with_filename: bool,

    /// Print only the matched (non-empty) parts of a matching line, with each such part on a separate output line.
    #[arg(short = 'o', long = "only-matching")]
    pub only_matching: bool,

    /// The pattern to search for
    #[arg(required = true)]
    pub pattern: String,

    /// Files to search
    #[arg(required = true)]
    pub files: Vec<String>,
}

impl Config {
    pub fn parse_args(args: impl IntoIterator<Item = std::ffi::OsString>) -> clap::error::Result<Self> {
        Self::try_parse_from(args)
    }
}
