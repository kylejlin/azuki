use crate::internal_prelude::*;

pub(super) struct FormatToken<'a> {
    pub raw: &'a str,
    pub leading: Gap<'a>,
    pub omitted: bool,
}

pub(super) struct Gap<'a> {
    pub comments: Vec<Comment<'a>>,
    /// Whether the whitespace after the last comment (or the whole gap, if
    /// there are no comments) contains an empty line.
    pub blank_before: bool,
}

pub(super) struct Comment<'a> {
    pub raw: &'a str,
    pub blank_before: bool,
    pub trailing: bool,
}

/// Recovers comments and whitespace from the gaps between the real lexer's
/// tokens. In particular, comment-looking text inside Rust strings is never
/// inspected as trivia. The caller must have successfully parsed `source`.
pub(super) fn tokenize(source: &str) -> (Vec<FormatToken<'_>>, Gap<'_>) {
    let file_source = FileSource::new(source);
    let mut previous_end = 0;
    let tokens = lex(source)
        .map(|token| {
            let token = token.expect("Successfully parsed source must lex");
            let fragment = token_fragment(token);
            let range = file_source
                .substr_range(fragment)
                .expect("Lexed token must belong to its source");
            let leading = Gap::new(&source[previous_end..range.start]);
            previous_end = range.end;
            FormatToken {
                raw: fragment.raw(),
                leading,
                omitted: false,
            }
        })
        .collect();
    (tokens, Gap::new(&source[previous_end..]))
}

impl<'a> Gap<'a> {
    fn new(mut raw: &'a str) -> Self {
        let mut comments = Vec::new();
        while let Some(start) = raw.find("//") {
            let whitespace = &raw[..start];
            let remainder = &raw[start..];
            let end = remainder.find('\n').unwrap_or(remainder.len());
            comments.push(Comment {
                raw: remainder[..end].trim_end_matches('\r'),
                blank_before: has_blank_line(whitespace),
                trailing: comments.is_empty() && !whitespace.contains('\n'),
            });
            raw = &remainder[end..];
        }
        Self {
            comments,
            blank_before: has_blank_line(raw),
        }
    }
}

fn has_blank_line(raw: &str) -> bool {
    raw.bytes().filter(|&b| b == b'\n').take(2).count() == 2
}

fn token_fragment(token: Token<'_>) -> SourceFragment<'_> {
    match token {
        Token::StartKw(token) => token.raw,
        Token::StructKw(token) => token.raw,
        Token::EnumKw(token) => token.raw,
        Token::ManualKw(token) => token.raw,
        Token::TerminalKw(token) => token.raw,
        Token::BridgeKw(token) => token.raw,
        Token::CrateKw(token) => token.raw,
        Token::UpperSelfKw(token) => token.raw,
        Token::Underscore(token) => token.raw,
        Token::LParen(token) => token.raw,
        Token::RParen(token) => token.raw,
        Token::LSquare(token) => token.raw,
        Token::RSquare(token) => token.raw,
        Token::LCurly(token) => token.raw,
        Token::RCurly(token) => token.raw,
        Token::Comma(token) => token.raw,
        Token::DoubleColon(token) => token.raw,
        Token::Colon(token) => token.raw,
        Token::Equal(token) => token.raw,
        Token::Octothorpe(token) => token.raw,
        Token::UpperName(token) => token.raw,
        Token::LowerName(token) => token.raw,
        Token::DollarUpperName(token) => token.raw,
        Token::AtUpperName(token) => token.raw,
        Token::AtLowerName(token) => token.raw,
        Token::SingleQuoteLowerName(token) => token.raw,
        Token::RustStringLiteral(token) => token.raw,
    }
}
