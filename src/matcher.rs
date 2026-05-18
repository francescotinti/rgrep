// rgrep — GNU grep ported to Rust
// Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
//
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Gemini Antigravity (Google)
//
// https://github.com/francescotinti/rgrep

use crate::cli::Config;
use crate::error::RgrepError;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;

/// Uniform behavior every concrete regex/literal engine must expose so
/// [`Matcher`] can delegate without enumerating the [`Engine`] variants.
///
/// The trait is the spec-authorized fallback for D-Ondata-3.2 (sealed-trait +
/// single-match dispatch helper, used in place of `enum_dispatch` which does
/// not compose cleanly with the `cfg`-gated `Engine::Pcre2` variant).
pub trait MatchEngine {
    fn engine_is_match(&self, line: &str) -> bool;
    fn engine_highlight<'a>(&self, line: &'a str, ms_code: &str) -> Cow<'a, str>;
    fn engine_find_offsets(&self, line: &str) -> Vec<(usize, String)>;
}

pub enum Engine {
    Regex(Regex),
    Fancy(fancy_regex::Regex),
    AhoCorasick(AhoCorasick),
    #[cfg(feature = "perl-regexp")]
    Pcre2(pcre2::bytes::Regex),
}

impl Engine {
    /// Single point of variant dispatch: every per-engine match arm lives here,
    /// so the [`Matcher`] methods are 1-liner trait calls.
    fn as_match_engine(&self) -> &dyn MatchEngine {
        match self {
            Self::Regex(re) => re,
            Self::Fancy(re) => re,
            Self::AhoCorasick(ac) => ac,
            #[cfg(feature = "perl-regexp")]
            Self::Pcre2(re) => re,
        }
    }
}

impl MatchEngine for Regex {
    fn engine_is_match(&self, line: &str) -> bool {
        self.is_match(line)
    }

    fn engine_highlight<'a>(&self, line: &'a str, ms_code: &str) -> Cow<'a, str> {
        let rep = format!("\x1b[{ms_code}m\x1b[K$0\x1b[m\x1b[K");
        self.replace_all(line, rep.as_str())
    }

    fn engine_find_offsets(&self, line: &str) -> Vec<(usize, String)> {
        self.find_iter(line)
            .map(|m| (m.start(), m.as_str().to_string()))
            .collect()
    }
}

impl MatchEngine for fancy_regex::Regex {
    fn engine_is_match(&self, line: &str) -> bool {
        self.is_match(line).unwrap_or(false)
    }

    fn engine_highlight<'a>(&self, line: &'a str, ms_code: &str) -> Cow<'a, str> {
        Cow::Owned(build_highlighted(
            line,
            ms_code,
            self.find_iter(line).flatten().map(|m| (m.start(), m.end())),
        ))
    }

    fn engine_find_offsets(&self, line: &str) -> Vec<(usize, String)> {
        self.find_iter(line)
            .flatten()
            .map(|m| (m.start(), m.as_str().to_string()))
            .collect()
    }
}

impl MatchEngine for AhoCorasick {
    fn engine_is_match(&self, line: &str) -> bool {
        self.is_match(line)
    }

    fn engine_highlight<'a>(&self, line: &'a str, ms_code: &str) -> Cow<'a, str> {
        Cow::Owned(build_highlighted(
            line,
            ms_code,
            self.find_iter(line).map(|m| (m.start(), m.end())),
        ))
    }

    fn engine_find_offsets(&self, line: &str) -> Vec<(usize, String)> {
        self.find_iter(line)
            .map(|m| (m.start(), line[m.start()..m.end()].to_string()))
            .collect()
    }
}

#[cfg(feature = "perl-regexp")]
impl MatchEngine for pcre2::bytes::Regex {
    fn engine_is_match(&self, line: &str) -> bool {
        self.is_match(line.as_bytes()).unwrap_or(false)
    }

    fn engine_highlight<'a>(&self, line: &'a str, ms_code: &str) -> Cow<'a, str> {
        Cow::Owned(build_highlighted(
            line,
            ms_code,
            self.find_iter(line.as_bytes())
                .flatten()
                .map(|m| (m.start(), m.end())),
        ))
    }

    fn engine_find_offsets(&self, line: &str) -> Vec<(usize, String)> {
        self.find_iter(line.as_bytes())
            .flatten()
            .map(|m| (m.start(), line[m.start()..m.end()].to_string()))
            .collect()
    }
}

