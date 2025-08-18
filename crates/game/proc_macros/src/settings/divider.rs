use super::*;
use quote::*;
use syn::Result;
use syn::Attribute;
use proc_macro2::TokenStream;

#[derive(Debug, Clone, Default)]
pub(super) struct DividerItem {
    common: CommonItems,
}
impl DividerItem {
    pub fn common(&self) -> &CommonItems { &self.common }
    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = true;
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()); }

            Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
        })?;

        Ok(self)
    }
    
    pub fn write(&self) -> TokenStream {
        quote! { 
            BuildableSettingType::Divider
        }
    }
}
