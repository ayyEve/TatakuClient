use syn::*;
use super::*;
use quote::*;
use proc_macro2::TokenStream;


const MIN_ATTRIBUTE:&str = "min";
const MAX_ATTRIBUTE:&str = "max";
const RANGE_ATTRIBUTE:&str = "range";
const PASSWORD_ATTRIBUTE:&str = "password";

#[derive(Debug, Clone, Default)]
pub(super) struct NormalItem {
    common: CommonItems,

    /// the inner type
    inner: NormalItemType,

    /// if this is a text input, should it be a password?
    password_input: Option<bool>,

    // optional input-setting variables
    range_min: Option<f32>,
    range_max: Option<f32>,
}
impl NormalItem {
    pub fn new(field: &Field) -> Self {
        let mut inner = NormalItemType::Unknown;
        if let Type::Path(p) = &field.ty {
            inner = NormalItemType::from(p.path.get_ident());
        }

        Self {
            inner,
            ..Default::default()
        }
    }
    pub fn common(&self) -> &CommonItems { &self.common }
    
    
    pub fn read(mut self, attr: &Attribute) -> Result<Self> {
        self.common.add_item = true;
        
        attr.parse_nested_meta(|meta| {
            if self.common.try_read(&meta)? { return Ok(()) }

            if meta.path.is_ident(MIN_ATTRIBUTE) {
                let _ = meta.value()?;

                if let Ok(value) = meta.input.parse::<LitFloat>() {
                    self.range_min = Some(value.base10_parse::<i64>()? as f32);
                } else {
                    panic!("bad num")
                }
            } else if meta.path.is_ident(MAX_ATTRIBUTE) {
                let _ = meta.value()?;

                if let Ok(value) = meta.input.parse::<LitFloat>() {
                    self.range_max = Some(value.base10_parse::<i64>()? as f32);
                } else {
                    panic!("bad num")
                }
            } else if meta.path.is_ident(PASSWORD_ATTRIBUTE) {
                let _ = meta.value()?;
                if let Ok(value) = meta.input.parse::<LitBool>() {
                    self.password_input = Some(value.value);
                }
            } else if meta.path.is_ident(RANGE_ATTRIBUTE) {
                use syn::punctuated::Punctuated;
                let list;
            
                parenthesized!(list in meta.input);
                let list: Punctuated<LitFloat, Token![,]> = list
                    .call(Punctuated::parse_separated_nonempty)?;

                let list = list
                    .into_iter()
                    .filter_map(|i| i.base10_parse::<f32>().ok())
                    .collect::<Vec<_>>();

                self.range_min = list.first().copied();
                self.range_max = list.last().copied();
            } 

            else {
                return Err(meta.error(format!("Invalid attribute: {}", meta.path.get_ident().unwrap())))
            }

            Ok(())
        })?;

        Ok(self)
    }

    pub fn write(&self) -> TokenStream {
        match &self.inner {
            // checkbox
            NormalItemType::Bool => quote! { BuildableSettingType::Bool },

            // slider
            f 
            @(NormalItemType::U32 
            | NormalItemType::U64 
            | NormalItemType::Usize 
            | NormalItemType::F32 
            | NormalItemType::F64) => {
                let ty = f.to_str();
                let min = self.range_min.unwrap_or(0.0) as f32;
                let max = self.range_max.unwrap_or(100.0) as f32;
                let step = if f.is_float() {0.01f32} else {1.0};

                quote! {
                    BuildableSettingType::Number {
                        num_type: #ty.to_string(),
                        min: #min,
                        max: #max,
                        step: Some(#step),
                    }
                }
            }

            // text input
            NormalItemType::String => {
                let do_password = self.password_input == Some(true);
                quote! {
                    BuildableSettingType::String {
                        password: #do_password,
                    }
                }
            }

            // color input
            NormalItemType::Color => quote! { BuildableSettingType::String {
                password: false,
            }},

            NormalItemType::SettingsColor => quote! { BuildableSettingType::String {
                password: false,
            }},

            // 
            NormalItemType::Key => quote! { BuildableSettingType::Key {
                optional: false,
            }},

            NormalItemType::OptionalKey => quote! { BuildableSettingType::Key {
                optional: true,
            }},
            
            NormalItemType::Unknown => {
                quote! {}
            }
        }
    }
}


#[derive(Debug, Clone, Default)]
enum NormalItemType {
    Bool,
    U32,
    U64,
    F32,
    F64,
    Usize,
    String,
    
    OptionalKey,
    Key,

    SettingsColor,
    Color,

    #[default] Unknown
}
impl NormalItemType {
    fn from(s: Option<&Ident>) -> Self {
        let Some(s) = s else { return Self::Unknown };

        match &*s.to_string() {
            "Key" => Self::Key,
            "u32"  => Self::U32,
            "u64"  => Self::U64,
            "f32"  => Self::F32,
            "f64"  => Self::F64,
            "bool" => Self::Bool,
            "usize" => Self::Usize,
            "Color" => Self::Color,
            "String" => Self::String,
            "Option<Key>" => Self::OptionalKey,
            "SettingsColor" => Self::SettingsColor,
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
