use syn::*;
use quote::*;

#[allow(clippy::useless_format)]
pub(crate) fn impl_settings_deserializer(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let struct_name = ast.ident.to_string();
    let de_struct_name = format!("{struct_name}Deserializer");

    let mut convert_def = Vec::new();

    let mut de_impl = Vec::new();
    de_impl.push(format!("impl<'de> Deserialize<'de> for {struct_name} {{"));
    de_impl.push(format!("fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {{"));
    de_impl.push(format!("#[derive(serde::Deserialize, Default)]"));
    de_impl.push(format!("#[serde(default)]"));
    de_impl.push(format!("pub struct {de_struct_name} {{"));


    if let Data::Struct(s) = &ast.data {
        for f in &s.fields {
            if f.attrs.iter().any(|a|a.path.is_ident("serde") && a.tokens.to_string().contains("skip")) { continue }
            let ident = f.ident.as_ref().unwrap().to_string();
            let ty = f.ty.to_token_stream().to_string();
            de_impl.push(format!("{ident}: TatakuSettingOptional<{ty}>,"));

            convert_def.extend([
                format!("// {ident}"),
                format!("match other.{ident} {{"),
                format!("TatakuSettingOptional::NoValue => (),"),
                format!("TatakuSettingOptional::Err(e) => warn!(\"Error reading {struct_name}.{ident}: {{}}\", e),"),
                format!("TatakuSettingOptional::Value(v) => output.{ident} = v,"),
                format!("}}\n"),
            ]);
        }
    }
    
    // finish off the struct def
    de_impl.push(format!("}}"));

    // start the convert def
    de_impl.push(format!("let other = {de_struct_name}::deserialize(de)?;"));
    de_impl.push(format!("let mut output = Self::default();"));
    // do convert
    de_impl.extend(convert_def);

    // return output
    de_impl.push(format!("Ok(output)\n}}\n}}"));

    // output
    let str = de_impl.join("\n");
    #[cfg(feature="extra_debugging")] {
        std::fs::create_dir_all("debug").unwrap();
        std::fs::write(format!("debug/{struct_name}"), &str).unwrap();
    }

    let tokens = str.parse::<proc_macro2::TokenStream>().unwrap();
    proc_macro::TokenStream::from(quote! {#tokens})
}
