use azuki::{cst, lexer::lex, source_text::FileSource, source_text::SourceFragment, token::Token};
use serde::Serialize;
use std::collections::HashSet;

use crate::analysis::{Analysis, Source, Span};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub name: String,
    pub kind: &'static str,
    pub span: Span,
    pub scope: String,
    pub preview: String,
    pub implicit_bridge: Option<String>,
    pub ambiguous: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Occurrence {
    pub span: Span,
    pub target: usize,
    pub is_self: bool,
}

#[derive(Serialize)]
pub struct UnresolvedReference {
    pub name: String,
    pub scope: &'static str,
    pub span: Span,
}

struct Reference<'a> {
    name: &'a str,
    scope: &'static str,
    span: Span,
}

struct Indexer<'a, 'b> {
    source: &'b Source<'a>,
    out: &'b mut Analysis,
    references: Vec<Reference<'a>>,
    non_terminals: HashSet<usize>,
}

pub fn index(source: &Source<'_>, file: &cst::File<'_>, out: &mut Analysis) {
    let mut indexer = Indexer {
        source,
        out,
        references: Vec::new(),
        non_terminals: HashSet::new(),
    };
    for item in &file.items {
        indexer.item(item);
    }
    for reference in indexer.references {
        let targets: Vec<_> = indexer
            .out
            .definitions
            .iter()
            .enumerate()
            .filter(|(i, def)| {
                def.scope == reference.scope
                    && def.name == reference.name
                    && (reference.scope != "item" || indexer.non_terminals.contains(i))
            })
            .map(|(i, _)| i)
            .collect();
        if let [target] = targets.as_slice() {
            indexer.out.occurrences.push(Occurrence {
                span: reference.span,
                target: *target,
                is_self: false,
            });
        } else {
            indexer.out.unresolved_references.push(UnresolvedReference {
                name: reference.name.to_owned(),
                scope: reference.scope,
                span: reference.span,
            });
        }
    }
    for i in 0..indexer.out.definitions.len() {
        let def = &indexer.out.definitions[i];
        let ambiguous = indexer
            .out
            .definitions
            .iter()
            .enumerate()
            .any(|(j, other)| i != j && def.scope == other.scope && def.name == other.name);
        indexer.out.definitions[i].ambiguous = ambiguous;
    }
    for occurrence in &indexer.out.occurrences {
        if occurrence.is_self {
            continue;
        }
        if let Some(token) = indexer
            .out
            .tokens
            .iter_mut()
            .find(|token| token.span.start == occurrence.span.start)
        {
            token.kind = indexer.out.definitions[occurrence.target].kind;
        }
    }
}

