# Azuki

[![crates.io](https://img.shields.io/crates/v/azuki.svg)](https://crates.io/crates/azuki)

Azuki is a minimalist parser generator for Rust.
It is the successor to [Kiki](https://github.com/kylejlin/kiki).

## VS Code extension

[Azuki VS Code](azuki-vscode/README.md) provides highlighting, definition navigation, rename, hover, and diagnostics for `.azuki` files. Its source, build scripts, and packaging instructions are in `azuki-vscode/`. The [Rust analysis helper](azuki_ide/README.md) is a separate crate.

## TODO: Write proper docs

I don't have time to hand-write proper docs right now.
I tried vibe-writing docs with Codex, but all attempts so far resulted in slop.

So, instead, I wrote a [migration guide](#migrating-from-kiki).
This guide assumes familiarity with Kiki.
If you are unfamiliar with Kiki, you can read its [docs](https://github.com/kylejlin/kiki) to learn more.

## Migrating from Kiki

This section assumes you are already familiar with Kiki.
Azuki is similar to Kiki, but it differs in several ways.
I divide these into _key changes_ and _new features_.
You must understand the key changes to port your Kiki code to Azuki.
The new features are not essential, but they may greatly improve your quality of life.

Key changes:

- [#1.](#the-bridge) External references are relative to a bridge root.
- [#2.](#external-non-terminals) You can use externally defined non-terminal types and semantic actions.
- [#3.](#no-auto-box-ing) There is no auto-`Box`-ing.
- [#4.](#code-injection) Outer attributes are no longer supported. Use code injection instead.
- [#5.](#non-terminal-namespace) Non-terminals are namespaced under `non_terminals`.

New features:

- [#6.](#lifetimes) Your AST can have lifetimes.
- [#7.](#external-terminal-enum) You can use an externally defined terminal enum.
- [#8.](#self) You can use `Self` to refer to the enclosing non-terminal.

### The bridge

In Kiki, you wrote

```kiki
terminal Token {
    $NumberLiteral: crate::lexer::token::NumberLiteral
    $LParen: ()
    $RParen: ()
    $InfixOp: crate::lexer::token::InfixOp
}
```

Then, in `src/lexer/token.rs`, you wrote

```rs
pub struct NumberLiteral {
    // ...
}

pub enum InfixOp {
    // ...
}
```

In Azuki, you write

```azuki
bridge crate::azuki_bridge

terminal Token {
    $NumberLiteral = @NumberLiteral
    $LParen = @Unit
    $RParen = @Unit
    $InfixOp = @InfixOp
}
```

Then, in `src/azuki_bridge.rs`, you write

```rs
pub struct NumberLiteral {
    // ...
}

pub type Unit = ();

pub enum InfixOp {
    // ...
}
```

The bridge does not need to be named `crate::azuki_bridge`.
You can make any crate-relative path the bridge.
The path must begin with the `crate` keyword.
For example, these are all legal `bridge` statements:

- `bridge crate`
- `bridge crate::cst::azuki_bridge`
- `bridge crate::foo::bar::baz::lorem::ipsum::dolor`

#### Terminal enum variant shorthand

You may shorten `$Foo = @Foo` to `$Foo`.

For example, here are two semantically equivalent code snippets.

Verbose way:

```azuki
terminal Token {
    $NumberLiteral = @NumberLiteral
    $LParen = @Unit
    $RParen = @Unit
    $InfixOp = @InfixOp
}
```

Concise way:

```azuki
terminal Token {
    // Notice the elision of the `= @NumberLiteral`.
    $NumberLiteral
    $LParen = @Unit
    $RParen = @Unit
    // Notice the elision of the `= @InfixOp`.
    $InfixOp
}
```

### External non-terminals

You can now write

```azuki
manual OneOrMoreNumbers = @OneOrMoreNumbers {
    @one($NumberLiteral)
    @snoc(OneOrMoreNumbers $NumberLiteral)
}
```

Then, in your bridge file:

```rs
pub type OneOrMoreNumbers = Vec<NumberLiteral>;

pub fn one(x: NumberLiteral) -> Vec<NumberLiteral> {
    vec![x]
}

pub fn snoc(mut v: OneOrMoreNumbers, x: NumberLiteral) -> OneOrMoreNumbers {
    v.push(x);
    v
}
```

#### Shorthand

You may shorten `manual Foo = @Foo {...}` to `manual Foo {...}`.

For example, you could rewrite the above example as

```azuki
// Notice the elision of the ` = @OneOrMoreNumbers`.
manual OneOrMoreNumbers {
    @one($NumberLiteral)
    @snoc(OneOrMoreNumbers $NumberLiteral)
}
```

#### Skipping semantic action arguments

By default, all arguments are passed to the specified semantic action.
For example, consider the `@snoc` case of the following example:

```azuki
manual OneOrMoreCommaSeparatedNumbers = @OneOrMoreCommaSeparatedNumbers {
    @one($NumberLiteral)
    @snoc(OneOrMoreCommaSeparatedNumbers $Comma $NumberLiteral)
}
```

You might write a bridge like this:

```rs
pub type OneOrMoreCommaSeparatedNumbers = Vec<NumberLiteral>;
pub type Comma = ();

pub fn one(x: NumberLiteral) -> Vec<NumberLiteral> {
    vec![x]
}

pub fn snoc(mut v: OneOrMoreCommaSeparatedNumbers, _: Comma, x: NumberLiteral) -> OneOrMoreCommaSeparatedNumbers {
    v.push(x);
    v
}
```

But notice we never use the comma argument.
You can instead write:

```azuki
manual OneOrMoreCommaSeparatedNumbers = @OneOrMoreCommaSeparatedNumbers {
    @one($NumberLiteral)
    @snoc(
        OneOrMoreCommaSeparatedNumbers
        // Notice the added `_: `.
        _: $Comma
        $NumberLiteral
    )
}
```

Then

```rs
pub type OneOrMoreCommaSeparatedNumbers = Vec<NumberLiteral>;
pub type Comma = ();

pub fn one(x: NumberLiteral) -> Vec<NumberLiteral> {
    vec![x]
}

// Notice we removed the comma parameter.
pub fn snoc(mut v: OneOrMoreCommaSeparatedNumbers, x: NumberLiteral) -> OneOrMoreCommaSeparatedNumbers {
    v.push(x);
    v
}
```

### No auto-`Box`-ing

In Kiki,

```kiki
enum Term {
    Var(Ident)
    Abs($Backslash Ident Term)
    App(
        _: $LParen
        Term
        Term
        _: $RParen
    )
}
```

generates

```rs
pub enum Term {
    Var(Box<Ident>),
    Abs(crate::...::Backslash, Box<Ident>, Box<Term>),
    App(Box<Term>, Box<Term>),
}
```

Observe how every non-terminal is boxed.

In contrast, Azuki generates

```rs
pub enum Term {
    Var(Ident),
    Abs(crate::...::Backslash, Ident, Term),
    App(Term, Term),
}
// Notice the lack of `Box`.
```

This obviously triggers a compilation error, since `Term` has infinite size.

To solve this, you can manually box:

```azuki
enum Term {
    Var(Ident)
    Abs($Backslash Ident BoxedTerm)
    App(_: $LParen BoxedTerm BoxedTerm _: $RParen)
}

manual BoxedTerm {
    @box_new(Term)
}
```

With the generated `Term` type imported into your bridge:

```rs
pub type BoxedTerm = Box<Term>;

pub fn box_new(term: Term) -> BoxedTerm {
    Box::new(term)
}
```

This achieves behavior similar to before.
(...but not quite identical--`Ident` is not boxed. However, if you want to box `Ident`, just repeat the same process.)

### Code injection

In Kiki, you wrote

```kiki
#[derive(Debug)]
struct Number {
    value: $NumberLiteral
}
```

In Azuki, you write

```azuki
rust"#[derive(Debug)]"
struct Number {
    value: $NumberLiteral
}
```

The contents of the `rust"..."` string are inserted verbatim before the generated definition.
You can put a Rust string before a `struct`, `enum`, or `terminal` item.

#### Multiple attributes

Put all the attributes in a single string.
For example:

```azuki
rust"
#[derive(Debug, Clone, Copy)]
#[repr(C)]
"
struct Number {
    value: $NumberLiteral
}
```

Multiple strings are not allowed.
For example, this is illegal:

```azuki
// Illegal! A single `struct` cannot have multiple Rust strings.
rust"#[derive(Debug)]"
rust"#[repr(C)]"
struct Number {
    value: $NumberLiteral
}
```

#### Strings containing quotes

If the injected code contains quotes, you can use `rust#"..."#`.
For example:

```azuki
rust#"
#[doc = "A number."]
#[derive(Debug)]
"#
struct Number {
    value: $NumberLiteral
}
```

You can add more `#`s if needed, just like Rust's raw strings.
The number of `#`s on each side must match.

### Non-terminal namespace

Suppose you had the following `src/parser.kiki`:

```kiki
start Foo

enum Foo {
    // ...
}

struct Bar {
    // ...
}

struct Qux {
    // ...
}

// ...

terminal Token {
    // ...
}
```

In `src/lib.rs`, you might write:

```rs
mod parser;

pub use parser::{Foo, Bar, Token, parse};
```

In Azuki, non-terminals are namespaced under a `non_terminals` submodule.

So, you must update `src/lib.rs` to:

```rs
mod parser;

pub use parser::{non_terminals::{Foo, Bar}, Token, parse};
```

### Lifetimes

Your terminal datatype (and by extension, the rest of the AST) can have lifetimes.

```azuki
terminal Token 'a {
    $StringLiteral 'a
    $InfixOp
}
```

Bridge:

```rs
pub type StringLiteral<'a> = &'a str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp {
    Plus,
    Minus,
}
```

If a `manual` non-terminal has lifetimes, you must explicitly annotate them.
For example:

```azuki
// Notice the `'a` annotation
manual StringSequence 'a {
    @one($StringLiteral)
    @snoc(StringSequence $StringLiteral)
}
```

Bridge:

```rs
pub type StringSequence<'a> = Vec<StringLiteral<'a>>;

pub fn one<'a>(x: StringLiteral<'a>) -> StringSequence<'a> {
    vec![x]
}

pub fn snoc<'a>(mut v: StringSequence<'a>, x: StringLiteral<'a>) -> StringSequence<'a> {
    v.push(x);
    v
}
```

#### Multiple lifetimes

Separate the lifetimes with spaces (no commas needed).
For example:

```azuki
terminal Token 'a 'b 'c {
    // ...
}
```

#### Required annotations

You must annotate lifetimes for:

1. The terminal enum
2. Each terminal enum variant
3. Each `manual` non-terminal

The set of names appearing in (2) and (3) must be a non-strict subset of the set of names appearing in (1).

For example, this is illegal:

```azuki
terminal Token 'a 'b 'c {
    $Foo 'a 'c
    $Bar 'b

    // Illegal! `'x` is undefined.
    $Baz 'x
}
```

Similarly, this is also illegal:

```azuki
terminal Token 'a 'b 'c {
    // ...
}

// Illegal! `'x` is undefined.
manual Lorem 'x {
    // ...
}
```

#### Lifetime inference

The lifetimes of `struct`s and `enum`s are inferred from their (non-`_`) fields.

For example:

```azuki
terminal Token 'a 'b 'c {
    $Foo 'a
    $Bar 'b
    $Qux 'c
}

enum Lorem {
    Ipsum($Foo)
    Dolor(
        _: $Bar
        $Qux
    )
}
// We infer that Lorem is polymorphic over `'a 'c`.
```

### External terminal enum

You can now write

```azuki
manual terminal Token = @MyCustomToken {
    $NumberLiteral
    $LParen = @Unit
    $RParen = @Unit
    $InfixOp
}
```

Then, in your bridge file:

```rs
pub enum MyCustomToken {
    NumberLiteral(NumberLiteral),
    LParen(Unit),
    RParen(Unit),
    InfixOp(InfixOp),
}

// ...
```

When you write `manual terminal` (instead of `terminal`), Azuki uses _your_ terminal enum (instead of generating one).
The provided enum must have exactly the variants listed in the grammar.
Each variant must have one unnamed field of the specified type.
For example, `LParen(Unit)` is legal, but `LParen` is not (even though `Unit = ()`).

Azuki does not enforce these requirements.
However, if you violate them, then `cargo check` will probably fail with a type error.

#### Shorthand

You may shorten `manual terminal Token = @Token {...}` to `manual terminal Token {...}`.

For example, the following code snippets are semantically equivalent.

Verbose:

```azuki
manual terminal Token = @Token {
    $NumberLiteral
    $LParen = @Unit
    $RParen = @Unit
    $InfixOp
}
```

With shorthand:

```azuki
// Notice the elision of the ` = @Token`.
manual terminal Token {
    $NumberLiteral
    $LParen = @Unit
    $RParen = @Unit
    $InfixOp
}
```

### `Self`

You can use `Self` to refer to the enclosing non-terminal.
For example, the following two examples are semantically equivalent.

Without `Self`:

```azuki
manual OneOrMoreCommaSeparatedNumbers {
    @one($NumberLiteral)
    @snoc(OneOrMoreCommaSeparatedNumbers $Comma $NumberLiteral)
}
```

With `Self`:

```azuki
manual OneOrMoreCommaSeparatedNumbers {
    @one($NumberLiteral)
    @snoc(Self $Comma $NumberLiteral)
}
```
