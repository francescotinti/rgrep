use crate::cli::Config;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use regex::{Regex, RegexBuilder};
use std::error::Error;

pub enum Engine {
    Regex(Regex),
    AhoCorasick(AhoCorasick),
}

pub struct Matcher<'a> {
    config: &'a Config,
    engine: Engine,
}

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
    pub fn new(config: &'a Config, raw_patterns: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let is_basic = config.basic_regexp || (!config.extended_regexp && !config.fixed_strings && !config.perl_regexp);
        
        let final_patterns: Vec<String> = if is_basic {
            raw_patterns.into_iter().map(|p| bre_to_ere(&p)).collect()
        } else {
            raw_patterns
        };
        
        let ignore_case = config.ignore_case && !config.no_ignore_case;
        
        if config.perl_regexp {
            return Err("-P only supported when compiled with --features perl-regexp".into());
        }
        
        if config.fixed_strings && !config.word_regexp && !config.line_regexp {
            let ac = AhoCorasickBuilder::new()
                .ascii_case_insensitive(ignore_case)
                .build(&final_patterns)
                .map_err(|e| Box::<dyn Error>::from(format!("{}", e)))?;
            return Ok(Self { config, engine: Engine::AhoCorasick(ac) });
        }
        
        let final_patterns_escaped: Vec<String> = if config.fixed_strings {
            final_patterns.into_iter().map(|p| regex::escape(&p)).collect()
        } else {
            final_patterns
        };
        
        let mut combined = final_patterns_escaped.join("|");
        
        if config.line_regexp {
            combined = format!(r"^(?:{})$", combined);
        } else if config.word_regexp {
            combined = format!(r"\b(?:{})\b", combined);
        }
        
        let re = RegexBuilder::new(&combined)
            .case_insensitive(ignore_case)
            .build()
            .map_err(|e| Box::<dyn Error>::from(format!("{}", e)))?;

        Ok(Self { config, engine: Engine::Regex(re) })
    }

    pub fn is_match(&self, line: &str) -> bool {
        let matches = match &self.engine {
            Engine::Regex(re) => re.is_match(line),
            Engine::AhoCorasick(ac) => ac.is_match(line),
        };

        if self.config.invert_match {
            !matches
        } else {
            matches
        }
    }

    pub fn highlight(&self, line: &str, colors: &crate::output::GrepColors) -> String {
        if self.config.invert_match {
            return line.to_string();
        }
        
        let ms_code = &colors.ms;
        if ms_code.is_empty() {
            return line.to_string();
        }

        match &self.engine {
            Engine::Regex(re) => {
                let rep = format!("\x1b[{}m\x1b[K$0\x1b[m\x1b[K", ms_code);
                re.replace_all(line, rep.as_str()).into_owned()
            },
            Engine::AhoCorasick(ac) => {
                let mut result = String::with_capacity(line.len());
                let mut last_match = 0;
                for mat in ac.find_iter(line) {
                    result.push_str(&line[last_match..mat.start()]);
                    result.push_str(&format!("\x1b[{}m\x1b[K", ms_code));
                    result.push_str(&line[mat.start()..mat.end()]);
                    result.push_str("\x1b[m\x1b[K");
                    last_match = mat.end();
                }
                result.push_str(&line[last_match..]);
                result
            }
        }
    }

    pub fn find_match_offsets(&self, line: &str) -> Vec<(usize, String)> {
        if self.config.invert_match {
            return vec![]; 
        }
        match &self.engine {
            Engine::Regex(re) => re.find_iter(line).map(|m| (m.start(), m.as_str().to_string())).collect(),
            Engine::AhoCorasick(ac) => ac.find_iter(line).map(|m| (m.start(), line[m.start()..m.end()].to_string())).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Config;

    #[test]
    fn test_find_match_offsets() {
        let config = Config::parse_args(vec![std::ffi::OsString::from("rgrep"), std::ffi::OsString::from("foo")]).unwrap();
        let matcher = Matcher::new(&config, vec!["foo".to_string()]).unwrap();
        
        let offsets = matcher.find_match_offsets("foo bar foo");
        assert_eq!(offsets, vec![(0, "foo".to_string()), (8, "foo".to_string())]);
    }

    #[test]
    fn test_bre_to_ere() {
        assert_eq!(bre_to_ere("foo\\?"), "foo?");
        assert_eq!(bre_to_ere("foo?"), "foo\\?");
        assert_eq!(bre_to_ere("\\(a\\|b\\)"), "(a|b)");
        assert_eq!(bre_to_ere("(a|b)"), "\\(a\\|b\\)");
        assert_eq!(bre_to_ere("^foo$"), "^foo$");
    }

    fn get_base_config(pattern: &str) -> Config {
        Config {
            extended_regexp: false,
            basic_regexp: false,
            no_ignore_case: false,
            line_regexp: false,
            dereference_recursive: false,
            directories: crate::cli::DirectoriesAction::Read,
            devices: crate::cli::DevicesAction::Read,
            no_messages: false,
            group_separator: "--".to_string(),
            no_group_separator: false,
            exclude_from: None,
            binary_files: None,
            binary: false,
            initial_tab: false,
            label: "(standard input)".to_string(),
            line_buffered: false,
            pattern: Some(pattern.to_string()),
            files: vec![],
            ignore_case: false,
            invert_match: false,
            line_number: false,
            count: false,
            word_regexp: false,
            recursive: false,
            color: "never".to_string(),
            files_with_matches: false,
            files_without_match: false,
            quiet: false,
            no_filename: false,
            with_filename: false,
            only_matching: false,
            after_context: 0,
            before_context: 0,
            context: 0,
            max_count: None,
            mmap: false,
            regexp: vec![],
            file_patterns: vec![],
            fixed_strings: false,
            perl_regexp: false,
            exclude: vec![],
            include: vec![],
            exclude_dir: vec![],
            byte_offset: false,
            null: false,
            null_data: false,
            text: false,
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
        config.ignore_case = true;
        let matcher = Matcher::new(&config, vec!["HELLO".to_string()]).unwrap();
        assert!(matcher.is_match("hello world"));
    }

    #[test]
    fn test_is_match_invert() {
        let mut config = get_base_config("hello");
        config.invert_match = true;
        let matcher = Matcher::new(&config, vec!["hello".to_string()]).unwrap();
        assert!(!matcher.is_match("hello world"));
        assert!(matcher.is_match("bye world"));
    }
    
    #[test]
    fn test_is_match_word_regexp() {
        let mut config = get_base_config("hello");
        config.word_regexp = true;
        let matcher = Matcher::new(&config, vec!["hello".to_string()]).unwrap();
        assert!(matcher.is_match("say hello to him"));
        assert!(!matcher.is_match("say helloworld to him"));
    }
    
    #[test]
    fn test_is_match_line_regexp() {
        let mut config = get_base_config("hello");
        config.line_regexp = true;
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
        config.regexp = vec!["world".to_string()];
        let matcher = Matcher::new(&config, vec!["hello".to_string(), "world".to_string()]).unwrap();
        assert!(matcher.is_match("say hello to him"));
        assert!(matcher.is_match("what a beautiful world"));
        assert!(!matcher.is_match("something else entirely"));
    }

    #[test]
    fn test_fixed_strings() {
        let mut config = get_base_config("h.*o");
        config.fixed_strings = true;
        let matcher = Matcher::new(&config, vec!["h.*o".to_string()]).unwrap();
        assert!(!matcher.is_match("say hello to him"));
        assert!(matcher.is_match("literal h.*o string"));
    }
}
