use quote::*;
use syn::*;

const WIDTH:f64 = 600.0;

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
                                check!(val, dropdown_value, "dropdown_value");
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

    // TODO: !!!!! categories !!!!!
    let struct_name = ast.ident.to_string();
    let mut get_menu_items_lines = Vec::new();
    get_menu_items_lines.push(format!("impl {struct_name} {{"));
    get_menu_items_lines.push("pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {".to_owned());
    get_menu_items_lines.push("let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();".to_owned());
    get_menu_items_lines.push("let font = Font::Main;".to_owned());

    // pulling vals back from the menu
    let mut from_menu_lines = Vec::new();
    from_menu_lines.push("pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {".to_owned());

    for setting in settings {
        let text = setting.setting_text.unwrap_or_default();
        let property = setting.setting_name.clone();
        let mut add = true;

        if let Some(category) = setting.category {
            get_menu_items_lines.push(format!("list.push(Box::new(MenuSection::new(p, 80.0, \"{category}\", Color::BLACK, Font::Main)));"));
        }

        // comment what this item is
        get_menu_items_lines.push(format!("\n// {property}"));
        from_menu_lines.push(format!("\n// {property}"));

        match setting.setting_type {
            // checkbox
            SettingsType::Bool => {
                let w = float(setting.width.unwrap_or(600.0));
                let size = format!("Vector2::new({w}, 50.0)");
                get_menu_items_lines.push(format!("let mut i = Checkbox::new(p, {size}, \"{text}\", self.{property}, Font::Main);"));

                from_menu_lines.push(format!("
                if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                    let val = val.downcast_ref::<bool>().expect(&format!(\"error downcasting for {property}\"));

                    self.{property} = val.clone();
                }}"))
            }

            // slider
            f
            @(SettingsType::U32
            | SettingsType::U64
            | SettingsType::F32
            | SettingsType::Usize
            | SettingsType::F64) => {
                let t = f.to_str();

                let w = float(setting.width.unwrap_or(WIDTH));
                let size = format!("Vector2::new({w}, 50.0)");

                let range = if let Some((min, max)) = setting.range_min.zip(setting.range_max) {
                    let min = float(min);
                    let max = float(max);
                    format!("Some({min}..{max})")
                } else {
                    "None".to_owned()
                };

                // TODO: snapping?
                get_menu_items_lines.push(format!("let mut i = Slider::new(p, {size}, \"{text}\", self.{property} as f64, {range}, None, Font::Main);"));

                from_menu_lines.push(format!("
                if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                    let val = val.downcast_ref::<f64>().expect(&format!(\"error downcasting for {property}\"));

                    self.{property} = (*val) as {t};
                }}"))
            }

            // text input
            SettingsType::String => {
                let w = float(setting.width.unwrap_or(WIDTH));
                let size = format!("Vector2::new({w}, 50.0)");

                get_menu_items_lines.push(format!("let mut i = TextInput::new(p, {size}, \"{text}\", &self.{property}, Font::Main);"));

                if setting.password_input == Some(true) {
                    get_menu_items_lines.push("i.is_password = true;".to_owned());
                }

                from_menu_lines.push(format!("
                if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                    let val = val.downcast_ref::<String>().expect(&format!(\"error downcasting for {property}\"));
                    self.{property} = val.clone();
                }}"))
            }

            // color input
            SettingsType::Color => {
                let w = float(setting.width.unwrap_or(WIDTH));
                let size = format!("Vector2::new({w}, 50.0)");
                get_menu_items_lines.push(format!("let s:String = self.{property}.into(); let mut i = TextInput::new(p, {size}, \"{text}\", &s, Font::Main);"));

                from_menu_lines.push(format!("
                {{
                    let val = list.get_tagged(prefix.clone() + \"{property}\"); // get item from list
                    let val = val.first().expect(\"error getting tagged\"); // unwrap
                    let val = val.get_value(); // get the value from the item
                    let val = val.downcast_ref::<String>().expect(&format!(\"error downcasting for Color (String)\"));

                    self.{property} = val.clone().into();
                }}"));
            }
            //
            SettingsType::Key => {
                let w = float(setting.width.unwrap_or(WIDTH));
                let size = format!("Vector2::new({w}, 50.0)");
                get_menu_items_lines.push(format!("let mut i = KeyButton::new(p, {size}, self.{property}, \"{text}\", Font::Main);"));

                from_menu_lines.push(format!("
                if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                    let val = val.downcast_ref::<Key>().expect(&format!(\"error downcasting for {property}\"));
                    self.{property} = val.clone();
                }}"))
            }

            // dropdown menu (obviously)
            SettingsType::Dropdown(enum_name) => {
                let width = float(setting.width.unwrap_or(WIDTH));
                let font_size = "20.0";

                let e = if let Some(s) = setting.dropdown_value.clone() {
                    format!("{enum_name}::{s}(self.{property}.clone())")
                } else {
                    format!("self.{property}.clone()")
                };

                get_menu_items_lines.push(format!("let mut i = Dropdown::<{enum_name}>::new(p, {width}, {font_size}, \"{text}\", Some({e}), Font::Main);"));

                if let Some(override_) = setting.dropdown_value {
                    from_menu_lines.push(format!("
                    if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                        let val = val.downcast_ref::<Option<{enum_name}>>().expect(&format!(\"error downcasting for {property}\"));

                        if let Some({enum_name}::{override_}(val)) = val {{
                            self.{property} = val.to_owned();
                        }}
                    }}"))
                } else {
                    from_menu_lines.push(format!("
                    if let Some(val) = list.get_tagged(prefix.clone() + \"{property}\").first().map(|i|i.get_value()) {{
                        let val = val.downcast_ref::<Option<{enum_name}>>().expect(&format!(\"error downcasting for {property}\"));

                        if let Some(val) = val {{
                            self.{property} = val.to_owned();
                        }}
                    }}"))
                }
            }

            // sub settings, ie mania or taiko settings
            SettingsType::SubSetting => {
                get_menu_items_lines.push(format!("list.extend(self.{property}.get_menu_items(p, \"{property}\".to_owned() + &prefix, sender.clone()));"));
                add = false;

                from_menu_lines.push(format!("self.{property}.from_menu(\"{property}\".to_owned() + &prefix, list);"));
            }

            // button that performs an action
            SettingsType::Button => {
                let w = float(setting.width.unwrap_or(600.0));
                let size = format!("Vector2::new({w}, 50.0)");
                get_menu_items_lines.push(format!("let mut i = MenuButton::new(p, {size}, \"{text}\", Font::Main);"));
                if let Some(action) = setting.action {
                    get_menu_items_lines.push(format!("i.on_click = Arc::new({action});"));
                }
                get_menu_items_lines.push(format!("list.push(Box::new(i));"));

                add = false;
            }

            // shrug
            // SettingsType::Vec(_) => {},
            SettingsType::Unknown => {},
        }

        if add {
            get_menu_items_lines.push(format!("i.set_tag(&(prefix.clone() + \"{property}\"));"));

            get_menu_items_lines.push(format!("let c = sender.clone();"));
            get_menu_items_lines.push(format!("i.on_change = Arc::new(move|_|{{c.send(()).unwrap()}});"));

            get_menu_items_lines.push(format!("list.push(Box::new(i));"));
        }
    }

    if let Some(extra) = get_items_extra { get_menu_items_lines.push("list.extend(self.".to_owned() + &extra + "(p, prefix, sender));"); }

    get_menu_items_lines.push("list".to_owned());
    get_menu_items_lines.push("}".to_owned());


    if let Some(extra) = from_menu_extra { from_menu_lines.push("self.".to_owned() + &extra + "(prefix, list);"); }
    from_menu_lines.push("}".to_owned());
    get_menu_items_lines.extend(from_menu_lines);

    get_menu_items_lines.push("}".to_owned());

    let all_lines = get_menu_items_lines.join("\n");

    #[cfg(feature="extra_debugging")] {
        std::fs::create_dir_all("./debug").unwrap();
        std::fs::write(format!("./debug/{struct_name}.rs", ), &all_lines).unwrap();
    }

    let impl_tokens = all_lines.parse::<proc_macro2::TokenStream>().unwrap();
    quote! { #impl_tokens }
}


fn float(n:f64) -> String {
    let mut n = n.to_string();
    if !n.contains(".") { n += ".0" }
    n
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

    /// what dropdown value to use if this is not a default dropdown value
    dropdown_value: Option<String>,

    /// if this is a text input, should it be a password?
    password_input: Option<bool>,

    // optional input-setting variables
    range_min: Option<f64>,
    range_max: Option<f64>,
    width: Option<f64>,

    // used for buttons
    action: Option<String>
}


#[derive(Debug, Clone)]
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
    Color,

    Button,

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
}

impl Default for SettingsType {
    fn default() -> Self {
        Self::Unknown
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
