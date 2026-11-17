use crate::internal_prelude::*;

use super::{FormatOptions, trivia::*};

pub(super) struct Layout<'a> {
    pub tokens: Vec<FormatToken<'a>>,
    pub lines: Vec<Line>,
    pub end: Gap<'a>,
}

pub(super) struct Line {
    pub start: usize,
    pub end: usize,
    pub indent: usize,
    pub comment_indent: usize,
    pub follows_prefix: bool,
}

struct LayoutBuilder<'a> {
    layout: Layout<'a>,
    cursor: usize,
    options: FormatOptions,
}

pub(super) fn build<'a>(
    source: &'a str,
    file: &cst::File<'a>,
    options: FormatOptions,
) -> Layout<'a> {
    let (tokens, end) = tokenize(source);
    let mut builder = LayoutBuilder {
        layout: Layout {
            tokens,
            lines: Vec::new(),
            end,
        },
        cursor: 0,
        options,
    };
    let cst::File { items } = file;
    for item in items {
        builder.item(item);
    }
    assert_eq!(builder.cursor, builder.layout.tokens.len());
    builder.layout
}

impl LayoutBuilder<'_> {
    fn item(&mut self, item: &cst::FileItem<'_>) {
        match item {
            cst::FileItem::Start(_) => {
                self.line(2, 0);
            }
            cst::FileItem::Struct(def) => {
                let cst::Struct {
                    prefix,
                    name: _,
                    fieldset,
                } = def;
                self.prefix(*prefix);
                let header = self.line(2, 0);
                self.layout.lines[header].follows_prefix = prefix.is_some();
                self.fieldset(fieldset, 0);
            }
            cst::FileItem::Enum(def) => {
                let cst::Enum {
                    prefix,
                    name: _,
                    variants,
                } = def;
                self.prefix(*prefix);
                let header = self.line(3, 0);
                self.layout.lines[header].follows_prefix = prefix.is_some();
                for variant in variants {
                    let cst::EnumVariant { name: _, fieldset } = variant;
                    self.line(1, 1);
                    self.fieldset(fieldset, 1);
                }
                self.close(0);
            }
            cst::FileItem::ManualNonTerminal(def) => {
                let cst::ManualNonTerminal {
                    name,
                    override_,
                    lifetimes,
                    variants,
                } = def;
                self.shorthand(name.raw.raw(), *override_, 2);
                self.line(3 + override_.is_some() as usize * 2 + lifetimes.len(), 0);
                for variant in variants {
                    let cst::ManualNonTerminalVariant {
                        function: _,
                        inputs,
                    } = variant;
                    self.line(1, 1);
                    if !inputs.is_empty() {
                        self.tuple(inputs, 1);
                    }
                }
                self.close(0);
            }
            cst::FileItem::GeneratedTerminal(def) => {
                let cst::GeneratedTerminal {
                    prefix,
                    name: _,
                    global_lifetimes,
                    variants,
                } = def;
                self.prefix(*prefix);
                let header = self.line(3 + global_lifetimes.len(), 0);
                self.layout.lines[header].follows_prefix = prefix.is_some();
                self.terminal_variants(variants);
                self.close(0);
            }
            cst::FileItem::ManualTerminal(def) => {
                let cst::ManualTerminal {
                    name,
                    override_,
                    global_lifetimes,
                    variants,
                } = def;
                self.shorthand(name.raw.raw(), *override_, 3);
                self.line(
                    4 + override_.is_some() as usize * 2 + global_lifetimes.len(),
                    0,
                );
                self.terminal_variants(variants);
                self.close(0);
            }
            cst::FileItem::Bridge(path) => {
                self.line(path.len() * 2, 0);
            }
        }
    }

    fn prefix(&mut self, prefix: Option<token::RustStringLiteral<'_>>) {
        if prefix.is_some() {
            self.line(1, 0);
        }
    }

    fn fieldset(&mut self, fieldset: &cst::Fieldset<'_>, indent: usize) {
        match fieldset {
            cst::Fieldset::Empty => {}
            cst::Fieldset::Named(fieldset) => {
                let cst::NamedFieldset { fields } = fieldset;
                self.extend_line(1);
                for field in fields {
                    let cst::NamedField { name: _, symbol: _ } = field;
                    self.line(3, indent + 1);
                }
                self.close(indent);
            }
            cst::Fieldset::Tuple(fieldset) => {
                let cst::TupleFieldset { fields } = fieldset;
                self.tuple(fields, indent);
            }
        }
    }

    fn tuple(&mut self, fields: &[cst::TupleField<'_>], indent: usize) {
        let field_tokens = fields.iter().map(tuple_field_len).sum::<usize>();
        let end = self.cursor + field_tokens + 2;
        let line = self.layout.lines.last().expect("Tuple must have a header");
        let used = fields
            .iter()
            .all(|field| matches!(field, cst::TupleField::Used(_)));
        // A comment inside the parentheses keeps the tuple multiline, including
        // when every field is used. Comments above the whole tuple do not.
        let has_comments = self.layout.tokens[self.cursor + 1..end]
            .iter()
            .any(|token| !token.leading.comments.is_empty());
        let width = line.indent * 4
            + line_text(&self.layout.tokens[line.start..end])
                .chars()
                .count();
        if used && !has_comments && width <= self.options.max_width {
            self.extend_line(field_tokens + 2);
        } else {
            self.extend_line(1);
            for field in fields {
                self.line(tuple_field_len(field), indent + 1);
            }
            self.close(indent);
        }
    }

    fn terminal_variants(&mut self, variants: &[cst::TerminalEnumVariant<'_>]) {
        for variant in variants {
            let cst::TerminalEnumVariant {
                name,
                override_,
                lifetimes,
            } = variant;
            let name = name.raw.raw().strip_prefix('$').unwrap();
            self.shorthand(name, *override_, 1);
            self.line(1 + override_.is_some() as usize * 2 + lifetimes.len(), 1);
        }
    }

    fn shorthand(&mut self, name: &str, override_: Option<token::AtUpperName<'_>>, offset: usize) {
        if override_.is_some_and(|override_| override_.raw.raw().strip_prefix('@') == Some(name)) {
            self.layout.tokens[self.cursor + offset].omitted = true;
            self.layout.tokens[self.cursor + offset + 1].omitted = true;
        }
    }

    fn line(&mut self, count: usize, indent: usize) -> usize {
        let index = self.layout.lines.len();
        self.layout.lines.push(Line {
            start: self.cursor,
            end: self.cursor + count,
            indent,
            comment_indent: indent,
            follows_prefix: false,
        });
        self.cursor += count;
        index
    }

    fn extend_line(&mut self, count: usize) {
        self.cursor += count;
        self.layout
            .lines
            .last_mut()
            .expect("Extended line must exist")
            .end = self.cursor;
    }

    fn close(&mut self, indent: usize) {
        let index = self.line(1, indent);
        self.layout.lines[index].comment_indent = indent + 1;
    }
}

fn tuple_field_len(field: &cst::TupleField<'_>) -> usize {
    match field {
        cst::TupleField::Used(_) => 1,
        cst::TupleField::Skipped(_) => 3,
    }
}

pub(super) fn line_text(tokens: &[FormatToken<'_>]) -> String {
    let mut out = String::new();
    let mut previous = None;
    for token in tokens.iter().filter(|token| !token.omitted) {
        if previous.is_some_and(|previous| needs_space(previous, token.raw)) {
            out.push(' ');
        }
        out.push_str(token.raw);
        previous = Some(token.raw);
    }
    out
}

fn needs_space(previous: &str, next: &str) -> bool {
    !matches!(previous, "(" | "::") && !matches!(next, "(" | ")" | ":" | "::")
}
