use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct BuildableTextTag {
    #[serde(alias = "$value", alias = "$text")] 
    pub value: BuildableText
}
crate::impl_tag!(BuildableTextTag, BuildableText, value);


#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct BuildableText {
    #[serde(rename="@join", default)] pub join: Option<String>,
    #[serde(rename="$value", alias="$text")] pub text: Vec<BuildableTextInner>
}
impl BuildableText {
    pub fn compute(&mut self) -> ShuntingYardResult<()> {
        for i in self.text.iter_mut() {
            i.compute()?
        }

        Ok(())
    }

    pub fn to_string(&self, values: &dyn Reflect) -> String {
        self.text
            .iter()
            .map(|i| i.to_string(values))
            .collect::<Vec<_>>()
            .join(self.join.as_deref().unwrap_or(""))
    }
}
impl From<BuildableTextInner> for BuildableText {
    fn from(value: BuildableTextInner) -> Self {
        Self {
            join: None,
            text: vec![value]
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub enum BuildableTextInner {
    Variable(String),
    Locale(String),
    Calc(String),

    /// calc but parsed, should not be read into
    #[serde(skip)] CalcParsed(Arc<BuildableCalc>, String),
    #[serde(alias = "$text")] Text(String),
    
    TextIter {
        #[serde(alias = "@variable")] variable: String,
        #[serde(alias = "@property")] property: Option<String>,
        #[serde(alias = "@join")] join: String,
    },
}
impl BuildableTextInner {
    /// Parses Self::Calc into Self::CalcParsed
    pub fn compute(&mut self) -> ShuntingYardResult<()> {
        match self {
            Self::Calc(s) => {
                let s = s.clone();
                *self = Self::CalcParsed(Arc::new(BuildableCalc::parse(&s)?), s)
            }
            // because json pointers use '/' and not '.', but '.' is nicer for locale
            // "dialog.confirmation.yes" (us) vs "dialog/confirmation/yes" (json)
            Self::Locale(s) => *s = s.replace('.', "/"),

            _ => {}
        }

        Ok(())
    }

    pub fn as_buildable(self) -> BuildableText {
        self.into()
    }

    pub fn to_string(&self, values: &dyn Reflect) -> String {
        match self {
            Self::Variable(t) => values.reflect_get::<String>(t).as_deref().cloned().unwrap_or_else(|_| format!("Invalid property: '{t}'")),
            Self::Text(t) | Self::Locale(t) => t.clone(),

            Self::CalcParsed(calc, calc_str) => {
                match calc.resolve(values) {
                    Ok(val) => val.as_string(),
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{calc_str}', error: {e:?}");
                        "Calc error! See console.".to_string()
                    }
                }
            }

            Self::TextIter { 
                variable, 
                property, 
                join 
            } => {
                match values.reflect_iter(variable) {
                    Ok(iter) => {
                        let mut list = Vec::new();

                        for i in iter {
                            if let Some(property) = &property {
                                let Ok(v) = i.impl_get(ReflectPath::new(property))
                                    .inspect_err(|e| error!("error with text iter prop: {e:?}")) 
                                    else { return String::new() };
                                
                                let str = match v {
                                    MaybeOwnedReflect::Borrowed(reflect) => try_get_string(reflect),
                                    MaybeOwnedReflect::Owned(reflect) => try_get_string(&*reflect),
                                };
                                
                                if let Some(s) = str {
                                    list.push(s);
                                }
                            } else if let Some(s) = try_get_string(i) {
                                list.push(s);
                            }
                        }

                        list.join(join)
                    }
                    Err(e) => {
                        error!("error with text iter: {e:?}");
                        "ERROR!!!".into()
                    }
                }
            }

            Self::Calc(_t) => panic!("You forgot to parse a calc!"),
        }
    }
}
impl Default for BuildableTextInner {
    fn default() -> Self {
        Self::Text(String::new())
    }
}


fn try_get_string(r: &dyn Reflect) -> Option<String> {
    match r.downcast_ref::<String>().cloned() {
        Some(s) => Some(s),
        None => r.downcast_ref::<&str>().map(|s| s.to_string())
    }
}
