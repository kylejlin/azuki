use azuki::{
    cst,
    lexer::{LexError, UnexpectedCharOrEndOfInput},
    machine::Machine,
    preflight::{PreflightCheckError, preflight_check},
    source_text::SourceFragment,
    table::{Table, TableConflictDiagnostic},
    takeoff::TakeoffCertificate,
};
use serde::Serialize;

use crate::analysis::{Source, Span, fragment};

#[derive(Serialize)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
    pub related: Vec<Related>,
}

#[derive(Serialize)]
pub struct Related {
    pub span: Span,
    pub message: String,
}

pub fn syntax(source: &Source<'_>, error: Result<cst::ParseError<'_>, LexError>) -> Diagnostic {
    let (span, message) = match error {
        Ok(cst::ParseError {
            unexpected_token: Some(token),
        }) => {
            let raw = fragment(token);
            (
                source.span(raw),
                format!("Unexpected token {:?}.", raw.raw()),
            )
        }
        Ok(cst::ParseError {
            unexpected_token: None,
        }) => (
            source.bytes(source.raw.len(), source.raw.len()),
            "Unexpected end of input while parsing.".to_owned(),
        ),
        Err(LexError {
            character,
            position,
        }) => {
            let start = position.byte_index;
            let (end, message) = match character {
                UnexpectedCharOrEndOfInput::EndOfInput => (start, "Unexpected end of input while lexing; check for an unterminated Rust injection string.".to_owned()),
                UnexpectedCharOrEndOfInput::Char(c) => (start + c.len_utf8(), format!("Unexpected character {c:?}.")),
            };
            (source.bytes(start, end), message)
        }
    };
    Diagnostic {
        span,
        message,
        related: Vec::new(),
    }
}

pub fn check<'a>(source: &Source<'a>, file: &'a cst::File<'a>, out: &mut Vec<Diagnostic>) {
    let preflight = match preflight_check(source.raw, file) {
        Ok(preflight) => preflight,
        Err(error) => {
            out.push(preflight_error(source, error));
            return;
        }
    };
    let anchor = source.span(preflight.start_symbol.raw);
    let takeoff = TakeoffCertificate::new(preflight);
    let machine = Machine::new(&takeoff);
    if let Err(error) = Table::new(&machine, &takeoff) {
        let message = format!(
            "{:?}",
            TableConflictDiagnostic {
                error,
                machine: &machine,
                takeoff: &takeoff,
                file_name: "grammar",
            }
        );
        out.push(Diagnostic {
            span: anchor,
            message,
            related: Vec::new(),
        });
    }
}

fn preflight_error(source: &Source<'_>, error: PreflightCheckError<'_>) -> Diagnostic {
    let eoi = source.bytes(source.raw.len(), source.raw.len());
    let single = |message: String, fragment: SourceFragment<'_>| Diagnostic {
        span: source.span(fragment),
        message,
        related: Vec::new(),
    };
    let duplicate =
        |message: String, first: SourceFragment<'_>, second: SourceFragment<'_>| Diagnostic {
            span: source.span(second),
            message,
            related: vec![Related {
                span: source.span(first),
                message: "First occurrence is here.".to_owned(),
            }],
        };
    let missing = |message: &str| Diagnostic {
        span: eoi,
        message: message.to_owned(),
        related: Vec::new(),
    };
    match error {
        PreflightCheckError::NoStartItem => {
            missing("Missing `start` item; exactly one is required.")
        }
        PreflightCheckError::MultipleStartItems(first, second) => {
            duplicate("Multiple `start` items.".to_owned(), first.raw, second.raw)
        }
        PreflightCheckError::NoTerminalItem => missing(
            "Missing terminal item; exactly one `terminal` or `manual terminal` item is required.",
        ),
        PreflightCheckError::MultipleTerminalItems(first, second) => {
            duplicate("Multiple terminal items.".to_owned(), first.raw, second.raw)
        }
        PreflightCheckError::NoBridgeItem => {
            missing("Missing `bridge` item; exactly one is required.")
        }
        PreflightCheckError::MultipleBridgeItems(first, second) => {
            duplicate("Multiple `bridge` items.".to_owned(), first, second)
        }
        PreflightCheckError::ItemNameClash(first, second) => duplicate(
            format!("Duplicate item name `{}`.", second.raw),
            first.raw,
            second.raw,
        ),
        PreflightCheckError::EnumVariantNameClash(first, second) => duplicate(
            format!("Duplicate enum variant name `{}`.", second.raw),
            first.raw,
            second.raw,
        ),
        PreflightCheckError::TerminalVariantNameClash(first, second) => duplicate(
            format!("Duplicate terminal variant name `{}`.", second.raw),
            first.raw,
            second.raw,
        ),
        PreflightCheckError::VariantSymbolSequenceClash(first, second) => duplicate(
            format!("Variant `{second}` has the same grammar symbol sequence as `{first}`."),
            first,
            second,
        ),
        PreflightCheckError::FieldNameClash(first, second) => duplicate(
            format!("Duplicate field name `{}`.", second.raw),
            first.raw,
            second.raw,
        ),
        PreflightCheckError::UndefinedTerminal(name) => {
            single(format!("Undefined terminal `{}`.", name.raw), name.raw)
        }
        PreflightCheckError::UndefinedNonTerminal(name) => {
            single(format!("Undefined non-terminal `{}`.", name.raw), name.raw)
        }
        PreflightCheckError::LifetimeNameClash(first, second) => duplicate(
            format!("Duplicate lifetime `{}`.", second.raw),
            first.raw,
            second.raw,
        ),
        PreflightCheckError::UndefinedLifetime(name) => single(
            format!(
                "Undefined lifetime `{}`. Declare it in the terminal item's global lifetime list.",
                name.raw
            ),
            name.raw,
        ),
    }
}
