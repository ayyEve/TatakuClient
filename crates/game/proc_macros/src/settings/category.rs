use super::*;
use quote::*;
use proc_macro2::TokenStream;
use syn::{ Result, Attribute, LitStr };

const CATEGORY_ICON_ATTRIBUTE:&str = "icon";

#[derive(Debug, Clone, Default)]
pub(super) struct CategoryItem {
    common: CommonItems,
    icon: Option<String>,
}
impl CategoryItem {
    pub fn common(&self) -> &CommonItems { &self.common }
    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = false;
        
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()); }

            if meta.path.is_ident(CATEGORY_ICON_ATTRIBUTE) {
                let _ = meta.value()?;
                let value: LitStr = meta.input.parse()?;
                self.icon = Some(value.value());
            } 
            else {
                return Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(self)
    }
    
    pub fn write(&self) -> TokenStream {
        let name = &self.common.text;
        let icon = if let Some(icon) = &self.icon {
            quote!{ Some(#icon) }
        } else {
            quote!{ None::<&str> }
        };
        quote!{
            builder.add_category(#name, #icon);
        }
    }
}
