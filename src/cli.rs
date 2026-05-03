// rgrep — GNU grep ported to Rust
// Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
//
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Gemini Antigravity (Google)
//
// https://github.com/francescotinti/rgrep

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum DirectoriesAction {
    Read,
    Recurse,
    Skip,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum DevicesAction {
    Read,
    Skip,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryAction {
    Binary,
    Text,
    WithoutMatch,
}

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about = "A Rust implementation of GNU grep", disable_help_flag = true)]
pub struct Config {
    /// Interpret PATTERNS as extended regular expressions.
    #[arg(short = 'E', long = "extended-regexp", overrides_with_all = ["basic_regexp", "fixed_strings", "perl_regexp"])]
    pub extended_regexp: bool,

    /// Interpret PATTERNS as basic regular expressions.
    #[arg(short = 'G', long = "basic-regexp", overrides_with_all = ["extended_regexp", "fixed_strings", "perl_regexp"])]
    pub basic_regexp: bool,

    /// Ignore case distinctions in patterns and input data
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Do not ignore case distinctions.
    #[arg(long = "no-ignore-case")]
    pub no_ignore_case: bool,

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

    /// Select only those matches that exactly match the whole line.
    #[arg(short = 'x', long = "line-regexp")]
    pub line_regexp: bool,

    /// Read all files under each directory, recursively
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Read all files under each directory, recursively. Follow all symlinks.
    #[arg(short = 'R', long = "dereference-recursive")]
    pub dereference_recursive: bool,

    /// How to handle directories (read, recurse, skip).
    #[arg(short = 'd', long = "directories", default_value = "read")]
    pub directories: DirectoriesAction,

    /// How to handle devices, FIFOs and sockets (read, skip).
    #[arg(short = 'D', long = "devices", default_value = "read")]
    pub devices: DevicesAction,

    /// Highlight matches in output
    #[arg(long = "color", default_value = "never", default_missing_value = "always", num_args = 0..=1)]
    pub color: String,

    /// Suppress normal output; instead print the name of each input file from which output would normally have been printed.
    #[arg(short = 'l', long = "files-with-matches")]
    pub files_with_matches: bool,

    /// Suppress normal output; instead print the name of each input file from which no output would normally have been printed.
    #[arg(short = 'L', long = "files-without-match")]
    pub files_without_match: bool,

    /// Quiet; do not write anything to standard output. Exit immediately with zero status if any match is found.
    #[arg(short = 'q', long = "quiet", visible_alias = "silent")]
    pub quiet: bool,

    /// Suppress error messages about nonexistent or unreadable files.
    #[arg(short = 's', long = "no-messages")]
    pub no_messages: bool,

    /// Suppress the prefixing of file names on output.
    #[arg(short = 'h', long = "no-filename", overrides_with("with_filename"))]
    pub no_filename: bool,

    /// Print the file name for each match.
    #[arg(short = 'H', long = "with-filename", overrides_with("no_filename"))]
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

    /// Use SEP as a group separator (default is `--`).
    #[arg(long = "group-separator", default_value = "--")]
    pub group_separator: String,

    /// Do not use a group separator.
    #[arg(long = "no-group-separator")]
    pub no_group_separator: bool,

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
    #[arg(short = 'F', long = "fixed-strings", overrides_with_all = ["extended_regexp", "basic_regexp", "perl_regexp"])]
    pub fixed_strings: bool,

    /// Interpret PATTERN as a Perl-compatible regular expression (PCRE).
    #[arg(short = 'P', long = "perl-regexp", overrides_with_all = ["extended_regexp", "basic_regexp", "fixed_strings"])]
    pub perl_regexp: bool,

    /// Skip files matching GLOB.
    #[arg(long = "exclude", action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Read exclude patterns from FILE.
    #[arg(long = "exclude-from")]
    pub exclude_from: Option<String>,

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

    /// Process a binary file as if it did not contain matching data.
    #[arg(short = 'I')]
    pub without_match: bool,

    /// Treat binary files as TYPE (binary, text, without-match).
    #[arg(long = "binary-files")]
    pub binary_files: Option<String>,

    /// Treat the file(s) as binary.
    #[arg(short = 'U', long = "binary")]
    pub binary: bool,

    /// Line up tabs so that tabs align.
    #[arg(short = 'T', long = "initial-tab")]
    pub initial_tab: bool,

    /// Use LABEL as the standard input file name prefix.
    #[arg(long = "label", default_value = "(standard input)")]
    pub label: String,

    /// Flush output on every line.
    #[arg(long = "line-buffered")]
    pub line_buffered: bool,

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

    pub fn get_binary_action(&self) -> BinaryAction {
        if let Some(bf) = &self.binary_files {
            match bf.as_str() {
                "text" => return BinaryAction::Text,
                "without-match" => return BinaryAction::WithoutMatch,
                "binary" => return BinaryAction::Binary,
                _ => return BinaryAction::Binary, // Default if invalid
            }
        }
        if self.text {
            return BinaryAction::Text;
        }
        if self.without_match {
            return BinaryAction::WithoutMatch;
        }
        BinaryAction::Binary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directories_action_parsing() {
        let config = Config::parse_args(vec![std::ffi::OsString::from("rgrep"), std::ffi::OsString::from("-d"), std::ffi::OsString::from("read"), std::ffi::OsString::from("pat")]).unwrap();
        assert_eq!(config.directories, DirectoriesAction::Read);

        let config2 = Config::parse_args(vec![std::ffi::OsString::from("rgrep"), std::ffi::OsString::from("--directories=skip"), std::ffi::OsString::from("pat")]).unwrap();
        assert_eq!(config2.directories, DirectoriesAction::Skip);

        let config3 = Config::parse_args(vec![std::ffi::OsString::from("rgrep"), std::ffi::OsString::from("-d"), std::ffi::OsString::from("recurse"), std::ffi::OsString::from("pat")]).unwrap();
        assert_eq!(config3.directories, DirectoriesAction::Recurse);
    }

    #[test]
    fn test_devices_action_parsing() {
        let config = Config::parse_args(vec![std::ffi::OsString::from("rgrep"), std::ffi::OsString::from("-D"), std::ffi::OsString::from("read"), std::ffi::OsString::from("pat")]).unwrap();
        assert_eq!(config.devices, DevicesAction::Read);

        let config2 = Config::parse_args(vec![std::ffi::OsString::from("rgrep"), std::ffi::OsString::from("--devices=skip"), std::ffi::OsString::from("pat")]).unwrap();
        assert_eq!(config2.devices, DevicesAction::Skip);
    }
}
