// rgrep — GNU grep ported to Rust
// Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
//
// AI-assisted port:
//   Architect: Claude Opus 4.7 (1M context, Anthropic)
//   Implementer: Gemini Antigravity (Google)
//
// https://github.com/francescotinti/rgrep

use std::io;

#[derive(thiserror::Error, Debug)]
pub enum RgrepError {
    #[error("invalid regex: {0}")]
    InvalidRegex(String),

    #[error("{path}: {source}")]
    Io { path: String, source: io::Error },

    #[error("perl-regexp not available in this build")]
    PcreUnavailable,

    #[error("invalid glob pattern: {0}")]
    InvalidGlob(String),

    // Marker used when an error message has already been emitted to stderr;
    // the caller exits non-zero without printing again. Matches the previous
    // `Err("".into())` idiom carried over from the original Box<dyn Error> code.
    #[error("")]
    Silent,
}

impl From<io::Error> for RgrepError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: String::new(),
            source,
        }
    }
}

impl From<globset::Error> for RgrepError {
    fn from(e: globset::Error) -> Self {
        Self::InvalidGlob(e.to_string())
    }
}
