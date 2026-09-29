#![feature(iter_intersperse)]

use std::fs;

use proc_macro::{
    Span,
    TokenStream,
};
use syn::parse_macro_input;

use crate::summary::SummaryArgs;

mod adt_reflect;
mod desugared;
mod ensure_full_reexport;
mod sealed;
mod summary;
mod utils;

#[proc_macro]
pub fn raw_summary(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as syn::parse::Nothing);
    let file = Span::call_site().file();
    let contents = fs::read_to_string(file)
        .expect("source file for macro invocation to exist");
    summary::expand_raw(syn::parse_str(&contents).unwrap()).into()
}

#[proc_macro_attribute]
pub fn summary(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SummaryArgs);
    match args {
        SummaryArgs::Toplevel => {
            summary::expand(parse_macro_input!(input)).into()
        }
        SummaryArgs::Skip => input,
    }
}

#[proc_macro]
pub fn adt_reflect(input: TokenStream) -> TokenStream {
    adt_reflect::expand(parse_macro_input!(input)).into()
}

/// Adds the body of a function to its documentation.
#[proc_macro_attribute]
pub fn desugared(args: TokenStream, input: TokenStream) -> TokenStream {
    desugared::expand(parse_macro_input!(args), parse_macro_input!(input))
        .into()
}

#[proc_macro]
pub fn ensure_full_reexport(input: TokenStream) -> TokenStream {
    ensure_full_reexport::expand(parse_macro_input!(input)).into()
}

/// Make a trait or impl sealed.
#[proc_macro_attribute]
pub fn sealed(args: TokenStream, input: TokenStream) -> TokenStream {
    sealed::expand(parse_macro_input!(args), parse_macro_input!(input)).into()
}
