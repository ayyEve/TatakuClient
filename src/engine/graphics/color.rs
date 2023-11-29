use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub, Rem, AddAssign, SubAssign, MulAssign, DivAssign, RemAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq,serde::Serialize, serde::Deserialize)]
#[serde(from = "String", into = "String")]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
// constant colors
#[allow(dead_code)]
impl Color {
    #[inline]
    pub const fn new(r:f32, g:f32, b:f32, a:f32) -> Self {Self{r, g, b, a}}

    pub fn alpha(mut self, a:f32) -> Color {
        self.a = a;
        self
    }

    pub fn clamp(self) -> Self {
        Self::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
            self.a.clamp(0.0, 1.0),
        )
    }

    pub fn from_hex(hex:impl AsRef<str>) -> Self {
        let hex = hex.as_ref();
        Self::try_from_hex(hex).unwrap_or_else(|| {
            println!("malformed hex: '{hex}'"); 
            Color::new(0.0, 0.0, 0.0, 0.0)
        })
    }

    pub fn try_from_hex(hex:impl AsRef<str>) -> Option<Color> {
        let hex = hex.as_ref();
        let chars = hex.trim_matches('#').chars().collect::<Vec<char>>();
        fn parse(c1:char, c2:char) -> Option<f32> {
            let n = u8::from_str_radix(&format!("{c1}{c2}"), 16).ok()?;
            Some(n as f32 / 255.0)
        }

        match chars.len() {
            3 => { // rgb
                let r = parse(chars[0], chars[0])?;
                let g = parse(chars[1], chars[1])?;
                let b = parse(chars[2], chars[2])?;
                let a = 1.0;

                Some(Self::new(r, g, b, a))
            }
            4 => { // rgba
                let r = parse(chars[0], chars[0])?;
                let g = parse(chars[1], chars[1])?;
                let b = parse(chars[2], chars[2])?;
                let a = parse(chars[3], chars[3])?;

                Some(Color::new(r, g, b, a))
            }
            6 => { //rrggbb
                let r = parse(chars[0], chars[1])?;
                let g = parse(chars[2], chars[3])?;
                let b = parse(chars[4], chars[5])?;
                let a = 1.0;

                Some(Color::new(r, g, b, a))
            }
            8 => { //rrggbbaa
                let r = parse(chars[0], chars[1])?;
                let g = parse(chars[2], chars[3])?;
                let b = parse(chars[4], chars[5])?;
                let a = parse(chars[6], chars[7])?;

                Some(Color::new(r, g, b, a))
            }

            _ => None
        }
    }

    pub fn from_rgb_bytes(r:u8, g:u8, b:u8) -> Color {
        Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            1.0
        )
    }

    pub fn to_hex(self) -> String {
        let r = (self.r * 255.0) as u8;
        let g = (self.g * 255.0) as u8;
        let b = (self.b * 255.0) as u8;
        let a = (self.a * 255.0) as u8;

        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}
// list of colors generated from the table found at https://www.computerhope.com/htmcolor.htm
impl Color {
    // probably dont need black but w/e
    pub const TRANSPARENT_WHITE:Color = Color {r:1.0,g:1.0,b:1.0,a:0.0};
    pub const TRANSPARENT_BLACK:Color = Color {r:0.0,g:0.0,b:0.0,a:0.0};

