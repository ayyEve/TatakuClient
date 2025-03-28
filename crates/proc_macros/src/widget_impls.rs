use quote::*;
use syn::*;
use syn::punctuated::Punctuated;

pub const WIDGET_ATTRIBUTE: &str = "widget";
pub const TYPE_ATTRIBUTE: &str = "type";
pub const CONTAINER_TYPE: &str = "container";
pub const TEXT_TYPE: &str = "text";

pub const STYLE_PATH: &str = "style_path";
pub const TEXT_STYLE_PATH: &str = "text_style_path";

macro_rules! try_error {
    ($($t:tt)+) => {
        match $($t)+ {
            Ok(ok) => ok,
            Err(e) => return e.into_compile_error(),
        }
    };
}


pub fn derive(derive: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let type_name = &derive.ident;
    let (impl_generics, ty_generics, where_clause) = derive.generics.split_for_impl();

    let global_attributes = try_error!(WidgetAttributes::parse_from_attrs(derive.attrs.as_slice(), true));

    let style_path = global_attributes.global_style_path.unwrap_or("style".to_owned());
    let style_path = style_path.split(".").map(|i| format_ident!("{i}")).collect::<Vec<_>>();
    let style_path = quote! { #(#style_path.)* };

    let text_style = global_attributes.global_text_style_path.unwrap_or("text_style".to_owned());
    let text_style = text_style.split(".").map(|i| format_ident!("{i}")).collect::<Vec<_>>();
    let text_style = quote! { #(#text_style.)* };

    let mut tokens = proc_macro2::TokenStream::new();

    tokens.extend(quote! {
        pub fn width(mut self, width: impl Into<Dimension>) -> Self {
            self.#style_path size.width = width.into();
            self
        }

        pub fn height(mut self, height: impl Into<Dimension>) -> Self {
            self.#style_path size.height = height.into();
            self
        }

        pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
            self.#style_path margin = margin.into().0;
            self
        }
        pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
            self.#style_path padding = padding.into().0;
            self
        }
    });
    if global_attributes.global_container {
        tokens.extend(quote! {

            pub fn spacing(mut self, amount: LengthPercentage) -> Self {
                // TODO!!!!!!!!!
                self.#style_path gap = Size {
                    width: amount,
                    height: amount,
                };
                self
            }

            pub fn flex_direction(mut self, direction: impl Into<FlexDirection>) -> Self {
                self.#style_path flex_direction = direction.into();
                self
            }

            pub fn is_vertical(&self) -> bool {
                match &self.#style_path flex_direction {
                    FlexDirection::Row
                    | FlexDirection::RowReverse => {
                        true
                    }
                    FlexDirection::Column
                    | FlexDirection::ColumnReverse => {
                        false
                    }
                }
            }
            pub fn is_horizontal(&self) -> bool {
                !self.is_vertical()
            }

            pub fn horizontal_align(mut self, align: taffy::AlignContent) -> Self {
                match &self.#style_path flex_direction {
                    FlexDirection::Row
                    | FlexDirection::RowReverse => {
                        self.#style_path justify_content = Some(align);
                    }
                    FlexDirection::Column
                    | FlexDirection::ColumnReverse => {
                        self.#style_path align_content = Some(align);
                    }
                }

                self
            }

            pub fn vertical_align(mut self, align: taffy::AlignContent) -> Self {
                match &self.#style_path flex_direction {
                    FlexDirection::Row
                    | FlexDirection::RowReverse => {
                        self.#style_path align_content = Some(align);
                    }
                    FlexDirection::Column
                    | FlexDirection::ColumnReverse => {
                        self.#style_path justify_content = Some(align);
                    }
                }

                self
            }

            pub fn horizontal_overflow(mut self, overflow: taffy::Overflow) -> Self {
                self.#style_path overflow.x = overflow;
                self
            }
            pub fn vertical_overflow(mut self, overflow: taffy::Overflow) -> Self {
                self.#style_path overflow.y = overflow;
                self
            }

            pub fn flex_wrap(mut self, wrap: taffy::FlexWrap) -> Self {
                self.#style_path flex_wrap = wrap;
                self
            }
            
            pub fn scrollbar_width(mut self, width: f32) -> Self {
                self.#style_path scrollbar_width = width;
                self
            }



        });
    }

    if global_attributes.global_text {
        tokens.extend(quote! {
            pub fn font(mut self, font: Font) -> Self {
                self.#text_style font = font;
                self
            }

            pub fn font_size(mut self, size: f32) -> Self {
                self.#text_style font_size = size;
                self
            }
            pub fn font_size_maybe(mut self, size: Option<f32>) -> Self {
                if let Some(size) = size {
                    self.#text_style font_size = size;
                }
                self
            }

            pub fn text_color(mut self, color: Color) -> Self {
                self.#text_style color = color;
                self
            }
            
            pub fn line_height(mut self, height: f32) -> Self {
                self.#text_style line_height = height;
                self
            }

            pub fn text_align(mut self, align: Alignment) -> Self {
                self.#text_style alignment = align;
                self
            }

            pub fn text_h_align(mut self, align: HorizontalAlign) -> Self {
                self.#text_style alignment.horizontal = align;
                self
            }
            pub fn text_v_align(mut self, align: VerticalAlign) -> Self {
                self.#text_style alignment.vertical = align;
                self
            }
        });
    }

    quote! {
        impl #impl_generics #type_name #ty_generics where #where_clause {
            #tokens
        }
    }
}




#[derive(Default)]
struct WidgetAttributes {
    // global_element: bool, // really should always be true
    global_container: bool,
    global_text: bool,
    global_style_path: Option<String>,
    global_text_style_path: Option<String>,


    // style_path: bool,
    // text_style_path: bool, 
}
impl WidgetAttributes {
    fn parse_from_attrs(attrs: &[Attribute], _global: bool) -> Result<Self> {
        let mut a = Self::default();

        for attr in attrs {
            if !attr.path().is_ident(WIDGET_ATTRIBUTE) { continue; }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(STYLE_PATH) {
                    let _ = meta.value()?;

                    let path = meta.input.parse::<LitStr>()?.value();
                    a.global_style_path = Some(path);
                }
                else if meta.path.is_ident(TEXT_STYLE_PATH) {
                    let _ = meta.value()?;
                    let path = meta.input.parse::<LitStr>()?.value();
                    a.global_text_style_path = Some(path);
                }

                else if meta.path.is_ident(TYPE_ATTRIBUTE) {
                    let types;
                    parenthesized!(types in meta.input);
                    let types: Punctuated<LitStr, Token![,]> = types.call(Punctuated::parse_separated_nonempty)?;

                    for i in types.iter() {
                        match &*i.value() {
                            CONTAINER_TYPE => a.global_container = true,
                            TEXT_TYPE => a.global_text = true,
                            _ => return Err(meta.error("invalid type"))
                        }
                    }
                }
                else {
                    return Err(meta.error("Invalid attribute"))
                }

                Ok(())
            })?;
        }

        Ok(a)
    }
}