use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct GamemodeSettingsCollection(HashMap<String, serde_json::Value>);
impl Deref for GamemodeSettingsCollection {
    type Target = HashMap<String, serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for GamemodeSettingsCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


#[cfg(feature="graphics")]
impl MakeSettingsMenu for GamemodeSettingsCollection {
    fn into_elements(
        &self, 
        _prefix: String,
        owner: MessageOwner, 
        builder: &mut SettingsBuilder<'_>,
    ) {
        let infos = builder.values.reflect_get::<GamemodeInfos>("global.infos").unwrap().cloned();

        for info in infos.by_id.values() {
            let playmode = info.id;
            let value = self.0.get(playmode).cloned().unwrap_or_default();
            let path = format!("var.{playmode}_config");
            let value_path = format!("var.{playmode}_config_value");

            let Some(settings) = info.deserialize_settings(value.clone()) else {
                warn!("couldnt deserialize settings for playmode {playmode}");
                continue
            };

            builder.add_category(info.display_name);
            settings.into_elements(path.clone(), owner, builder);

            builder.values.reflect_insert(
                &path, 
                settings
            )
            .unwrap();

            builder.values.reflect_insert(
                &value_path, 
                ReflectJsonValue(value)
            )
            .unwrap();
        }
    }

    fn from_elements(
        &mut self,
        tags: &mut ReflectPath,
        message: Message,
        extras: &mut FromElementsExtra<'_>,
    ) {
        let Some(playmode) = tags.next() else { return println!("aaaa"); };
        let playmode = playmode.trim_end_matches("_config");

        // let Some(value) = self.0.get(playmode) else { return };
        let info = extras.values
            .reflect_get::<GamemodeInfos>("global.infos")
            .unwrap().cloned()
            .get_info(playmode)
            .cloned().unwrap();
        
        let path = format!("var.{playmode}_config");
        let value_path = format!("var.{playmode}_config_value");
        let tmp = extras.values.reflect_get::<ReflectJsonValue>(
            &value_path
        ).unwrap().0.clone();

        let mut settings = info.deserialize_settings(tmp).unwrap();
        settings.from_elements(tags, message, extras);
        let value = settings.to_value();

        *self.0.entry(playmode.to_owned()).or_default() = value.clone();
        extras.values.reflect_insert(&path, settings).unwrap();
        extras.values.reflect_insert(&value_path, ReflectJsonValue(value)).unwrap();
    }
}


// yay dirty hack
struct ReflectJsonValue(serde_json::Value);
impl Reflect for ReflectJsonValue {
    fn impl_get<'s, 'v>(&'s self, _path: ReflectPath<'v>) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        Ok(MaybeOwnedReflect::Borrowed(self))
    }

    fn impl_get_mut<'s, 'v>(&'s mut self, _path: ReflectPath<'v>) -> ReflectResult<'v, &'s mut dyn Reflect> {
        unimplemented!()
    }

    fn impl_insert<'v>(&mut self, _path: ReflectPath<'v>, _value: Box<dyn Reflect>) -> ReflectResult<'v, ()> {
        unimplemented!()
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        unimplemented!()
    }

    fn from_string(_str: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        unimplemented!()
    }
}
