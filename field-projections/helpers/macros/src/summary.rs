use proc_macro2::{
    Literal,
    Span,
    TokenStream,
    TokenTree,
};
use quote::{
    ToTokens,
    quote,
};
use syn::{
    AttrStyle,
    Attribute,
    Error,
    Expr,
    ExprLit,
    File,
    Item,
    ItemMod,
    Lit,
    parse::{
        End,
        Parse,
    },
    parse_quote,
    parse2,
    spanned::Spanned,
    visit_mut::{
        VisitMut,
        visit_item_fn_mut,
        visit_item_mut,
        visit_trait_item_fn_mut,
    },
};

use crate::utils::rustfmt;

#[derive(Debug)]
pub(crate) enum SummaryArgs {
    Toplevel,
    Skip,
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(skip);
}

impl Parse for SummaryArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(End) {
            Ok(Self::Toplevel)
        } else if lh.peek(kw::skip) {
            let _: kw::skip = input.parse()?;
            Ok(Self::Skip)
        } else {
            Err(lh.error())
        }
    }
}

pub(crate) fn expand(mut module: ItemMod) -> TokenStream {
    let ItemMod {
        ref mut attrs,
        content: Some((_, ref items)),
        ..
    } = module
    else {
        let err = Error::new(
            module.span(),
            "must be an inline module, use `#![summary]` inside of files",
        )
        .into_compile_error();
        return quote!(#module #err);
    };
    let Some(anchor) = find_summary_anchor(attrs) else {
        return module.into_token_stream();
    };
    let mut full_summary: Vec<String> = vec![];
    let mut errors = vec![];
    for item in items {
        if let Some(last) = full_summary.last()
            && !last.is_empty()
        {
            full_summary.push(String::new());
        }
        let item_summary = generate_summary_lines(item, &mut errors);
        full_summary.extend(item_summary);
    }
    attrs.splice(
        anchor..=anchor,
        full_summary
            .into_iter()
            .map(|content| parse_quote!(#![doc = #content])),
    );
    let errors = errors.into_iter().map(|err| err.into_compile_error());
    quote!(#module #(#errors)*)
}

pub(crate) fn expand_raw(file: File) -> TokenStream {
    let items = &file.items;
    let mut summary: String = String::with_capacity(items.len() * 10 * 50);
    let mut errors = vec![];
    for item in items {
        if !summary.is_empty() {
            if !summary.ends_with('\n') {
                summary.push('\n');
                summary.push('\n');
            } else if !summary.ends_with("\n\n") {
                summary.push('\n');
            }
        }
        summary.push_str(&generate_summary(item, &mut errors));
    }
    if !errors.is_empty() {
        let errors = errors.into_iter().map(|err| err.into_compile_error());
        return quote!((#(#errors),*));
    }
    TokenTree::Literal(Literal::string(&summary)).into()
}

fn find_summary_anchor<'a>(attrs: &[Attribute]) -> Option<usize> {
    let mut res = None;
    for (idx, attr) in attrs.iter().enumerate() {
        if matches!(attr.style, AttrStyle::Inner(_))
            && let Ok(meta) = attr.meta.require_name_value()
            && meta.path.is_ident("doc")
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(contents), ..
            }) = &meta.value
            && contents.value() == " ====== SUMMARY ANCHOR ======"
        {
            if res.is_some() {
                todo!("report error that there are two anchors!");
            }
            res = Some(idx);
        }
    }
    res
}

fn generate_summary(item: &Item, errors: &mut Vec<Error>) -> String {
    let mut stripped = item.clone();
    StripDistractions { skip: false }.visit_item_mut(&mut stripped);
    let content = stripped.into_token_stream().to_string();
    match rustfmt(&content) {
        Ok(Ok(formatted)) => formatted,
        Ok(Err((code, stderr))) => {
            errors.push(Error::new(
                def_span(item),
                format!(
                    "While formatting the summary for this item, \
                    encountered an error: code={code:?}, stderr:\n{stderr}"
                ),
            ));
            String::new()
        }
        Err(err) => {
            errors.push(Error::new(
                def_span(item),
                format!(
                    "While formatting the summary of this item, \
                    encountered an error: {err}"
                ),
            ));
            String::new()
        }
    }
}

fn generate_summary_lines(item: &Item, errors: &mut Vec<Error>) -> Vec<String> {
    generate_summary(item, errors)
        .split('\n')
        .map(|s| format!(" {}", s.trim_end()))
        .collect()
}

struct StripDistractions {
    skip: bool,
}

impl StripDistractions {
    fn is_skip(attr: &Attribute) -> bool {
        if let Ok(syn::MetaList {
            path: syn::Path { segments, .. },
            tokens,
            ..
        }) = attr.meta.require_list()
            && segments.iter().all(|seg| seg.arguments.is_empty())
            && segments
                .iter()
                .rev()
                .zip(["summary", "macros"])
                .all(|(seg, expected)| seg.ident == expected)
            && let Ok(SummaryArgs::Skip) = parse2(tokens.clone())
        {
            true
        } else {
            false
        }
    }
}

impl VisitMut for StripDistractions {
    fn visit_attributes_mut(&mut self, i: &mut Vec<Attribute>) {
        if i.iter().any(Self::is_skip) {
            self.skip = true;
        }
        i.clear();
    }

    fn visit_item_mut(&mut self, i: &mut Item) {
        if self.skip {
            *i = Item::Verbatim(quote!());
        } else {
            let attrs = match i {
                Item::Const(item_const) => &item_const.attrs,
                Item::Enum(item_enum) => &item_enum.attrs,
                Item::ExternCrate(item_extern_crate) => {
                    &item_extern_crate.attrs
                }
                Item::Fn(item_fn) => &item_fn.attrs,
                Item::ForeignMod(item_foreign_mod) => &item_foreign_mod.attrs,
                Item::Impl(item_impl) => &item_impl.attrs,
                Item::Macro(item_macro) => &item_macro.attrs,
                Item::Mod(item_mod) => &item_mod.attrs,
                Item::Static(item_static) => &item_static.attrs,
                Item::Struct(item_struct) => &item_struct.attrs,
                Item::Trait(item_trait) => &item_trait.attrs,
                Item::TraitAlias(item_trait_alias) => &item_trait_alias.attrs,
                Item::Type(item_type) => &item_type.attrs,
                Item::Union(item_union) => &item_union.attrs,
                Item::Use(item_use) => &item_use.attrs,
                _ => todo!("unsupported item: {i:?}"),
            };
            if attrs.iter().any(Self::is_skip) {
                *i = Item::Verbatim(quote!());
            } else {
                visit_item_mut(self, i);
            }
            self.skip = false;
        }
    }

    fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
        visit_item_fn_mut(self, i);
        i.block.stmts.clear();
    }

    fn visit_trait_item_fn_mut(&mut self, i: &mut syn::TraitItemFn) {
        visit_trait_item_fn_mut(self, i);
        if let Some(block) = &mut i.default {
            block.stmts.clear();
        }
    }
}

fn def_span(item: &Item) -> Span {
    match item {
        Item::Const(syn::ItemConst {
            vis, const_token, ident, ..
        }) => quote!(#vis #const_token #ident).span(),
        Item::Enum(syn::ItemEnum {
            vis, enum_token, ident, ..
        }) => quote!(#vis #enum_token #ident).span(),
        Item::ExternCrate(syn::ItemExternCrate {
            vis,
            extern_token,
            crate_token,
            ident,
            ..
        }) => quote!(#vis #extern_token, #crate_token #ident).span(),
        Item::Fn(syn::ItemFn { vis, sig, .. }) => quote!(#vis #sig).span(),
        Item::ForeignMod(syn::ItemForeignMod { unsafety, abi, .. }) => {
            quote!(#unsafety #abi).span()
        }
        Item::Impl(syn::ItemImpl {
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty,
            ..
        }) => match trait_ {
            None => quote!(
                #defaultness
                #unsafety
                #impl_token
                #generics
                #self_ty
            )
            .span(),
            Some((not, path, for_)) => quote!(
                #defaultness
                #unsafety
                #impl_token
                #generics
                #not
                #path
                #for_
                #self_ty
            )
            .span(),
        },
        Item::Macro(syn::ItemMacro { ident, .. }) => quote!(#ident).span(),
        Item::Mod(syn::ItemMod {
            vis,
            unsafety,
            mod_token,
            ident,
            ..
        }) => quote!(#vis #unsafety #mod_token #ident).span(),
        Item::Static(syn::ItemStatic {
            vis,
            static_token,
            mutability,
            ident,
            ..
        }) => quote!(#vis #static_token #mutability #ident).span(),
        Item::Struct(syn::ItemStruct {
            vis, struct_token, ident, ..
        }) => quote!(#vis #struct_token #ident).span(),
        Item::Trait(syn::ItemTrait {
            vis,
            unsafety,
            auto_token,
            trait_token,
            ident,
            ..
        }) => quote!(#vis #unsafety #auto_token #trait_token #ident).span(),
        Item::TraitAlias(syn::ItemTraitAlias {
            vis,
            trait_token,
            ident,
            ..
        }) => quote!(#vis #trait_token #ident).span(),
        Item::Type(syn::ItemType {
            vis, type_token, ident, ..
        }) => quote!(#vis #type_token #ident).span(),
        Item::Union(syn::ItemUnion {
            vis, union_token, ident, ..
        }) => quote!(#vis #union_token #ident).span(),
        Item::Use(item_use) => item_use.use_token.span,
        Item::Verbatim(ts) => ts.span(),
        _ => {
            eprintln!(
                "WARN: novel `syn::Item` variant not handled \
                in `{}:{} fn def_span`",
                file!(),
                line!(),
            );
            item.span()
        }
    }
}
