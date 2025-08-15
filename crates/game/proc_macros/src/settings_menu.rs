use proc_macro2::TokenStream;
use quote::*;
use syn::{ meta::ParseNestedMeta, spanned::Spanned, * };


const TEXT_ATTRIBUTE:&str = "text";
const ACTION_ATTRIBUTE:&str = "action";
const CLICK_ATTRIBUTE:&str = "click";

const MIN_ATTRIBUTE:&str = "min";
const MAX_ATTRIBUTE:&str = "max";
const WIDTH_ATTRIBUTE:&str = "width";
const PASSWORD_ATTRIBUTE:&str = "password";
const SETTING_ATTRIBUTE:&str = "setting";
const SUBSETTING_ATTRIBUTE:&str = "subsetting";

// dropdown attrs
const DROPDOWN_ATTRIBUTE:&str = "dropdown";
const DROPDOWN_PATH_ATTRIBUTE:&str = "path";

// category attrs
const CATEGORY_ATTRIBUTE:&str = "category";
const CATEGORY_NAME_ATTRIBUTE:&str = "name";


pub(crate) fn impl_settings(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &ast.data else {
        return Err(Error::new(ast.span(), "Settings can only be derived on a struct"));
    };

    let settings = data.fields
        .iter()
        .map(SettingsItem::read)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|i| i)
        .collect::<Vec<_>>();

    let struct_name = &ast.ident;
    let mut into_elements_lines = proc_macro2::TokenStream::new();

    for setting in settings {
        let text = setting.setting_text.unwrap_or_default();
        let property = setting.setting_name.clone().unwrap();

        let prop_string = property.to_string();

        match setting.setting_type {
            // checkbox
            SettingsType::Bool => {
                into_elements_lines.extend(quote! {
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::Bool,

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }

            // slider
            f 
            @(SettingsType::U32 
            | SettingsType::U64 
            | SettingsType::Usize 
            | SettingsType::F32 
            | SettingsType::F64) => {
                let ty = f.to_str();
                let min = setting.range_min.unwrap_or(0.0) as f32;
                let max = setting.range_max.unwrap_or(100.0) as f32;
                let step = if f.is_float() {0.01f32} else {1.0};

                into_elements_lines.extend(quote! {
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::Number {
                            num_type: #ty.to_string(),
                            min: #min,
                            max: #max,
                            step: Some(#step),
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }

            // text input
            SettingsType::String => {
                let do_password = setting.password_input == Some(true);
                
                into_elements_lines.extend(quote! {
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::String {
                            password: #do_password,
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }

            // color input
            SettingsType::Color => {
                into_elements_lines.extend(quote! {{
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::String {
                            password: false,
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                }});

            }
            SettingsType::SettingsColor => {
                into_elements_lines.extend(quote! {
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::String {
                            password: false,
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }

            // 
            SettingsType::Key => {
                into_elements_lines.extend(quote! {
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::Key {
                            optional: false,
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }

            SettingsType::OptionalKey => {
                into_elements_lines.extend(quote! {
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::Key {
                            optional: true,
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }

            // dropdown menu
            SettingsType::Dropdown(d) => {
                if let Some(path) = d.path {
                    into_elements_lines.extend(quote! {
                        builder.add_item(BuildableSetting {
                            name: #text.to_owned(),
                            path: format!("{prefix}.{}", #prop_string),
                            setting_type: BuildableSettingType::Dropdown {
                                options: BuildableSettingDropdownOptions::Variable {
                                    var: #path .to_string(),
                                },
                            },

                            tooltip: None,
                            enabled_if: None,
                            visible_if: None,
                        });
                    });
                }
            }

            // sub settings, ie mania or taiko settings
            SettingsType::SubSetting => {
                // TODO:!!!!
                into_elements_lines.extend(quote! { 
                    builder.add_category(#text, None::<&str>);

                    self.#property.create_provider(
                        format!("{prefix}.{}", #prop_string),
                        builder,
                    );
                });
            }

            // button that performs an action
            SettingsType::Button => {
                let click = setting.click
                    .or(setting.action)
                    .expect("no click action??")
                    .parse::<TokenStream>()
                    .expect("invalid click action");

                into_elements_lines.extend(quote! { 
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::Button {
                            action: TatakuAction::from(#click).into(),
                        },

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }
            
            SettingsType::Divider => {
                into_elements_lines.extend(quote! { 
                    builder.add_item(BuildableSetting {
                        name: #text.to_owned(),
                        path: format!("{prefix}.{}", #prop_string),
                        setting_type: BuildableSettingType::Divider,

                        tooltip: None,
                        enabled_if: None,
                        visible_if: None,
                    });
                });
            }


            SettingsType::Category(c) => {
                let name = c.name;
                let icon = if let Some(icon) = c.icon {
                    quote!{ Some(#icon) }
                } else {
                    quote!{ None::<&str> }
                };
                into_elements_lines.extend(quote!{
                    builder.add_category(#name, #icon);
                });
            }



            // shrug
            SettingsType::Unknown => {},
        }

    }

    let all_lines = quote!{
        impl MakeSettingsMenu for #struct_name {
            fn create_provider(
                &self, 
                prefix: String,
                builder: &mut SettingsBuilder,
            ) {
                use crate::prelude::*;
                #into_elements_lines
            }
        }
    };

    std::fs::create_dir_all("/tmp/debug").unwrap();
    std::fs::write(format!("/tmp/debug/{struct_name}-settings_impl.rs"), all_lines.to_string()).unwrap();
    
    Ok(all_lines)
}


#[derive(Default)]
struct SettingsItem {
    /// the type for this setting item
    setting_type: SettingsType, 

    /// what is the name of the setting? 
    setting_name: Option<Ident>,

    /// what text to display
    setting_text: Option<String>,

    // /// what dropdown value to use if this is not a default dropdown value
    // dropdown_value: Option<String>,

    /// if this is a text input, should it be a password?
    password_input: Option<bool>,

    // optional input-setting variables
    range_min: Option<f64>,
    range_max: Option<f64>,
    width: Option<f64>,

    // used for buttons
    click: Option<String>,
    action: Option<String>,
}
impl SettingsItem {
    fn read(f: &Field) -> Result<Option<Self>> {
        let Some(field_name) = f.ident.as_ref()
        else { return Ok(None) };

        let mut setting = SettingsItem {
            setting_name: Some(field_name.clone()),
            ..Default::default()
        };

        // read the type
        if let Type::Path(path) = &f.ty {
            setting.setting_type = SettingsType::from(path.path.get_ident());
        }
    
        // read the attributes
        for attr in &f.attrs {
            let path = attr.path();
            if !(path.is_ident(SUBSETTING_ATTRIBUTE) || path.is_ident(SETTING_ATTRIBUTE)) { continue }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(CATEGORY_ATTRIBUTE) {
                    if let Ok(_) = meta.value() {
                        let name: LitStr = meta.input.parse()?;
                        let name = name.value();
                        setting.setting_type = SettingsType::Category(CategoryItem { 
                            name, 
                            ..Default::default() 
                        });
                    } else {
                        let c = CategoryItem::read(&meta)?;
                        setting.setting_type = SettingsType::Category(c);
                    }
                }
                else if meta.path.is_ident(TEXT_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let value: LitStr = meta.input.parse()?;

                    setting.setting_text = Some(value.value());
                }
                else if meta.path.is_ident(CLICK_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let value: LitStr = meta.input.parse()?;

                    setting.click = Some(value.value());
                }
                else if meta.path.is_ident(ACTION_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let value: LitStr = meta.input.parse()?;

                    setting.action = Some(value.value());
                }
                else if meta.path.is_ident(DROPDOWN_ATTRIBUTE) {
                    let d = DropdownItem::read(&meta)?;
                    setting.setting_type = SettingsType::Dropdown(d);
                }
                else if meta.path.is_ident(MIN_ATTRIBUTE) {
                    let _ = meta.value()?;

                    if let Ok(value) = meta.input.parse::<LitInt>() {
                        setting.range_min = Some(value.base10_parse::<u64>()? as f64);
                    } else if let Ok(value) = meta.input.parse::<LitFloat>() {
                        setting.range_min = Some(value.base10_parse::<f64>()?);
                    }
                }
                else if meta.path.is_ident(MAX_ATTRIBUTE) {
                    let _ = meta.value()?;

                    if let Ok(value) = meta.input.parse::<LitInt>() {
                        setting.range_max = Some(value.base10_parse::<u64>()? as f64);
                    } else if let Ok(value) = meta.input.parse::<LitFloat>() {
                        setting.range_max = Some(value.base10_parse::<f64>()?);
                    }
                }
                else if meta.path.is_ident(WIDTH_ATTRIBUTE) {
                    let _ = meta.value()?;

                    if let Ok(value) = meta.input.parse::<LitInt>() {
                        setting.width = Some(value.base10_parse::<u64>()? as f64);
                    } else if let Ok(value) = meta.input.parse::<LitFloat>() {
                        setting.width = Some(value.base10_parse::<f64>()?);
                    }
                }
                else if meta.path.is_ident(PASSWORD_ATTRIBUTE) {
                    let _ = meta.value()?;

                    if let Ok(value) = meta.input.parse::<LitBool>() {
                        setting.password_input = Some(value.value);
                    }
                } else {
                    return Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
                }

                Ok(())
            })?;

            if attr.path().is_ident(SUBSETTING_ATTRIBUTE) { 
                setting.setting_type = SettingsType::SubSetting;
            }

            return Ok(Some(setting));
        }
    
        Ok(None)
    }

}



#[derive(Debug, Clone, Default)]
enum SettingsType {
    Bool,
    U32,
    U64,
    F32,
    F64,
    Usize,
    String,
    
    OptionalKey,
    Key,
    Dropdown(DropdownItem),

    SettingsColor,
    Color,

    // special
    SubSetting,

    Button,
    Divider,
    Category(CategoryItem),

    #[default]
    Unknown
}
impl SettingsType {
    fn from(s: Option<&Ident>) -> Self {
        let Some(s) = s else { return Self::Unknown };

        match &*s.to_string() {
            "bool" => Self::Bool,
            "u32"  => Self::U32,
            "u64"  => Self::U64,
            "f32"  => Self::F32,
            "f64"  => Self::F64,
            "usize" => Self::Usize,
            "String" => Self::String,
            "Color" => Self::Color,
            "SettingsColor" => Self::SettingsColor,
            "Key" => Self::Key,
            "Option<Key>" => Self::OptionalKey,
            "SettingsButton" => Self::Button,
            "SettingsDivider" => Self::Divider,
            _ => Self::Unknown
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Usize => "usize",
            _ => ""
        }
    }

    fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }
}

#[derive(Debug, Clone, Default)]
struct DropdownItem {
    path: Option<String>,
}
impl DropdownItem {
    fn read(meta: &ParseNestedMeta<'_>) -> Result<Self> {
        let mut setting = Self::default();

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident(DROPDOWN_PATH_ATTRIBUTE) {
                let _ = meta.value()?;
                let value: LitStr = meta.input.parse()?;
                setting.path = Some(value.value());
            }  
            else {
                return Err(meta.error(format!("Invalid dropdown attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(setting)
    }
}


#[derive(Debug, Clone, Default)]
struct CategoryItem {
    name: String,
    icon: Option<String>,
}
impl CategoryItem {
    fn read(meta: &ParseNestedMeta<'_>) -> Result<Self> {
        let mut setting = Self::default();

        meta.parse_nested_meta(|meta| {
            if meta.path.is_ident(CATEGORY_NAME_ATTRIBUTE) {
                let _ = meta.value()?;
                let value: LitStr = meta.input.parse()?;
                setting.name = value.value();
            } 
            else {
                return Err(meta.error(format!("Invalid Category attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(setting)
    }
}
