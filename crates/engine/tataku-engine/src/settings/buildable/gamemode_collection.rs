use crate::*;
use common::reflect::*;
use engine::gameplay::{
    GamemodeInfos, 
    GamemodeSettings,
};

#[derive(Debug, Default)]
#[derive(Serialize, Deserialize)]
#[serde(from="HashMap<String, serde_json::Value>")]
#[serde(into="HashMap<String, serde_json::Value>")]
pub struct GamemodeSettingsCollection {
    collection: HashMap<String, serde_json::Value>,
    built: HashMap<String, Box<dyn GamemodeSettings>>,
    infos: Option<GamemodeInfos>,
}
impl GamemodeSettingsCollection {
    pub fn build(&mut self, infos: GamemodeInfos) {
        self.infos = Some(infos);
        self.rebuild();
    }

    pub fn rebuild(&mut self) {
        let infos = self.infos.as_ref().unwrap();

        for info in infos.by_id.values() {
            let playmode = info.id;
            let value = self.collection
                .get(playmode)
                .cloned()
                .unwrap_or_default();

            let Some(settings) = info
                .deserialize_settings(value.clone()) 
            else {
                warn!("Couldn't deserialize settings for playmode {playmode}");
                continue
            };

            self.built.insert(playmode.to_string(), settings);
        }
    }

    pub fn update(&mut self) {
        for (id, settings) in self.built.iter() {
            let value = settings.to_value();
            let old = self
                .collection
                .entry(id.clone())
                .or_default();

            *old = value;
        }
    }
}

impl Reflect for GamemodeSettingsCollection {
    fn impl_get<'s, 'v>(
        &'s self, 
        mut path: ReflectPath<'v>,
    ) -> reflect::Result<'v, MaybeOwnedReflect<'s>> {
        let Some(key) = path.next() else {
            return Ok((self as &dyn Reflect).into())
        };
        let b = self
            .built
            .get(key)
            .ok_or(ReflectError::EntryNotExist { entry: Cow::Borrowed(key) })?;

        b.impl_get(path)
    }

    fn impl_get_mut<'s, 'v>(
        &'s mut self, 
        mut path: ReflectPath<'v>
    ) -> reflect::Result<'v, &'s mut dyn Reflect> {
        let Some(key) = path.next() else {
            return Ok(self as &mut dyn Reflect)
        };
        let b = self
            .built
            .get_mut(key)
            .ok_or(ReflectError::EntryNotExist { entry: Cow::Borrowed(key) })?;

        b.impl_get_mut(path)
    }

    fn impl_insert<'v>(
        &mut self, 
        path: ReflectPath<'v>, 
        value: Box<dyn Reflect>,
    ) -> reflect::Result<'v, ()> {
        self
            .impl_get_mut(path)?
            .impl_insert(ReflectPath::new(""), value)?;
        Ok(())
    }

    fn impl_as_number<'v>(
        &self, 
        mut path: ReflectPath<'v>
    ) -> reflect::Result<'v, ReflectNumber> {
        let Some(key) = path.next() else {
            return Err(ReflectError::NotANumber)
        };
        let b = self
            .built
            .get(key)
            .ok_or(ReflectError::EntryNotExist { entry: Cow::Borrowed(key) })?;

        b.impl_as_number(path)
    }

    fn impl_display<'v>(
        &self, 
        mut path: ReflectPath<'v>, 
        precision: Option<usize>
    ) -> reflect::Result<'v, String> {
        let Some(key) = path.next() else {
            return Err(ReflectError::NoDisplay)
        };
        let b = self
            .built
            .get(key)
            .ok_or(ReflectError::EntryNotExist { entry: Cow::Borrowed(key) })?;

        b.impl_display(path, precision)
    }
    
    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.clone()))
    }
}

impl Deref for GamemodeSettingsCollection {
    type Target = HashMap<String, serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.collection
    }
}
impl DerefMut for GamemodeSettingsCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.collection
    }
}


impl From<HashMap<String, serde_json::Value>> for GamemodeSettingsCollection {
    fn from(value: HashMap<String, serde_json::Value>) -> Self {
        Self {
            collection: value,
            built: HashMap::default(),
            infos: None,
        }
    }
}
impl From<GamemodeSettingsCollection> for HashMap<String, serde_json::Value> {
    fn from(value: GamemodeSettingsCollection) -> Self {
        value.collection
    }
}
impl PartialEq for GamemodeSettingsCollection {
    fn eq(&self, _other: &Self) -> bool {
        // optimization because json::Value::eq is slow af
        true
        // self.collection.eq(&other.collection)
    }
}
impl Clone for GamemodeSettingsCollection {
    fn clone(&self) -> Self {
        Self {
            collection: self.collection.clone(),
            infos: self.infos.clone(),

            built: self
                .built
                .iter()
                .map(|(k, v)| (
                    k.clone(),
                    v.duplicate_settings()
                ))
                .collect()
        }
    }
}


#[cfg(feature="graphics")]
impl settings::MakeSettingsMenu for GamemodeSettingsCollection {
    fn create_provider(
        &self, 
        prefix: String,
        builder: &mut settings::SettingsBuilder,
    ) {
        let infos = self.infos.as_ref().unwrap();

        for (playmode, settings) in self.built.iter() {
            let Ok(info) = infos
                .get_info(playmode)
            else { continue };

            let path = format!("{prefix}.{playmode}");
            builder.add_category(info.display_name, None::<&str>);
            settings.create_provider(path, builder);
        }
    }
}
