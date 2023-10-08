use syn::*;
use quote::*;

pub(crate) fn impl_settings_format(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let struct_name = ast.ident.to_string();
    let de_struct_name = format!("{struct_name}Deserializer");

    let mut struct_fields = Vec::new();
    struct_fields.push(format!("#[derive(serde::Deserialize, Default)]"));
    struct_fields.push(format!("#[serde(default)]"));
    struct_fields.push(format!("pub struct {de_struct_name} {{"));

    let mut into_lines = Vec::new();
    into_lines.push(format!("impl From<{de_struct_name}> for {struct_name} {{"));
    into_lines.push(format!("fn from(other: {de_struct_name}) -> Self {{"));
    into_lines.push(format!("let mut output = Self::default();"));

    if let Data::Struct(s) = &ast.data {
        for f in &s.fields {
            if f.attrs.iter().find(|a|a.path.is_ident("serde") && a.tokens.to_string().contains("skip")).is_some() { continue }
            let ident = f.ident.as_ref().unwrap().to_string();
            let ty = f.ty.to_token_stream().to_string();
            struct_fields.push(format!("{ident}: TatakuSettingOptional<{ty}>,"));

            into_lines.extend([
                format!("// {ident}"),
                format!("match other.{ident} {{"),
                format!("TatakuSettingOptional::NoValue => (),"),
                format!("TatakuSettingOptional::Err(e) => warn!(\"Error reading {struct_name}.{ident}: {{}}\", e),"),
                format!("TatakuSettingOptional::Value(v) => output.{ident} = v,"),
                format!("}}\n"),
            ]);
        }
    }


    struct_fields.push(format!("}}"));
    into_lines.push(format!("output\n}}\n}}"));
    struct_fields.extend(into_lines);
    let str = struct_fields.join("\n");

    let tokens = str.parse::<proc_macro2::TokenStream>().unwrap();
    proc_macro::TokenStream::from(quote! {#tokens})
}
