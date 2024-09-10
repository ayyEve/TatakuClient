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
/// allowed variant attribute values:
/// - ignore
/// - display = String (default is variant name)
#[proc_macro_derive(Dropdown, attributes(Dropdown))]
pub fn dropdown(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast:DeriveInput = syn::parse(input).unwrap();
    let mut entries = Vec::new();
    let mut debug = false;

    struct DropdownVariant {
        /// text to be displayed
        display: String,
        /// name of variant
        name: String,
    }


    for attr in &ast.attrs {
        if !attr.path.is_ident("Dropdown") {continue}

        if let Ok(Meta::List(list)) = attr.parse_meta() {
            for i in list.nested {
                if let NestedMeta::Meta(Meta::Path(p)) = &i {
                    if p.is_ident("debug") {
                        debug = true;
                    } else {
                        panic!("Unknown enum attribute {}", p.get_ident().unwrap())
                    }
                }
            }
        }
    }


    // find all the variant data
    if let Data::Enum(data) = &ast.data {
        for v in data.variants.iter() {
            let mut variant = DropdownVariant {display: v.ident.to_string(), name: v.ident.to_string()};
            let mut ignore = false;

            // find the id of the packet
            for a in v.attrs.iter() {
                if !a.path.is_ident("Dropdown") {continue}

                if let Ok(Meta::List(list)) = a.parse_meta() {
                    for i in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(name_value)) = &i {
                            // display
                            if let Lit::Str(display) = &name_value.lit {
                                variant.display = display.value()
                            }
                        }

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
            if ignore {continue}

            // create packet data
            entries.push(variant)
        }
    }

    // make sure the enum isnt empty
    if entries.is_empty() {panic!("enum is empty or all variants are ignored")}

    let enum_name = ast.ident.to_string();
    let mut to_string = String::new();
    let mut from_string = String::new();
    let mut variants = String::new();

    let t = "    ";

    for variant in entries {
        to_string += &format!("//{t}{t}{t}{t}Self::{} => \"{}\".to_owned(), \n", variant.name, variant.display);
        from_string += &format!("//{t}{t}{t}{t}\"{}\" => Self::{}, \n", variant.display, variant.name);
        variants += &format!("//{t}{t}{t}{t}Self::{},\n", variant.name);
    }
    let variants = variants.trim();
    let to_string = to_string.trim();
    let from_string = from_string.trim();

    let gen = format!(r#"
    impl Dropdownable2 for {enum_name} {{
        type T = {enum_name};
        fn variants() -> Vec<Self::T> {{
            vec![
                {variants}
            ]
        }}
        // fn display_text(&self) -> String {{
        //     match self {{
        //         {to_string}
        //         other => panic!("variant {{:?}} was ignored", other)
        //     }}
        // }}
        // fn from_string(string: String) -> Self {{
        //     match &*string {{
        //         {from_string}
        //         other => panic!("variant does not exist for enum {enum_name} and string {{}}", other)
        //     }}
        // }}
    }}

    "#);

    if debug {println!("generated: \n{}", gen)}

    // Return the generated impl
    let gen = gen.parse::<proc_macro2::TokenStream>().unwrap();
    proc_macro::TokenStream::from(quote!{#gen})
}

/// allowed attribute values:
/// - selectable=bool (default true)
/// - multi_selectable=bool (default false)
#[proc_macro_derive(ScrollableGettersSetters, attributes(Scrollable))]
pub fn scrollable_getters_setters(input: TokenStream) -> TokenStream {
    let ast:DeriveInput = syn::parse(input).unwrap();

    let mut has_pos = false;
    let mut has_tag = false;
    let mut has_hover = false;
    let mut has_selected = false;
    let mut has_size = false;

    let mut selectable = true;
    let mut multi_selectable = false;

    let mut generics = String::new();

    for attr in &ast.attrs {
        if !attr.path.is_ident("Scrollable") {continue}

        if let Ok(Meta::List(list)) = attr.parse_meta() {
            for i in list.nested {
                if let NestedMeta::Meta(Meta::NameValue(name_value)) = &i {
                    if let Lit::Bool(i) = &name_value.lit {
                        if name_value.path.is_ident("selectable") {
                            selectable = i.value()
                        } else if name_value.path.is_ident("multi_selectable") {
                            multi_selectable = i.value()
                        } else {
                            panic!("unknown attribute {}", name_value.path.get_ident().unwrap())
                        }
                    } else if let Lit::Str(i) = &name_value.lit {
                        if name_value.path.is_ident("generics") {
                            generics = i.value()
                        } else {
                            panic!("unknown attribute {}", name_value.path.get_ident().unwrap())
                        }
                    }
                }
            }
        }
    }

    if let Data::Struct(s) = &ast.data {
        for f in &s.fields {
            if let Some(ident) = &f.ident {
                has_size |= ident.to_string() == "size";
                has_pos |= ident.to_string() == "pos";
                has_tag |= ident.to_string() == "tag";
                has_hover |= ident.to_string() == "hover";
                has_selected |= ident.to_string() == "selected";
            }
            if has_pos && has_tag && has_hover && has_selected {break}
        }
    }
    if !has_size {panic!("Scrollable does not have a size field")}

    let generics = {
        if generics.len() > 0 {format!("<{generics}>")}
        else {String::new()}
    };
    let generics2 = generics.split(",").map(|g|g.split(":").next().unwrap()).collect::<Vec<_>>().join(",") + if generics.len() > 0 {">"} else {""};

    let struct_name = ast.ident.to_string();

    let mut str = String::new();
    str += &format!("impl{generics} ScrollableItemGettersSetters for {struct_name}{generics2} {{\n");

    // size (required)
    str += "    fn size(&self) -> Vector2 {self.size}\n    fn set_size(&mut self, new_size: Vector2) {self.size = new_size}\n";

    if has_pos {
        str += "    fn get_pos(&self) -> Vector2 {self.pos}\n    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}\n"
    }
    if has_tag {
        str += "    fn get_tag(&self) -> String {self.tag.clone()}\n    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}\n"
    }
    if has_hover {
        str += "    fn get_hover(&self) -> bool {self.hover}\n    fn set_hover(&mut self, hover:bool) {self.hover = hover}\n"
    }
    if has_selected {
        str += "    fn get_selected(&self) -> bool {self.selected}\n    fn set_selected(&mut self, selected:bool) {self.selected = selected}\n";
    }

    str += &format!("    fn get_selectable(&self) -> bool {{{selectable}}}\n    fn get_multi_selectable(&self) -> bool {{{multi_selectable}}}\n");

    str += "}";


    let tokens = str.parse::<proc_macro2::TokenStream>().unwrap();
    proc_macro::TokenStream::from(quote! {#tokens})
}



#[proc_macro_derive(SettingsDeserialize)]
pub fn impl_settings_deserializer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = settings_deserializer::impl_settings_deserializer(&ast);

    // Return the generated impl
    proc_macro::TokenStream::from(gen)
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
        if !f.attrs.iter().find(|a|a.path.is_ident("chain")).is_some() { continue }
        tys.push(&f.ty);
        idents.push(f.ident.as_ref().unwrap());
        idents_maybe.push(format_ident!("{}_maybe", f.ident.as_ref().unwrap()))
    }

    let tokens = quote! {
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
    };

    // std::fs::write("debug/test.rs", tokens.to_string()).unwrap();

    proc_macro::TokenStream::from(tokens)
}