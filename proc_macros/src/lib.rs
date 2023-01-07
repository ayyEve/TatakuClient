mod settings_menu;

#[proc_macro_derive(Settings, attributes(Setting, Subsetting))]
pub fn create_setting(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = settings_menu::impl_settings(&ast);
    
    // Return the generated impl
    proc_macro::TokenStream::from(gen)
}