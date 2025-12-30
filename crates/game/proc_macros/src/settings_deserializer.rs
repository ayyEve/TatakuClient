use syn::*;
use quote::*;

#[allow(clippy::useless_format)]
pub(crate) fn impl_settings_deserializer(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let struct_name = &ast.ident;

    let mut idents = Vec::new();
    let mut tys = Vec::new();

    if let Data::Struct(s) = &ast.data {
        'field: for f in &s.fields {

            for attr in &f.attrs {
                if !attr.path().is_ident("serde") { continue; }
                let mut skip = false;
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        skip = true;
                    }
                    
                    Ok(())
                });

                if skip { continue 'field }
            }
            let Some(ident) = f.ident.as_ref() else { continue }; 
            tys.push(&f.ty);
            idents.push(ident);
        }
    }


    Ok(quote! {
        impl<'de> Deserialize<'de> for #struct_name {
            fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                use engine::settings::TatakuSettingOptional;

                #[derive(serde::Deserialize, Default)]
                #[serde(default)]
                struct De {
                    #(#idents: TatakuSettingOptional<#tys>,)*
                }

                let other = De::deserialize(de)?;
                let mut output = Self::default();
                
                #(
                    match other.#idents {
                        TatakuSettingOptional::NoValue => (),
                        TatakuSettingOptional::Err(e) => warn!("Error reading {}.{}: {{e}}", stringify!(#struct_name), stringify!(#idents)),
                        TatakuSettingOptional::Value(v) => output.#idents = v,
                    }
                )*

                Ok(output)
            }
        }
    })
    
}
