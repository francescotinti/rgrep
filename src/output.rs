// rgrep — GNU grep ported to Rust
// Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
//
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Gemini Antigravity (Google)
//
// https://github.com/francescotinti/rgrep

#[derive(Debug, Clone)]
pub struct GrepColors {
    pub ms: String,
    pub mc: String,
    pub fn_color: String,
    pub ln: String,
    pub bn: String,
    pub se: String,
    pub sl: String,
    pub cx: String,
}

impl Default for GrepColors {
    fn default() -> Self {
        Self {
            ms: "01;31".to_string(),
            mc: "01;31".to_string(),
            fn_color: "35".to_string(),
            ln: "32".to_string(),
            bn: "32".to_string(),
            se: "36".to_string(),
            sl: "".to_string(),
            cx: "".to_string(),
        }
    }
}

impl GrepColors {
    pub fn from_env() -> Self {
        let mut colors = Self::default();
        if let Ok(val) = std::env::var("GREP_COLORS") {
            for part in val.split(':') {
                if let Some((k, v)) = part.split_once('=') {
                    match k {
                        "ms" => colors.ms = v.to_string(),
                        "mc" => colors.mc = v.to_string(),
                        "fn" => colors.fn_color = v.to_string(),
                        "ln" => colors.ln = v.to_string(),
                        "bn" => colors.bn = v.to_string(),
                        "se" => colors.se = v.to_string(),
                        "sl" => colors.sl = v.to_string(),
                        "cx" => colors.cx = v.to_string(),
                        _ => {} // Ignore unknown
                    }
                }
            }
        }
        colors
    }
}

pub fn ansi_wrap(text: &str, code: &str) -> String {
    if code.is_empty() {
        text.to_string()
    } else {
        format!("\x1b[{}m\x1b[K{}\x1b[m\x1b[K", code, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_colors_parser() {
        unsafe { std::env::set_var("GREP_COLORS", "ms=1;33:fn=34:unknown=99"); }
        let colors = GrepColors::from_env();
        assert_eq!(colors.ms, "1;33");
        assert_eq!(colors.fn_color, "34");
        assert_eq!(colors.ln, "32"); // default
        unsafe { std::env::remove_var("GREP_COLORS"); }
    }

    #[test]
    fn test_ansi_wrap() {
        assert_eq!(ansi_wrap("foo", "01;31"), "\x1b[01;31m\x1b[Kfoo\x1b[m\x1b[K");
        assert_eq!(ansi_wrap("bar", ""), "bar");
    }
}
