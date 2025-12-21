use syn::*;
use quote::*;
use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;

const CSS_ATTRIBUTE: &str = "css";
const NAME_ATTRIBUTE: &str = "name";
const SIZE_ATTRIBUTE: &str = "size";
const SHORTHAND_ATTRIBUTE: &str = "shorthand";
const SHORTHAND_FIELDS_ATTRIBUTE: &str = "shorthand_fields";
const DEFAULT_ATTRIBUTE: &str = "default";
const SKIP_ATTRIBUTE: &str = "skip";
const PARSE_WITH_ATTRIBUTE: &str = "parse_with";

macro_rules! try_error {
    ($($t:tt)+) => {
        match $($t)+ {
            Ok(ok) => ok,
            Err(e) => return e.into_compile_error(),
        }
    };
}

pub(crate) fn derive(derive: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let type_name = &derive.ident;
    let (impl_generics, ty_generics, where_clause) = derive.generics.split_for_impl();

    let mut match_tokens = proc_macro2::TokenStream::new();
    let mut shorthand_init_tokens = proc_macro2::TokenStream::new();
    let mut shorthand_cleanup_tokens = proc_macro2::TokenStream::new();
    
    let mut fields = Vec::new();
    let mut shorthand_fields = Vec::new();
    match &derive.data {
        syn::Data::Struct(s) => {
            for i in s.fields.iter() {
                let attributes = try_error!(CssAttributes::parse_from_attrs(i.attrs.as_slice()));
                
                if attributes.skip { continue }
                let parse_with = attributes
                    .parse_with
                    .unwrap_or("str::parse".to_owned())
                    .parse::<TokenStream>()
                    .unwrap();
                
                let ident = i.ident.as_ref().unwrap();
                let name = attributes.name.unwrap_or(
                    ident.to_string()
                        .trim_start_matches("_")
                        .replace('_', "-")
                );

                let default = attributes
                    .default
                    .unwrap_or("Default::default()".to_string())
                    .parse::<TokenStream>()
                    .unwrap();

                fn parse(s: &str) -> TokenStream {
                    s.parse::<TokenStream>().unwrap()
                }
                if let Some(params) = attributes.shorthand {
                    let ty = parse(&params);
                    let ident2 = ident.to_string();
                    let ident_trim = parse(ident2.trim_matches('_'));
                    shorthand_init_tokens.extend(quote! {
                        let mut #ident_trim = #ty::default();
                    });

                    match_tokens.extend(quote! {
                        #name => {
                            #ident_trim = std::str::FromStr::from_str(d.value)
                                .unwrap_or_default();
                        }
                    });
                    
                    if params == "DualShorthand" {
                        if attributes.size {
                            let ident2 = ident2.trim_start_matches("_").trim_end_matches("size");
                            let w = parse(&format!("{ident2}width"));
                            let h = parse(&format!("{ident2}height"));
                            shorthand_cleanup_tokens.extend(quote! {
                                this.#w = this.#w.merge(&#ident_trim.x).clone();
                                this.#h = this.#h.merge(&#ident_trim.y).clone();
                            });
                        } else {
                            let x = parse(&format!("{ident_trim}_x"));
                            let y = parse(&format!("{ident_trim}_y"));
                            shorthand_cleanup_tokens.extend(quote! {
                                this.#x = this.#x.merge(&#ident_trim.x).clone();
                                this.#y = this.#x.merge(&#ident_trim.y).clone();
                            });
                        }
                    } else if params == "QuadShorthand" {
                        let top = parse(&format!("{ident_trim}_top"));
                        let left = parse(&format!("{ident_trim}_left"));
                        let bottom = parse(&format!("{ident_trim}_bottom"));
                        let right = parse(&format!("{ident_trim}_right"));

                        shorthand_cleanup_tokens.extend(quote! {
                            this.#top = this.#top.merge(&#ident_trim.top).clone();
                            this.#left = this.#left.merge(&#ident_trim.left).clone();
                            this.#bottom = this.#bottom.merge(&#ident_trim.bottom).clone();
                            this.#right = this.#right.merge(&#ident_trim.right).clone();
                        });
                    } else {
                        for i in &attributes.shorthand_fields {
                            let field = parse(i);
                            shorthand_cleanup_tokens.extend(quote! {
                                this.#field = this.#field.merge(&#ident_trim.#field).clone();
                            });
                        }
                    }
                
                    shorthand_fields.push(i.ident.as_ref().unwrap());
                } else {
                    fields.push(i.ident.as_ref().unwrap());
                    match_tokens.extend(quote! {
                        #name => this.#ident = CssValue::parse(
                            d.value, 
                            #default, 
                            #parse_with
                        ),
                    });
                }
            }
        }

        _ => panic!("nope")
    }

    
    let tokens = quote! {
        impl #impl_generics #type_name #ty_generics where #where_clause {
            pub fn parse_css(rule: &simplecss::Rule) -> Self {
                let mut this = Self::default();
                #shorthand_init_tokens

                for d in rule.declarations.iter() {
                    match d.name {
                        #match_tokens
                        other => warn!("unknown css property: {other}"),
                    }
                }
                
                #shorthand_cleanup_tokens

                this
            }
            pub fn merge(self, parent: Self) -> Self {
                Self {
                    #(
                        #fields: self.#fields.check_unset(parent.#fields),
                    )*
                    #( #shorthand_fields: (), )*
                }
            }

            pub fn merge_parent(self, parent: Self) -> Self {
                Self {
                    #(
                        #fields: self.#fields.check_inherit(parent.#fields),
                    )*
                    #( #shorthand_fields: (), )*
                }
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
                        let id = stringify!(#fields);
                        let value = value_str(&self.#fields, values);

                        lines.push(format!("{spacing}<{id}>{value}</{id}>"));
                    )*
                }
                lines.push(format!("{spacing}</style>"));
            }
        }
    }; 
    // println!("{tokens}");

    tokens
}


#[derive(Default)]
struct CssAttributes {
    name: Option<String>,
    parse_with: Option<String>,
    default: Option<String>,
    shorthand: Option<String>,
    size: bool,
    skip: bool,

    shorthand_fields: Vec<String>,
}
impl CssAttributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut a = Self::default();

        for attr in attrs {
            if !attr.path().is_ident(CSS_ATTRIBUTE) { continue; }

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
                else if meta.path.is_ident(SHORTHAND_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let params = meta.input.parse::<LitStr>()?.value();
                    a.shorthand = Some(params);
                }
                else if meta.path.is_ident(PARSE_WITH_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let parse_with = meta.input.parse::<LitStr>()?.value();
                    a.parse_with = Some(parse_with);
                }
                else if meta.path.is_ident(SHORTHAND_FIELDS_ATTRIBUTE){
                    let fields;
                    parenthesized!(fields in meta.input);

                    let fields: Punctuated<LitStr, Token![,]> = fields
                        .call(Punctuated::parse_separated_nonempty)?;

                    a.shorthand_fields.extend(fields.into_iter().map(|i| i.value()));
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
