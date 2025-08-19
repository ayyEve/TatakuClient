/*
    its pretty annoying that the rust Default macro doesnt allow for this :/
*/
use proc_macro2::TokenStream;
use quote::*;
use syn::*;

const DEFAULT_ATTRIBUTE: &str = "default";

macro_rules! try_error {
    ($($t:tt)+) => {
        match $($t)+ {
            Ok(ok) => ok,
            Err(e) => return e.into_compile_error(),
        }
    };
}


pub(crate) fn derive(derive: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let type_name = &derive.ident;
    let (
        impl_generics, 
        ty_generics, 
        where_clause
    ) = derive.generics.split_for_impl();

    let mut tokens = proc_macro2::TokenStream::new();
    match &derive.data {
        syn::Data::Struct(s) => {
            for (n, field) in s.fields.iter().enumerate() {
                let ident = field.ident.clone().unwrap_or_else(|| format_ident!("{n}"));

                let attrs = try_error!(FieldAttributes::parse_from_attrs(field.attrs.as_slice(), false));
                if let Some(default) = attrs.default {
                    tokens.extend(quote! { #ident: #default, });
                } else {
                    tokens.extend(quote! { #ident: Default::default(), });
                }
            }
            tokens = quote! { Self { #tokens } }
        }
        syn::Data::Enum(e) => {
            for variant in e.variants.iter() {
                let ident = &variant.ident;
                let attrs = try_error!(FieldAttributes::parse_from_attrs(variant.attrs.as_slice(), true));
                if !attrs.is_variant { continue }
                
                let named = variant.fields.iter().all(|f| f.ident.is_some());

                if variant.fields.is_empty() {
                    tokens.extend(quote!{ Self::#ident });
                } else if named {
                    let mut field_tokens = proc_macro2::TokenStream::new();

                    for field in variant.fields.iter() {
                        let ident = field.ident.as_ref().unwrap();
                        let attrs = try_error!(FieldAttributes::parse_from_attrs(field.attrs.as_slice(), false));

                        if let Some(default) = attrs.default {
                            field_tokens.extend(quote! { #ident: #default });
                        } else {
                            field_tokens.extend(quote! {
                                #ident: Default::default(),
                            });
                        }
                    }

                    tokens.extend(quote! { Self::#ident { #field_tokens } });
                } else {
                    let mut field_tokens = proc_macro2::TokenStream::new();

                    for field in variant.fields.iter() {
                        let attrs = try_error!(FieldAttributes::parse_from_attrs(field.attrs.as_slice(), false));

                        if let Some(default) = attrs.default {
                            field_tokens.extend(quote! { #default, });
                        } else {
                            field_tokens.extend(quote! { Default::default(), });
                        }
                    }

                    tokens.extend(quote!{ Self::#ident(#field_tokens) });
                }
            }
        }

        syn::Data::Union(_u) => panic!("no unions >:c"),
    }

    let tokens = quote! {
        impl #impl_generics Default for #type_name #ty_generics where #where_clause {
            fn default() -> Self {
                #tokens
            }
        }
    };
    
    std::fs::write(format!("./debug/pain/{type_name}.rs"), tokens.to_string()).unwrap();

    tokens
}

#[derive(Default)]
struct FieldAttributes {
    is_variant: bool,
    default: Option<TokenStream>,
}
impl FieldAttributes {
    fn parse_from_attrs(
        attrs: &[Attribute], 
        is_enum: bool,
    ) -> Result<Self> {
        let mut a = Self::default();
    
        for attr in attrs {
            if !attr.path().is_ident(DEFAULT_ATTRIBUTE) { continue; }
            if is_enum {
                a.is_variant = true;
            } else {
                use syn::parse::Parse;
                let ts = attr.parse_args_with(TokenStream::parse)?;
                a.default = Some(ts);
            }
        }
    
        Ok(a)
    }
}
