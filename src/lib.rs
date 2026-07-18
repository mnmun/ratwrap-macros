//! # Macros for ratwrap
//!
//! This crate provides procedural macros used by `ratwrap`.
//!
//! ## Available macros
//!
//! - [`name!`]: Constructs nested enum variant expressions with automatic path prefixing
//!
//! ## Usage
//!
//! The macros are re‑exported from the main crate and can be used as:
//!
//! ```ignore
//! use ratwrap::name;
//! ```
//!
//! See individual macro docs for examples and detailed syntax.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, Path, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct Input {
    base: Path,
    rest: Vec<Ident>,
    fields: Option<proc_macro2::TokenStream>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let base: Path = input.parse()?;

        let mut rest = Vec::new();

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            rest.push(input.parse()?);
        }

        let fields = if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            Some(content.parse::<proc_macro2::TokenStream>()?)
        } else {
            None
        };

        Ok(Input { base, rest, fields })
    }
}

fn split_path(path: &Path) -> (Option<proc_macro2::TokenStream>, Ident) {
    let mut segments: Vec<_> = path.segments.iter().collect();

    let last = segments
        .pop()
        .expect("Path must contain at least one segment")
        .ident
        .clone();

    if segments.is_empty() {
        (None, last)
    } else {
        let prefix = quote! { #(#segments)::* };
        (Some(prefix), last)
    }
}

/// # Name macro
///
/// A procedural macro that constructs nested enum variant expressions from a
/// base path and a chain of identifiers.
///
/// ## Motivation
///
/// In `ratwrap`, it is convenient to use enums that form a hierarchy as widget
/// names in the tree:
///
/// ```ignore
/// enum Name  { Main(Main),               ... }
/// enum Main  { Itself,     Page(Page),   ... }
/// enum Page  { Itself,     Table(Table), ... }
/// enum Table { Itself,     ...               }
/// ```
///
/// Without the macro, constructing a deep path like `Name::Main ->
/// Main::Page -> Page::Table -> Table::Itself` is verbose and error‑prone.
/// The `name!` macro eliminates the repetition:
///
/// ```ignore
/// // Manual
/// Name::Main(Main::Page(Page::Table(Table::Itself)))
///
/// // Macro – equivalent
/// name!(Name, Main, Page, Table, Itself)
/// ```
///
/// ## Syntax
///
/// ```ignore
/// name!( [ <path>:: ] <ident>, <ident>, ..., <ident> [ { <fields> } ])
/// ```
///
/// ```plain
/// ┌──────────────┬───────────────────────────────────────────────────────┐
/// │ Part         │ Description                                           │
/// ├──────────────┼───────────────────────────────────────────────────────┤
/// │ <path>       │ Optional base path to the module with enums           │
/// │              │ (automatically added to all idents)                   │
/// ├──────────────┼───────────────────────────────────────────────────────┤
/// │ <ident>      │ Enum identifier (at least two)                        │
/// ├──────────────┼───────────────────────────────────────────────────────┤
/// │ { <fields> } │ Optional struct-initializer appended to the innermost │
/// │              │ variant                                               │
/// └──────────────┴───────────────────────────────────────────────────────┘
/// ```
///
/// ## Expansion
///
/// ```ignore
/// name!(A::B, X, Y, Z { n: 42 }) // -> A::B::X(A::X::Y(A::Y::Z { n: 42 }))
/// name!(Name, Itself) // -> Name::Itself
/// ```
///
/// The chain is built **right‑to‑left**: the last identifier is the innermost
/// variant, and each preceding identifier wraps the expression so far.
#[proc_macro]
pub fn name(input: TokenStream) -> TokenStream {
    let Input { base, rest, fields } = parse_macro_input!(input as Input);

    let (prefix, first) = split_path(&base);

    let mut segments = vec![first];
    segments.extend(rest);

    if segments.len() < 2 {
        return syn::Error::new_spanned(
            base,
            "macro requires at least two segments",
        )
        .to_compile_error()
        .into();
    }

    let mut iter = segments.into_iter().rev();

    let last = iter.next().unwrap();
    let mut prev = iter.next().unwrap();

    let mut expr = match (&prefix, &fields) {
        (Some(p), Some(f)) => quote! { #p::#prev::#last { #f } },
        (Some(p), None) => quote! { #p::#prev::#last },
        (None, Some(f)) => quote! { #prev::#last { #f } },
        (None, None) => quote! { #prev::#last },
    };

    for seg in iter {
        expr = match &prefix {
            Some(p) => quote! {
                #p::#seg::#prev(#expr)
            },
            None => quote! {
                #seg::#prev(#expr)
            },
        };

        prev = seg;
    }

    expr.into()
}
