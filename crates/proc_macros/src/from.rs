use quote::*;
use syn::*;

const FROM_ATTRIBUTE: &str = "from";
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

    let syn::Data::Enum(e) = &derive.data else { panic!("unsupported!") };

    for variant in e.variants.iter() {
        if variant.fields.is_empty() { continue }
        let ident = &variant.ident;
        
        let named = variant
            .fields
            .iter()
            .any(|f| f.ident.is_some());
        if named { continue }

        let attrs = try_error!(FieldAttributes::parse_from_attrs(variant.attrs.as_slice()));
        if attrs.skip { continue }

        let ty = &variant.fields
            .iter()
            .next()
            .unwrap()
            .ty;

        tokens.extend(quote!(
            impl #impl_generics From<#ty> for #type_name #ty_generics where #where_clause {
                fn from(value: #ty) -> Self {
                    Self::#ident (value)
                }
            }
        ));
    }
        

    tokens
}



#[derive(Default)]
struct FieldAttributes {
    skip: bool,
}
impl FieldAttributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut a = Self::default();
    
        for attr in attrs {
            if !attr.path().is_ident(FROM_ATTRIBUTE) { continue; }
    
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(SKIP_ATTRIBUTE) {
                    a.skip = true;
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
