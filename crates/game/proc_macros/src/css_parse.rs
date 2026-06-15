use syn::*;
use quote::*;
use syn::parse::Parse;
use proc_macro2::TokenStream;

// TODO: clean this up once the css parsing has been moved to PropertyCollection

const CSS_ATTRIBUTE: &str = "css";
const NAME_ATTRIBUTE: &str = "name";
const SIZE_ATTRIBUTE: &str = "size";
const SHORTHAND_ATTRIBUTE: &str = "shorthand";
const SHORTHAND_FIELDS_ATTRIBUTE: &str = "shorthand_fields";
const DEFAULT_ATTRIBUTE: &str = "default";
const SKIP_ATTRIBUTE: &str = "skip";
const PARSE_WITH_ATTRIBUTE: &str = "parse_with";

pub(crate) fn derive(derive: &syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let type_name = &derive.ident;
    let (
        impl_generics, 
        ty_generics, 
        where_clause
    ) = derive.generics.split_for_impl();

    let mut shorthand_init_tokens = proc_macro2::TokenStream::new();
    let mut shorthand_cleanup_tokens = proc_macro2::TokenStream::new();
    
    let mut defaults = Vec::new();
    let mut parse_withs = Vec::new();


    let mut field_idents = Vec::new();
    let mut property_texts = Vec::new();


    let mut shorthand_tys = Vec::new();
    let mut shorthand_variables = Vec::new();
    let mut shorthand_field_idents = Vec::new();
    let mut shorthand_property_texts = Vec::new();
    let mut shorthand_fuck = Vec::new();
    
    let mut enum_variants = Vec::new();
    let mut shorthand_enum_variants = Vec::new();

    let mut style_enum_variants = Vec::new();
    let mut style_enum_variant_css_prop = Vec::new();
    let mut style_enum_as_value = Vec::new();

    let mut into_property_list = Vec::new();
    
    match &derive.data {
        syn::Data::Struct(s) => {
            for i in s.fields.iter() {
                let attributes = CssAttributes::parse_from_attrs(i.attrs.as_slice())?;
                
                if attributes.skip { continue }
                let parse_with = attributes
                    .parse_with
                    .unwrap_or_else(|| quote!(str::parse))
                    ;
                
                let ident = i.ident.as_ref().unwrap();
                let property_text = attributes.name.unwrap_or(
                    ident.to_string()
                        .trim_start_matches("_")
                        .replace('_', "-")
                );

                let default = attributes
                    .default
                    .unwrap_or("Default::default()".to_string())
                    .parse::<TokenStream>()
                    .unwrap();

                if let Some(ty) = attributes.shorthand {
                    // let ty = parse(&ty);
                    let ident2 = ident.to_string();
                    let var_name = parse(ident2.trim_matches('_'));
                    shorthand_init_tokens.extend(quote! {
                        let mut #var_name = #ty::default();
                    });
                    shorthand_tys.push(ty.clone());
                    shorthand_variables.push(var_name.clone());

                    shorthand_property_texts.push(property_text.clone());

                    let mut do_shorthand = |fields: Vec<TokenStream>, props: Vec<TokenStream>| {
                        shorthand_cleanup_tokens.extend(quote! {
                            #(
                                this.#fields = this.#fields.merge(&#var_name.#props).clone();
                            )*
                        });
                        shorthand_fuck.push(quote! {
                            #(
                                self.#fields = #var_name.#props;
                            )*
                        });
                    };

                    match ty.to_string().as_str() {
                        "DualShorthand" if attributes.size => {
                            let ident2 = ident2.trim_start_matches("_").trim_end_matches("size");
                            let w = parse(&format!("{ident2}width"));
                            let h = parse(&format!("{ident2}height"));
                            
                            let fields = vec![w, h];
                            let props = vec![
                                quote!(x),
                                quote!(y),
                            ];
                            do_shorthand(fields, props);
                        }
                        "DualShorthand" => {
                            let x = parse(&format!("{var_name}_x"));
                            let y = parse(&format!("{var_name}_y"));

                            let fields = vec![x, y];
                            let props = vec![
                                quote!(x),
                                quote!(y),
                            ];

                            do_shorthand(fields, props);
                        }
                    
                        "QuadShorthand" => {
                            let top = parse(&format!("{var_name}_top"));
                            let left = parse(&format!("{var_name}_left"));
                            let bottom = parse(&format!("{var_name}_bottom"));
                            let right = parse(&format!("{var_name}_right"));

                            let fields = vec![
                                top,
                                left,
                                bottom,
                                right,
                            ];
                            let props = vec![
                                quote!(top),
                                quote!(left),
                                quote!(bottom),
                                quote!(right),
                            ];

                            do_shorthand(fields, props);
                        }

                        _ => {
                            let shorthand_fields = attributes.shorthand_fields
                                .iter()
                                .map(|i| parse(i.as_str()))
                                .collect::<Vec<_>>();

                            for field in &shorthand_fields {
                                shorthand_cleanup_tokens.extend(quote! {
                                    this.#field = this.#field.merge(&#var_name.#field).clone();
                                });
                            }

                            shorthand_fuck.push(quote! {
                                #(
                                    self.#shorthand_fields = #var_name.#shorthand_fields;
                                )*
                            });
                        }
                    }
                    
                    let ident = i.ident.as_ref().unwrap();
                    shorthand_field_idents.push(ident);
                    let enum_variant = serde_case::RenameRule::PascalCase.apply_to_field(&ident.to_string());
                    shorthand_enum_variants.push(parse(&enum_variant));
                } else {
                    let ident = i.ident.as_ref().unwrap();
                    field_idents.push(ident);
                    property_texts.push(property_text.clone());
                    defaults.push(default.clone());
                    parse_withs.push(parse_with.clone());

                    let variant_name = serde_case::RenameRule::PascalCase
                        .apply_to_field(ident.to_string().as_str());
                    let variant_name = parse(&variant_name);
                    enum_variants.push(variant_name.clone());
                    
                    let ty = &i.ty;

                    style_enum_variants.extend(quote! {
                        #variant_name (#ty),
                    });
                    style_enum_variant_css_prop.extend(quote! {
                        Self::#variant_name(_) => CssProperty::#variant_name,
                    });

                    into_property_list.extend(quote! {
                        match self.#ident {
                            CssValue::Unset => {}
                            CssValue::Inherit if skip_inherited => {},
                            other => list.push(StyleProperty::#variant_name (other)),
                        }
                    });

                    style_enum_as_value.extend(quote! {
                        Self::#variant_name(v) => (v as &dyn std::any::Any).downcast_ref::<CssValue<T>>().unwrap(),
                    });

                }
            }
        }

        _ => panic!("nope")
    }
    
    let tokens = quote! {
        impl #impl_generics #type_name #ty_generics where #where_clause {
            pub fn parse_css(rule: &simplecss::Rule<ArcStr>) -> Self {
                let mut this = Self::default();
                #shorthand_init_tokens

                for d in rule.declarations.iter() {
                    match &*d.name {
                        #(
                            #property_texts => this.#field_idents = CssValue::parse(
                                &d.value, 
                                #defaults, 
                                #parse_withs
                            ),
                        )*
                        #(
                            #shorthand_property_texts => {
                                #shorthand_variables = std::str::FromStr::from_str(&d.value)
                                    .unwrap_or_default();
                            }
                        )*

                        other => warn!("unknown css property: {other}"),
                    }
                }
                
                #shorthand_cleanup_tokens

                this
            }
            
            pub fn into_property_list(self, skip_inherited: bool) -> Vec<StyleProperty> {
                let mut list = Vec::new();

                #( #into_property_list )*

                list
            }

            pub fn export_xml(
                &self, 
                lines: &mut Vec<String>,
                indent: usize,
                values: &dyn Reflect,
            ) {
                fn value_str<V: std::fmt::Debug + Reflect>(
                    v: &CssValue<V>, 
                    values: &dyn Reflect
                ) -> String {
                    match v.resolve(values) {
                        Some(v) => format!("{:?}", *v),
                        None => format!("{v:?}")
                    }
                }
                
                let spacing = "  ".repeat(indent);
                lines.push(format!("{spacing}<style>"));
                {
                    let spacing = "  ".repeat(indent+1);
                    #(
                        let id = stringify!(#field_idents);
                        let value = value_str(&self.#field_idents, values);

                        lines.push(format!("{spacing}<{id}>{value}</{id}>"));
                    )*
                }
                lines.push(format!("{spacing}</style>"));
            }
        }

        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        #[repr(u8)]
        pub enum CssProperty {
            #( #enum_variants, )*
            #( #shorthand_enum_variants, )*
        }

        impl std::str::FromStr for CssProperty {
            type Err = ();
            fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                Ok(match s {
                    #(
                        #property_texts => Self::#enum_variants,
                    )*
                    #(
                        #shorthand_property_texts => Self::#shorthand_enum_variants,
                    )*
                    _ => return Err(())
                })
            }
        }
    
        #[derive(Clone, Debug)]
        pub enum StyleProperty {
            #( #style_enum_variants )*
        }
        impl StyleProperty {
            pub fn css_property(&self) -> CssProperty {
                match self {
                    #( #style_enum_variant_css_prop )*
                }
            }
            pub fn is_property(&self, prop: CssProperty) -> bool {
                self.css_property() == prop
            }

            pub fn value<T: std::any::Any>(&self) -> &CssValue<T> {
                match self {
                    #( #style_enum_as_value )*
                }
            }
        }
    
    }; 
    println!("{tokens}");

    Ok(tokens)
}


