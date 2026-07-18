# macros

Procedural macros for [ratwrap](https://github.com/mnmun/ratwrap).

## Overview

This crate provides the [`name!`] macro, used to construct nested enum variant expressions with automatic path prefixing.

## `name!`

Constructs deeply nested enum variant expressions from a flat chain of identifiers.

### Motivation

In `ratwrap`, widget names often form a hierarchy:

```rust
enum Name  { Main(Main),               ... }
enum Main  { Itself,     Page(Page),   ... }
enum Page  { Itself,     Table(Table), ... }
enum Table { Itself,     ...               }
```

Without the macro, constructing a deep path is verbose and error‑prone:

```rust
Name::Main(Main::Page(Page::Table(Table::Itself)))
```

With `name!`:

```rust
name!(Name, Main, Page, Table, Itself)
```

### Syntax

```ignore
name!( [ <path>:: ] <ident>, <ident>, ..., <ident> [ { <fields> } ])
```

| Part           | Description                                                                    |
|----------------|--------------------------------------------------------------------------------|
| `<path>`       | Optional base path to the module with enums automatically added to all idents) |
| `<ident>`      | Enum identifier (at least two)                                                 |
| `{ <fields> }` | Optional struct‑initializer appended to the innermost variant                |

### Examples

```rust
name!(A::B, X, Y, Z { n: 42 }) // -> A::B::X(A::X::Y(A::Y::Z { n: 42 }))
name!(Name, Itself)            // -> Name::Itself
```

## License

[MIT](LICENSE)
