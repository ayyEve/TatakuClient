
use syn::*;
use quote::*;
use super::*;
use proc_macro2::TokenStream;

#[derive(Debug, Clone, Default)]
pub(super) struct SubsettingItem {
    common: CommonItems,
}
impl SubsettingItem {
    pub fn common(&self) -> &CommonItems { &self.common }
    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = false;
        
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()); }

            // if meta.path.is_ident(CATEGORY_ICON_ATTRIBUTE) {
            //     let _ = meta.value()?;
            //     let value: LitStr = meta.input.parse()?;
            //     self.icon = Some(value.value());
            // } 
            // else {
                Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
            // }

        })?;

        Ok(self)
    }
    
    pub fn write(&self, property: &Ident) -> TokenStream {
        let text = &self.common.text;
        let prop_string = property.to_string();

        quote! { 
            builder.add_category(#text, None::<&str>);

            self.#property.create_provider(
                format!("{prefix}.{}", #prop_string),
                builder,
            );
        }
    }
}
