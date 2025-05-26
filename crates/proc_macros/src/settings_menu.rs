use quote::*;
use syn::{ spanned::Spanned, * };


const CATEGORY_ATTRIBUTE:&str = "category";
const TEXT_ATTRIBUTE:&str = "text";
const DROPDOWN_ATTRIBUTE:&str = "dropdown";
const ACTION_ATTRIBUTE:&str = "action";
const CLICK_ATTRIBUTE:&str = "click";

const MIN_ATTRIBUTE:&str = "min";
const MAX_ATTRIBUTE:&str = "max";
const WIDTH_ATTRIBUTE:&str = "width";
const PASSWORD_ATTRIBUTE:&str = "password";


const SETTING_ATTRIBUTE:&str = "setting";
const SUBSETTING_ATTRIBUTE:&str = "subsetting";


pub(crate) fn impl_settings(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let mut settings: Vec<SettingsItem> = Vec::new();

    // let mut categories = HashMap::new();

    // let mut get_items_extra = None;
    // let mut from_menu_extra = None;

    // for attr in &ast.attrs {
    //     if attr.path.is_ident("Setting") {
    //         if let Ok(Meta::List(list)) = attr.parse_meta() {
    //             for name_value in recurse_meta(list) {
    //                 match &name_value.lit {
    //                     Lit::Str(str) if name_value.path.is_ident("get_items") => get_items_extra = Some(str.value()),
    //                     Lit::Str(str) if name_value.path.is_ident("from_menu") => from_menu_extra = Some(str.value()),
    //                     _ => {}
    //                 }
    //             }
    //         }
    //     }
    // }

    if let Data::Struct(data) = &ast.data {
        // go through settings
        for f in data.fields.iter() {
            let Some(field_name) = f.ident.as_ref() else { continue };
            let mut setting = SettingsItem {
                setting_name: Some(field_name.clone()),
                ..Default::default()
            };

            // read the type
            match &f.ty {
                Type::Path(path) => setting.setting_type = SettingsType::from(path.path.get_ident()),
                Type::Tuple(_) => setting.setting_type = SettingsType::Button,
                _ => {}
            }
        
            // read the attributes
            for attr in &f.attrs {
                let path = attr.path();
                if !(path.is_ident(SUBSETTING_ATTRIBUTE) || path.is_ident(SETTING_ATTRIBUTE)) { continue }

                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(CATEGORY_ATTRIBUTE) {
                        let _ = meta.value()?;
                        let name: LitStr = meta.input.parse()?;

                        setting.category = Some(name.value());
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
                        let _ = meta.value()?;
                        let value: LitStr = meta.input.parse()?;

                        setting.setting_type = SettingsType::Dropdown(value.value());
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
                    }

                    Ok(())
                })?;


                if attr.path().is_ident(SUBSETTING_ATTRIBUTE) { 
                    setting.setting_type = SettingsType::SubSetting;

                    settings.push(setting);
                    break;
                }

                settings.push(setting);
                break;
            }
        }
    } else {
        return Err(Error::new(ast.span(), "Settings can only be derived on a struct"));
    }


    let struct_name = &ast.ident;
    let mut into_elements_lines = proc_macro2::TokenStream::new();
    let mut from_elements_lines = proc_macro2::TokenStream::new();

    for setting in settings {
        let text = setting.setting_text.unwrap_or_default();
        let property = setting.setting_name.clone().unwrap();

        if let Some(category) = setting.category {
            into_elements_lines.extend(quote!{
                builder.add_category(#category);
            });
        }

        let prop_string = property.to_string();

        match setting.setting_type {
            // checkbox
            SettingsType::Bool => {
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();

                    let prop_str = format!("{prefix}.{}", #prop_string);

                    let checkbox = builder.create_checkbox(
                        CheckboxBuilder::new(
                            #text,
                            prop_str.clone()
                        )
                        .on_change(Box::new(move |b| Message::new(
                            owner, 
                            prop_str.clone(), 
                            MessageValue::Toggle(b)
                        )))
                        .font_size(FONT_SIZE)
                    );

                    let other = builder.create_empty();
                    builder.add_item(checkbox, other, #text);
                }});
                
                from_elements_lines.extend(quote! {
                    #prop_string => if let Some(b) = message.value.as_toggle() { self.#property = b },
                });
            }

            // slider
            f 
            @(SettingsType::U32 
            | SettingsType::U64 
            | SettingsType::Usize 
            | SettingsType::F32 
            | SettingsType::F64) => {
                let ty = format_ident!("{}", f.to_str());

                let min = setting.range_min.unwrap_or(0.0);
                let max = setting.range_max.unwrap_or(100.0);
                
                let step = if f.is_float() {0.01f32} else {1.0};
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();

                    let text = builder.create_text(
                        TextBuilder::new(
                            TextBuilderValue::List(vec![
                                TextBuilderValue::Static(format!("{} (", #text)),
                                TextBuilderValue::Calc(format!("{prefix}.{}", #prop_string)),
                                TextBuilderValue::Static(String::from(")")),
                            ], String::new())
                        )
                        .font_size(FONT_SIZE)
                    );
                    let prop_str = format!("{prefix}.{}", #prop_string);
                    let prop_str2 = prop_str.clone();

                    let b: Box<dyn Fn(&f32) -> Message + Send + Sync> = Box::new(move |v| Message::new(owner, prop_str.clone(), MessageValue::Float(*v)));
                    let slider = builder.create_slider(
                        SliderBuilder::new(
                            (#min as f32)..=(#max as f32),
                            prop_str2,
                        )
                        .on_change(b)
                        .step(#step) 
                    );
                    builder.add_item(text, slider, #text);
                }});
                
                from_elements_lines.extend(quote! {
                    #prop_string => if let Some(n) = message.value.as_float() { self.#property = n as #ty },
                });
            }

            // text input
            SettingsType::String => {
                let do_password = setting.password_input == Some(true);
                
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();
                
                    let text = builder.create_text(
                        TextBuilder::new(#text)
                        .font_size(FONT_SIZE)
                    );

                    let prop_str = format!("{prefix}.{}", #prop_string);
                    let prop_str2 = prop_str.clone();
                    let b:Box<dyn Fn(&String) -> Message + Send + Sync> = Box::new(move |t| Message::new(
                        owner, 
                        prop_str.clone(), 
                        MessageValue::Text(t.to_string())
                    ));

                    let input = builder.create_text_input(
                        TextInputBuilder::new("", TextBuilderValue::Variable(prop_str2))
                        .on_input(b)
                        .secure(#do_password)
                        .font_size(FONT_SIZE)
                    );
                    builder.add_item(text, input, #text);
                }});
                
                from_elements_lines.extend(quote! {
                    #prop_string => if let Some(t) = message.value.as_text() { self.#property = t },
                });
            }

            // color input
            SettingsType::Color => {
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();
                    let color:String = self.#property.into();

                    let text = builder.create_text(
                        TextBuilder::new(#text)
                        .font_size(FONT_SIZE)
                    );

                    let b:Box<dyn Fn(&String) -> Message + Send + Sync> = Box::new(move|t| Message::new(owner, format!("{prefix}.{}", #prop_string), MessageValue::Text(t.to_string())));
                    // TODO: 
                    let input = builder.create_text_input(
                        TextInputBuilder::new("", color)
                        .on_input(b)
                        .font_size(FONT_SIZE)
                    );
                    builder.add_item(text, input, #text);
                }});

                from_elements_lines.extend(quote! { 
                    #prop_string => if let Some(t) = message.value.as_text() { self.#property = Color::from_hex(t) },
                });
            }
            SettingsType::SettingsColor => {
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();

                    let text = builder.create_text(
                        TextBuilder::new(#text)
                        .font_size(FONT_SIZE)
                    );

                    let b:Box<dyn Fn(&String) -> Message + Send + Sync> = Box::new(move|t| Message::new(owner, format!("{prefix}.{}", #prop_string), MessageValue::Text(t.to_string())));

                    // TODO: impl reflect on settings color (?)
                    let input = builder.create_text_input(
                        TextInputBuilder::new("", &self.#property.string)
                        .on_input(b)
                        .font_size(FONT_SIZE)
                    );
                    builder.add_item(text, input, #text);
                }});

                from_elements_lines.extend(quote! {
                    #prop_string => if let Some(t) = message.value.as_text() { self.#property.update(t) },
                });
            }

            // 
            SettingsType::Key => {
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();

                    let text = builder.create_text(
                        TextBuilder::new(#text)
                        .font_size(FONT_SIZE)
                    );

                    let prop_str = format!("{prefix}.{}", #prop_string);

                    let prop2 = prop_str.clone();
                    let change: Box<dyn Fn(&Option<Key>) -> Message + Send + Sync> = Box::new(move |key| Message::new(
                        owner, 
                        prop2.clone(), 
                        MessageValue::Key(key.unwrap())
                    ));

                    let key_button = builder.create_key_button(
                        KeyButtonBuilder::new(prop_str.clone())
                        .on_change(change)
                    );

                    builder.add_item(text, key_button, #text);
                }});
                
                from_elements_lines.extend(quote! {
                    #prop_string => if let Some(k) = message.value.as_key() { self.#property = k; },
                });
            }

            SettingsType::OptionalKey => {
                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();

                    let text = builder.create_text(
                        TextBuilder::new(#text)
                        .font_size(FONT_SIZE)
                    );

                    let prop_str = format!("{prefix}.{}", #prop_string);

                    let prop2 = prop_str.clone();
                    let change: Box<dyn Fn(&Option<Key>) -> Message + Send + Sync> = Box::new(move |key| Message::new(
                        owner, 
                        prop2.clone(), 
                        key.copied().map(MessageValue::Key).unwrap_or(MessageValue::Click)
                    ));

                    let key_button = builder.create_key_button(
                        KeyButtonBuilder::new(prop_str.clone())
                        .on_change(change)
                    );

                    builder.add_item(text, key_button, #text);
                }});
                
                from_elements_lines.extend(quote! {
                    #prop_string => self.#property = message.value.as_key(),
                });
            }

            // dropdown menu
            SettingsType::Dropdown(enum_name) => {
                // let enum_name = setting.dropdown_value.unwrap_or(enum_name);
                let enum_ident = format_ident!("{enum_name}");

                into_elements_lines.extend(quote! {{
                    let prefix = prefix.clone();
                    let prefix2 = prefix.clone();

                    let text = builder.create_text(
                        TextBuilder::new(#text)
                            .font_size(FONT_SIZE)
                    );

                    let variants = #enum_ident::variants();
                    let texts = variants.iter().map(|i| format!("{i}")).collect::<Vec<_>>();

                    // let current = variants.iter().enumerate()
                    //     .find(|(_, i)| *i == &self.#property)
                    //     .map(|(n,_)|n)
                    //     ;
                    let change: Box<dyn Fn(usize) -> Message + Send + Sync> = 
                        Box::new(move |i| Message::new(owner, format!("{prefix2}.{}", #prop_string), MessageValue::Custom(Arc::new(variants[i].clone()))));

                    let dropdown = builder.create_dropdown(
                        DropdownBuilder::new(
                            texts,
                            format!("{prefix}.{}", #prop_string)
                        )
                        .on_change(change)
                        .font_size(FONT_SIZE)
                    );

                    builder.add_item(text, dropdown, #text);
                }});

                from_elements_lines.extend(quote! {
                    #prop_string => {
                        let v = message.value.downcast::<<#enum_ident as Dropdownable2>::T>();
                        self.#property = (*v).clone();
                    }
                });
            }

            // sub settings, ie mania or taiko settings
            SettingsType::SubSetting => {
                into_elements_lines.extend(quote! { 
                    self.#property.into_elements(
                        format!("{prefix}.{}", #prop_string),
                        owner,
                        builder,
                    );
                });

                from_elements_lines.extend(quote! { 
                    #prop_string => self.#property.from_elements(tags, message, shell),
                });
            }

            // button that performs an action
            SettingsType::Button => {
                into_elements_lines.extend(quote! { 
                    let prefix = prefix.clone();
                    
                    let empty = builder.create_empty();
                    let text = builder.create_text(
                        TextBuilder::new(#text)
                        .font_size(FONT_SIZE)
                    );

                    let button = builder.create_button(
                        ButtonBuilder::new(text)
                        .on_press(Message::new(owner, format!("{prefix}.{}", #prop_string), MessageValue::Click))
                    );

                    builder.add_item(empty, button, #text);
                });

                if let Some(click) = setting.click {
                    let click = click.parse::<proc_macro2::TokenStream>().unwrap();
                    from_elements_lines.extend(quote! { 
                        #prop_string => { #click; },
                    });
                } else if let Some(action) = setting.action {
                    let action = action.parse::<proc_macro2::TokenStream>().unwrap();
                    from_elements_lines.extend(quote! { 
                        #prop_string => { shell.actions.push(#action); },
                    });
                }
            }

            // shrug
            // SettingsType::Vec(_) => {},
            SettingsType::Unknown => {},
        }

    }

    // if let Some(extra) = get_items_extra { get_menu_items_lines.push("list.extend(self.".to_owned() + &extra + "(p, prefix, sender));"); }
    // if let Some(extra) = from_menu_extra { from_menu_lines.push("self.".to_owned() + &extra + "(prefix, list);"); }


    let all_lines = quote!{
        impl MakeSettingsMenu for #struct_name {
            fn into_elements(
                &self, 
                prefix: String,
                owner: MessageOwner, 
                builder: &mut SettingsBuilder,
            ) {
                use crate::prelude::*;
                use crate::prelude::ui::*;
                const FONT_SIZE:f32 = 30.0;
                #into_elements_lines
            }
            
            fn from_elements<'a>(
                &mut self,
                // tags of the current property, with all previous prefixes removed 
                tags: &mut ReflectPath,//impl Iterator<Item = &'a str>,
                // message that contains the data
                message: Message,
                shell: &mut GenericShell
            ) {
                use crate::prelude::*;
                use crate::prelude::ui::*;
                let Some(tag) = tags.next() else { return };
                match tag {
                    #from_elements_lines
                    
                    _ => {}
                }
            }
        }
    };

    
    // std::fs::create_dir_all("/tmp/debug").unwrap();
    // std::fs::write(format!("/tmp/debug/{struct_name}-settings_impl.rs"), all_lines.to_string()).unwrap();
    
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

    /// does this setting belong to a category?
    category: Option<String>,

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
    Dropdown(String),
    SubSetting,

    SettingsColor,
    Color,

    Button,

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

