use azuki::{cst, lexer, source_text::FileSource, source_text::SourceFragment, token::Token};
use serde::Serialize;

use crate::{diagnostics, index};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub tokens: Vec<Highlight>,
    pub rust_regions: Vec<Span>,
    pub definitions: Vec<index::Definition>,
    pub occurrences: Vec<index::Occurrence>,
    pub unresolved_references: Vec<index::UnresolvedReference>,
    pub rename_available: bool,
    pub diagnostics: Vec<diagnostics::Diagnostic>,
}

#[derive(Clone, Copy, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize)]
pub struct Highlight {
    pub span: Span,
    pub kind: &'static str,
    pub declaration: bool,
}

pub struct Source<'a> {
    pub raw: &'a str,
    utf16_offsets: Vec<usize>,
}

impl<'a> Source<'a> {
    fn new(raw: &'a str) -> Self {
        let mut utf16_offsets = vec![0; raw.len() + 1];
        let mut offset = 0;
        for (byte, c) in raw.char_indices() {
            utf16_offsets[byte] = offset;
            offset += c.len_utf16();
        }
        utf16_offsets[raw.len()] = offset;
        Self { raw, utf16_offsets }
    }

    pub fn bytes(&self, start: usize, end: usize) -> Span {
        Span {
            start: self.utf16_offsets[start],
            end: self.utf16_offsets[end],
        }
    }

    pub fn span(&self, fragment: SourceFragment<'_>) -> Span {
        let range = FileSource::new(self.raw)
            .substr_range(fragment)
            .expect("Analysis fragments must belong to the source");
        self.bytes(range.start, range.end)
    }
}

pub fn analyze(raw: &str) -> Analysis {
    let source = Source::new(raw);
    let mut out = Analysis {
        tokens: Vec::new(),
        rust_regions: Vec::new(),
        definitions: Vec::new(),
        occurrences: Vec::new(),
        unresolved_references: Vec::new(),
        rename_available: false,
        diagnostics: Vec::new(),
    };
    let tokens = lex_recovering(raw);
    for &token in &tokens {
        let fragment = fragment(token);
        let kind = match token {
            Token::StartKw(_)
            | Token::StructKw(_)
            | Token::EnumKw(_)
            | Token::ManualKw(_)
            | Token::TerminalKw(_)
            | Token::BridgeKw(_)
            | Token::UpperSelfKw(_) => "keyword",
            Token::UpperName(_) | Token::AtUpperName(_) => "type",
            Token::LowerName(_) | Token::Underscore(_) => "variable",
            Token::DollarUpperName(_) => "enumMember",
            Token::AtLowerName(_) => "function",
            Token::SingleQuoteLowerName(_) => "typeParameter",
            Token::RustStringLiteral(_) => "string",
            Token::LParen(_)
            | Token::RParen(_)
            | Token::LSquare(_)
            | Token::RSquare(_)
            | Token::LCurly(_)
            | Token::RCurly(_)
            | Token::Comma(_)
            | Token::DoubleColon(_)
            | Token::Colon(_)
            | Token::Equal(_)
            | Token::Octothorpe(_) => "operator",
        };
        if let Token::RustStringLiteral(_) = token {
            let range = FileSource::new(raw).substr_range(fragment).unwrap();
            let opener = fragment.raw().find('"').unwrap() + 1;
            let closer = opener - 4;
            let body_start = range.start + opener;
            let body_end = range.end - closer;
            out.tokens.push(Highlight {
                span: source.bytes(range.start, body_start),
                kind: "string",
                declaration: false,
            });
            out.rust_regions.push(source.bytes(body_start, body_end));
            out.tokens.push(Highlight {
                span: source.bytes(body_end, range.end),
                kind: "string",
                declaration: false,
            });
        } else {
            out.tokens.push(Highlight {
                span: source.span(fragment),
                kind,
                declaration: false,
            });
        }
    }

    match cst::parse(FileSource::new(raw)) {
        Ok(file) => {
            out.rename_available = true;
            index::index(&source, &file, &mut out);
            diagnostics::check(&source, &file, &mut out.diagnostics);
        }
        Err(error) => {
            out.diagnostics.push(diagnostics::syntax(&source, error));
            // Index independently parseable items while the user is editing another item.
            let mut starts = Vec::new();
            let mut depth = 0usize;
            let mut previous = None;
            for &token in &tokens {
                if depth == 0
                    && is_item_keyword(token)
                    && !matches!(
                        (previous, token),
                        (Some(Token::ManualKw(_)), Token::TerminalKw(_))
                    )
                {
                    let start = match previous {
                        Some(Token::RustStringLiteral(prefix)) => prefix.raw,
                        _ => fragment(token),
                    };
                    starts.push(FileSource::new(raw).substr_range(start).unwrap().start);
                }
                match token {
                    Token::LCurly(_) | Token::LParen(_) => depth += 1,
                    Token::RCurly(_) | Token::RParen(_) => depth = depth.saturating_sub(1),
                    _ => {}
                }
                previous = Some(token);
            }
            starts.push(raw.len());
            let mut file = cst::File { items: Vec::new() };
            for pair in starts.windows(2) {
                if let Ok(mut parsed) = cst::parse(FileSource::new(&raw[pair[0]..pair[1]])) {
                    file.items.append(&mut parsed.items);
                }
            }
            index::index(&source, &file, &mut out);
        }
    }
    out
}

fn lex_recovering(source: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < source.len() {
        let mut next = source.len();
        for token in lexer::lex(&source[start..]) {
            match token {
                Ok(token) => out.push(token),
                Err(error) => {
                    let error_start = start + error.position.byte_index;
                    next = source[error_start..]
                        .chars()
                        .next()
                        .map_or(source.len(), |c| error_start + c.len_utf8());
                    break;
                }
            }
        }
        start = next;
    }
    out
}

fn is_item_keyword(token: Token<'_>) -> bool {
    matches!(
        token,
        Token::StartKw(_)
            | Token::StructKw(_)
            | Token::EnumKw(_)
            | Token::ManualKw(_)
            | Token::TerminalKw(_)
            | Token::BridgeKw(_)
    )
}

pub fn fragment(token: Token<'_>) -> SourceFragment<'_> {
    match token {
        Token::StartKw(t) => t.raw,
        Token::StructKw(t) => t.raw,
        Token::EnumKw(t) => t.raw,
        Token::ManualKw(t) => t.raw,
        Token::TerminalKw(t) => t.raw,
        Token::BridgeKw(t) => t.raw,
        Token::UpperSelfKw(t) => t.raw,
        Token::Underscore(t) => t.raw,
        Token::LParen(t) => t.raw,
        Token::RParen(t) => t.raw,
        Token::LSquare(t) => t.raw,
        Token::RSquare(t) => t.raw,
        Token::LCurly(t) => t.raw,
        Token::RCurly(t) => t.raw,
        Token::Comma(t) => t.raw,
        Token::DoubleColon(t) => t.raw,
        Token::Colon(t) => t.raw,
        Token::Equal(t) => t.raw,
        Token::Octothorpe(t) => t.raw,
        Token::UpperName(t) => t.raw,
        Token::LowerName(t) => t.raw,
        Token::DollarUpperName(t) => t.raw,
        Token::AtUpperName(t) => t.raw,
        Token::AtLowerName(t) => t.raw,
        Token::SingleQuoteLowerName(t) => t.raw,
        Token::RustStringLiteral(t) => t.raw,
    }
}
