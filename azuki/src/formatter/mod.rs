use crate::internal_prelude::*;

mod layout;
mod render;
mod trivia;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    /// Preferred line width, including indentation. Headers, comments, Rust
    /// strings, and individual fields can exceed this width.
    pub max_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { max_width: 80 }
    }
}

/// Formats an Azuki file, returning the original text on any lexer or parser
/// error. Does not perform preflight checks or construct a parsing table.
///
/// Uses four spaces per indent and LF line endings outside Rust strings.
/// Rust string tokens are preserved verbatim. Comments embedded in a header
/// or field move above it; ordinary end-of-line comments stay on that line.
/// Redundant manual and terminal-variant overrides use their shorthand.
pub fn format(source: &str, options: FormatOptions) -> String {
    let Ok(file) = cst::parse(FileSource::new(source)) else {
        return source.to_owned();
    };

    let layout = layout::build(source, &file, options);
    render::render(layout)
}
