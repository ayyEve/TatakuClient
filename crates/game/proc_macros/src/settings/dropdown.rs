use syn::*;
use super::*;
use quote::*;
use proc_macro2::TokenStream;


const DROPDOWN_PATH_ATTRIBUTE:&str = "path";

#[derive(Debug, Clone, Default)]
pub struct DropdownItem {
    common: CommonItems,
    path: Option<String>,
}
impl DropdownItem {
    pub fn common(&self) -> &CommonItems { &self.common }
    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = true;
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()); }

            if meta.path.is_ident(DROPDOWN_PATH_ATTRIBUTE) {
                let _ = meta.value()?;
                let value: LitStr = meta.input.parse()?;
                self.path = Some(value.value());
            } 
            else {
                return Err(meta.error(format!("Invalid dropdown attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(self)
    }

    pub fn write(&self) -> TokenStream {
        if let Some(path) = &self.path {
            quote! {
                BuildableSettingType::Dropdown {
                    options: BuildableSettingDropdownOptions::Variable {
                        var: #path .to_string(),
                    },
                }
            }
        } else {
            panic!("nope")
        }
    }
}
