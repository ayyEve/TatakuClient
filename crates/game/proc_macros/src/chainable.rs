use proc_macro2::TokenStream;
use syn::*;
use quote::*;

const CHAIN_ATTRIBUTE: &str = "chain";
const OPTION_ATTRIBUTE: &str = "option";


pub(crate) fn impl_chainable(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let (
        impl_generics, 
        ty_generics, 
        where_clause
    ) = ast.generics.split_for_impl();

    // Build the impl
    let Data::Struct(s) = &ast.data else { panic!("no") };
    let type_name = &ast.ident;

    let mut output = TokenStream::new();

    for f in s.fields.iter() {
        let mut attrs = Attributes::parse_from_attrs(&f.attrs)?;
        if !attrs.enabled { continue }

        let Some(ident) = &f.ident else { panic!("ghjskslgd") }; 
        let ident_maybe = format_ident!("{ident}_maybe");
        let ty = &f.ty;

        if let Type::Path(p) = &ty
        && let Some(s) = p.path.segments.first() 
        &&s.ident == "Option"
        {
            attrs.option = true;
        }
        
        // field is an Option<..>, handle it better
        if attrs.option {
            let Some(ty) = get_inner(&f.ty) 
            else { panic!("??") };

            output.extend(quote! {
                pub fn #ident(mut self, val: Option<impl Into<#ty>>) -> Self {
                    self.#ident = val.map(Into::into);
                    self
                }
            });
        } else {
            output.extend(quote! {
                pub fn #ident(mut self, val: impl Into<#ty>) -> Self {
                    self.#ident = val.into();
                    self
                }

                pub fn #ident_maybe(mut self, val: Option<impl Into<#ty>>) -> Self {
                    let Some(val) = val else { return self };
                    self.#ident = val.into();
                    self
                }
            });
        }
    }

    Ok(quote! {
        impl #impl_generics #type_name #ty_generics where #where_clause { 
            #output
        }
    })
}


#[derive(Default)]
struct Attributes {
    enabled: bool,
    option: bool,
}
impl Attributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut a = Self::default();
    
        for attr in attrs {
            if !attr.path().is_ident(CHAIN_ATTRIBUTE) { continue; }
            a.enabled = true;

            if let Meta::Path(_) = &attr.meta {
                return Ok(a)
            };
    
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(OPTION_ATTRIBUTE) {
                    a.option = true;
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


fn get_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty 
    else { return None };

    let a = path.path.segments.first()?;
    match &a.arguments {
        PathArguments::None => None,
        PathArguments::AngleBracketed(a) => {
            let GenericArgument::Type(b) = a.args.first()?
            else { return None };
            Some(b)
        }
        PathArguments::Parenthesized(_a) => {
            None
        }
    }
}