use proc_macro2::{
    Span,
    TokenStream,
};
use quote::{
    ToTokens,
    quote,
};
use syn::{
    AttrStyle,
    Attribute,
    Block,
    Error,
    Expr,
    ExprLit,
    ItemFn,
    Lit,
    parse::Nothing,
    parse_quote,
};

use crate::utils::rustfmt;

pub(crate) fn expand(args: Nothing, mut input: ItemFn) -> TokenStream {
    let _ = args;
    let mut error = None;
    let Some(anchor) = find_desugared_anchor(&input.attrs) else {
        return input.into_token_stream();
    };
    let desugared = desugared(&input.block, &mut error);
    input.attrs[anchor] = parse_quote!(#[doc = #desugared]);
    quote!(#error #input)
}

fn find_desugared_anchor<'a>(attrs: &[Attribute]) -> Option<usize> {
    let mut res = None;
    for (idx, attr) in attrs.iter().enumerate() {
        if matches!(attr.style, AttrStyle::Outer)
            && let Ok(meta) = attr.meta.require_name_value()
            && meta.path.is_ident("doc")
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(contents), ..
            }) = &meta.value
            && contents.value() == " ==== DESUGARED ===="
        {
            if res.is_some() {
                todo!("report error that there are two anchors!");
            }
            res = Some(idx);
        }
    }
    res
}

fn desugared(block: &Block, error: &mut Option<TokenStream>) -> String {
    let code = format!("fn main() {}", block.to_token_stream());
    let formatted = match rustfmt(&code) {
        Ok(Ok(formatted)) => formatted,
        Ok(Err((code, stderr))) => {
            *error = Some(
                Error::new(
                    Span::call_site(),
                    format!(
                        "While formatting the summary for this item, \
                        encountered an error: code={code:?}, stderr:\n{stderr}"
                    ),
                )
                .into_compile_error(),
            );
            String::from("fn main() {\n    ERROR\n}")
        }
        Err(err) => {
            *error = Some(
                Error::new(
                    Span::call_site(),
                    format!(
                        "While formatting the summary of this item, \
                        encountered an error: {err}"
                    ),
                )
                .into_compile_error(),
            );
            String::from("fn main() {\n    ERROR\n}")
        }
    };
    formatted
        .strip_prefix("fn main() {\n")
        .expect("we added this")
        .strip_suffix("\n}")
        .expect("we added this")
        .lines()
        .map(|line| {
            // removing only 3 spaces, because rustdoc expects an additional
            // space in each line
            line.strip_prefix("   ")
                .expect("rustfmt should've added indentation")
        })
        .intersperse("\n")
        .collect::<String>()
}