#[derive(Default)]
struct CssAttributes {
    name: Option<String>,
    parse_with: Option<TokenStream>,
    default: Option<String>,
    shorthand: Option<Ident>,
    size: bool,
    skip: bool,

    shorthand_fields: Vec<String>,
}
impl CssAttributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut a = Self::default();

        for attr in attrs {
            let path = attr.path();
            if path.is_ident(SHORTHAND_ATTRIBUTE) {

                struct ShorthandExtra {
                    _as: Token![as],
                    _value: Ident,
                }
                impl syn::parse::Parse for ShorthandExtra {
                    fn parse(input: parse::ParseStream) -> Result<Self> {
                        Ok(Self {
                            _as: input.parse()?,
                            _value: input.parse()?,
                        })
                    }
                }
                struct Shorthand {
                    value: Ident,
                    extra: Option<ShorthandExtra>
                }
                impl syn::parse::Parse for Shorthand {
                    fn parse(input: parse::ParseStream) -> Result<Self> {
                        Ok(Self {
                            value: input.parse()?,
                            extra: input.parse().ok(),
                        })
                    }
                }
                
                let s = attr.parse_args::<Shorthand>()?;
                a.shorthand = Some(s.value);
                if let Some(extra) = s.extra {
                    if extra._value == "size" {
                        a.size = true;
                    }
                }

                continue;
            }
            else if path.is_ident(SHORTHAND_FIELDS_ATTRIBUTE) {
                let fields = attr.parse_args_with(|s: &parse::ParseBuffer<'_>| {
                    let mut list = Vec::new();
                    list.push(s.parse::<Ident>()?);

                    while s.parse::<Token![,]>().is_ok() {
                        list.push(s.parse::<Ident>()?);
                    }

                    Ok(list)
                })?;
                a.shorthand_fields.extend(fields.into_iter().map(|i| i.to_string()));

                // attr.parse_nested_meta(|i| {
                //     let fields;
                //     parenthesized!(fields in i.input);
                        
                //     let fields: Punctuated<Ident, Token![,]> = fields
                //         .call(Punctuated::parse_separated_nonempty)?;

                //     a.shorthand_fields.extend(fields.into_iter().map(|i| i.value()));
                //     Ok(())
                // })?;

                continue;
            }
            else if path.is_ident(PARSE_WITH_ATTRIBUTE) {
                a.parse_with = Some(attr.parse_args_with(TokenStream::parse)?);
                // meta.parse_nested_meta(|m| {
                //     a.parse_with = Some(m.input.parse::<TokenStream>()?);
                //     Ok(())
                // })?;

                // let _ = meta.value()?;
                // let parse_with = meta.input.parse::<LitStr>()?.value();
                // a.parse_with = Some(parse_with);
                continue;
            }

            if !path.is_ident(CSS_ATTRIBUTE) { continue; }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(NAME_ATTRIBUTE) {
                    let _ = meta.value()?;

                    let name = meta.input.parse::<LitStr>()?.value();
                    a.name = Some(name);
                }
                else if meta.path.is_ident(SKIP_ATTRIBUTE) {
                    a.skip = true;
                }
                else if meta.path.is_ident(SIZE_ATTRIBUTE) {
                    a.size = true;
                }
                else if meta.path.is_ident(DEFAULT_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let default = meta.input.parse::<LitStr>()?.value();
                    a.default = Some(default);
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



fn parse(s: &str) -> TokenStream {
    s.parse::<TokenStream>().unwrap()
}


// from https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/case.rs
#[allow(unused)]
mod serde_case {
    //! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to the
    //! case of the source (e.g. `my-field`, `MY_FIELD`).
    use self::RenameRule::*;

    /// The different possible ways to change case of fields in a struct, or variants in an enum.
    #[derive(Copy, Clone, PartialEq)]
    pub enum RenameRule {
        /// Don't apply a default rename rule.
        None,
        /// Rename direct children to "lowercase" style.
        LowerCase,
        /// Rename direct children to "UPPERCASE" style.
        UpperCase,
        /// Rename direct children to "PascalCase" style, as typically used for
        /// enum variants.
        PascalCase,
        /// Rename direct children to "camelCase" style.
        CamelCase,
        /// Rename direct children to "snake_case" style, as commonly used for
        /// fields.
        SnakeCase,
        /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
        /// used for constants.
        ScreamingSnakeCase,
        /// Rename direct children to "kebab-case" style.
        KebabCase,
        /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
        ScreamingKebabCase,
    }

    // static RENAME_RULES: &[(&str, RenameRule)] = &[
    //     ("lowercase", LowerCase),
    //     ("UPPERCASE", UpperCase),
    //     ("PascalCase", PascalCase),
    //     ("camelCase", CamelCase),
    //     ("snake_case", SnakeCase),
    //     ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
    //     ("kebab-case", KebabCase),
    //     ("SCREAMING-KEBAB-CASE", ScreamingKebabCase),
    // ];

    impl RenameRule {
        /// Apply a renaming rule to an enum variant, returning the version expected in the source.
        pub fn apply_to_variant(self, variant: &str) -> String {
            match self {
                None | PascalCase => variant.to_owned(),
                LowerCase => variant.to_ascii_lowercase(),
                UpperCase => variant.to_ascii_uppercase(),
                CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
                SnakeCase => {
                    let mut snake = String::new();
                    for (i, ch) in variant.char_indices() {
                        if i > 0 && ch.is_uppercase() {
                            snake.push('_');
                        }
                        snake.push(ch.to_ascii_lowercase());
                    }
                    snake
                }
                ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
                KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
                ScreamingKebabCase => ScreamingSnakeCase
                    .apply_to_variant(variant)
                    .replace('_', "-"),
            }
        }

        /// Apply a renaming rule to a struct field, returning the version expected in the source.
        pub fn apply_to_field(self, field: &str) -> String {
            match self {
                None | LowerCase | SnakeCase => field.to_owned(),
                UpperCase => field.to_ascii_uppercase(),
                PascalCase => {
                    let mut pascal = String::new();
                    let mut capitalize = true;
                    for ch in field.chars() {
                        if ch == '_' {
                            capitalize = true;
                        } else if capitalize {
                            pascal.push(ch.to_ascii_uppercase());
                            capitalize = false;
                        } else {
                            pascal.push(ch);
                        }
                    }
                    pascal
                }
                CamelCase => {
                    let pascal = PascalCase.apply_to_field(field);
                    pascal[..1].to_ascii_lowercase() + &pascal[1..]
                }
                ScreamingSnakeCase => field.to_ascii_uppercase(),
                KebabCase => field.replace('_', "-"),
                ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
            }
        }

    }
}