impl<'a> Indexer<'a, '_> {
    fn item(&mut self, item: &cst::FileItem<'a>) {
        match item {
            cst::FileItem::Start(_, name) => self.reference(name.raw, "item"),
            cst::FileItem::Struct(cst::Struct {
                prefix: _,
                struct_kw: _,
                name,
                fieldset,
            }) => {
                let id = self.definition(
                    name.raw,
                    "struct",
                    "item".to_owned(),
                    None,
                    self.item_preview(name.raw, "struct"),
                );
                self.non_terminals.insert(id);
                self.fieldset(fieldset, id);
            }
            cst::FileItem::Enum(cst::Enum {
                prefix: _,
                enum_kw: _,
                name,
                lcurly: _,
                variants,
                rcurly: _,
            }) => {
                let id = self.definition(
                    name.raw,
                    "enum",
                    "item".to_owned(),
                    None,
                    self.item_preview(name.raw, "enum"),
                );
                self.non_terminals.insert(id);
                let scope = format!("variant:{id}");
                for cst::EnumVariant { name, fieldset } in variants {
                    self.definition(
                        name.raw,
                        "enumMember",
                        scope.clone(),
                        None,
                        self.item_preview(name.raw, ""),
                    );
                    self.fieldset(fieldset, id);
                }
            }
            cst::FileItem::ManualNonTerminal(cst::ManualNonTerminal {
                manual_kw: _,
                name,
                override_,
                lifetimes,
                lcurly: _,
                variants,
                rcurly: _,
            }) => {
                let implicit_bridge = override_.is_none().then(|| format!("@{}", name.raw));
                let id = self.definition(
                    name.raw,
                    "type",
                    "item".to_owned(),
                    implicit_bridge,
                    self.item_preview(name.raw, "manual"),
                );
                self.non_terminals.insert(id);
                self.lifetimes(lifetimes, false);
                for cst::ManualNonTerminalVariant {
                    function: _,
                    inputs,
                } in variants
                {
                    if let Some(cst::TupleFieldset {
                        lparen: _,
                        fields,
                        rparen: _,
                    }) = inputs
                    {
                        self.tuple_fields(fields, id);
                    }
                }
            }
            cst::FileItem::GeneratedTerminal(cst::GeneratedTerminal {
                prefix: _,
                terminal_kw: _,
                name,
                global_lifetimes,
                lcurly: _,
                variants,
                rcurly: _,
            }) => {
                self.definition(
                    name.raw,
                    "enum",
                    "item".to_owned(),
                    None,
                    self.item_preview(name.raw, "terminal"),
                );
                self.lifetimes(global_lifetimes, true);
                self.terminal_variants(variants);
            }
            cst::FileItem::ManualTerminal(cst::ManualTerminal {
                manual_kw: _,
                terminal_kw: _,
                name,
                override_,
                global_lifetimes,
                lcurly: _,
                variants,
                rcurly: _,
            }) => {
                let implicit_bridge = override_.is_none().then(|| format!("@{}", name.raw));
                self.definition(
                    name.raw,
                    "enum",
                    "item".to_owned(),
                    implicit_bridge,
                    self.item_preview(name.raw, "manual terminal"),
                );
                self.lifetimes(global_lifetimes, true);
                self.terminal_variants(variants);
            }
            cst::FileItem::Bridge(_, _) => {}
        }
    }

    fn terminal_variants(&mut self, variants: &[cst::TerminalEnumVariant<'a>]) {
        for cst::TerminalEnumVariant {
            name,
            override_,
            lifetimes,
        } in variants
        {
            let implicit_bridge = override_
                .is_none()
                .then(|| format!("@{}", &name.raw.raw()[1..]));
            let last = lifetimes
                .last()
                .map(|t| t.raw)
                .or_else(|| override_.as_ref().map(|override_| override_.name.raw))
                .unwrap_or(name.raw);
            let start = FileSource::new(self.source.raw)
                .substr_range(name.raw)
                .unwrap()
                .start;
            let end = FileSource::new(self.source.raw)
                .substr_range(last)
                .unwrap()
                .end;
            let preview = self.source.raw[start..end].to_owned();
            self.definition(
                name.raw,
                "enumMember",
                "terminal".to_owned(),
                implicit_bridge,
                preview,
            );
            self.lifetimes(lifetimes, false);
        }
    }

    fn lifetimes(
        &mut self,
        lifetimes: &[azuki::token::SingleQuoteLowerName<'a>],
        declarations: bool,
    ) {
        for lifetime in lifetimes {
            if declarations {
                self.definition(
                    lifetime.raw,
                    "typeParameter",
                    "lifetime".to_owned(),
                    None,
                    format!("terminal lifetime {}", lifetime.raw),
                );
            } else {
                self.reference(lifetime.raw, "lifetime");
            }
        }
    }

    fn fieldset(&mut self, fieldset: &cst::Fieldset<'a>, owner: usize) {
        match fieldset {
            cst::Fieldset::Empty => {}
            cst::Fieldset::Named(cst::NamedFieldset {
                lcurly: _,
                fields,
                rcurly: _,
            }) => {
                let scope = format!(
                    "field:{}",
                    self.source
                        .span(match fields[0].name {
                            cst::UnderscoreOrLowerName::Lower(t) => t.raw,
                            cst::UnderscoreOrLowerName::Underscore(t) => t.raw,
                        })
                        .start
                );
                for cst::NamedField {
                    name,
                    colon: _,
                    symbol,
                } in fields
                {
                    if let cst::UnderscoreOrLowerName::Lower(name) = name {
                        let preview = format!("{}: {}", name.raw, symbol_name(symbol));
                        self.definition(name.raw, "property", scope.clone(), None, preview);
                    }
                    self.symbol(symbol, owner);
                }
            }
            cst::Fieldset::Tuple(cst::TupleFieldset {
                lparen: _,
                fields,
                rparen: _,
            }) => self.tuple_fields(fields, owner),
        }
    }

    fn tuple_fields(&mut self, fields: &[cst::TupleField<'a>], owner: usize) {
        for field in fields {
            match field {
                cst::TupleField::Used(symbol) | cst::TupleField::Skipped(_, _, symbol) => {
                    self.symbol(symbol, owner)
                }
            }
        }
    }

    fn symbol(&mut self, symbol: &cst::GrammarSymbolOrSelf<'a>, owner: usize) {
        match symbol {
            cst::GrammarSymbolOrSelf::Upper(t) => self.reference(t.raw, "item"),
            cst::GrammarSymbolOrSelf::DollarUpper(t) => self.reference(t.raw, "terminal"),
            cst::GrammarSymbolOrSelf::Self_(t) => {
                self.out.occurrences.push(Occurrence {
                    span: self.source.span(t.raw),
                    target: owner,
                    is_self: true,
                });
            }
        }
    }

    fn definition(
        &mut self,
        fragment: SourceFragment<'a>,
        kind: &'static str,
        scope: String,
        implicit_bridge: Option<String>,
        preview: String,
    ) -> usize {
        let id = self.out.definitions.len();
        let span = self.source.span(fragment);
        self.out.definitions.push(Definition {
            name: fragment.raw().to_owned(),
            kind,
            span,
            scope,
            preview,
            implicit_bridge,
            ambiguous: false,
        });
        self.out.occurrences.push(Occurrence {
            span,
            target: id,
            is_self: false,
        });
        if let Some(token) = self
            .out
            .tokens
            .iter_mut()
            .find(|token| token.span.start == span.start && token.span.end == span.end)
        {
            token.kind = kind;
            token.declaration = true;
        }
        id
    }

    fn reference(&mut self, fragment: SourceFragment<'a>, scope: &'static str) {
        self.references.push(Reference {
            name: fragment.raw(),
            scope,
            span: self.source.span(fragment),
        });
    }

    fn item_preview(&self, name: SourceFragment<'a>, keyword: &str) -> String {
        let range = FileSource::new(self.source.raw).substr_range(name).unwrap();
        let mut end = range.end;
        let mut depth = 0usize;
        for token in lex(&self.source.raw[range.end..]) {
            let Ok(token) = token else { break };
            if depth == 0
                && matches!(
                    token,
                    Token::StartKw(_)
                        | Token::StructKw(_)
                        | Token::EnumKw(_)
                        | Token::ManualKw(_)
                        | Token::TerminalKw(_)
                        | Token::BridgeKw(_)
                        | Token::RustStringLiteral(_)
                        | Token::UpperName(_)
                        | Token::DollarUpperName(_)
                )
            {
                break;
            }
            match token {
                Token::LCurly(_) | Token::LParen(_) => depth += 1,
                Token::RCurly(_) | Token::RParen(_) if depth == 0 => break,
                Token::RCurly(_) | Token::RParen(_) => depth -= 1,
                _ => {}
            }
            end = FileSource::new(self.source.raw)
                .substr_range(crate::analysis::fragment(token))
                .unwrap()
                .end;
            if depth == 0 && matches!(token, Token::RCurly(_) | Token::RParen(_)) {
                break;
            }
        }
        let body = &self.source.raw[range.start..end];
        if keyword.is_empty() {
            body.to_owned()
        } else {
            format!("{keyword} {body}")
        }
    }
}

fn symbol_name<'a>(symbol: &cst::GrammarSymbolOrSelf<'a>) -> &'a str {
    match symbol {
        cst::GrammarSymbolOrSelf::Upper(t) => t.raw.raw(),
        cst::GrammarSymbolOrSelf::DollarUpper(t) => t.raw.raw(),
        cst::GrammarSymbolOrSelf::Self_(t) => t.raw.raw(),
    }
}