    pub const BLACK:Color = Color {r:0.00000000,g:0.00000000,b:0.00000000,a:1.0};
    pub const NIGHT:Color = Color {r:0.04705882,g:0.03529412,b:0.03921569,a:1.0};
    pub const CHARCOAL:Color = Color {r:0.20392157,g:0.15686275,b:0.17254902,a:1.0};
    pub const OIL:Color = Color {r:0.23137255,g:0.19215686,b:0.19215686,a:1.0};
    pub const DARK_GRAY:Color = Color {r:0.22745098,g:0.23137255,b:0.23529412,a:1.0};
    pub const LIGHT_BLACK:Color = Color {r:0.27058824,g:0.27058824,b:0.27058824,a:1.0};
    pub const BLACK_CAT:Color = Color {r:0.25490196,g:0.21960784,b:0.22352941,a:1.0};
    pub const IRIDIUM:Color = Color {r:0.23921569,g:0.23529412,b:0.22745098,a:1.0};
    pub const BLACK_EEL:Color = Color {r:0.27450980,g:0.24313725,b:0.24705882,a:1.0};
    pub const BLACK_COW:Color = Color {r:0.29803922,g:0.27450980,b:0.27450980,a:1.0};
    pub const GRAY_WOLF:Color = Color {r:0.31372549,g:0.29019608,b:0.29411765,a:1.0};
    pub const VAMPIRE_GRAY:Color = Color {r:0.33725490,g:0.31372549,b:0.31764706,a:1.0};
    pub const IRON_GRAY:Color = Color {r:0.32156863,g:0.34901961,b:0.36470588,a:1.0};
    pub const GRAY_DOLPHIN:Color = Color {r:0.36078431,g:0.34509804,b:0.34509804,a:1.0};
    pub const CARBON_GRAY:Color = Color {r:0.38431373,g:0.36470588,b:0.36470588,a:1.0};
    pub const ASH_GRAY:Color = Color {r:0.40000000,g:0.38823529,b:0.38431373,a:1.0};
    pub const CLOUDY_GRAY:Color = Color {r:0.42745098,g:0.41176471,b:0.40784314,a:1.0};
    pub const DIM_GRAY:Color = Color {r:0.41176471,g:0.41176471,b:0.41176471,a:1.0};
    pub const DIM_GREY:Color = Color {r:0.41176471,g:0.41176471,b:0.41176471,a:1.0};
    pub const SMOKEY_GRAY:Color = Color {r:0.44705882,g:0.43137255,b:0.42745098,a:1.0};
    pub const ALIEN_GRAY:Color = Color {r:0.45098039,g:0.43529412,b:0.43137255,a:1.0};
    pub const SONIC_SILVER:Color = Color {r:0.45882353,g:0.45882353,b:0.45882353,a:1.0};
    pub const PLATINUM_GRAY:Color = Color {r:0.47450980,g:0.47450980,b:0.47450980,a:1.0};
    pub const GRANITE:Color = Color {r:0.51372549,g:0.49411765,b:0.48627451,a:1.0};
    pub const GRAY:Color = Color {r:0.50196078,g:0.50196078,b:0.50196078,a:1.0};
    pub const GREY:Color = Color {r:0.50196078,g:0.50196078,b:0.50196078,a:1.0};
    pub const BATTLESHIP_GRAY:Color = Color {r:0.51764706,g:0.51764706,b:0.50980392,a:1.0};
    pub const DARK_GREY:Color = Color {r:0.66274510,g:0.66274510,b:0.66274510,a:1.0};
    pub const GRAY_CLOUD:Color = Color {r:0.71372549,g:0.71372549,b:0.70588235,a:1.0};
    pub const SILVER:Color = Color {r:0.75294118,g:0.75294118,b:0.75294118,a:1.0};
    pub const PALE_SILVER:Color = Color {r:0.78823529,g:0.75294118,b:0.73333333,a:1.0};
    pub const GRAY_GOOSE:Color = Color {r:0.81960784,g:0.81568627,b:0.80784314,a:1.0};
    pub const PLATINUM_SILVER:Color = Color {r:0.80784314,g:0.80784314,b:0.80784314,a:1.0};
    pub const LIGHT_GRAY:Color = Color {r:0.82745098,g:0.82745098,b:0.82745098,a:1.0};
    pub const LIGHT_GREY:Color = Color {r:0.82745098,g:0.82745098,b:0.82745098,a:1.0};
    pub const GAINSBORO:Color = Color {r:0.86274510,g:0.86274510,b:0.86274510,a:1.0};
    pub const PLATINUM:Color = Color {r:0.89803922,g:0.89411765,b:0.88627451,a:1.0};
    pub const METALLIC_SILVER:Color = Color {r:0.73725490,g:0.77647059,b:0.80000000,a:1.0};
    pub const BLUE_GRAY:Color = Color {r:0.59607843,g:0.68627451,b:0.78039216,a:1.0};
    pub const ROMAN_SILVER:Color = Color {r:0.51372549,g:0.53725490,b:0.58823529,a:1.0};
    pub const LIGHT_SLATE_GRAY:Color = Color {r:0.46666667,g:0.53333333,b:0.60000000,a:1.0};
    pub const LIGHT_SLATE_GREY:Color = Color {r:0.46666667,g:0.53333333,b:0.60000000,a:1.0};
    pub const SLATE_GRAY:Color = Color {r:0.43921569,g:0.50196078,b:0.56470588,a:1.0};
    pub const SLATE_GREY:Color = Color {r:0.43921569,g:0.50196078,b:0.56470588,a:1.0};
    pub const RAT_GRAY:Color = Color {r:0.42745098,g:0.48235294,b:0.55294118,a:1.0};
    pub const SLATE_GRANITE_GRAY:Color = Color {r:0.39607843,g:0.45098039,b:0.51372549,a:1.0};
    pub const JET_GRAY:Color = Color {r:0.38039216,g:0.42745098,b:0.49411765,a:1.0};
    pub const MIST_BLUE:Color = Color {r:0.39215686,g:0.42745098,b:0.49411765,a:1.0};
    pub const MARBLE_BLUE:Color = Color {r:0.33725490,g:0.42745098,b:0.49411765,a:1.0};
    pub const SLATE_BLUE_GREY:Color = Color {r:0.45098039,g:0.48627451,b:0.63137255,a:1.0};
    pub const LIGHT_PURPLE_BLUE:Color = Color {r:0.44705882,g:0.56078431,b:0.80784314,a:1.0};
    pub const AZURE_BLUE:Color = Color {r:0.28235294,g:0.38823529,b:0.62745098,a:1.0};
    pub const BLUE_JAY:Color = Color {r:0.16862745,g:0.32941176,b:0.49411765,a:1.0};
    pub const CHARCOAL_BLUE:Color = Color {r:0.21176471,g:0.27058824,b:0.30980392,a:1.0};
    pub const DARK_BLUE_GREY:Color = Color {r:0.16078431,g:0.27450980,b:0.35686275,a:1.0};
    pub const DARK_SLATE:Color = Color {r:0.16862745,g:0.21960784,b:0.33725490,a:1.0};
    pub const DEEP_SEA_BLUE:Color = Color {r:0.07058824,g:0.20392157,b:0.33725490,a:1.0};
    pub const NIGHT_BLUE:Color = Color {r:0.08235294,g:0.10588235,b:0.32941176,a:1.0};
    pub const MIDNIGHT_BLUE:Color = Color {r:0.09803922,g:0.09803922,b:0.43921569,a:1.0};
    pub const NAVY:Color = Color {r:0.00000000,g:0.00000000,b:0.50196078,a:1.0};
    pub const DENIM_DARK_BLUE:Color = Color {r:0.08235294,g:0.10588235,b:0.55294118,a:1.0};
    pub const DARK_BLUE:Color = Color {r:0.00000000,g:0.00000000,b:0.54509804,a:1.0};
    pub const LAPIS_BLUE:Color = Color {r:0.08235294,g:0.19215686,b:0.49411765,a:1.0};
    pub const NEW_MIDNIGHT_BLUE:Color = Color {r:0.00000000,g:0.00000000,b:0.62745098,a:1.0};
    pub const EARTH_BLUE:Color = Color {r:0.00000000,g:0.00000000,b:0.64705882,a:1.0};
    pub const COBALT_BLUE:Color = Color {r:0.00000000,g:0.12549020,b:0.76078431,a:1.0};
    pub const MEDIUM_BLUE:Color = Color {r:0.00000000,g:0.00000000,b:0.80392157,a:1.0};
    pub const BLUEBERRY_BLUE:Color = Color {r:0.00000000,g:0.25490196,b:0.76078431,a:1.0};
    pub const CANARY_BLUE:Color = Color {r:0.16078431,g:0.08627451,b:0.96078431,a:1.0};
    pub const BLUE:Color = Color {r:0.00000000,g:0.00000000,b:1.00000000,a:1.0};
    pub const BRIGHT_BLUE:Color = Color {r:0.03529412,g:0.03529412,b:1.00000000,a:1.0};
    pub const BLUE_ORCHID:Color = Color {r:0.12156863,g:0.27058824,b:0.98823529,a:1.0};
    pub const SAPPHIRE_BLUE:Color = Color {r:0.14509804,g:0.32941176,b:0.78039216,a:1.0};
    pub const BLUE_EYES:Color = Color {r:0.08235294,g:0.41176471,b:0.78039216,a:1.0};
    pub const BRIGHT_NAVY_BLUE:Color = Color {r:0.09803922,g:0.45490196,b:0.82352941,a:1.0};
    pub const BALLOON_BLUE:Color = Color {r:0.16862745,g:0.37647059,b:0.87058824,a:1.0};
    pub const ROYAL_BLUE:Color = Color {r:0.25490196,g:0.41176471,b:0.88235294,a:1.0};
    pub const OCEAN_BLUE:Color = Color {r:0.16862745,g:0.39607843,b:0.92549020,a:1.0};
    pub const BLUE_RIBBON:Color = Color {r:0.18823529,g:0.43137255,b:1.00000000,a:1.0};
    pub const BLUE_DRESS:Color = Color {r:0.08235294,g:0.49019608,b:0.92549020,a:1.0};
    pub const NEON_BLUE:Color = Color {r:0.08235294,g:0.53725490,b:1.00000000,a:1.0};
    pub const DODGER_BLUE:Color = Color {r:0.11764706,g:0.56470588,b:1.00000000,a:1.0};
    pub const GLACIAL_BLUE_ICE:Color = Color {r:0.21176471,g:0.54509804,b:0.75686275,a:1.0};
    pub const STEELB_LUE:Color = Color {r:0.27450980,g:0.50980392,b:0.70588235,a:1.0};
    pub const SILK_BLUE:Color = Color {r:0.28235294,g:0.54117647,b:0.78039216,a:1.0};
    pub const WINDOWS_BLUE:Color = Color {r:0.20784314,g:0.49411765,b:0.78039216,a:1.0};
    pub const BLUE_IVY:Color = Color {r:0.18823529,g:0.56470588,b:0.78039216,a:1.0};
    pub const BLUE_KOI:Color = Color {r:0.39607843,g:0.61960784,b:0.78039216,a:1.0};
    pub const COLUMBIA_BLUE:Color = Color {r:0.52941176,g:0.68627451,b:0.78039216,a:1.0};
    pub const BABY_BLUE:Color = Color {r:0.58431373,g:0.72549020,b:0.78039216,a:1.0};
    pub const CORNFLOWER_BLUE:Color = Color {r:0.39215686,g:0.58431373,b:0.92941176,a:1.0};
    pub const SKY_BLUE_DRESS:Color = Color {r:0.40000000,g:0.59607843,b:1.00000000,a:1.0};
    pub const ICEBERG:Color = Color {r:0.33725490,g:0.64705882,b:0.92549020,a:1.0};
    pub const BUTTERFLY_BLUE:Color = Color {r:0.21960784,g:0.67450980,b:0.92549020,a:1.0};
    pub const DEEPSKY_BLUE:Color = Color {r:0.00000000,g:0.74901961,b:1.00000000,a:1.0};
    pub const MIDDAY_BLUE:Color = Color {r:0.23137255,g:0.72549020,b:1.00000000,a:1.0};
    pub const CRYSTAL_BLUE:Color = Color {r:0.36078431,g:0.70196078,b:1.00000000,a:1.0};
    pub const DENIM_BLUE:Color = Color {r:0.47450980,g:0.72941176,b:0.92549020,a:1.0};
    pub const DAY_SKY_BLUE:Color = Color {r:0.50980392,g:0.79215686,b:1.00000000,a:1.0};
    pub const LIGHTSKY_BLUE:Color = Color {r:0.52941176,g:0.80784314,b:0.98039216,a:1.0};
    pub const SKY_BLUE:Color = Color {r:0.52941176,g:0.80784314,b:0.92156863,a:1.0};
    pub const JEANS_BLUE:Color = Color {r:0.62745098,g:0.81176471,b:0.92549020,a:1.0};
    pub const BLUE_ANGEL:Color = Color {r:0.71764706,g:0.80784314,b:0.92549020,a:1.0};
    pub const PASTEL_BLUE:Color = Color {r:0.70588235,g:0.81176471,b:0.92549020,a:1.0};
    pub const LIGHT_DAY_BLUE:Color = Color {r:0.67843137,g:0.87450980,b:1.00000000,a:1.0};
    pub const SEA_BLUE:Color = Color {r:0.76078431,g:0.87450980,b:1.00000000,a:1.0};
    pub const HEAVENLY_BLUE:Color = Color {r:0.77647059,g:0.87058824,b:1.00000000,a:1.0};
    pub const ROBIN_EGG_BLUE:Color = Color {r:0.74117647,g:0.92941176,b:1.00000000,a:1.0};
    pub const POWDER_BLUE:Color = Color {r:0.69019608,g:0.87843137,b:0.90196078,a:1.0};
    pub const CORAL_BLUE:Color = Color {r:0.68627451,g:0.86274510,b:0.92549020,a:1.0};
    pub const LIGHT_BLUE:Color = Color {r:0.67843137,g:0.84705882,b:0.90196078,a:1.0};
    pub const LIGHT_STEEL_BLUE:Color = Color {r:0.69019608,g:0.81176471,b:0.87058824,a:1.0};
    pub const GULF_BLUE:Color = Color {r:0.78823529,g:0.87450980,b:0.92549020,a:1.0};
    pub const PASTEL_LIGHT_BLUE:Color = Color {r:0.83529412,g:0.83921569,b:0.91764706,a:1.0};
    pub const LAVENDER_BLUE:Color = Color {r:0.89019608,g:0.89411765,b:0.98039216,a:1.0};
    pub const LAVENDER:Color = Color {r:0.90196078,g:0.90196078,b:0.98039216,a:1.0};
    pub const WATER:Color = Color {r:0.92156863,g:0.95686275,b:0.98039216,a:1.0};
    pub const ALICE_BLUE:Color = Color {r:0.94117647,g:0.97254902,b:1.00000000,a:1.0};
    pub const GHOST_WHITE:Color = Color {r:0.97254902,g:0.97254902,b:1.00000000,a:1.0};
    pub const AZURE:Color = Color {r:0.94117647,g:1.00000000,b:1.00000000,a:1.0};
    pub const LIGHT_CYAN:Color = Color {r:0.87843137,g:1.00000000,b:1.00000000,a:1.0};
    pub const LIGHT_SLATE:Color = Color {r:0.80000000,g:1.00000000,b:1.00000000,a:1.0};
    pub const ELECTRIC_BLUE:Color = Color {r:0.60392157,g:0.99607843,b:1.00000000,a:1.0};
    pub const TRON_BLUE:Color = Color {r:0.49019608,g:0.99215686,b:0.99607843,a:1.0};
    pub const BLUE_ZIRCON:Color = Color {r:0.34117647,g:0.99607843,b:1.00000000,a:1.0};
    pub const AQUA:Color = Color {r:0.00000000,g:1.00000000,b:1.00000000,a:1.0};
    pub const CYAN:Color = Color {r:0.00000000,g:1.00000000,b:1.00000000,a:1.0};
    pub const BRIGHT_CYAN:Color = Color {r:0.03921569,g:1.00000000,b:1.00000000,a:1.0};
    pub const CELESTE:Color = Color {r:0.31372549,g:0.92156863,b:0.92549020,a:1.0};
    pub const BLUE_DIAMOND:Color = Color {r:0.30588235,g:0.88627451,b:0.92549020,a:1.0};
    pub const BRIGHT_TURQUOISE:Color = Color {r:0.08627451,g:0.88627451,b:0.96078431,a:1.0};
    pub const BLUE_LAGOON:Color = Color {r:0.55686275,g:0.92156863,b:0.92549020,a:1.0};
    pub const PALE_TURQUOISE:Color = Color {r:0.68627451,g:0.93333333,b:0.93333333,a:1.0};
    pub const PALE_BLUE_LILY:Color = Color {r:0.81176471,g:0.92549020,b:0.92549020,a:1.0};
    pub const TIFFANY_BLUE:Color = Color {r:0.50588235,g:0.84705882,b:0.81568627,a:1.0};
    pub const BLUE_HOSTA:Color = Color {r:0.46666667,g:0.74901961,b:0.78039216,a:1.0};
    pub const CYAN_OPAQUE:Color = Color {r:0.57254902,g:0.78039216,b:0.78039216,a:1.0};
    pub const NORTHERN_LIGHTS_BLUE:Color = Color {r:0.47058824,g:0.78039216,b:0.78039216,a:1.0};
    pub const BLUE_GREEN:Color = Color {r:0.48235294,g:0.80000000,b:0.70980392,a:1.0};
    pub const MEDIUM_AQUAMARINE:Color = Color {r:0.40000000,g:0.80392157,b:0.66666667,a:1.0};
    pub const MAGIC_MINT:Color = Color {r:0.66666667,g:0.94117647,b:0.81960784,a:1.0};
    pub const AQUAMARINE:Color = Color {r:0.49803922,g:1.00000000,b:0.83137255,a:1.0};
    pub const LIGHT_AQUAMARINE:Color = Color {r:0.57647059,g:1.00000000,b:0.90980392,a:1.0};
    pub const TURQUOISE:Color = Color {r:0.25098039,g:0.87843137,b:0.81568627,a:1.0};
    pub const MEDIUM_TURQUOISE:Color = Color {r:0.28235294,g:0.81960784,b:0.80000000,a:1.0};
    pub const DEEP_TURQUOISE:Color = Color {r:0.28235294,g:0.80000000,b:0.80392157,a:1.0};
    pub const JELLYFISH:Color = Color {r:0.27450980,g:0.78039216,b:0.78039216,a:1.0};
    pub const BLUE_TURQUOISE:Color = Color {r:0.26274510,g:0.77647059,b:0.85882353,a:1.0};
    pub const DARK_TURQUOISE:Color = Color {r:0.00000000,g:0.80784314,b:0.81960784,a:1.0};
    pub const MACAW_BLUE_GREEN:Color = Color {r:0.26274510,g:0.74901961,b:0.78039216,a:1.0};
    pub const LIGHT_SEAGREEN:Color = Color {r:0.12549020,g:0.69803922,b:0.66666667,a:1.0};
    pub const SEAFOAM_GREEN:Color = Color {r:0.24313725,g:0.66274510,b:0.62352941,a:1.0};
    pub const CADET_BLUE:Color = Color {r:0.37254902,g:0.61960784,b:0.62745098,a:1.0};
    pub const DEEP_SEA:Color = Color {r:0.23137255,g:0.61176471,b:0.61176471,a:1.0};
    pub const DARK_CYAN:Color = Color {r:0.00000000,g:0.54509804,b:0.54509804,a:1.0};
    pub const TEAL:Color = Color {r:0.00000000,g:0.50196078,b:0.50196078,a:1.0};
    pub const MEDIUM_TEAL:Color = Color {r:0.01568627,g:0.37254902,b:0.37254902,a:1.0};
    pub const DEEP_TEAL:Color = Color {r:0.01176471,g:0.24313725,b:0.24313725,a:1.0};
    pub const DARK_SLATE_GRAY:Color = Color {r:0.14509804,g:0.21960784,b:0.23529412,a:1.0};
    pub const DARK_SLATE_GREY:Color = Color {r:0.14509804,g:0.21960784,b:0.23529412,a:1.0};
    pub const GUNMETAL:Color = Color {r:0.17254902,g:0.20784314,b:0.22352941,a:1.0};
    pub const BLUE_MOSS_GREEN:Color = Color {r:0.23529412,g:0.33725490,b:0.35686275,a:1.0};
    pub const BEETLE_GREEN:Color = Color {r:0.29803922,g:0.47058824,b:0.49411765,a:1.0};
    pub const GRAYISH_TURQUOISE:Color = Color {r:0.36862745,g:0.49019608,b:0.49411765,a:1.0};
    pub const GREENISH_BLUE:Color = Color {r:0.18823529,g:0.49019608,b:0.49411765,a:1.0};
    pub const AQUAMARINE_STONE:Color = Color {r:0.20392157,g:0.52941176,b:0.50588235,a:1.0};
    pub const SEA_TURTLE_GREEN:Color = Color {r:0.26274510,g:0.55294118,b:0.50196078,a:1.0};
    pub const DULL_SEA_GREEN:Color = Color {r:0.30588235,g:0.53725490,b:0.45882353,a:1.0};
    pub const DEEP_SEA_GREEN:Color = Color {r:0.18823529,g:0.40392157,b:0.32941176,a:1.0};
    pub const SEAGREEN:Color = Color {r:0.18039216,g:0.54509804,b:0.34117647,a:1.0};
    pub const DARK_MINT:Color = Color {r:0.19215686,g:0.56470588,b:0.43137255,a:1.0};
    pub const JADE:Color = Color {r:0.00000000,g:0.63921569,b:0.42352941,a:1.0};
    pub const EARTH_GREEN:Color = Color {r:0.20392157,g:0.64705882,b:0.43529412,a:1.0};
    pub const EMERALD:Color = Color {r:0.31372549,g:0.78431373,b:0.47058824,a:1.0};
    pub const MINT:Color = Color {r:0.24313725,g:0.70588235,b:0.53725490,a:1.0};
    pub const MEDIUM_SEA_GREEN:Color = Color {r:0.23529412,g:0.70196078,b:0.44313725,a:1.0};
    pub const CAMOUFLAGE_GREEN:Color = Color {r:0.47058824,g:0.52549020,b:0.41960784,a:1.0};
    pub const SAGE_GREEN:Color = Color {r:0.51764706,g:0.54509804,b:0.47450980,a:1.0};
    pub const HAZEL_GREEN:Color = Color {r:0.38039216,g:0.48627451,b:0.34509804,a:1.0};
    pub const VENOM_GREEN:Color = Color {r:0.44705882,g:0.54901961,b:0.00000000,a:1.0};
    pub const OLIVEDRAB:Color = Color {r:0.41960784,g:0.55686275,b:0.13725490,a:1.0};
    pub const OLIVE:Color = Color {r:0.50196078,g:0.50196078,b:0.00000000,a:1.0};
    pub const DARK_OLIVE_GREEN:Color = Color {r:0.33333333,g:0.41960784,b:0.18431373,a:1.0};
    pub const ARMY_GREEN:Color = Color {r:0.29411765,g:0.32549020,b:0.12549020,a:1.0};
    pub const FERN_GREEN:Color = Color {r:0.40000000,g:0.48627451,b:0.14901961,a:1.0};
    pub const FALL_FOREST_GREEN:Color = Color {r:0.30588235,g:0.57254902,b:0.34509804,a:1.0};
    pub const PINE_GREEN:Color = Color {r:0.21960784,g:0.48627451,b:0.26666667,a:1.0};
    pub const MEDIUM_FOREST_GREEN:Color = Color {r:0.20392157,g:0.44705882,b:0.20784314,a:1.0};
    pub const JUNGLE_GREEN:Color = Color {r:0.20392157,g:0.48627451,b:0.17254902,a:1.0};
    pub const FOREST_GREEN:Color = Color {r:0.13333333,g:0.54509804,b:0.13333333,a:1.0};
    pub const GREEN:Color = Color {r:0.00000000,g:0.50196078,b:0.00000000,a:1.0};
    pub const DARK_GREEN:Color = Color {r:0.00000000,g:0.39215686,b:0.00000000,a:1.0};
    pub const DEEP_EMERALD_GREEN:Color = Color {r:0.01568627,g:0.38823529,b:0.02745098,a:1.0};
    pub const DARK_FOREST_GREEN:Color = Color {r:0.14509804,g:0.25490196,b:0.09019608,a:1.0};
    pub const SEAWEED_GREEN:Color = Color {r:0.26274510,g:0.48627451,b:0.09019608,a:1.0};
    pub const SHAMROCK_GREEN:Color = Color {r:0.20392157,g:0.48627451,b:0.09019608,a:1.0};
    pub const GREEN_ONION:Color = Color {r:0.41568627,g:0.63137255,b:0.12941176,a:1.0};
    pub const GREEN_PEPPER:Color = Color {r:0.29019608,g:0.62745098,b:0.17254902,a:1.0};
    pub const DARK_LIME_GREEN:Color = Color {r:0.25490196,g:0.63921569,b:0.09019608,a:1.0};
    pub const PARROT_GREEN:Color = Color {r:0.07058824,g:0.67843137,b:0.16862745,a:1.0};
    pub const CLOVER_GREEN:Color = Color {r:0.24313725,g:0.62745098,b:0.33333333,a:1.0};
    pub const DINOSAUR_GREEN:Color = Color {r:0.45098039,g:0.63137255,b:0.42352941,a:1.0};
    pub const GREEN_SNAKE:Color = Color {r:0.42352941,g:0.73333333,b:0.23529412,a:1.0};
    pub const ALIEN_GREEN:Color = Color {r:0.42352941,g:0.76862745,b:0.09019608,a:1.0};
    pub const GREEN_APPLE:Color = Color {r:0.29803922,g:0.76862745,b:0.09019608,a:1.0};
    pub const LIME_GREEN:Color = Color {r:0.19607843,g:0.80392157,b:0.19607843,a:1.0};
    pub const PEA_GREEN:Color = Color {r:0.32156863,g:0.81568627,b:0.09019608,a:1.0};
    pub const KELLY_GREEN:Color = Color {r:0.29803922,g:0.77254902,b:0.32156863,a:1.0};
    pub const ZOMBIE_GREEN:Color = Color {r:0.32941176,g:0.77254902,b:0.44313725,a:1.0};
    pub const FROG_GREEN:Color = Color {r:0.60000000,g:0.77647059,b:0.55686275,a:1.0};
    pub const DARK_SEA_GREEN:Color = Color {r:0.56078431,g:0.73725490,b:0.56078431,a:1.0};
    pub const GREEN_PEAS:Color = Color {r:0.53725490,g:0.76470588,b:0.36078431,a:1.0};
    pub const DOLLAR_BILL_GREEN:Color = Color {r:0.52156863,g:0.73333333,b:0.39607843,a:1.0};
    pub const IGUANA_GREEN:Color = Color {r:0.61176471,g:0.69019608,b:0.44313725,a:1.0};
    pub const ACID_GREEN:Color = Color {r:0.69019608,g:0.74901961,b:0.10196078,a:1.0};
    pub const AVOCADO_GREEN:Color = Color {r:0.69803922,g:0.76078431,b:0.28235294,a:1.0};
    pub const PISTACHIO_GREEN:Color = Color {r:0.61568627,g:0.76078431,b:0.03529412,a:1.0};
    pub const SALAD_GREEN:Color = Color {r:0.63137255,g:0.78823529,b:0.20784314,a:1.0};
    pub const YELLOW_GREEN:Color = Color {r:0.60392157,g:0.80392157,b:0.19607843,a:1.0};
    pub const PASTEL_GREEN:Color = Color {r:0.46666667,g:0.86666667,b:0.46666667,a:1.0};
    pub const HUMMINGBIRD_GREEN:Color = Color {r:0.49803922,g:0.90980392,b:0.09019608,a:1.0};
    pub const NEBULA_GREEN:Color = Color {r:0.34901961,g:0.90980392,b:0.09019608,a:1.0};
    pub const STOPLIGHT_GO_GREEN:Color = Color {r:0.34117647,g:0.91372549,b:0.39215686,a:1.0};
    pub const NEON_GREEN:Color = Color {r:0.08627451,g:0.96078431,b:0.16078431,a:1.0};
    pub const JADE_GREEN:Color = Color {r:0.36862745,g:0.98431373,b:0.43137255,a:1.0};
    pub const LIME_MINT_GREEN:Color = Color {r:0.21176471,g:0.96078431,b:0.49803922,a:1.0};
    pub const SPRING_GREEN:Color = Color {r:0.00000000,g:1.00000000,b:0.49803922,a:1.0};
    pub const MEDIUM_SPRING_GREEN:Color = Color {r:0.00000000,g:0.98039216,b:0.60392157,a:1.0};
    pub const EMERALD_GREEN:Color = Color {r:0.37254902,g:0.98431373,b:0.09019608,a:1.0};
    pub const LIME:Color = Color {r:0.00000000,g:1.00000000,b:0.00000000,a:1.0};
    pub const LAWN_GREEN:Color = Color {r:0.48627451,g:0.98823529,b:0.00000000,a:1.0};
    pub const BRIGHT_GREEN:Color = Color {r:0.40000000,g:1.00000000,b:0.00000000,a:1.0};
    pub const CHARTREUSE:Color = Color {r:0.49803922,g:1.00000000,b:0.00000000,a:1.0};
    pub const YELLOW_LAWN_GREEN:Color = Color {r:0.52941176,g:0.96862745,b:0.09019608,a:1.0};
    pub const ALOE_VERA_GREEN:Color = Color {r:0.59607843,g:0.96078431,b:0.08627451,a:1.0};
    pub const DULL_GREEN_YELLOW:Color = Color {r:0.69411765,g:0.98431373,b:0.09019608,a:1.0};
    pub const GREEN_YELLOW:Color = Color {r:0.67843137,g:1.00000000,b:0.18431373,a:1.0};
    pub const CHAMELEON_GREEN:Color = Color {r:0.74117647,g:0.96078431,b:0.08627451,a:1.0};
    pub const NEON_YELLOW_GREEN:Color = Color {r:0.85490196,g:0.93333333,b:0.00392157,a:1.0};
    pub const YELLOW_GREEN_GROSBEAK:Color = Color {r:0.88627451,g:0.96078431,b:0.08627451,a:1.0};
    pub const TEA_GREEN:Color = Color {r:0.80000000,g:0.98431373,b:0.36470588,a:1.0};
    pub const SLIME_GREEN:Color = Color {r:0.73725490,g:0.91372549,b:0.32941176,a:1.0};
    pub const ALGAE_GREEN:Color = Color {r:0.39215686,g:0.91372549,b:0.52549020,a:1.0};
    pub const LIGHT_GREEN:Color = Color {r:0.56470588,g:0.93333333,b:0.56470588,a:1.0};
    pub const DRAGON_GREEN:Color = Color {r:0.41568627,g:0.98431373,b:0.57254902,a:1.0};
    pub const PALE_GREEN:Color = Color {r:0.59607843,g:0.98431373,b:0.59607843,a:1.0};
    pub const MINT_GREEN:Color = Color {r:0.59607843,g:1.00000000,b:0.59607843,a:1.0};
    pub const GREEN_THUMB:Color = Color {r:0.70980392,g:0.91764706,b:0.66666667,a:1.0};
    pub const ORGANIC_BROWN:Color = Color {r:0.89019608,g:0.97647059,b:0.65098039,a:1.0};
    pub const LIGHT_JADE:Color = Color {r:0.76470588,g:0.99215686,b:0.72156863,a:1.0};
    pub const LIGHT_ROSE_GREEN:Color = Color {r:0.85882353,g:0.97647059,b:0.85882353,a:1.0};
    pub const HONEYDEW:Color = Color {r:0.94117647,g:1.00000000,b:0.94117647,a:1.0};
    pub const MINTCREAM:Color = Color {r:0.96078431,g:1.00000000,b:0.98039216,a:1.0};
    pub const LEMONCHIFFON:Color = Color {r:1.00000000,g:0.98039216,b:0.80392157,a:1.0};
    pub const PARCHMENT:Color = Color {r:1.00000000,g:1.00000000,b:0.76078431,a:1.0};
    pub const CREAM:Color = Color {r:1.00000000,g:1.00000000,b:0.80000000,a:1.0};
    pub const LIGHT_GOLDENROD_YELLOW:Color = Color {r:0.98039216,g:0.98039216,b:0.82352941,a:1.0};
    pub const LIGHT_YELLOW:Color = Color {r:1.00000000,g:1.00000000,b:0.87843137,a:1.0};
    pub const BEIGE:Color = Color {r:0.96078431,g:0.96078431,b:0.86274510,a:1.0};
    pub const CORNSILK:Color = Color {r:1.00000000,g:0.97254902,b:0.86274510,a:1.0};
    pub const BLONDE:Color = Color {r:0.98431373,g:0.96470588,b:0.85098039,a:1.0};
    pub const CHAMPAGNE:Color = Color {r:0.96862745,g:0.90588235,b:0.80784314,a:1.0};
    pub const ANTIQUE_WHITE:Color = Color {r:0.98039216,g:0.92156863,b:0.84313725,a:1.0};
    pub const PAPAYAWHIP:Color = Color {r:1.00000000,g:0.93725490,b:0.83529412,a:1.0};
    pub const BLANCHED_ALMOND:Color = Color {r:1.00000000,g:0.92156863,b:0.80392157,a:1.0};
    pub const BISQUE:Color = Color {r:1.00000000,g:0.89411765,b:0.76862745,a:1.0};
    pub const WHEAT:Color = Color {r:0.96078431,g:0.87058824,b:0.70196078,a:1.0};
    pub const MOCCASIN:Color = Color {r:1.00000000,g:0.89411765,b:0.70980392,a:1.0};
    pub const PEACH:Color = Color {r:1.00000000,g:0.89803922,b:0.70588235,a:1.0};
    pub const LIGHT_ORANGE:Color = Color {r:0.99607843,g:0.84705882,b:0.69411765,a:1.0};
    pub const PEACHPUFF:Color = Color {r:1.00000000,g:0.85490196,b:0.72549020,a:1.0};
    pub const NAVAJO_WHITE:Color = Color {r:1.00000000,g:0.87058824,b:0.67843137,a:1.0};
    pub const GOLDEN_BLONDE:Color = Color {r:0.98431373,g:0.90588235,b:0.63137255,a:1.0};
    pub const GOLDEN_SILK:Color = Color {r:0.95294118,g:0.89019608,b:0.76470588,a:1.0};
    pub const DARK_BLONDE:Color = Color {r:0.94117647,g:0.88627451,b:0.71372549,a:1.0};
    pub const LIGHT_GOLD:Color = Color {r:0.94509804,g:0.89803922,b:0.67450980,a:1.0};
    pub const VANILLA:Color = Color {r:0.95294118,g:0.89803922,b:0.67058824,a:1.0};
    pub const TAN_BROWN:Color = Color {r:0.92549020,g:0.89803922,b:0.71372549,a:1.0};
    pub const PALE_GOLDENROD:Color = Color {r:0.93333333,g:0.90980392,b:0.66666667,a:1.0};
    pub const KHAKI:Color = Color {r:0.94117647,g:0.90196078,b:0.54901961,a:1.0};
    pub const CARDBOARD_BROWN:Color = Color {r:0.92941176,g:0.85490196,b:0.45490196,a:1.0};
    pub const HARVEST_GOLD:Color = Color {r:0.92941176,g:0.88627451,b:0.45882353,a:1.0};
    pub const SUN_YELLOW:Color = Color {r:1.00000000,g:0.90980392,b:0.48627451,a:1.0};
    pub const CORN_YELLOW:Color = Color {r:1.00000000,g:0.95294118,b:0.50196078,a:1.0};
    pub const PASTEL_YELLOW:Color = Color {r:0.98039216,g:0.97254902,b:0.51764706,a:1.0};
    pub const NEON_YELLOW:Color = Color {r:1.00000000,g:1.00000000,b:0.20000000,a:1.0};
    pub const YELLOW:Color = Color {r:1.00000000,g:1.00000000,b:0.00000000,a:1.0};
    pub const CANARY_YELLOW:Color = Color {r:1.00000000,g:0.93725490,b:0.00000000,a:1.0};
    pub const BANANA_YELLOW:Color = Color {r:0.96078431,g:0.88627451,b:0.08627451,a:1.0};
    pub const MUSTARD_YELLOW:Color = Color {r:1.00000000,g:0.85882353,b:0.34509804,a:1.0};
    pub const GOLDEN_YELLOW:Color = Color {r:1.00000000,g:0.87450980,b:0.00000000,a:1.0};
    pub const BOLD_YELLOW:Color = Color {r:0.97647059,g:0.85882353,b:0.14117647,a:1.0};
    pub const RUBBER_DUCKY_YELLOW:Color = Color {r:1.00000000,g:0.84705882,b:0.00392157,a:1.0};
    pub const GOLD:Color = Color {r:1.00000000,g:0.84313725,b:0.00000000,a:1.0};
    pub const BRIGHT_GOLD:Color = Color {r:0.99215686,g:0.81568627,b:0.09019608,a:1.0};
    pub const GOLDEN_BROWN:Color = Color {r:0.91764706,g:0.75686275,b:0.09019608,a:1.0};
    pub const DEEP_YELLOW:Color = Color {r:0.96470588,g:0.74509804,b:0.00000000,a:1.0};
    pub const MACARONI_AND_CHEESE:Color = Color {r:0.94901961,g:0.73333333,b:0.40000000,a:1.0};
    pub const SAFFRON:Color = Color {r:0.98431373,g:0.72549020,b:0.09019608,a:1.0};
    pub const BEER:Color = Color {r:0.98431373,g:0.69411765,b:0.09019608,a:1.0};
    pub const YELLOW_ORANGE:Color = Color {r:1.00000000,g:0.68235294,b:0.25882353,a:1.0};
    pub const ORANGE_YELLOW:Color = Color {r:1.00000000,g:0.68235294,b:0.25882353,a:1.0};
    pub const CANTALOUPE:Color = Color {r:1.00000000,g:0.65098039,b:0.18431373,a:1.0};
    pub const ORANGE:Color = Color {r:1.00000000,g:0.64705882,b:0.00000000,a:1.0};
    pub const BROWN_SAND:Color = Color {r:0.93333333,g:0.60392157,b:0.30196078,a:1.0};
    pub const SANDY_BROWN:Color = Color {r:0.95686275,g:0.64313725,b:0.37647059,a:1.0};
    pub const BROWN_SUGAR:Color = Color {r:0.88627451,g:0.65490196,b:0.43529412,a:1.0};
    pub const CAMEL_BROWN:Color = Color {r:0.75686275,g:0.60392157,b:0.41960784,a:1.0};
    pub const DEER_BROWN:Color = Color {r:0.90196078,g:0.74901961,b:0.51372549,a:1.0};
    pub const BURLYWOOD:Color = Color {r:0.87058824,g:0.72156863,b:0.52941176,a:1.0};
    pub const TAN:Color = Color {r:0.82352941,g:0.70588235,b:0.54901961,a:1.0};
    pub const LIGHT_FRENCH_BEIGE:Color = Color {r:0.78431373,g:0.67843137,b:0.49803922,a:1.0};
    pub const SAND:Color = Color {r:0.76078431,g:0.69803922,b:0.50196078,a:1.0};
    pub const SAGE:Color = Color {r:0.73725490,g:0.72156863,b:0.54117647,a:1.0};
    pub const FALL_LEAF_BROWN:Color = Color {r:0.78431373,g:0.70980392,b:0.37647059,a:1.0};
    pub const GINGER_BROWN:Color = Color {r:0.78823529,g:0.74509804,b:0.38431373,a:1.0};
    pub const DARK_KHAKI:Color = Color {r:0.74117647,g:0.71764706,b:0.41960784,a:1.0};
    pub const OLIVE_GREEN:Color = Color {r:0.72941176,g:0.72156863,b:0.42352941,a:1.0};
    pub const BRASS:Color = Color {r:0.70980392,g:0.65098039,b:0.25882353,a:1.0};
    pub const COOKIE_BROWN:Color = Color {r:0.78039216,g:0.63921569,b:0.09019608,a:1.0};
    pub const METALLIC_GOLD:Color = Color {r:0.83137255,g:0.68627451,b:0.21568627,a:1.0};
    pub const BEE_YELLOW:Color = Color {r:0.91372549,g:0.67058824,b:0.09019608,a:1.0};
    pub const SCHOOL_BUS_YELLOW:Color = Color {r:0.90980392,g:0.63921569,b:0.09019608,a:1.0};
    pub const GOLDENROD:Color = Color {r:0.85490196,g:0.64705882,b:0.12549020,a:1.0};
    pub const ORANGE_GOLD:Color = Color {r:0.83137255,g:0.62745098,b:0.09019608,a:1.0};
    pub const CARAMEL:Color = Color {r:0.77647059,g:0.55686275,b:0.09019608,a:1.0};
    pub const DARK_GOLDENROD:Color = Color {r:0.72156863,g:0.52549020,b:0.04313725,a:1.0};
    pub const CINNAMON:Color = Color {r:0.77254902,g:0.53725490,b:0.09019608,a:1.0};
    pub const PERU:Color = Color {r:0.80392157,g:0.52156863,b:0.24705882,a:1.0};
    pub const BRONZE:Color = Color {r:0.80392157,g:0.49803922,b:0.19607843,a:1.0};
    pub const TIGER_ORANGE:Color = Color {r:0.78431373,g:0.50588235,b:0.25490196,a:1.0};
    pub const COPPER:Color = Color {r:0.72156863,g:0.45098039,b:0.20000000,a:1.0};
    pub const WOOD:Color = Color {r:0.58823529,g:0.43529412,b:0.20000000,a:1.0};
    pub const OAK_BROWN:Color = Color {r:0.50196078,g:0.39607843,b:0.09019608,a:1.0};
    pub const ANTIQUE_BRONZE:Color = Color {r:0.40000000,g:0.36470588,b:0.11764706,a:1.0};
    pub const HAZEL:Color = Color {r:0.55686275,g:0.46274510,b:0.09411765,a:1.0};
    pub const DARK_YELLOW:Color = Color {r:0.54509804,g:0.50196078,b:0.00000000,a:1.0};
    pub const DARK_MOCCASIN:Color = Color {r:0.50980392,g:0.47058824,b:0.22352941,a:1.0};
    pub const BULLET_SHELL:Color = Color {r:0.68627451,g:0.60784314,b:0.37647059,a:1.0};
    pub const ARMY_BROWN:Color = Color {r:0.50980392,g:0.48235294,b:0.37647059,a:1.0};
    pub const SANDSTONE:Color = Color {r:0.47058824,g:0.42745098,b:0.37254902,a:1.0};
    pub const TAUPE:Color = Color {r:0.28235294,g:0.23529412,b:0.19607843,a:1.0};
    pub const MOCHA:Color = Color {r:0.28627451,g:0.23921569,b:0.14901961,a:1.0};
    pub const MILK_CHOCOLATE:Color = Color {r:0.31764706,g:0.23137255,b:0.10980392,a:1.0};
    pub const GRAY_BROWN:Color = Color {r:0.23921569,g:0.21176471,b:0.20784314,a:1.0};
    pub const DARK_COFFEE:Color = Color {r:0.23137255,g:0.18431373,b:0.18431373,a:1.0};
    pub const OLD_BURGUNDY:Color = Color {r:0.26274510,g:0.18823529,b:0.18039216,a:1.0};
    pub const WESTERN_CHARCOAL:Color = Color {r:0.28627451,g:0.25490196,b:0.24705882,a:1.0};
    pub const BAKERS_BROWN:Color = Color {r:0.36078431,g:0.20000000,b:0.09019608,a:1.0};
    pub const DARK_BROWN:Color = Color {r:0.39607843,g:0.26274510,b:0.12941176,a:1.0};
    pub const SEPIA_BROWN:Color = Color {r:0.43921569,g:0.25882353,b:0.07843137,a:1.0};
    pub const COFFEE:Color = Color {r:0.43529412,g:0.30588235,b:0.21568627,a:1.0};
    pub const BROWN_BEAR:Color = Color {r:0.51372549,g:0.36078431,b:0.23137255,a:1.0};
    pub const RED_DIRT:Color = Color {r:0.49803922,g:0.32156863,b:0.09019608,a:1.0};
    pub const SEPIA:Color = Color {r:0.49803922,g:0.27450980,b:0.17254902,a:1.0};
    pub const SIENNA:Color = Color {r:0.62745098,g:0.32156863,b:0.17647059,a:1.0};
    pub const SADDLE_BROWN:Color = Color {r:0.54509804,g:0.27058824,b:0.07450980,a:1.0};
    pub const DARK_SIENNA:Color = Color {r:0.54117647,g:0.25490196,b:0.09019608,a:1.0};
    pub const SANGRIA:Color = Color {r:0.49411765,g:0.21960784,b:0.09019608,a:1.0};
    pub const BLOOD_RED:Color = Color {r:0.49411765,g:0.20784314,b:0.09019608,a:1.0};
    pub const CHESTNUT:Color = Color {r:0.58431373,g:0.27058824,b:0.20784314,a:1.0};
    pub const CHESTNUT_RED:Color = Color {r:0.76470588,g:0.29019608,b:0.17254902,a:1.0};
    pub const MAHOGANY:Color = Color {r:0.75294118,g:0.25098039,b:0.00000000,a:1.0};
    pub const RED_FOX:Color = Color {r:0.76470588,g:0.34509804,b:0.09019608,a:1.0};
    pub const DARK_BISQUE:Color = Color {r:0.72156863,g:0.39607843,b:0.00000000,a:1.0};
    pub const LIGHT_BROWN:Color = Color {r:0.70980392,g:0.39607843,b:0.11372549,a:1.0};
    pub const RUST:Color = Color {r:0.76470588,g:0.38431373,b:0.25490196,a:1.0};
    pub const COPPER_RED:Color = Color {r:0.79607843,g:0.42745098,b:0.31764706,a:1.0};
    pub const ORANGE_SALMON:Color = Color {r:0.76862745,g:0.45490196,b:0.31764706,a:1.0};
    pub const CHOCOLATE:Color = Color {r:0.82352941,g:0.41176471,b:0.11764706,a:1.0};
    pub const SEDONA:Color = Color {r:0.80000000,g:0.40000000,b:0.00000000,a:1.0};
    pub const PAPAYA_ORANGE:Color = Color {r:0.89803922,g:0.40392157,b:0.09019608,a:1.0};
    pub const HALLOWEEN_ORANGE:Color = Color {r:0.90196078,g:0.42352941,b:0.17254902,a:1.0};
    pub const NEON_ORANGE:Color = Color {r:1.00000000,g:0.40392157,b:0.00000000,a:1.0};
    pub const BRIGHT_ORANGE:Color = Color {r:1.00000000,g:0.37254902,b:0.12156863,a:1.0};
    pub const PUMPKIN_ORANGE:Color = Color {r:0.97254902,g:0.44705882,b:0.09019608,a:1.0};
    pub const CARROT_ORANGE:Color = Color {r:0.97254902,g:0.50196078,b:0.09019608,a:1.0};
    pub const DARK_ORANGE:Color = Color {r:1.00000000,g:0.54901961,b:0.00000000,a:1.0};
    pub const CONSTRUCTION_CONE_ORANGE:Color = Color {r:0.97254902,g:0.45490196,b:0.19215686,a:1.0};
    pub const INDIAN_SAFFRON:Color = Color {r:1.00000000,g:0.46666667,b:0.13333333,a:1.0};
    pub const SUNRISE_ORANGE:Color = Color {r:0.90196078,g:0.45490196,b:0.31764706,a:1.0};
    pub const MANGO_ORANGE:Color = Color {r:1.00000000,g:0.50196078,b:0.25098039,a:1.0};
    pub const CORAL:Color = Color {r:1.00000000,g:0.49803922,b:0.31372549,a:1.0};
    pub const BASKET_BALL_ORANGE:Color = Color {r:0.97254902,g:0.50588235,b:0.34509804,a:1.0};
    pub const LIGHT_SALMON_ROSE:Color = Color {r:0.97647059,g:0.58823529,b:0.41960784,a:1.0};
    pub const LIGHT_SALMON:Color = Color {r:1.00000000,g:0.62745098,b:0.47843137,a:1.0};
    pub const DARK_SALMON:Color = Color {r:0.91372549,g:0.58823529,b:0.47843137,a:1.0};
    pub const TANGERINE:Color = Color {r:0.90588235,g:0.54117647,b:0.38039216,a:1.0};
    pub const LIGHT_COPPER:Color = Color {r:0.85490196,g:0.54117647,b:0.40392157,a:1.0};
    pub const SALMON:Color = Color {r:0.98039216,g:0.50196078,b:0.44705882,a:1.0};
    pub const LIGHT_CORAL:Color = Color {r:0.94117647,g:0.50196078,b:0.50196078,a:1.0};
    pub const PASTEL_RED:Color = Color {r:0.96470588,g:0.44705882,b:0.50196078,a:1.0};
    pub const PINK_CORAL:Color = Color {r:0.90588235,g:0.45490196,b:0.44313725,a:1.0};
    pub const BEAN_RED:Color = Color {r:0.96862745,g:0.36470588,b:0.34901961,a:1.0};
    pub const VALENTINE_RED:Color = Color {r:0.89803922,g:0.32941176,b:0.31764706,a:1.0};
    pub const INDIAN_RED:Color = Color {r:0.80392157,g:0.36078431,b:0.36078431,a:1.0};
    pub const TOMATO:Color = Color {r:1.00000000,g:0.38823529,b:0.27843137,a:1.0};
    pub const SHOCKING_ORANGE:Color = Color {r:0.89803922,g:0.35686275,b:0.23529412,a:1.0};
    pub const ORANGE_RED:Color = Color {r:1.00000000,g:0.27058824,b:0.00000000,a:1.0};
    pub const RED:Color = Color {r:1.00000000,g:0.00000000,b:0.00000000,a:1.0};
    pub const NEON_RED:Color = Color {r:0.99215686,g:0.10980392,b:0.01176471,a:1.0};
    pub const SCARLET:Color = Color {r:1.00000000,g:0.14117647,b:0.00000000,a:1.0};
    pub const RUBY_RED:Color = Color {r:0.96470588,g:0.13333333,b:0.09019608,a:1.0};
    pub const FERRARI_RED:Color = Color {r:0.96862745,g:0.05098039,b:0.10196078,a:1.0};
    pub const FIRE_ENGINE_RED:Color = Color {r:0.96470588,g:0.15686275,b:0.09019608,a:1.0};
    pub const LAVA_RED:Color = Color {r:0.89411765,g:0.13333333,b:0.09019608,a:1.0};
    pub const LOVE_RED:Color = Color {r:0.89411765,g:0.10588235,b:0.09019608,a:1.0};
    pub const GRAPEFRUIT:Color = Color {r:0.86274510,g:0.21960784,b:0.12156863,a:1.0};
    pub const CHERRY_RED:Color = Color {r:0.76078431,g:0.27450980,b:0.25490196,a:1.0};
    pub const CHILLI_PEPPER:Color = Color {r:0.75686275,g:0.10588235,b:0.09019608,a:1.0};
    pub const FIREBRICK:Color = Color {r:0.69803922,g:0.13333333,b:0.13333333,a:1.0};
    pub const TOMATO_SAUCE_RED:Color = Color {r:0.69803922,g:0.09411765,b:0.02745098,a:1.0};
    pub const BROWN:Color = Color {r:0.64705882,g:0.16470588,b:0.16470588,a:1.0};
    pub const CARBON_RED:Color = Color {r:0.65490196,g:0.05098039,b:0.16470588,a:1.0};
    pub const CRANBERRY:Color = Color {r:0.62352941,g:0.00000000,b:0.05882353,a:1.0};
    pub const SAFFRON_RED:Color = Color {r:0.57647059,g:0.07450980,b:0.07843137,a:1.0};
    pub const RED_WINE:Color = Color {r:0.60000000,g:0.00000000,b:0.07058824,a:1.0};
    pub const WINE_RED:Color = Color {r:0.60000000,g:0.00000000,b:0.07058824,a:1.0};
    pub const DARK_RED:Color = Color {r:0.54509804,g:0.00000000,b:0.00000000,a:1.0};
    pub const MAROON:Color = Color {r:0.50196078,g:0.00000000,b:0.00000000,a:1.0};
    pub const BURGUNDY:Color = Color {r:0.54901961,g:0.00000000,b:0.10196078,a:1.0};
    pub const DEEP_RED:Color = Color {r:0.50196078,g:0.01960784,b:0.09019608,a:1.0};
    pub const RED_BLOOD:Color = Color {r:0.40000000,g:0.00000000,b:0.00000000,a:1.0};
    pub const BLOOD_NIGHT:Color = Color {r:0.33333333,g:0.08627451,b:0.02352941,a:1.0};
    pub const BLACK_BEAN:Color = Color {r:0.23921569,g:0.04705882,b:0.00784314,a:1.0};
    pub const CHOCOLATE_BROWN:Color = Color {r:0.24705882,g:0.00000000,b:0.05882353,a:1.0};
    pub const MIDNIGHT:Color = Color {r:0.16862745,g:0.10588235,b:0.09019608,a:1.0};
    pub const PURPLE_LILY:Color = Color {r:0.33333333,g:0.03921569,b:0.20784314,a:1.0};
    pub const PURPLE_MAROON:Color = Color {r:0.50588235,g:0.01960784,b:0.25490196,a:1.0};
    pub const PLUM_PIE:Color = Color {r:0.49019608,g:0.01960784,b:0.25490196,a:1.0};
    pub const PLUM_VELVET:Color = Color {r:0.49019608,g:0.01960784,b:0.32156863,a:1.0};
    pub const DARK_RASPBERRY:Color = Color {r:0.52941176,g:0.14901961,b:0.34117647,a:1.0};
    pub const VELVET_MAROON:Color = Color {r:0.49411765,g:0.20784314,b:0.30196078,a:1.0};
    pub const ROSY_FINCH:Color = Color {r:0.49803922,g:0.30588235,b:0.32156863,a:1.0};
    pub const DULL_PURPLE:Color = Color {r:0.49803922,g:0.32156863,b:0.36470588,a:1.0};
    pub const PUCE:Color = Color {r:0.49803922,g:0.35294118,b:0.34509804,a:1.0};
    pub const ROSE_DUST:Color = Color {r:0.60000000,g:0.43921569,b:0.43921569,a:1.0};
    pub const ROSY_PINK:Color = Color {r:0.70196078,g:0.51764706,b:0.50588235,a:1.0};
    pub const ROSY_BROWN:Color = Color {r:0.73725490,g:0.56078431,b:0.56078431,a:1.0};
    pub const KHAKI_ROSE:Color = Color {r:0.77254902,g:0.56470588,b:0.55686275,a:1.0};
    pub const PINK_BROWN:Color = Color {r:0.76862745,g:0.50588235,b:0.53725490,a:1.0};
    pub const LIPSTICK_PINK:Color = Color {r:0.76862745,g:0.52941176,b:0.57647059,a:1.0};
    pub const ROSE:Color = Color {r:0.90980392,g:0.67843137,b:0.66666667,a:1.0};
    pub const SILVER_PINK:Color = Color {r:0.76862745,g:0.68235294,b:0.67843137,a:1.0};
    pub const ROSE_GOLD:Color = Color {r:0.92549020,g:0.77254902,b:0.75294118,a:1.0};
    pub const DEEP_PEACH:Color = Color {r:1.00000000,g:0.79607843,b:0.64313725,a:1.0};
    pub const PASTEL_ORANGE:Color = Color {r:0.97254902,g:0.72156863,b:0.54509804,a:1.0};
    pub const DESERT_SAND:Color = Color {r:0.92941176,g:0.78823529,b:0.68627451,a:1.0};
    pub const UNBLEACHED_SILK:Color = Color {r:1.00000000,g:0.86666667,b:0.79215686,a:1.0};
    pub const PIG_PINK:Color = Color {r:0.99215686,g:0.84313725,b:0.89411765,a:1.0};
    pub const BLUSH:Color = Color {r:1.00000000,g:0.90196078,b:0.90980392,a:1.0};
    pub const MISTY_ROSE:Color = Color {r:1.00000000,g:0.89411765,b:0.88235294,a:1.0};
    pub const PINK_BUBBLE_GUM:Color = Color {r:1.00000000,g:0.87450980,b:0.86666667,a:1.0};
    pub const LIGHT_RED:Color = Color {r:1.00000000,g:0.80000000,b:0.79607843,a:1.0};
    pub const LIGHT_ROSE:Color = Color {r:0.98431373,g:0.81176471,b:0.80392157,a:1.0};
    pub const DEEP_ROSE:Color = Color {r:0.98431373,g:0.73333333,b:0.72549020,a:1.0};
    pub const PINK:Color = Color {r:1.00000000,g:0.75294118,b:0.79607843,a:1.0};
    pub const LIGHT_PINK:Color = Color {r:1.00000000,g:0.71372549,b:0.75686275,a:1.0};
    pub const DONUT_PINK:Color = Color {r:0.98039216,g:0.68627451,b:0.74509804,a:1.0};
    pub const BABY_PINK:Color = Color {r:0.98039216,g:0.68627451,b:0.72941176,a:1.0};
    pub const FLAMINGO_PINK:Color = Color {r:0.97647059,g:0.65490196,b:0.69019608,a:1.0};
    pub const PASTEL_PINK:Color = Color {r:0.99607843,g:0.63921569,b:0.66666667,a:1.0};
    pub const PINK_ROSE:Color = Color {r:0.90588235,g:0.63137255,b:0.69019608,a:1.0};
    pub const PINK_DAISY:Color = Color {r:0.90588235,g:0.60000000,b:0.63921569,a:1.0};
    pub const CADILLAC_PINK:Color = Color {r:0.89019608,g:0.54117647,b:0.68235294,a:1.0};
    pub const CARNATION_PINK:Color = Color {r:0.96862745,g:0.47058824,b:0.63137255,a:1.0};
    pub const BLUSH_RED:Color = Color {r:0.89803922,g:0.43137255,b:0.58039216,a:1.0};
    pub const PALE_VIOLET_RED:Color = Color {r:0.85882353,g:0.43921569,b:0.57647059,a:1.0};
    pub const PURPLE_PINK:Color = Color {r:0.81960784,g:0.39607843,b:0.52941176,a:1.0};
    pub const TULIP_PINK:Color = Color {r:0.76078431,g:0.35294118,b:0.48627451,a:1.0};
    pub const BASHFUL_PINK:Color = Color {r:0.76078431,g:0.32156863,b:0.51372549,a:1.0};
    pub const DARK_PINK:Color = Color {r:0.90588235,g:0.32941176,b:0.50196078,a:1.0};
    pub const DARK_HOT_PINK:Color = Color {r:0.96470588,g:0.37647059,b:0.67058824,a:1.0};
    pub const HOT_PINK:Color = Color {r:1.00000000,g:0.41176471,b:0.70588235,a:1.0};
    pub const WATERMELON_PINK:Color = Color {r:0.98823529,g:0.42352941,b:0.52156863,a:1.0};
    pub const VIOLET_RED:Color = Color {r:0.96470588,g:0.20784314,b:0.54117647,a:1.0};
    pub const HOT_DEEP_PINK:Color = Color {r:0.96078431,g:0.15686275,b:0.52941176,a:1.0};
    pub const DEEP_PINK:Color = Color {r:1.00000000,g:0.07843137,b:0.57647059,a:1.0};
    pub const NEON_PINK:Color = Color {r:0.96078431,g:0.20784314,b:0.66666667,a:1.0};
    pub const NEON_HOT_PINK:Color = Color {r:0.99215686,g:0.20392157,b:0.61176471,a:1.0};
    pub const PINK_CUPCAKE:Color = Color {r:0.89411765,g:0.36862745,b:0.61568627,a:1.0};
    pub const DIMORPHOTHECA_MAGENTA:Color = Color {r:0.89019608,g:0.19215686,b:0.61568627,a:1.0};
    pub const PINK_LEMONADE:Color = Color {r:0.89411765,g:0.15686275,b:0.48627451,a:1.0};
    pub const RASPBERRY:Color = Color {r:0.89019608,g:0.04313725,b:0.36470588,a:1.0};
    pub const CRIMSON:Color = Color {r:0.86274510,g:0.07843137,b:0.23529412,a:1.0};
    pub const BRIGHT_MAROON:Color = Color {r:0.76470588,g:0.12941176,b:0.28235294,a:1.0};
    pub const ROSE_RED:Color = Color {r:0.76078431,g:0.11764706,b:0.33725490,a:1.0};
    pub const ROGUE_PINK:Color = Color {r:0.75686275,g:0.15686275,b:0.41176471,a:1.0};
    pub const BURNT_PINK:Color = Color {r:0.75686275,g:0.13333333,b:0.40392157,a:1.0};
    pub const PINK_VIOLET:Color = Color {r:0.79215686,g:0.13333333,b:0.41960784,a:1.0};
    pub const MEDIUM_VIOLET_RED:Color = Color {r:0.78039216,g:0.08235294,b:0.52156863,a:1.0};
    pub const DARK_CARNATION_PINK:Color = Color {r:0.75686275,g:0.13333333,b:0.51372549,a:1.0};
    pub const RASPBERRY_PURPLE:Color = Color {r:0.70196078,g:0.26666667,b:0.42352941,a:1.0};
    pub const PINK_PLUM:Color = Color {r:0.72549020,g:0.23137255,b:0.56078431,a:1.0};
    pub const ORCHID:Color = Color {r:0.85490196,g:0.43921569,b:0.83921569,a:1.0};
    pub const DEEP_MAUVE:Color = Color {r:0.87450980,g:0.45098039,b:0.83137255,a:1.0};
    pub const VIOLET:Color = Color {r:0.93333333,g:0.50980392,b:0.93333333,a:1.0};
    pub const BRIGHT_NEON_PINK:Color = Color {r:0.95686275,g:0.20000000,b:1.00000000,a:1.0};
    pub const FUCHSIA:Color = Color {r:1.00000000,g:0.00000000,b:1.00000000,a:1.0};
    pub const MAGENTA:Color = Color {r:1.00000000,g:0.00000000,b:1.00000000,a:1.0};
    pub const CRIMSON_PURPLE:Color = Color {r:0.88627451,g:0.21960784,b:0.92549020,a:1.0};
    pub const HELIOTROPE_PURPLE:Color = Color {r:0.83137255,g:0.38431373,b:1.00000000,a:1.0};
    pub const TYRIAN_PURPLE:Color = Color {r:0.76862745,g:0.35294118,b:0.92549020,a:1.0};
    pub const MEDIUM_ORCHID:Color = Color {r:0.72941176,g:0.33333333,b:0.82745098,a:1.0};
    pub const PURPLE_FLOWER:Color = Color {r:0.65490196,g:0.29019608,b:0.78039216,a:1.0};
    pub const ORCHID_PURPLE:Color = Color {r:0.69019608,g:0.28235294,b:0.70980392,a:1.0};
    pub const PASTEL_VIOLET:Color = Color {r:0.82352941,g:0.56862745,b:0.73725490,a:1.0};
    pub const MAUVE_TAUPE:Color = Color {r:0.56862745,g:0.37254902,b:0.42745098,a:1.0};
    pub const VIOLA_PURPLE:Color = Color {r:0.49411765,g:0.34509804,b:0.49411765,a:1.0};
    pub const EGGPLANT:Color = Color {r:0.38039216,g:0.25098039,b:0.31764706,a:1.0};
    pub const PLUM_PURPLE:Color = Color {r:0.34509804,g:0.21568627,b:0.34901961,a:1.0};
    pub const GRAPE:Color = Color {r:0.36862745,g:0.35294118,b:0.50196078,a:1.0};
    pub const PURPLE_NAVY:Color = Color {r:0.30588235,g:0.31764706,b:0.50196078,a:1.0};
    pub const SLATE_BLUE:Color = Color {r:0.41568627,g:0.35294118,b:0.80392157,a:1.0};
    pub const BLUE_LOTUS:Color = Color {r:0.41176471,g:0.37647059,b:0.92549020,a:1.0};
    pub const LIGHT_SLATE_BLUE:Color = Color {r:0.45098039,g:0.41568627,b:1.00000000,a:1.0};
    pub const MEDIUM_SLATE_BLUE:Color = Color {r:0.48235294,g:0.40784314,b:0.93333333,a:1.0};
    pub const PERIWINKLE_PURPLE:Color = Color {r:0.45882353,g:0.45882353,b:0.81176471,a:1.0};
    pub const PURPLE_AMETHYST:Color = Color {r:0.42352941,g:0.17647059,b:0.78039216,a:1.0};
    pub const BRIGHT_PURPLE:Color = Color {r:0.41568627,g:0.05098039,b:0.67843137,a:1.0};
    pub const DEEP_PERIWINKLE:Color = Color {r:0.32941176,g:0.32549020,b:0.65098039,a:1.0};
    pub const DARK_SLATE_BLUE:Color = Color {r:0.28235294,g:0.23921569,b:0.54509804,a:1.0};
    pub const PURPLE_HAZE:Color = Color {r:0.30588235,g:0.21960784,b:0.49411765,a:1.0};
    pub const PURPLE_IRIS:Color = Color {r:0.34117647,g:0.10588235,b:0.49411765,a:1.0};
    pub const DARK_PURPLE:Color = Color {r:0.29411765,g:0.00392157,b:0.31372549,a:1.0};
    pub const DEEP_PURPLE:Color = Color {r:0.21176471,g:0.00392157,b:0.24705882,a:1.0};
    pub const PURPLE_MONSTER:Color = Color {r:0.27450980,g:0.10588235,b:0.49411765,a:1.0};
    pub const INDIGO:Color = Color {r:0.29411765,g:0.00000000,b:0.50980392,a:1.0};
    pub const BLUE_WHALE:Color = Color {r:0.20392157,g:0.17647059,b:0.49411765,a:1.0};
    pub const REBECCA_PURPLE:Color = Color {r:0.40000000,g:0.20000000,b:0.60000000,a:1.0};
    pub const PURPLE_JAM:Color = Color {r:0.41568627,g:0.15686275,b:0.49411765,a:1.0};
    pub const DARK_MAGENTA:Color = Color {r:0.54509804,g:0.00000000,b:0.54509804,a:1.0};
    pub const PURPLE:Color = Color {r:0.50196078,g:0.00000000,b:0.50196078,a:1.0};
    pub const FRENCH_LILAC:Color = Color {r:0.52549020,g:0.37647059,b:0.55686275,a:1.0};
    pub const DARK_ORCHID:Color = Color {r:0.60000000,g:0.19607843,b:0.80000000,a:1.0};
    pub const DARK_VIOLET:Color = Color {r:0.58039216,g:0.00000000,b:0.82745098,a:1.0};
    pub const PURPLE_VIOLET:Color = Color {r:0.55294118,g:0.21960784,b:0.78823529,a:1.0};
    pub const JASMINE_PURPLE:Color = Color {r:0.63529412,g:0.23137255,b:0.92549020,a:1.0};
    pub const PURPLE_DAFFODIL:Color = Color {r:0.69019608,g:0.25490196,b:1.00000000,a:1.0};
    pub const CLEMANTIS_VIOLET:Color = Color {r:0.51764706,g:0.17647059,b:0.80784314,a:1.0};
    pub const BLUE_VIOLET:Color = Color {r:0.54117647,g:0.16862745,b:0.88627451,a:1.0};
    pub const PURPLE_SAGE_BUSH:Color = Color {r:0.47843137,g:0.36470588,b:0.78039216,a:1.0};
    pub const LOVELY_PURPLE:Color = Color {r:0.49803922,g:0.21960784,b:0.92549020,a:1.0};
    pub const NEON_PURPLE:Color = Color {r:0.61568627,g:0.00000000,b:1.00000000,a:1.0};
    pub const PURPLE_PLUM:Color = Color {r:0.55686275,g:0.20784314,b:0.93725490,a:1.0};
    pub const AZTECH_PURPLE:Color = Color {r:0.53725490,g:0.23137255,b:1.00000000,a:1.0};
    pub const LAVENDER_PURPLE:Color = Color {r:0.58823529,g:0.48235294,b:0.71372549,a:1.0};
    pub const MEDIUM_PURPLE:Color = Color {r:0.57647059,g:0.43921569,b:0.85882353,a:1.0};
    pub const LIGHT_PURPLE:Color = Color {r:0.51764706,g:0.40392157,b:0.84313725,a:1.0};
    pub const CROCUS_PURPLE:Color = Color {r:0.56862745,g:0.44705882,b:0.92549020,a:1.0};
    pub const PURPLE_MIMOSA:Color = Color {r:0.61960784,g:0.48235294,b:1.00000000,a:1.0};
    pub const PERIWINKLE:Color = Color {r:0.80000000,g:0.80000000,b:1.00000000,a:1.0};
    pub const PALE_LILAC:Color = Color {r:0.86274510,g:0.81568627,b:1.00000000,a:1.0};
    pub const MAUVE:Color = Color {r:0.87843137,g:0.69019608,b:1.00000000,a:1.0};
    pub const BRIGHT_LILAC:Color = Color {r:0.84705882,g:0.56862745,b:0.93725490,a:1.0};
    pub const RICH_LILAC:Color = Color {r:0.71372549,g:0.40000000,b:0.82352941,a:1.0};
    pub const PURPLE_DRAGON:Color = Color {r:0.76470588,g:0.55686275,b:0.78039216,a:1.0};
    pub const LILAC:Color = Color {r:0.78431373,g:0.63529412,b:0.78431373,a:1.0};
    pub const PLUM:Color = Color {r:0.86666667,g:0.62745098,b:0.86666667,a:1.0};
    pub const BLUSH_PINK:Color = Color {r:0.90196078,g:0.66274510,b:0.92549020,a:1.0};
    pub const PASTEL_PURPLE:Color = Color {r:0.94901961,g:0.63529412,b:0.90980392,a:1.0};
    pub const BLOSSOM_PINK:Color = Color {r:0.97647059,g:0.71764706,b:1.00000000,a:1.0};
    pub const WISTERIA_PURPLE:Color = Color {r:0.77647059,g:0.68235294,b:0.78039216,a:1.0};
    pub const PURPLE_THISTLE:Color = Color {r:0.82352941,g:0.72549020,b:0.82745098,a:1.0};
    pub const THISTLE:Color = Color {r:0.84705882,g:0.74901961,b:0.84705882,a:1.0};
    pub const PERIWINKLE_PINK:Color = Color {r:0.91372549,g:0.81176471,b:0.92549020,a:1.0};
    pub const COTTON_CANDY:Color = Color {r:0.98823529,g:0.87450980,b:1.00000000,a:1.0};
    pub const LAVENDER_PINOCCHIO:Color = Color {r:0.92156863,g:0.86666667,b:0.88627451,a:1.0};
    pub const ASH_WHITE:Color = Color {r:0.91372549,g:0.89411765,b:0.83137255,a:1.0};
    pub const WHITE_CHOCOLATE:Color = Color {r:0.92941176,g:0.90196078,b:0.83921569,a:1.0};
    pub const SOFT_IVORY:Color = Color {r:0.98039216,g:0.94117647,b:0.86666667,a:1.0};
    pub const OFF_WHITE:Color = Color {r:0.97254902,g:0.94117647,b:0.89019608,a:1.0};
    pub const LAVENDER_BLUSH:Color = Color {r:1.00000000,g:0.94117647,b:0.96078431,a:1.0};
    pub const PEARL:Color = Color {r:0.99215686,g:0.93333333,b:0.95686275,a:1.0};
    pub const EGG_SHELL:Color = Color {r:1.00000000,g:0.97647059,b:0.89019608,a:1.0};
    pub const OLDLACE:Color = Color {r:0.99215686,g:0.96078431,b:0.90196078,a:1.0};
    pub const LINEN:Color = Color {r:0.98039216,g:0.94117647,b:0.90196078,a:1.0};
    pub const SEASHELL:Color = Color {r:1.00000000,g:0.96078431,b:0.93333333,a:1.0};
    pub const RICE:Color = Color {r:0.98039216,g:0.96078431,b:0.93725490,a:1.0};
    pub const FLORAL_WHITE:Color = Color {r:1.00000000,g:0.98039216,b:0.94117647,a:1.0};
    pub const IVORY:Color = Color {r:1.00000000,g:1.00000000,b:0.94117647,a:1.0};
    pub const LIGHT_WHITE:Color = Color {r:1.00000000,g:1.00000000,b:0.96862745,a:1.0};
    pub const WHITE_SMOKE:Color = Color {r:0.96078431,g:0.96078431,b:0.96078431,a:1.0};
    pub const COTTON:Color = Color {r:0.98431373,g:0.98431373,b:0.97647059,a:1.0};
    pub const SNOW:Color = Color {r:1.00000000,g:0.98039216,b:0.98039216,a:1.0};
    pub const MILK_WHITE:Color = Color {r:0.99607843,g:0.98823529,b:1.00000000,a:1.0};
    pub const WHITE:Color = Color {r:1.00000000,g:1.00000000,b:1.00000000,a:1.0};
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Color {{r:{}, g:{}, b:{}, a:{}}}", self.r, self.g, self.b, self.a)
    }
}

