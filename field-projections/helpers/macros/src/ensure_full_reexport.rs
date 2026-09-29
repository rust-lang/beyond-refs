use proc_macro2::TokenStream;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    Error,
    ItemMod,
    Result,
    Visibility,
    parse::{
        End,
        Parse,
        ParseStream,
    },
    spanned::Spanned,
};

pub(crate) struct Input(Vec<ItemMod>);

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut res = vec![];
        while !input.peek(End) {
            let mod_: ItemMod = input.parse()?;
            if mod_.vis != Visibility::Inherited {
                return Err(Error::new_spanned(
                    mod_.vis,
                    "expected no visibility",
                ));
            }
            if let Some((brace, _)) = &mod_.content {
                return Err(Error::new(brace.span.span(), "expected no body"));
            }
            if mod_.semi.is_none() {
                return Err(Error::new(
                    mod_.ident.span(),
                    "expected semicolon after this",
                ));
            }
            res.push(mod_);
        }
        Ok(Self(res))
    }
}

pub(crate) fn expand(Input(mods): Input) -> TokenStream {
    let ensures = mods.iter().map(|mod_| {
        let span = mod_.span();
        let ident = &mod_.ident;
        quote_spanned! {span=>
            #[expect(
                unused_imports,
                reason = "ensure all items are explicitly exported and none \
                are forgotten"
            )]
            pub use self::#ident::*;
        }
    });
    quote! {
        #(#mods)*
        #(#ensures)*
    }
}
