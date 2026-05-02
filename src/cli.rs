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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_basic_args() {
        let args = vec![
            OsString::from("rgrep"),
            OsString::from("pattern"),
            OsString::from("file.txt"),
        ];
        let config = Config::parse_args(args).unwrap();
        assert_eq!(config.pattern, "pattern");
        assert_eq!(config.files, vec!["file.txt"]);
        assert!(!config.ignore_case);
    }

    #[test]
    fn test_flags() {
        let args = vec![
            OsString::from("rgrep"),
            OsString::from("-i"),
            OsString::from("-v"),
            OsString::from("pattern"),
            OsString::from("file.txt"),
        ];
        let config = Config::parse_args(args).unwrap();
        assert!(config.ignore_case);
        assert!(config.invert_match);
    }
}
