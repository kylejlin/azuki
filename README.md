# Azuki

[![crates.io](https://img.shields.io/crates/v/azuki.svg)](https://crates.io/crates/azuki)

Azuki is a minimalist parser generator for Rust.
It is the successor to [Kiki](https://github.com/kylejlin/kiki).

## TODO: Write proper docs

I don't have time to hand-write proper docs right now.
I tried vibe-writing docs with Codex, but all attempts so far resulted in slop.

So, instead, I wrote a [migration guide](#migrating-from-kiki).
This guide assumes familiarity with Kiki.
If you are unfamiliar with Kiki, you can read its [docs](https://github.com/kylejlin/kiki) to learn more.

## Migrating from Kiki

This section assumes you are already familiar with Kiki.
Azuki differs from Kiki in the following ways:

- External references are relative to a bridge root.
- You can use externally defined non-terminal types and semantic actions.
- Your AST can have lifetimes.
- There is no auto-`Box`-ing.
- You can use an externally defined terminal enum.
- You can use `Self` to refer to the enclosing non-terminal.
- Outer attributes are deprecated in favor of code injection.

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

```
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

#### Shorthand

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

pub fn snoc<T>(mut v: OneOrMoreNumbers, x: NumberLiteral) -> OneOrMoreNumbers {
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
pub type OneOrMoreNumbers = Vec<NumberLiteral>;
pub type Comma = ();

pub fn one(x: NumberLiteral) -> Vec<NumberLiteral> {
    vec![x]
}

pub fn snoc<T>(mut v: OneOrMoreNumbers, _: Comma, x: NumberLiteral) -> OneOrMoreNumbers {
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
pub type OneOrMoreNumbers = Vec<NumberLiteral>;
pub type Comma = ();

pub fn one(x: NumberLiteral) -> Vec<NumberLiteral> {
    vec![x]
}

// Notice we removed the comma parameter.
pub fn snoc<T>(mut v: OneOrMoreNumbers, x: NumberLiteral) -> OneOrMoreNumbers {
    v.push(x);
    v
}
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

```
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

The lifetimes of `struct` and `enum`s are inferred from their (non-`_`) fields.

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
