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

    /// Print NUM lines of trailing context after matching lines.
    #[arg(short = 'A', long = "after-context", default_value_t = 0)]
    pub after_context: usize,

    /// Print NUM lines of leading context before matching lines.
    #[arg(short = 'B', long = "before-context", default_value_t = 0)]
    pub before_context: usize,

    /// Print NUM lines of output context.
    #[arg(short = 'C', long = "context", default_value_t = 0)]
    pub context: usize,

    /// Stop reading a file after NUM matching lines.
    #[arg(short = 'm', long = "max-count")]
    pub max_count: Option<usize>,

    /// Use memory-mapped I/O to read input files if possible.
    #[arg(long = "mmap")]
    pub mmap: bool,

    /// Use PATTERN as the pattern.
    #[arg(short = 'e', long = "regexp", action = clap::ArgAction::Append)]
    pub regexp: Vec<String>,

    /// Obtain PATTERN from FILE.
    #[arg(short = 'f', long = "file", action = clap::ArgAction::Append)]
    pub file_patterns: Vec<String>,

    /// Interpret PATTERN as fixed strings, not regular expressions.
    #[arg(short = 'F', long = "fixed-strings")]
    pub fixed_strings: bool,

    /// Interpret PATTERN as a Perl-compatible regular expression (PCRE).
    #[arg(short = 'P', long = "perl-regexp")]
    pub perl_regexp: bool,

    /// Skip files matching GLOB.
    #[arg(long = "exclude", action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Search only files that match GLOB.
    #[arg(long = "include", action = clap::ArgAction::Append)]
    pub include: Vec<String>,

    /// Exclude directories matching GLOB.
    #[arg(long = "exclude-dir", action = clap::ArgAction::Append)]
    pub exclude_dir: Vec<String>,

    /// Print the 0-based byte offset within the input file before each line of output.
    #[arg(short = 'b', long = "byte-offset")]
    pub byte_offset: bool,

    /// Output a zero byte (the ASCII NUL character) instead of the character that normally follows a file name.
    #[arg(short = 'Z', long = "null")]
    pub null: bool,

    /// Treat input and output data as sequences of lines, each terminated by a zero byte instead of a newline.
    #[arg(short = 'z', long = "null-data")]
    pub null_data: bool,

    /// Process a binary file as if it were text.
    #[arg(short = 'a', long = "text")]
    pub text: bool,

    /// A pattern to search for (if -e or -f is not provided)
    #[arg(required_unless_present_any = ["regexp", "file_patterns"])]
    pub pattern: Option<String>,

    /// Files to search
    #[arg()]
    pub files: Vec<String>,
}

impl Config {
    pub fn parse_args(args: impl IntoIterator<Item = std::ffi::OsString>) -> clap::error::Result<Self> {
        Self::try_parse_from(args)
    }

    pub fn get_after_context(&self) -> usize {
        std::cmp::max(self.after_context, self.context)
    }

    pub fn get_before_context(&self) -> usize {
        std::cmp::max(self.before_context, self.context)
    }
}
