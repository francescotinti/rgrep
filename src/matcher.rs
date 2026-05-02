use crate::cli::Config;
use regex::{Regex, RegexBuilder};

pub struct Matcher<'a> {
    config: &'a Config,
    re: Regex,
}

impl<'a> Matcher<'a> {
    pub fn new(config: &'a Config) -> Result<Self, regex::Error> {
        let mut pattern_str = config.pattern.clone();

        if config.word_regexp {
            pattern_str = format!(r"\b(?:{})\b", pattern_str);
        }

        let re = RegexBuilder::new(&pattern_str)
            .case_insensitive(config.ignore_case)
            .build()?;

        Ok(Self { config, re })
    }

    pub fn is_match(&self, line: &str) -> bool {
        let matches = self.re.is_match(line);

        if self.config.invert_match {
            !matches
        } else {
            matches
        }
    }

    pub fn highlight(&self, line: &str) -> String {
        if self.config.invert_match {
            return line.to_string();
        }
        
        self.re.replace_all(line, "\x1b[31;1m$0\x1b[0m").into_owned()
    }

    pub fn find_matches(&self, line: &str) -> Vec<String> {
        if self.config.invert_match {
            return vec![]; // -o doesn't usually make sense with -v, but we return empty for simplicity
        }
        self.re.find_iter(line).map(|m| m.as_str().to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Config;

    fn get_base_config(pattern: &str) -> Config {
        Config {
            pattern: pattern.to_string(),
            files: vec![],
            ignore_case: false,
            invert_match: false,
            line_number: false,
            count: false,
            word_regexp: false,
            recursive: false,
            color: false,
            files_with_matches: false,
            files_without_match: false,
            quiet: false,
            no_filename: false,
            with_filename: false,
            only_matching: false,
        }
    }

    #[test]
    fn test_is_match_basic() {
        let config = get_base_config("hello");
        let matcher = Matcher::new(&config).unwrap();
        assert!(matcher.is_match("hello world"));
        assert!(!matcher.is_match("bye world"));
    }

    #[test]
    fn test_is_match_ignore_case() {
        let mut config = get_base_config("HELLO");
        config.ignore_case = true;
        let matcher = Matcher::new(&config).unwrap();
        assert!(matcher.is_match("hello world"));
    }

    #[test]
    fn test_is_match_invert() {
        let mut config = get_base_config("hello");
        config.invert_match = true;
        let matcher = Matcher::new(&config).unwrap();
        assert!(!matcher.is_match("hello world"));
        assert!(matcher.is_match("bye world"));
    }
    
    #[test]
    fn test_is_match_word_regexp() {
        let mut config = get_base_config("hello");
        config.word_regexp = true;
        let matcher = Matcher::new(&config).unwrap();
        assert!(matcher.is_match("say hello to him"));
        assert!(!matcher.is_match("say helloworld to him"));
    }
    
    #[test]
    fn test_is_match_regex() {
        let config = get_base_config("h.*o");
        let matcher = Matcher::new(&config).unwrap();
        assert!(matcher.is_match("say hello to him"));
    }
}
