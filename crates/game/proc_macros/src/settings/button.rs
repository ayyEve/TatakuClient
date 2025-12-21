use syn::*;
use quote::*;
use super::*;
use proc_macro2::TokenStream;

const BUTTON_ACTION_ATTRIBUTE:&str = "action";

#[derive(Debug, Clone, Default)]
pub(super) struct ButtonItem {
    common: CommonItems,
    action: Option<Expr>,
}
impl ButtonItem {
    pub fn common(&self) -> &CommonItems { &self.common }

    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = true;
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()) }

            if meta.path.is_ident(BUTTON_ACTION_ATTRIBUTE) {
                let _ = meta.value()?;
                // fixme: parse_nested_meta expects LitStr here
                let value: LitStr = meta.input.parse()?;
                let value: Expr = value.parse()?;
                self.action = Some(value);
            } 
            else {
                return Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(self)
    }
    pub fn write(&self) -> TokenStream {
        let action = if let Some(action) = &self.action {
            quote! {
                Some(#action.into())
            }
        } else {
            quote! { None }
        };

        quote! { 
            engine::settings::BuildableSettingType::Button {
                action: engine::settings::BuildableSettingsAction {
                    inner: Arc::new(|| #action),
                },
            }
        }
    }
}
