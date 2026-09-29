use proc_macro2::{
    Span,
    TokenStream,
};
use quote::{
    ToTokens,
    format_ident,
    quote,
};
use syn::{
    Error,
    Ident,
    Item,
    ItemImpl,
    ItemTrait,
    parse::Nothing,
    parse_quote,
    spanned::Spanned,
};

pub(crate) fn expand(_args: Nothing, input: Item) -> TokenStream {
    match input {
        Item::Trait(item_trait) => handle_trait(item_trait),
        Item::Impl(item_impl) => handle_impl(item_impl),
        item => {
            let mut res = Error::new(
                item.span(),
                "`#[sealed]` only supports traits and impls",
            )
            .into_compile_error();
            item.to_tokens(&mut res);
            res
        }
    }
}

fn sealed_mod_name(trait_name: &Ident) -> Ident {
    format_ident!("__{trait_name}_sealed")
}

fn handle_trait(mut item_trait: ItemTrait) -> TokenStream {
    let sealed_mod_name = sealed_mod_name(&item_trait.ident);

    let mut sealed_trait = item_trait.clone();
    sealed_trait.items.clear();
    sealed_trait.ident = Ident::new("Sealed", item_trait.ident.span());
    let (_, ty_gen, _) = item_trait.generics.split_for_impl();
    item_trait
        .supertraits
        .push(parse_quote!(#sealed_mod_name::Sealed #ty_gen ));

    quote! {
        mod #sealed_mod_name {
            use super::*;
            #[allow(unnameable_types)]
            #sealed_trait
        }

        #item_trait
    }
}

fn handle_impl(item_impl: ItemImpl) -> TokenStream {
    let err = |span: Span, msg: &str| -> TokenStream {
        let mut res = Error::new(span, msg).into_compile_error();
        item_impl.to_tokens(&mut res);
        res
    };

    let mut sealed_impl = item_impl.clone();
    sealed_impl.items.clear();
    let Some((None, path, _)) = sealed_impl.trait_.as_mut() else {
        return err(
            item_impl.impl_token.span(),
            "`#[sealed]` impl must impl a trait",
        );
    };

    if let Some(leading) = path.leading_colon {
        return err(leading.span(), "`#[sealed]` impl expects a local trait");
    }
    if path.segments.len() > 1 {
        return err(path.span(), "`#[sealed]` impl expects a local trait");
    }

    let mut last = path
        .segments
        .pop()
        .expect("paths should have at least one segment")
        .into_value();
    let sealed_mod_name = sealed_mod_name(&last.ident);
    path.segments.push(sealed_mod_name.into());
    last.ident = Ident::new("Sealed", last.ident.span());
    path.segments.push(last);

    quote! {
        #item_impl
        #sealed_impl
    }
}
