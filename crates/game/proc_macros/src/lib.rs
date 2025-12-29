mod from;
mod chainable;
mod css_parse;
mod custom_debug;
mod custom_default;
mod settings_deserializer;
#[cfg(feature="graphics")] mod settings;

#[proc_macro_derive(From, attributes(from))]
pub fn impl_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    from::derive(&ast).into()
}

#[proc_macro_derive(Debug2, attributes(debug))]
pub fn impl_debug2(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    custom_debug::derive(&ast).into()
}

#[proc_macro_derive(Default2, attributes(default))]
pub fn impl_default2(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    custom_default::derive(&ast).into()
}

#[proc_macro_derive(ParseCss, attributes(css))]
pub fn impl_parse_css(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    css_parse::derive(&ast).into()
}


#[proc_macro_derive(
    Settings, 
    attributes(
        setting, 
        subsetting, 
        dropdown, 
        button, 
        category, 
        divider,
    )
)]
pub fn create_setting(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    #[cfg(feature="graphics")]
    let ast = syn::parse(input).unwrap();

    #[cfg(not(feature="graphics"))]
    return proc_macro::TokenStream::from(quote! {});

    #[cfg(feature="graphics")]
    wrap_result(settings::impl_settings(&ast))
}

#[proc_macro_derive(DeserializeSettings)]
pub fn impl_settings_deserializer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    wrap_result(settings_deserializer::impl_settings_deserializer(&ast))
}


#[proc_macro_derive(ChainableInitializer, attributes(chain))]
pub fn impl_chainable_initializer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    wrap_result(chainable::impl_chainable(&ast))
}

fn wrap_result(r: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    let tokens = match r {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error(),
    };
    proc_macro::TokenStream::from(tokens)
}