impl From<String> for Color {
    fn from(s: String) -> Self {
        Self::from_hex(s)
    }
}
impl Into<String> for Color {
    fn into(self) -> String {
        self.to_hex()
    }
}

impl From<iced::Color> for Color {
    fn from(value: iced::Color) -> Self {
        Self::new(value.r, value.g, value.b, value.a)
    }
}
impl From<&iced::Color> for Color {
    fn from(value: &iced::Color) -> Self {
        Self::new(value.r, value.g, value.b, value.a)
    }
}
impl Into<iced::Color> for Color {
    fn into(self) -> iced::Color {
        iced::Color::from_rgba(self.r, self.g, self.b, self.a)
    }
}


impl Into<[f32;4]> for Color {
    fn into(self) -> [f32;4] {
        [self.r,self.g,self.b,self.a]
    }
}

// bad math ahead!!!!

// negative (invert color?)
impl Neg for Color {
    type Output = Color;
    fn neg(self) -> Self::Output {
        Color::new(
            1.0 - self.r, 
            1.0 - self.g,
            1.0 - self.b,
            1.0 - self.a,
        ).clamp()
    }
}

// add
impl Add<f32> for Color {
    type Output = Color;
    fn add(self, rhs: f32) -> Self::Output {
        Color::new(
            self.r + rhs, 
            self.g + rhs,
            self.b + rhs,
            self.a + rhs,
        ).clamp()
    }
}
impl Add<f64> for Color {
    type Output = Color;
    fn add(self, rhs: f64) -> Self::Output {
        self.add(rhs as f32)
    }
}
impl Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r + rhs.r, 
            self.g + rhs.g,
            self.b + rhs.b, 
            self.a + rhs.a,
        ).clamp()
    }
}
impl AddAssign<f32> for Color {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}
impl AddAssign<f64> for Color {
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs as f32;
    }
}
impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        *self = *self + rhs;
    }
}

