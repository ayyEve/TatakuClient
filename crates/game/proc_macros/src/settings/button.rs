use syn::*;
use quote::*;
use super::*;
use proc_macro2::TokenStream;

const BUTTON_ACTION_ATTRIBUTE:&str = "action";

#[derive(Debug, Clone, Default)]
pub(super) struct ButtonItem {
    common: CommonItems,
    action: String,
}
impl ButtonItem {
    pub fn common(&self) -> &CommonItems { &self.common }

    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = true;
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()) }

            if meta.path.is_ident(BUTTON_ACTION_ATTRIBUTE) {
                let _ = meta.value()?;
                let value: LitStr = meta.input.parse()?;
                self.action = value.value();
            } 
            else {
                return Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(self)
    }
    pub fn write(&self) -> TokenStream {
        let action = self.action.parse::<TokenStream>().unwrap();
        quote! { 
            BuildableSettingType::Button {
                action: TatakuAction::from(#action).into(),
            }
        }
    }
}

