use proc_macro2::TokenStream;
use quote::*;
use syn::*;

const DEBUG_ATTRIBUTE: &str = "debug";
const SKIP_ATTRIBUTE: &str = "skip";


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
    let (impl_generics, ty_generics, where_clause) = derive.generics.split_for_impl();

    let mut tokens = proc_macro2::TokenStream::new();
    match &derive.data {
        syn::Data::Struct(s) => {
            for (n, field) in s.fields.iter().enumerate() {
                let ident = field.ident.clone().unwrap_or_else(|| format_ident!("{n}"));

                let attrs = try_error!(FieldAttributes::parse_from_attrs(field.attrs.as_slice()));
                if attrs.skip {
                    tokens.extend(quote! {
                        .field(stringify!(#ident), &())
                    })
                } else {
                    tokens.extend(quote! {
                        .field(stringify!(#ident), &self.#ident)
                    })
                }
            }
            
            quote! {
                impl #impl_generics std::fmt::Debug for #type_name #ty_generics where #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f
                        .debug_struct(stringify!(#type_name))
                        #tokens
                        .finish()
                    }
                }
            }
        }
        syn::Data::Enum(e) => {
            for variant in e.variants.iter() {
                let ident = &variant.ident;
                
                let named = variant.fields.iter().all(|f| f.ident.is_some());

                let attrs = try_error!(FieldAttributes::parse_from_attrs(variant.attrs.as_slice()));

                if attrs.skip {

                    let a = if variant.fields.is_empty() {
                        TokenStream::new()
                    } else if named {
                        "{..}".parse().unwrap()
                    } else {
                        "(..)".parse().unwrap()
                    };

                    tokens.extend(quote!(
                        Self::#ident #a => {
                            f
                            .debug_tuple(stringify!(#ident))
                            .finish()
                        },
                    ));
                } else if variant.fields.is_empty() {
                    tokens.extend(quote!(
                        Self::#ident => {
                            f
                            .debug_tuple(stringify!(#ident))
                            .finish()
                        },
                    ));
                } else if named {
                    let mut match_tokens = proc_macro2::TokenStream::new();
                    for field in variant.fields.iter() {
                        let ident = field.ident.as_ref().unwrap();
                        let attrs = try_error!(FieldAttributes::parse_from_attrs(field.attrs.as_slice()));
                        if attrs.skip {
                            match_tokens.extend(quote! {
                                .field(stringify!(#ident), &())
                            })
                        } else {
                            match_tokens.extend(quote! {
                                .field(stringify!(#ident), &#ident)
                            })
                        }
                    }

                    let fields = variant.fields.iter().map(|f|f.ident.as_ref().unwrap()).collect::<Vec<_>>();
                    tokens.extend(quote!(
                        Self::#ident { #(#fields,)* } => {
                            f
                            .debug_struct(stringify!(#type_name))
                            #match_tokens
                            .finish()
                        },
                    ));
                } else {
                    let mut match_tokens = proc_macro2::TokenStream::new();
                    for (n, field) in variant.fields.iter().enumerate() {
                        let ident = format_ident!("_{n}");
                        let attrs = try_error!(FieldAttributes::parse_from_attrs(field.attrs.as_slice()));
                        if attrs.skip {
                            match_tokens.extend(quote! {
                                .field(&())
                            })
                        } else {
                            match_tokens.extend(quote! {
                                .field(&#ident)
                            })
                        }
                    }

                    let fields = (0..variant.fields.len()).map(|n| format_ident!("_{n}")).collect::<Vec<_>>();
                    
                    tokens.extend(quote!(
                        Self::#ident ( #(#fields,)* ) => {
                            f
                            .debug_tuple(stringify!(#type_name))
                            #match_tokens
                            .finish()
                        },
                    ));
                }
            }

            quote! {
                impl #impl_generics std::fmt::Debug for #type_name #ty_generics where #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #tokens
                        }
                    }
                }
            }
        }

        syn::Data::Union(_u) => {
            panic!("no unions >:c")
        }
    }
}

#[derive(Default)]
struct FieldAttributes {
    skip: bool,
}
impl FieldAttributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut a = Self::default();
    
        for attr in attrs {
            if !attr.path().is_ident(DEBUG_ATTRIBUTE) { continue; }
    
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(SKIP_ATTRIBUTE) {
                    a.skip = true
                }
                else {
                    return Err(meta.error("Invalid attribute"))
                }
    
                Ok(())
            })?;
        }
    
        Ok(a)
    }
}