// sub
impl Sub<f32> for Color {
    type Output = Color;
    fn sub(self, rhs: f32) -> Self::Output {
        self + -rhs
    }
}
impl Sub<f64> for Color {
    type Output = Color;
    fn sub(self, rhs: f64) -> Self::Output {
        self.sub(rhs as f32)
    }
}
impl Sub<Color> for Color {
    type Output = Color;
    fn sub(self, rhs: Color) -> Self::Output {
        Self::new(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a,
        ).clamp()
    }
}
impl SubAssign<f32> for Color {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}
impl SubAssign<f64> for Color {
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs as f32;
    }
}
impl SubAssign<Color> for Color {
    fn sub_assign(&mut self, rhs: Color) {
        *self = *self - rhs;
    }
}

// mul
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(
            self.r * rhs,
            self.g * rhs,
            self.b * rhs,
            self.a * rhs,
        ).clamp()
    }
}
impl Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        self.mul(rhs as f32)
    }
}
impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Self::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        ).clamp()
    }
}
impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}
impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs as f32;
    }
}
impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        *self = *self * rhs;
    }
}

// div
impl Div<f32> for Color {
    type Output = Color;
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(
            self.r / rhs,
            self.g / rhs,
            self.b / rhs,
            self.a / rhs,
        ).clamp()
    }
}
impl Div<f64> for Color {
    type Output = Color;
    fn div(self, rhs: f64) -> Self::Output {
        self.div(rhs as f32)
    }
}
impl Div<Color> for Color {
    type Output = Color;
    fn div(self, rhs: Color) -> Self::Output {
        Self::new(
            self.r / rhs.r,
            self.g / rhs.g,
            self.b / rhs.b,
            self.a / rhs.a,
        ).clamp()
    }
}
impl DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
impl DivAssign<f64> for Color {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs as f32;
    }
}
impl DivAssign<Color> for Color {
    fn div_assign(&mut self, rhs: Color) {
        *self = *self / rhs;
    }
}

// rem (mod)
impl Rem<f32> for Color {
    type Output = Color;
    fn rem(self, rhs: f32) -> Self::Output {
        Color::new(
            self.r % rhs, 
            self.g % rhs,
            self.b % rhs, 
            self.a % rhs,
        ).clamp()
    }
}
impl Rem<f64> for Color {
    type Output = Color;
    fn rem(self, rhs: f64) -> Self::Output {
        self.rem(rhs as f32)
    }
}
impl Rem<Color> for Color {
    type Output = Color;
    fn rem(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r % rhs.r, 
            self.g % rhs.g,
            self.b % rhs.b, 
            self.a % rhs.a,
        ).clamp()
    }
}
impl RemAssign<f32> for Color {
    fn rem_assign(&mut self, rhs: f32) {
        *self = *self % rhs;
    }
}
impl RemAssign<f64> for Color {
    fn rem_assign(&mut self, rhs: f64) {
        *self = *self % rhs as f32;
    }
}
impl RemAssign<Color> for Color {
    fn rem_assign(&mut self, rhs: Color) {
        *self = *self % rhs;
    }
}
