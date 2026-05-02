use crate::cli::Config;

pub struct Matcher<'a> {
    config: &'a Config,
    pattern: String,
}

impl<'a> Matcher<'a> {
    pub fn new(config: &'a Config) -> Self {
        let pattern = if config.ignore_case {
            config.pattern.to_lowercase()
        } else {
            config.pattern.clone()
        };
        
        Self { config, pattern }
    }

    pub fn is_match(&self, line: &str) -> bool {
        let matches = if self.config.ignore_case {
            line.to_lowercase().contains(&self.pattern)
        } else {
            line.contains(&self.pattern)
        };

        if self.config.invert_match {
            !matches
        } else {
            matches
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Config;

    #[test]
    fn test_is_match_basic() {
        let config = Config {
            pattern: "hello".to_string(),
            files: vec![],
            ignore_case: false,
            invert_match: false,
            line_number: false,
            count: false,
            word_regexp: false,
        };
        let matcher = Matcher::new(&config);
        assert!(matcher.is_match("hello world"));
        assert!(!matcher.is_match("bye world"));
    }

    #[test]
    fn test_is_match_ignore_case() {
        let config = Config {
            pattern: "HELLO".to_string(),
            files: vec![],
            ignore_case: true,
            invert_match: false,
            line_number: false,
            count: false,
            word_regexp: false,
        };
        let matcher = Matcher::new(&config);
        assert!(matcher.is_match("hello world"));
    }

    #[test]
    fn test_is_match_invert() {
        let config = Config {
            pattern: "hello".to_string(),
            files: vec![],
            ignore_case: false,
            invert_match: true,
            line_number: false,
            count: false,
            word_regexp: false,
        };
        let matcher = Matcher::new(&config);
        assert!(!matcher.is_match("hello world"));
        assert!(matcher.is_match("bye world"));
    }
}
