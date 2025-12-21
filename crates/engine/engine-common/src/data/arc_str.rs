use std::sync::Arc;
use std::sync::LazyLock;
use tataku_common::reflection::*;

static EMPTY: LazyLock<ArcStr> = LazyLock::new(|| ArcStr(String::new().into()));

#[derive(Clone, Eq)]
pub struct ArcStr(Arc<str>);
impl ArcStr {
    pub fn unknown() -> Self {
        static STR: LazyLock<ArcStr> = LazyLock::new(|| ArcStr("unknown".into()));
        STR.clone()
    }

    pub fn join(slice: &[Self], sep: &str) -> String {
        slice
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }
}

impl Default for ArcStr {
    fn default() -> Self {
        EMPTY.clone()
    }
}
impl std::fmt::Debug for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for ArcStr {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl PartialEq<String> for ArcStr {
    fn eq(&self, other: &String) -> bool {
        &*self.0 == other
    }
}
impl PartialEq<&str> for ArcStr {
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}
impl PartialEq<std::borrow::Cow<'_, str>> for ArcStr {
    fn eq(&self, other: &std::borrow::Cow<'_, str>) -> bool {
        **self == **other
    }
}


impl core::hash::Hash for ArcStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::ops::Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl From<String> for ArcStr {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
impl From<&str> for ArcStr {
    fn from(value: &str) -> Self {
        Self(value.to_string().into())
    }
}
impl From<Arc<str>> for ArcStr {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl AsRef<std::ffi::OsStr> for ArcStr {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_ref().as_ref()
    }
}

impl<'de> serde::Deserialize<'de> for ArcStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(s.into())
    }
}
impl serde::Serialize for ArcStr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        <&str>::serialize(&&*self.0, serializer)
    }
}

impl Reflect for ArcStr {
    fn impl_get<'s, 'v>(&'s self, path: ReflectPath<'v>) -> reflect::Result<'v, MaybeOwnedReflect<'s>> {
        self.0.impl_get(path)
    }

    fn impl_get_mut<'s, 'v>(&'s mut self, path: ReflectPath<'v>) -> reflect::Result<'v, &'s mut dyn Reflect> {
        self.0.impl_get_mut(path)
    }

    fn impl_insert<'v>(&mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> reflect::Result<'v, ()> {
        self.0.impl_insert(path, value)
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.clone()))
    }

    fn impl_display<'v>(&self, _: ReflectPath<'v>, _: Option<usize>) -> reflect::Result<'v, String> {
        Ok(self.to_string())
    }

}
impl Stringable for ArcStr {
    type Err = ();
    fn parse_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}
impl<'a> From<&'a ArcStr> for ReflectPath<'a> {
    fn from(value: &'a ArcStr) -> Self {
        ReflectPath::new(&value.0)
    }
}
