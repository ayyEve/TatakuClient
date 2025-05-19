use proc_macro2::TokenStream;
use quote::*;
use syn::*;

const CSS_ATTRIBUTE: &str = "css";
const NAME_ATTRIBUTE: &str = "name";
const DEFAULT_ATTRIBUTE: &str = "default";
const SKIP_ATTRIBUTE: &str = "skip";
const PARSE_WITH_ATTRIBUTE: &str = "parse_with";

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

    let mut match_tokens = proc_macro2::TokenStream::new();
    
    let mut fields = Vec::new();
    match &derive.data {
        syn::Data::Struct(s) => {
            for i in s.fields.iter() {
                let attributes = try_error!(CssAttributes::parse_from_attrs(i.attrs.as_slice()));
                
                if attributes.skip { continue }
                let parse_with = attributes
                    .parse_with
                    .unwrap_or("str::parse".to_owned())
                    .parse::<TokenStream>()
                    .unwrap();
                
                let ident = i.ident.as_ref().unwrap();
                let name = attributes.name.unwrap_or(
                    ident.to_string().replace('_', "-")
                );

                let default = attributes
                    .default
                    .unwrap_or("Default::default()".to_string())
                    .parse::<TokenStream>()
                    .unwrap();

                match_tokens.extend(quote! {
                    #name => this.#ident = CssValue::parse(
                        d.value, 
                        #default, 
                        #parse_with
                    ),
                });

                fields.push(i.ident.as_ref().unwrap());
            }
        }

        _ => panic!("nope")
    }
    
    quote! {
        impl #impl_generics #type_name #ty_generics where #where_clause {
            pub fn parse_css(rule: &simplecss::Rule) -> Self {
                let mut this = Self::default();

                for d in rule.declarations.iter() {
                    match d.name {
                        #match_tokens
                        other => warn!("unknown css property: {other}"),
                    }
                }
                
                this
            }
        
            pub fn merge(self, parent: Self) -> Self {
                Self {
                    #(
                        #fields: self.#fields.check_unset(parent.#fields),
                    )*
                }
            }

            pub fn merge_parent(self, parent: Self) -> Self {
                Self {
                    #(
                        #fields: self.#fields.check_inherit(parent.#fields),
                    )*
                }
            }
        }
    }
}


#[derive(Default)]
struct CssAttributes {
    name: Option<String>,
    parse_with: Option<String>,
    default: Option<String>,
    skip: bool,
}
impl CssAttributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut a = Self::default();

        for attr in attrs {
            if !attr.path().is_ident(CSS_ATTRIBUTE) { continue; }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(NAME_ATTRIBUTE) {
                    let _ = meta.value()?;

                    let name = meta.input.parse::<LitStr>()?.value();
                    a.name = Some(name);
                }
                else if meta.path.is_ident(SKIP_ATTRIBUTE) {
                    a.skip = true;
                }
                else if meta.path.is_ident(DEFAULT_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let default = meta.input.parse::<LitStr>()?.value();
                    a.default = Some(default);
                }
                else if meta.path.is_ident(PARSE_WITH_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let parse_with = meta.input.parse::<LitStr>()?.value();
                    a.parse_with = Some(parse_with);
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
