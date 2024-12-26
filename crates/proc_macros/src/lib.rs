mod settings_menu;
mod settings_deserializer;

use proc_macro::TokenStream;
use quote::*;
use syn::*;

#[proc_macro_derive(Settings, attributes(Setting, Subsetting))]
pub fn create_setting(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = settings_menu::impl_settings(&ast);

    // Return the generated impl
    proc_macro::TokenStream::from(gen)
}


/// allowed enum attribute values:
/// - debug
/// 
/// allowed variant attribute values:
/// - ignore
/// - display = String (default is variant name)
#[proc_macro_derive(Dropdown, attributes(Dropdown))]
pub fn dropdown(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast:DeriveInput = syn::parse(input).unwrap();
    let mut entries = Vec::new();

    // find all the variant data
    if let Data::Enum(data) = &ast.data {
        for v in data.variants.iter() {
            let mut ignore = false;

            // find the id of the packet
            for a in v.attrs.iter() {
                if !a.path.is_ident("Dropdown") { continue }

                if let Ok(Meta::List(list)) = a.parse_meta() {
                    for i in list.nested {
                        // ignore
                        if let NestedMeta::Meta(Meta::Path(name)) = &i {
                            if name.is_ident("ignore") {
                                ignore = true;
                                break
                            }
                        }

                    }
                }
            }

            // skip this variant if it should be ignored
            if ignore { continue }

            // create packet data
            entries.push(&v.ident)
        }
    }

    // make sure the enum isnt empty
    if entries.is_empty() { panic!("enum is empty or all variants are ignored") }

    let enum_name = ast.ident;
    quote! {
        impl Dropdownable2 for #enum_name {
            type T = Self;

            fn variants() -> Vec<Self::T> {
                vec![
                    #(Self::#entries,)*
                ]
            }
        }
    }.into()
}

#[proc_macro_derive(SettingsDeserialize)]
pub fn impl_settings_deserializer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build and return the impl
    settings_deserializer::impl_settings_deserializer(&ast)
}


#[proc_macro_derive(ChainableInitializer, attributes(chain))]
pub fn impl_chainable_initializer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast:DeriveInput = syn::parse(input).unwrap();

    // Build the impl
    let Data::Struct(s) = &ast.data else { panic!("no") };
    let struct_name = &ast.ident;

    let mut tys = Vec::new();
    let mut idents = Vec::new();
    let mut idents_maybe = Vec::new();

    for f in s.fields.iter() {
        if !f.attrs.iter().any(|a| a.path.is_ident("chain")) { continue }
        let Some(ident) = &f.ident else { panic!("ghjskslgd") }; 
        tys.push(&f.ty);
        idents.push(ident);
        idents_maybe.push(format_ident!("{ident}_maybe"))
    }

    quote! {
        impl #struct_name { #(
            pub fn #idents(mut self, val: impl Into<#tys>) -> Self {
                self.#idents = val.into();
                self
            }

            pub fn #idents_maybe(mut self, val: Option<impl Into<#tys>>) -> Self {
                let Some(val) = val else { return self };
                self.#idents = val.into();
                self
            }
        )* }
    }.into()
}