use quote::*;
use syn::*;

pub(crate) fn impl_settings(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let mut settings:Vec<SettingsItem> = Vec::new();
    // let mut categories = HashMap::new();

    let mut get_items_extra = None;
    let mut from_menu_extra = None;

    for attr in &ast.attrs {
        if attr.path.is_ident("Setting") {
            if let Ok(Meta::List(list)) = attr.parse_meta() {
                for name_value in recurse_meta(list) {
                    match &name_value.lit {
                        Lit::Str(str) if name_value.path.is_ident("get_items") => get_items_extra = Some(str.value()),
                        Lit::Str(str) if name_value.path.is_ident("from_menu") => from_menu_extra = Some(str.value()),
                        _ => {}
                    }
                }
            }
        }
    }

    if let Data::Struct(data) = &ast.data {
        // go through settings
        for f in data.fields.iter() {
            let mut setting = SettingsItem::default();
            let field_name = f.ident.as_ref().unwrap().to_string();
            setting.setting_name = field_name.clone();

            // read the type
            match &f.ty {
                Type::Path(path) => setting.setting_type = SettingsType::from(path.path.get_ident()),
                Type::Tuple(_) => setting.setting_type = SettingsType::Button,
                _ => {}
            }
        
            // read the attributes
            for attr in &f.attrs {
                if attr.path.is_ident("Subsetting") { 
                    setting.setting_type = SettingsType::SubSetting;

                    // check for category
                    if let Ok(Meta::List(list)) = attr.parse_meta() {
                        for name_value in recurse_meta(list) {
                            match &name_value.lit {
                                Lit::Str(str) if name_value.path.is_ident("category") => setting.category = Some(str.value()),
                                _ => {}
                            }
                        }
                    }

                    settings.push(setting);
                    break;
                }

                if !attr.path.is_ident("Setting") { continue }

                if let Ok(Meta::List(list)) = attr.parse_meta() {

                    for name_value in recurse_meta(list) {
                        macro_rules! check {
                            ($val:expr, $setting:ident, $ident:expr) => {
                                if name_value.path.is_ident($ident) {
                                    setting.$setting = Some($val.clone());
                                    continue;
                                }
                            };

                            ($i:expr, $setting:ident, $ident:expr, $n:ident) => {
                                if name_value.path.is_ident($ident) {
                                    setting.$setting = Some($i.base10_parse::<$n>().unwrap());
                                    continue;
                                }
                            }
                        }

                        match &name_value.lit {
                            Lit::Str(str) => {
                                let val = str.value();

                                // check!(val, setting_path, "path");
                                check!(val, category, "category");
                                check!(val, setting_text, "text");
                                // check!(val, dropdown_value, "dropdown_value");
                                check!(val, action, "action");

                                if name_value.path.is_ident("dropdown") {
                                    setting.setting_type = SettingsType::Dropdown(val);
                                    continue;
                                }
                            }
                            Lit::Int(i) => {
                                check!(i, range_min, "min", f64);
                                check!(i, range_max, "max", f64);
                                check!(i, width, "width", f64);
                            }
                            Lit::Float(f) => {
                                check!(f, range_min, "min", f64);
                                check!(f, range_max, "max", f64);
                                check!(f, width, "width", f64);
                            }
                            Lit::Bool(b) => {
                                let val = b.value;
                                check!(val, password_input, "password");
                            },
                            
                            _ => {}
                        }
                        
                        panic!("Unknown parameter {}={}", name_value.path.get_ident().unwrap().to_string(), name_value.lit.to_token_stream().to_string())
                    }
                
                }
                // println!("{:#?}", setting);

                settings.push(setting);
                break;
            }
        
        }
    } else {
        panic!("tf you doin")
    }


    let struct_name = ast.ident.to_string();
    let mut into_elements_lines = vec!["
        pub fn into_elements(
            &self, 
            prefix: String,
            filter: &ItemFilter, 
            owner: MessageOwner, 
            builder: &mut SettingsBuilder,
        ) {
            use crate::prelude::iced_elements::*;
            const FONT_SIZE:f32 = 30.0;
    ".to_owned()];
    
    // pulling vals back from the menu
    let mut from_elements_lines = vec!["
        pub fn from_elements<'a>(
            &mut self,
            // tags of the current property, with all previous prefixes removed 
            tags: &mut impl Iterator<Item = &'a str>,
            // message that contains the data
            message: Message,
        ) {
            let Some(tag) = tags.next() else { return };
            match tag {
    ".to_owned()];

    for setting in settings {
        let text = setting.setting_text.unwrap_or_default();
        let property = setting.setting_name.clone();

        if let Some(category) = setting.category {
            into_elements_lines.push(format!(r#"builder.add_category("{category}");"#));
        }

        // comment what this item is
        into_elements_lines.push(format!("\n// {property}"));
        from_elements_lines.push(format!("\n// {property}"));

        match setting.setting_type {
            // checkbox
            SettingsType::Bool => {
                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();

                        builder.add_item(
                            // Text::new("{text}").size(FONT_SIZE).into_element(),
                            Checkbox::new(
                                "{text}",
                                self.{property}
                            )
                            .on_toggle(move|b| Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Toggle(b)))
                            .text_size(FONT_SIZE)
                            .into_element(),
                            Space::new(Shrink, Shrink).into_element(),
                        );
                    }}
                "#));
                
                from_elements_lines.push(format!(r#"
                    "{property}" => if let Some(b) = message.message_type.as_toggle() {{ self.{property} = b }},
                "#));
            }

            // slider
            f 
            @(SettingsType::U32 
            | SettingsType::U64 
            | SettingsType::Usize 
            | SettingsType::F32 
            | SettingsType::F64) => {
                let ty = f.to_str();

                let min = setting.range_min.unwrap_or(0.0);
                let max = setting.range_max.unwrap_or(100.0);
                
                let step = if f.is_float() {"0.01"} else {"1.0"};


                // TODO: step?
                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();
                        
                        builder.add_item(
                            Text::new(format!("{text} ({{:.2}})", self.{property})).size(FONT_SIZE).into_element(),
                            Slider::new(
                                ({min}f32)..=({max}f32),
                                self.{property} as f32,
                                move|v| Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Float(v))
                            )
                            .step({step})
                            .into_element()
                        );
                    }}
                "#));
                
                from_elements_lines.push(format!(r#"
                    "{property}" => if let Some(n) = message.message_type.as_float() {{ self.{property} = n as {ty} }},
                "#));
            }

            // text input
            SettingsType::String => {
                let do_password = if setting.password_input == Some(true) {"true"} else {"false"};
                
                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();
                        
                        builder.add_item(
                            Text::new("{text}").size(FONT_SIZE).into_element(),
                            TextInput::new(
                                "  ", // no placeholder
                                &self.{property},
                            )
                            .size(FONT_SIZE)
                            .on_input(move|t| Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Text(t)))
                            .secure({do_password})
                            .into_element()
                        );
                    }}
                "#));
                
                from_elements_lines.push(format!(r#"
                    "{property}" => if let Some(t) = message.message_type.as_text() {{ self.{property} = t }},
                "#));
            }

            // color input
            SettingsType::Color => {
                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();
                        let color:String = self.{property}.into();

                        builder.add_item(
                            Text::new("{text}").size(FONT_SIZE).into_element(),
                            TextInput::new(
                                "  ", // no placeholder
                                &color,
                            )
                            .size(FONT_SIZE)
                            .on_input(move|t| Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Text(t)))
                            .into_element()
                        );
                    }}
                "#));

                from_elements_lines.push(format!(r#"
                    "{property}" => if let Some(t) = message.message_type.as_text() {{ self.{property} = Color::from_hex(t) }},
                "#));
            }
            SettingsType::SettingsColor => {
                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();
                        
                        builder.add_item(
                            Text::new("{text}").size(FONT_SIZE).into_element(),
                            TextInput::new(
                                "  ", // no placeholder
                                &self.{property}.string,
                            )
                            .size(FONT_SIZE)
                            .on_input(move|t| Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Text(t)))
                            .into_element()
                        );
                    }}
                "#));

                from_elements_lines.push(format!(r#"
                    "{property}" => if let Some(t) = message.message_type.as_text() {{ self.{property}.update(t) }},
                "#));
            }

            // 
            SettingsType::Key => {
                //TODO: !!!!!!!!!!!!!

                // get_menu_items_lines.push(format!("let mut i = KeyButton::new(p, {size}, self.{property}, \"{text}\", Font::Main);"));

                // from_menu_lines.push(format!("
                // if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                //     let val = val.downcast_ref::<Key>().expect(&format!(\"error downcasting for {property}\"));
                //     self.{property} = val.clone(); 
                // }}"))
            }

            // dropdown menu
            SettingsType::Dropdown(enum_name) => {
                // let enum_name = setting.dropdown_value.unwrap_or(enum_name);

                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();
                        
                        builder.add_item(
                            Text::new("{text}").size(FONT_SIZE).into_element(),
                            Dropdown::new(
                                {enum_name}::variants(),
                                Some(self.{property}.clone()),
                                move|v| Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Custom(Arc::new(v)))
                            )
                            .text_size(FONT_SIZE)
                            .into_element()
                        );
                    }}
                "#));

                from_elements_lines.push(format!(r#"
                    "{property}" => {{
                        let v = message.message_type.downcast::<<{enum_name} as Dropdownable2>::T>();
                        self.{property} = (*v).clone();
                    }}
                "#));
            }

            // sub settings, ie mania or taiko settings
            SettingsType::SubSetting => {
                into_elements_lines.push(format!(r#"
                    self.{property}.into_elements(
                        format!("{{prefix}}.{property}"),
                        filter,
                        owner,
                        builder,
                    );
                "#));

                from_elements_lines.push(format!(r#"
                    "{property}" => self.{property}.from_elements(tags, message),
                "#));
            }

            // button that performs an action
            SettingsType::Button => {
                into_elements_lines.push(format!(r#"
                    if filter.check("{text}") {{
                        let prefix = prefix.clone();
                        
                        builder.add_item(
                            Text::new(" ").size(FONT_SIZE).into_element(),
                            Button::new(Text::new("{text}").size(FONT_SIZE))
                            .on_press(Message::new(owner, format!("{{prefix}}.{property}"), MessageType::Click))
                            .into_element()
                        );
                    }}
                "#));

                if let Some(action) = setting.action {
                    from_elements_lines.push(format!(r#"
                        "{property}" => {{ {action}; }},
                    "#));
                }

            }

            // shrug
            // SettingsType::Vec(_) => {},
            SettingsType::Unknown => {},
        }

    }

    // if let Some(extra) = get_items_extra { get_menu_items_lines.push("list.extend(self.".to_owned() + &extra + "(p, prefix, sender));"); }
    // if let Some(extra) = from_menu_extra { from_menu_lines.push("self.".to_owned() + &extra + "(prefix, list);"); }

    into_elements_lines.push("}".to_owned());
    let into_elements_lines = into_elements_lines.join("\n");


    from_elements_lines.push(" _ => {}".to_owned());
    from_elements_lines.push("}".to_owned());
    from_elements_lines.push("}".to_owned());
    let from_elements_lines = from_elements_lines.join("\n");
    
    // from_menu_lines.push("}".to_owned());
    // get_menu_items_lines.extend(from_menu_lines);

    // get_menu_items_lines.push("}".to_owned());
    // let all_lines = get_menu_items_lines.join("\n");

    let all_lines = format!(r#"
        impl {struct_name} {{
            {into_elements_lines}
            
            {from_elements_lines}
        }}
    "#);

    #[cfg(feature="extra_debugging")] {
        std::fs::create_dir_all("./debug").unwrap();
        std::fs::write(format!("./debug/{struct_name}-settings_impl.rs", ), &all_lines).unwrap();
    }

    let impl_tokens = all_lines.parse::<proc_macro2::TokenStream>().unwrap();
    quote! { #impl_tokens }
}


#[derive(Default)]
struct SettingsItem {
    /// the type for this setting item
    setting_type: SettingsType, 

    /// what is the name of the setting? 
    setting_name: String,

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
    action: Option<String>
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
    // Vec(Box<SettingsType>),
    
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
    fn from(s:Option<&Ident>) -> Self {
        if let None = s { return Self::Unknown }

        match &*s.unwrap().to_string() {
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
        match self {
            Self::F32 | Self::F64 => true,
            _ => false,
        }
    }
}



fn recurse_meta(meta: MetaList) -> Vec<MetaNameValue> {
    let mut list = Vec::new();

    for i in meta.nested {
        if let NestedMeta::Meta(m) = i {
            // println!("meta: {}", m.to_token_stream().to_string());
            match m {
                Meta::List(l) => list.extend(recurse_meta(l)),
                Meta::NameValue(nv) => { list.push(nv) }
                _o => {
                    // println!("got other: {}", _o.to_token_stream().to_string())
                }
            }
        }
    }
    
    list
}
