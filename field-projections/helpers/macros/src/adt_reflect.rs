use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::{
    Fields,
    Generics,
    Ident,
    Item,
    ItemEnum,
    parse::{
        Parse,
        ParseStream,
    },
    spanned::Spanned,
};

pub(crate) struct Input {
    items: Vec<Item>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}

pub(crate) fn expand(Input { items }: Input) -> TokenStream {
    let mut lang_limits_items = Vec::new();
    let mut variant_macro_arms = Vec::new();
    let mut field_macro_arms = Vec::new();
    let mut errors = TokenStream::new();

    for item in &items {
        match item {
            Item::Struct(struct_) => {
                generate_fields(
                    &struct_.ident,
                    None,
                    &struct_.generics,
                    &struct_.fields,
                    &mut lang_limits_items,
                    &mut field_macro_arms,
                    &mut errors,
                );
            }
            Item::Enum(enum_) => {
                generate_for_enum(
                    enum_,
                    &mut lang_limits_items,
                    &mut variant_macro_arms,
                    &mut field_macro_arms,
                    &mut errors,
                );
            }
            _ => {
                let err_msg = "unsupported item in `adt_reflect!`";
                errors.extend(quote_spanned!(item.span()=>
                    ::core::compile_error!(#err_msg);
                ));
            }
        }
    }

    quote! {
        #(#items)*

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        mod ___lang_limits {
            use super::*;
            #(#lang_limits_items)*
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! variant_of {
            #(#variant_macro_arms)*
            ($($fallback:tt)*) => {
                ::core::compile_error!(::core::concat!(
                    "unknown type or variant: `",
                    ::core::stringify!($($fallback)*),
                    "`",
                ))
            };
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! field_of {
            #(#field_macro_arms)*
            ($($fallback:tt)*) => {
                ::core::compile_error!(::core::concat!(
                    "unknown type, variant, or field: `",
                    ::core::stringify!($($fallback)*),
                    "`",
                ))
            };
        }

        #errors
    }
}

fn generics_to_fragments(generics: &Generics) -> (TokenStream, TokenStream) {
    if generics.params.is_empty() {
        return (quote!(), quote!());
    }

    let mut fragment_decl_gen = Vec::new();
    let mut fragment_ty_gen = Vec::new();

    for param in &generics.params {
        match param {
            syn::GenericParam::Type(ty) => {
                let ident = &ty.ident;
                fragment_decl_gen.push(quote! { $#ident:ty });
                fragment_ty_gen.push(ident);
            }
            syn::GenericParam::Lifetime(lt) => {
                let ident = &lt.lifetime.ident;
                fragment_decl_gen.push(quote! { $#ident:lifetime });
                fragment_ty_gen.push(ident);
            }
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                fragment_decl_gen.push(quote! { $#ident:expr });
                fragment_ty_gen.push(ident);
            }
        }
    }

    (
        quote! { $(::)? < #( #fragment_decl_gen ),* $(,)? > },
        quote! { ::< #( #fragment_ty_gen ),* > },
    )
}

fn generate_fields(
    adt_ident: &Ident,
    variant: Option<&Ident>,
    generics: &Generics,
    fields: &Fields,
    lang_limits: &mut Vec<TokenStream>,
    field_arms: &mut Vec<TokenStream>,
    errors: &mut TokenStream,
) {
    let decl_gen = {
        let mut tmp = generics.clone();
        tmp.where_clause = None;
        tmp
    };
    let (impl_gen, ty_gen, whr) = generics.split_for_impl();
    let (fragment_decl_gen, fragment_ty_gen) = generics_to_fragments(generics);

    let variant_ty = if let Some(variant) = variant {
        format_ident!("___{}__{}", adt_ident, variant)
    } else {
        format_ident!("___{}", adt_ident)
    };

    let phantom_ty = quote!(#adt_ident #ty_gen);

    let source_ty = if variant.is_some() {
        quote!(#variant_ty #ty_gen)
    } else {
        quote!(#adt_ident #ty_gen)
    };

    let Fields::Named(fields_named) = fields else {
        errors.extend(quote_spanned!(fields.span()=>
            ::core::compile_error!("only named fields are supported");
        ));
        return;
    };

    for field in &fields_named.named {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let field_ty = &field.ty;
        let meta_ident = format_ident!("{}__{}", variant_ty, field_ident);

        let field_access = match variant {
            Some(variant) => quote!(#variant . #field_ident),
            None => quote!(#field_ident),
        };

        lang_limits.push(quote! {
            pub struct #meta_ident #decl_gen
                (::core::marker::PhantomData::<#phantom_ty>)
            #whr;

            impl #impl_gen ::core::default::Default for #meta_ident #ty_gen
            #whr
            {
                fn default() -> Self { Self(::core::marker::PhantomData) }
            }

            unsafe impl #impl_gen
                ::design::place::Subplace
                for #meta_ident #ty_gen
            #whr
            {
                type Source = #source_ty;
                type Target = #field_ty;

                fn offset(self, (): ()) -> (::core::primitive::usize, ()) {
                    (
                        ::core::mem::offset_of!(
                            #adt_ident #ty_gen,
                            #field_access,
                        ),
                        (),
                    )
                }
            }

            unsafe impl #impl_gen
                ::design::place::Field
                for #meta_ident #ty_gen
            #whr
            {
                const NAME: &'static ::core::primitive::str = #field_name;
            }
        });

        let macro_lhs = if let Some(variant) = variant {
            quote!(#adt_ident #fragment_decl_gen::#variant)
        } else {
            quote!(#adt_ident #fragment_decl_gen)
        };

        field_arms.push(quote! {
            (#macro_lhs, #field_ident) => {
                ___lang_limits::#meta_ident #fragment_ty_gen
            };
        });
    }
}

fn generate_for_enum(
    enum_: &ItemEnum,
    lang_limits: &mut Vec<TokenStream>,
    variant_arms: &mut Vec<TokenStream>,
    field_arms: &mut Vec<TokenStream>,
    errors: &mut TokenStream,
) {
    let enum_ident = &enum_.ident;
    let (impl_gen, ty_gen, whr) = enum_.generics.split_for_impl();
    let (fragment_decl_gen, fragment_ty_gen) =
        generics_to_fragments(&enum_.generics);

    let mut variants = Vec::new();

    for variant in &enum_.variants {
        let variant_ident = &variant.ident;
        variants.push(variant_ident);
        let variant_name_str = variant_ident.to_string();

        let variant_meta_ident =
            format_ident!("___{}__{}", enum_ident, variant_ident);

        lang_limits.push(quote! {
            #[repr(transparent)]
            pub struct #variant_meta_ident #ty_gen (#enum_ident #ty_gen);

            unsafe impl #impl_gen
                ::design::place::HasVariant<#variant_name_str>
                for #enum_ident #ty_gen
            #whr
            {
                type VariantType = #variant_meta_ident #ty_gen;
            }
        });

        variant_arms.push(quote! {
            (#enum_ident #fragment_decl_gen, #variant_ident) => {
                ___lang_limits::#variant_meta_ident #fragment_ty_gen
            };
        });

        generate_fields(
            enum_ident,
            Some(variant_ident),
            &enum_.generics,
            &variant.fields,
            lang_limits,
            field_arms,
            errors,
        );
    }

    let variant_names = variants
        .iter()
        .map(|variant| variant.to_string())
        .collect::<Vec<_>>();

    lang_limits.push(quote! {
        unsafe impl #impl_gen
            ::design::place::Matchable
            for #enum_ident #ty_gen
        #whr
        {
            const VARIANTS: &'static [&'static str] = &[ #(#variant_names),* ];

            unsafe fn variant_at(ptr: *const Self) -> &'static str {
                // FIXME: this is unsound, but nothing else that we could do
                // here instead :(
                match unsafe { &*ptr } {
                    #(Self::#variants {..} => #variant_names,)*
                }
            }
        }
    });
}