/// Build an ANSI-decorated copy of `line` by walking the (start, end) ranges
/// produced by any concrete engine. Shared by the manual-loop engines
/// (`fancy_regex`, `AhoCorasick`, `pcre2`); the `regex` crate uses its own
/// `replace_all` fast path.
fn build_highlighted(
    line: &str,
    ms_code: &str,
    matches: impl Iterator<Item = (usize, usize)>,
) -> String {
    let mut result = String::with_capacity(line.len());
    let mut last_match = 0;
    for (start, end) in matches {
        result.push_str(&line[last_match..start]);
        result.push_str("\x1b[");
        result.push_str(ms_code);
        result.push_str("m\x1b[K");
        result.push_str(&line[start..end]);
        result.push_str("\x1b[m\x1b[K");
        last_match = end;
    }
    result.push_str(&line[last_match..]);
    result
}

pub struct Matcher<'a> {
    config: &'a Config,
    engine: Engine,
}

#[must_use]
pub fn bre_to_ere(pattern: &str) -> String {
    let mut ere = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    '?' | '+' | '(' | ')' | '{' | '}' | '|' => {
                        ere.push(next);
                        chars.next();
                    }
                    _ => {
                        ere.push('\\');
                    }
                }
            } else {
                ere.push('\\');
            }
        } else {
            match c {
                '?' | '+' | '(' | ')' | '{' | '}' | '|' => {
                    ere.push('\\');
                    ere.push(c);
                }
                _ => {
                    ere.push(c);
                }
            }
        }
    }
    ere
}

impl<'a> Matcher<'a> {
    /// Build a `Matcher` from the user `Config` and the collected raw patterns.
    ///
    /// # Errors
    /// Returns `RgrepError::InvalidRegex` if a pattern cannot be compiled by
    /// any of the supported engines, or `RgrepError::PcreUnavailable` if `-P`
    /// is requested in a build without the `perl-regexp` feature.
    pub fn new(config: &'a Config, raw_patterns: Vec<String>) -> Result<Self, RgrepError> {
        let is_basic = config.pattern_opts.basic_regexp
            || (!config.pattern_opts.extended_regexp
                && !config.pattern_opts.fixed_strings
                && !config.pattern_opts.perl_regexp);

        let final_patterns: Vec<String> = if is_basic {
            raw_patterns.into_iter().map(|p| bre_to_ere(&p)).collect()
        } else {
            raw_patterns
        };

        let ignore_case = config.pattern_opts.ignore_case && !config.pattern_opts.no_ignore_case;

        #[cfg(not(feature = "perl-regexp"))]
        if config.pattern_opts.perl_regexp {
            return Err(RgrepError::PcreUnavailable);
        }

        #[cfg(feature = "perl-regexp")]
        if config.pattern_opts.perl_regexp {
            let pat = final_patterns.join("|");
            let mut builder = pcre2::bytes::RegexBuilder::new();
            builder.caseless(ignore_case);
            builder.utf(true).jit(true);
            let re = builder
                .build(&pat)
                .map_err(|e| RgrepError::InvalidRegex(e.to_string()))?;
            return Ok(Self {
                config,
                engine: Engine::Pcre2(re),
            });
        }

        if config.pattern_opts.fixed_strings
            && !config.pattern_opts.word_regexp
            && !config.pattern_opts.line_regexp
        {
            let ac = AhoCorasickBuilder::new()
                .ascii_case_insensitive(ignore_case)
                .build(&final_patterns)
                .map_err(|e| RgrepError::InvalidRegex(e.to_string()))?;
            return Ok(Self {
                config,
                engine: Engine::AhoCorasick(ac),
            });
        }

        let final_patterns_escaped: Vec<String> = if config.pattern_opts.fixed_strings {
            final_patterns
                .into_iter()
                .map(|p| regex::escape(&p))
                .collect()
        } else {
            final_patterns
        };

        let mut fancy_needed = false;

        // 1. Try compile each pattern individually as syntax check
        for pat in &final_patterns_escaped {
            let mut p = pat.clone();
            if config.pattern_opts.line_regexp {
                p = format!(r"^(?:{p})$");
            } else if config.pattern_opts.word_regexp {
                p = format!(r"\b(?:{p})\b");
            }
            if RegexBuilder::new(&p)
                .case_insensitive(ignore_case)
                .build()
                .is_err()
            {
                if fancy_regex::Regex::new(&p).is_err() {
                    return Err(RgrepError::InvalidRegex(pat.clone()));
                }
                fancy_needed = true;
            }
        }

        let mut combined = final_patterns_escaped.join("|");

        if config.pattern_opts.line_regexp {
            combined = format!(r"^(?:{combined})$");
        } else if config.pattern_opts.word_regexp {
            combined = format!(r"\b(?:{combined})\b");
        }

        if fancy_needed {
            let p = if ignore_case {
                format!("(?i){combined}")
            } else {
                combined
            };
            let re =
                fancy_regex::Regex::new(&p).map_err(|e| RgrepError::InvalidRegex(e.to_string()))?;
            Ok(Self {
                config,
                engine: Engine::Fancy(re),
            })
        } else {
            let re = RegexBuilder::new(&combined)
                .case_insensitive(ignore_case)
                .build()
                .map_err(|e| RgrepError::InvalidRegex(e.to_string()))?;
            Ok(Self {
                config,
                engine: Engine::Regex(re),
            })
        }
    }

