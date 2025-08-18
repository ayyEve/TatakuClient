use quote::*;
use super::*;
use proc_macro2::TokenStream;
use syn::{ spanned::Spanned, * };

// common attributes
const BUTTON_ATTRIBUTE:&str = "button";
const DIVIDER_ATTRIBUTE:&str = "divider";
const SETTING_ATTRIBUTE:&str = "setting";
const DROPDOWN_ATTRIBUTE:&str = "dropdown";
const CATEGORY_ATTRIBUTE:&str = "category";
const SUBSETTING_ATTRIBUTE:&str = "subsetting";

pub(crate) fn impl_settings(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &ast.data else {
        return Err(Error::new(ast.span(), "Settings can only be derived on a struct"));
    };

    let settings = data.fields
        .iter()
        .map(SettingsItem::read)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let struct_name = &ast.ident;
    let mut output = proc_macro2::TokenStream::new();

    for setting in settings {
        let property = setting.field_name.clone().unwrap();
        setting.write(&property, &mut output);
    }

    let all_lines = quote!{
        impl MakeSettingsMenu for #struct_name {
            fn create_provider(
                &self, 
                prefix: String,
                builder: &mut SettingsBuilder,
            ) {
                use crate::prelude::*;
                #output
            }
        }
    };

    // std::fs::create_dir_all("/tmp/debug").unwrap();
    // std::fs::write(format!("/tmp/debug/{struct_name}-settings_impl.rs"), all_lines.to_string()).unwrap();
    
    Ok(all_lines)
}

struct SettingsItem {
    /// the type of this setting item
    setting_type: SettingType, 

    /// what is the field name of the setting? 
    field_name: Option<Ident>,
}
impl SettingsItem {
    fn read(f: &Field) -> Result<Option<Self>> {
        let Some(field_name) = f.ident.as_ref()
        else { return Ok(None) };

        // read the attributes
        match SettingType::read(f)? {
            Some(a) => Ok(Some(SettingsItem {
                field_name: Some(field_name.clone()),
                setting_type: a
            })),

            None => Ok(None),
        }
    }

    fn write(&self, property: &Ident, output: &mut TokenStream) {
        let common = self.setting_type.common();
        let tokens = self.setting_type.write(property);
        
        if common.add_item {
            let text = &common.text;
            let prop_string = property.to_string();

            output.extend(quote! {
                builder.add_item(BuildableSetting {
                    name: #text.to_owned(),
                    path: format!("{prefix}.{}", #prop_string),
                    setting_type: #tokens,

                    tooltip: None,
                    enabled_if: None,
                    visible_if: None,
                });
            });
        } else {
            output.extend(tokens);
        }
    }

}

enum SettingType {
    Normal(NormalItem),
    /// sub settings, ie mania or taiko settings
    SubSetting(SubsettingItem),

    Button(ButtonItem),
    Dropdown(DropdownItem),
    Category(CategoryItem),
    Divider(DividerItem),
}
impl SettingType {
    fn read(field: &Field) -> Result<Option<Self>> {
        let mut a = None;

        for attr in &field.attrs {
            let path = attr.path();

            // Normal item (number, string, bool, etc)
            if path.is_ident(SETTING_ATTRIBUTE) { 
                a = Some(Self::Normal(NormalItem::new(field).read(attr)?));
            } 
            // subsetting
            else if path.is_ident(SUBSETTING_ATTRIBUTE) { 
                a = Some(Self::SubSetting(SubsettingItem::default().read(attr)?));
            } 
            // dropdown
            else if path.is_ident(DROPDOWN_ATTRIBUTE) {
                a = Some(Self::Dropdown(DropdownItem::default().read(attr)?));
            } 
            // category
            else if path.is_ident(CATEGORY_ATTRIBUTE) {
                a = Some(Self::Category(CategoryItem::default().read(attr)?));
            } 
            // divider
            else if path.is_ident(DIVIDER_ATTRIBUTE) {
                a = Some(Self::Divider(DividerItem::default().read(attr)?));
            } 
            // button
            else if path.is_ident(BUTTON_ATTRIBUTE) {
                a = Some(Self::Button(ButtonItem::default().read(attr)?));
            } 
        }

        Ok(a)
    }

    fn common(&self) -> &CommonItems {
        match self {
            Self::Normal(i) => i.common(),
            Self::SubSetting(i) => i.common(),
            Self::Dropdown(i) => i.common(),
            Self::Category(i) => i.common(),
            Self::Button(i) => i.common(),
            Self::Divider(i) => i.common(),
        }
    }
    fn write(&self, property: &Ident) -> TokenStream {
        match self {
            Self::Normal(i) => i.write(),
            Self::SubSetting(i) => i.write(property),
            Self::Dropdown(i) => i.write(),
            Self::Category(i) => i.write(),
            Self::Button(i) => i.write(),
            Self::Divider(i) => i.write(),
        }
    }
}