    #[must_use]
    pub fn is_match(&self, line: &str) -> bool {
        let matches = self.engine.as_match_engine().engine_is_match(line);
        if self.config.filter_opts.invert_match {
            !matches
        } else {
            matches
        }
    }

    #[must_use]
    pub fn highlight<'b>(&self, line: &'b str, colors: &crate::output::GrepColors) -> Cow<'b, str> {
        if self.config.filter_opts.invert_match || colors.is_disabled() || colors.ms.is_empty() {
            return Cow::Borrowed(line);
        }
        self.engine
            .as_match_engine()
            .engine_highlight(line, &colors.ms)
    }

    #[must_use]
    pub fn find_match_offsets(&self, line: &str) -> Vec<(usize, String)> {
        if self.config.filter_opts.invert_match {
            return vec![];
        }
        self.engine.as_match_engine().engine_find_offsets(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Config;

    #[test]
    fn test_find_match_offsets() {
        let config = Config::parse_args(vec![
            std::ffi::OsString::from("rgrep"),
            std::ffi::OsString::from("foo"),
        ])
        .unwrap();
        let matcher = Matcher::new(&config, vec!["foo".to_string()]).unwrap();

        let offsets = matcher.find_match_offsets("foo bar foo");
        assert_eq!(
            offsets,
            vec![(0, "foo".to_string()), (8, "foo".to_string())]
        );
    }

    #[test]
    fn test_bre_to_ere() {
        assert_eq!(bre_to_ere("foo\\?"), "foo?");
        assert_eq!(bre_to_ere("foo?"), "foo\\?");
        assert_eq!(bre_to_ere("\\(a\\|b\\)"), "(a|b)");
        assert_eq!(bre_to_ere("(a|b)"), "\\(a\\|b\\)");
        assert_eq!(bre_to_ere("^foo$"), "^foo$");
    }

    #[cfg(feature = "perl-regexp")]
    #[test]
    fn test_pcre_lookahead() {
        let mut config = get_base_config("foo(?=bar)");
        config.pattern_opts.perl_regexp = true;
        let matcher = Matcher::new(&config, vec!["foo(?=bar)".to_string()]).unwrap();
        assert!(matcher.is_match("foobar"));
        assert!(!matcher.is_match("foobaz"));
    }

    #[cfg(feature = "perl-regexp")]
    #[test]
    fn test_pcre_backreference() {
        let mut config = get_base_config(r"(\w+)\s+\1");
        config.pattern_opts.perl_regexp = true;
        let matcher = Matcher::new(&config, vec![r"(\w+)\s+\1".to_string()]).unwrap();
        assert!(matcher.is_match("foo foo"));
        assert!(!matcher.is_match("foo bar"));
    }

    fn get_base_config(pattern: &str) -> Config {
        Config {
            pattern_opts: crate::cli::PatternOpts {
                extended_regexp: false,
                basic_regexp: false,
                fixed_strings: false,
                perl_regexp: false,
                ignore_case: false,
                no_ignore_case: false,
                word_regexp: false,
                line_regexp: false,
                regexp: vec![],
                file_patterns: vec![],
                pattern: Some(pattern.to_string()),
            },
            output_opts: crate::cli::OutputOpts {
                line_number: false,
                count: false,
                files_with_matches: false,
                files_without_match: false,
                quiet: false,
                no_filename: false,
                with_filename: false,
                only_matching: false,
                byte_offset: false,
                null: false,
                null_data: false,
                initial_tab: false,
                color: "never".to_string(),
                label: "(standard input)".to_string(),
                line_buffered: false,
                group_separator: "--".to_string(),
                no_group_separator: false,
            },
            context_opts: crate::cli::ContextOpts {
                after_context: 0,
                before_context: 0,
                context: 0,
            },
            filter_opts: crate::cli::FilterOpts {
                invert_match: false,
                recursive: false,
                dereference_recursive: false,
                directories: crate::cli::DirectoriesAction::Read,
                devices: crate::cli::DevicesAction::Read,
                include: vec![],
                exclude: vec![],
                exclude_from: None,
                exclude_dir: vec![],
                text: false,
                without_match: false,
                no_messages: false,
            },
            binary_opts: crate::cli::BinaryOpts {
                binary_files: None,
                binary: false,
                max_count: None,
                mmap: false,
            },
            files: vec![],
        }
    }

    #[test]
    fn test_is_match_basic() {
        let config = get_base_config("hello");
        let matcher = Matcher::new(&config, vec!["hello".to_string()]).unwrap();
        assert!(matcher.is_match("hello world"));
        assert!(!matcher.is_match("bye world"));
    }

    #[test]
    fn test_is_match_ignore_case() {
        let mut config = get_base_config("HELLO");
        config.pattern_opts.ignore_case = true;
        let matcher = Matcher::new(&config, vec!["HELLO".to_string()]).unwrap();
        assert!(matcher.is_match("hello world"));
    }

    #[test]
    fn test_is_match_invert() {
        let mut config = get_base_config("hello");
        config.filter_opts.invert_match = true;
        let matcher = Matcher::new(&config, vec!["hello".to_string()]).unwrap();
        assert!(!matcher.is_match("hello world"));
        assert!(matcher.is_match("bye world"));
    }

    #[test]
    fn test_is_match_word_regexp() {
        let mut config = get_base_config("hello");
        config.pattern_opts.word_regexp = true;
        let matcher = Matcher::new(&config, vec!["hello".to_string()]).unwrap();
        assert!(matcher.is_match("say hello to him"));
        assert!(!matcher.is_match("say helloworld to him"));
    }

    #[test]
    fn test_is_match_line_regexp() {
        let mut config = get_base_config("hello");
        config.pattern_opts.line_regexp = true;
        let matcher = Matcher::new(&config, vec!["hello".to_string()]).unwrap();
        assert!(matcher.is_match("hello"));
        assert!(!matcher.is_match("say hello"));
    }

    #[test]
    fn test_is_match_regex() {
        let config = get_base_config("h.*o");
        let matcher = Matcher::new(&config, vec!["h.*o".to_string()]).unwrap();
        assert!(matcher.is_match("say hello to him"));
    }

    #[test]
    fn test_multiple_patterns() {
        let mut config = get_base_config("hello");
        config.pattern_opts.regexp = vec!["world".to_string()];
        let matcher =
            Matcher::new(&config, vec!["hello".to_string(), "world".to_string()]).unwrap();
        assert!(matcher.is_match("say hello to him"));
        assert!(matcher.is_match("what a beautiful world"));
        assert!(!matcher.is_match("something else entirely"));
    }

    #[test]
    fn test_fixed_strings() {
        let mut config = get_base_config("h.*o");
        config.pattern_opts.fixed_strings = true;
        let matcher = Matcher::new(&config, vec!["h.*o".to_string()]).unwrap();
        assert!(!matcher.is_match("say hello to him"));
        assert!(matcher.is_match("literal h.*o string"));
    }

    #[test]
    fn test_backref_simple() {
        let mut config = get_base_config(r"(.+)\1");
        config.pattern_opts.extended_regexp = true;
        let matcher = Matcher::new(&config, vec![r"(.+)\1".to_string()]).unwrap();
        assert!(matcher.is_match("abab"));
        assert!(!matcher.is_match("abc"));
    }

    #[test]
    fn test_backref_named_in_pattern() {
        let mut config = get_base_config(r"(\d+)-\1");
        config.pattern_opts.extended_regexp = true;
        let matcher = Matcher::new(&config, vec![r"(\d+)-\1".to_string()]).unwrap();
        assert!(matcher.is_match("42-42"));
        assert!(!matcher.is_match("42-43"));
    }

    #[test]
    fn test_multi_e_invalid_validation() {
        let config = get_base_config("[");
        let matcher = Matcher::new(&config, vec!["[".to_string(), "]".to_string()]);
        assert!(matcher.is_err());
    }
}
