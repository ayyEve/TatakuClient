use crate::prelude::*;

#[allow(unused)]
#[cfg(test)]
mod old {
    use crate::prelude::*;

    #[derive(Default, Debug)]
    pub struct ValueCollection(HashMap<String, TatakuVariable>);
    impl ValueCollection {
        // initialize with some basic values
        pub fn new() -> Self {
            Self::default()
                .set_chained("true", TatakuVariable::new(true))
                .set_chained("false", TatakuVariable::new(false))
        }


        pub fn set_chained(mut self, key: impl ToString, value: TatakuVariable) -> Self {
            self.set(key, value);
            self
        }

        pub fn set(&mut self, key: impl ToString, value: TatakuVariable) {
            let key = key.to_string();

            // // we shouldnt do thing.thing.thing inserts anymore, it should always be { map { map { value }}}
            // check_key(&key);

            let val = self.ensure_tree(&key, || value.clone());
            *val = value;

            // self.0.insert(key, value);
        }

        pub fn update_multiple(&mut self, access: TatakuVariableWriteSource, list: impl Iterator<Item=(impl AsRef<str>, impl Into<TatakuValue>)>) {
            for (key, value) in list {
                self.update(key.as_ref(), access, value.into());
            }
        }

        pub fn remove(&mut self, key: &str) { self.0.remove(key); }
        pub fn exists(&self, key: &str) -> bool { self.get_raw(key).is_ok() }



        /// set the value in insert to None, this will set it after
        pub fn update_or_insert(&mut self, key: &str, access: TatakuVariableWriteSource, value: impl Into<TatakuValue>, insert: impl Fn() -> TatakuVariable) {
            let Ok(variable) = self.get_raw_mut(key) else {
                // check_key(key);
                let val = self.ensure_tree(key, insert);
                val.value = value.into();
                return;
            };

            if !variable.access.check_access(&access) { return warn!("{access:?} trying to write to variable {key}") }
            variable.value = value.into();
        }

        pub fn update(&mut self, key: &str, access: TatakuVariableWriteSource, value: impl Into<TatakuValue>) {
            let Ok(variable) = self.get_raw_mut(key) else { return error!("value {key} doesnt exist in collection") };
            if !variable.access.check_access(&access) { return warn!("{access:?} trying to write to variable {key}") }
            variable.value = value.into()
        }
        pub fn update_display(&mut self, key: &str, access: TatakuVariableWriteSource, value: impl Into<TatakuValue>, display: Option<impl Into<Cow<'static, str>>>) {
            let Ok(variable) = self.get_raw_mut(key) else { return error!("value {key} doesnt exist in collection") };
            if !variable.access.check_access(&access) { return warn!("{access:?} trying to write to variable {key}") }
            variable.value = value.into();
            variable.display = display.map(|d| d.into());
        }


        pub fn ensure_tree(&mut self, key: &str, insert: impl Fn() -> TatakuVariable) -> &mut TatakuVariable {
            let mut split = key.split(".").collect::<VecDeque<_>>();
            let first = split.pop_front().unwrap().to_owned();
            // let _ = split.pop_back(); // remove the variable portion to make sure we dont accidentally set it

            let mut last = self.0.entry(first).or_insert(insert());

            while let Some(i) = split.pop_front() {
                let map = match &mut last.value {
                    TatakuValue::Map(m) => m,
                    val @ TatakuValue::None => {
                        warn!("creating {i}");
                        *val = TatakuValue::Map(HashMap::new());
                        let TatakuValue::Map(m) = val else { unreachable!("how??") };
                        m
                    }

                    _ => panic!("trying to create property on non-map")
                };

                last = map.entry(i.to_owned()).or_insert(insert());
            }

            last
        }

    }

    // getters
    impl ValueCollection {
        pub fn get_raw_mut(&mut self, key: &str) -> Result<&mut TatakuVariable, ShuntingYardError> {
            // if let Some(v) = self.0.get_mut(key) { return Ok(v) }

            let mut split = key.split(".").collect::<VecDeque<_>>();
            let mut last = self.0.get_mut(split.pop_front().unwrap());

            while let Some(i) = split.pop_front() {
                let Some(TatakuVariable { value: TatakuValue::Map(map), ..}) = last else { return Err(ShuntingYardError::EntryDoesntExist(key.to_owned())) };
                last = map.get_mut(i);
            }

            last.ok_or_else(|| ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }


        pub fn get_raw(&self, key: &str) -> Result<&TatakuVariable, ShuntingYardError> {
            // debug!("got {key}");
            let mut split = key.split(".").collect::<VecDeque<_>>();
            let mut last = self.0.get(split.pop_front().unwrap());

            while let Some(i) = split.pop_front() {
                // debug!("checking > {i}");
                let Some(TatakuVariable { value: TatakuValue::Map(map), ..}) = last else { return Err(ShuntingYardError::EntryDoesntExist(key.to_owned())) };
                last = map.get(i);
                // if last.is_none() { debug!("failed.") }
            }

            last.ok_or_else(|| ShuntingYardError::EntryDoesntExist(key.to_owned()))

            // if let Some(v) = self.0.get(key) {
            //     return Ok(v)
            // }

            // // TODO: optimize this, this is quite bad
            // let mut remaining = key.split(".").collect::<Vec<_>>();
            // if remaining.len() > 1 {
            //     let k2 = remaining.pop().unwrap();
            //     let key = remaining.join(".");

            //     if let TatakuValue::Map(m) = &self.get_raw(&key)?.value {
            //         if let Some(v) = m.get(k2) {
            //             return Ok(v);
            //         }
            //     }
            // }

            // Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }

        pub fn get_f32(&self, key: &str) -> Result<f32, ShuntingYardError> {
            match self.get_raw(key) {
                Ok(TatakuVariable { value: TatakuValue::String(_), .. }) => Err(ShuntingYardError::ValueIsntANumber(key.to_owned())),
                Ok(other) => other.as_f32(),
                Err(_) => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
            }
        }
        pub fn get_u32(&self, key: &str) -> Result<u32, ShuntingYardError> {
            match self.get_raw(key) {
                Ok(TatakuVariable { value: TatakuValue::U32(n), .. }) => Ok(*n),
                Ok(_) => Err(ShuntingYardError::ValueIsntANumber(key.to_owned())),
                Err(_) => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
            }
        }
        pub fn get_string(&self, key: &str) -> Result<String, ShuntingYardError> {
            self
                .get_raw(key)
                .map(|i| i.as_string())
                // .ok_or_else(|| ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }

        pub fn get_bool<'a>(&self, key: &str) -> Result<bool, ShuntingYardError> {
            match self.get_raw(key) {
                Ok(TatakuVariable { value: TatakuValue::Bool(b), .. }) => Ok(*b),
                Ok(_) => Err(ShuntingYardError::ValueIsntABool),
                _ => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
            }
        }


        pub fn try_get<'a, T>(&'a self, key: &str) -> Result<T, ShuntingYardError>
            where
                &'a TatakuValue: TryInto<T>,
                <&'a TatakuValue as TryInto<T>>::Error: ToString
        {
            let raw = self.get_raw(key)?;
            (&raw.value).try_into().map_err(|e| ShuntingYardError::ConversionError(e.to_string()))
        }

    }


}

#[derive(Reflect)]
#[derive(Debug, Default)]
pub struct GameValues {
    pub settings: Settings,

    pub song: SongInfo,
    pub game: GameInfo,
    pub global: GlobalInfo,
    pub enums: EnumInfo,

    pub lobby: Option<CurrentLobbyInfo>,

    pub score: IngameScore,
    pub mods: ModManager,

    /// Beatmap manager, its here instead of in Game to keep the lists in one place
    #[reflect(alias("beatmaps"))]
    pub beatmap_manager: BeatmapManager,

    /// list of retreived scored 
    pub score_list: ScoreList,
}
impl GameValues {
    pub fn current_beatmap_prop<T>(&self, f: impl FnOnce(&BeatmapMeta)->T) -> Option<T> {
        self
            .beatmap_manager
            .current_beatmap
            .as_ref()
            .map(|b| f(b))
    }
}


#[derive(Debug, Clone, Default)]
#[derive(Reflect)]
pub struct ScoreList {
    #[reflect(flatten)]
    pub scores: Vec<IngameScore>,
    pub loaded: bool,
}


#[derive(Default)]
pub struct ValueCollection {
    pub values: GameValues,
    pub custom: DynMap,
}
impl Deref for ValueCollection {
    type Target = GameValues;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl DerefMut for ValueCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl Reflect for ValueCollection {
    fn impl_get<'v>(&self, path: ReflectPath<'v>) -> Result<&dyn Reflect, ReflectError<'v>> {
        self.values.impl_get(path.clone())
            .or_else(|_| self.custom.impl_get(path))
    }

    fn impl_get_mut<'v>(&mut self, path: ReflectPath<'v>) -> Result<&mut dyn Reflect, ReflectError<'v>> {
        self.values.impl_get_mut(path.clone())
            .or_else(|_| self.custom.impl_get_mut(path))
    }

    fn impl_insert<'v>(&mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'v>> {
        self.values.impl_insert(path.clone(), todo!())
            .or_else(|_| self.custom.impl_insert(path, todo!()))
    }

    fn impl_iter<'v>(&self, path: ReflectPath<'v>) -> Result<IterThing<'_>, ReflectError<'v>> {
        match (self.values.impl_iter(path.clone()), self.custom.impl_iter(path)) {
            (Ok(v), Ok(c)) => Ok(v.chain(c).collect::<Vec<_>>().into()),
            (Ok(v), Err(_)) => Ok(v),
            (Err(_), Ok(c)) => Ok(c),
            (Err(ReflectError::EntryNotExist { .. }), Err(e)) => Err(e),
            (Err(e), Err(ReflectError::EntryNotExist { .. })) => Err(e),
            // todo: is this correct?
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn impl_iter_mut<'v>(&mut self, path: ReflectPath<'v>) -> Result<IterThingMut<'_>, ReflectError<'v>> {
        match (self.values.impl_iter_mut(path.clone()), self.custom.impl_iter_mut(path)) {
            (Ok(v), Ok(c)) => Ok(v.chain(c).collect::<Vec<_>>().into()),
            (Ok(v), Err(_)) => Ok(v),
            (Err(_), Ok(c)) => Ok(c),
            (Err(ReflectError::EntryNotExist { .. }), Err(e)) => Err(e),
            (Err(e), Err(ReflectError::EntryNotExist { .. })) => Err(e),
            // todo: is this correct?
            (Err(e), Err(_)) => Err(e),
        }
    }
}


#[derive(Reflect)]
#[derive(Default, Debug)]
pub struct SongInfo {
    pub position: f32,
    pub paused: bool,
    pub playing: bool,
    pub stopped: bool,
    pub exists: bool,

    pub state: AudioState
}
impl SongInfo {
    pub fn update(&mut self, audio: Option<Arc<dyn AudioInstance>>) {
        if let Some(audio) = audio {
            self.position = audio.get_position();
            self.set_state(audio.get_state());
            self.exists = true;
        } else {
            self.position = 0.0;
            self.set_state(AudioState::Unknown);
            self.exists = false;
        }
    }
    pub fn set_state(&mut self, state: AudioState) -> bool {
        if self.state == state { return false }

        self.paused = state == AudioState::Paused;
        self.playing = state == AudioState::Playing;
        self.stopped = state == AudioState::Stopped;
        self.exists = self.paused || self.playing || self.stopped;

        self.state = state;

        true
    }
}

#[derive(Reflect)]
#[derive(Default, Debug)]
pub struct GameInfo {
    pub time: f32,
}


#[derive(Reflect)]
#[derive(Default, Debug)]
pub struct GlobalInfo {
    pub lobbies: Vec<LobbyInfo>,

    pub playmode: String,
    pub playmode_display: String,
    pub playmode_actual: String,
    pub playmode_actual_display: String,

    pub username: String,
    pub user_id: u32,
    pub menu_list: Vec<String>,

    pub new_beatmap_hash: Option<Md5Hash>,
}
impl GlobalInfo {
    pub fn update_playmode(&mut self, playmode: String) {
        self.playmode = playmode.clone();
        let Some(info) = get_gamemode_info(&playmode) else { return };
        self.playmode_display = info.display_name().to_owned();
    }
    pub fn update_playmode_actual(&mut self, playmode: String) {
        self.playmode_actual = playmode.clone();
        let Some(info) = get_gamemode_info(&playmode) else { return };
        self.playmode_actual_display = info.display_name().to_owned();
    }
}


#[derive(Reflect)]
#[derive(Debug)]
pub struct EnumInfo {
    pub sort_by: Vec<SortBy>,
    pub group_by: Vec<GroupBy>,
    pub score_methods: Vec<ScoreRetreivalMethod>,

    pub playmodes: Vec<&'static str>,
    pub playmodes_display: Vec<&'static str>,
}
impl Default for EnumInfo {
    fn default() -> Self {
        Self {
            sort_by: SortBy::list(),
            group_by: GroupBy::list(),
            score_methods: ScoreRetreivalMethod::list(),

            playmodes: AVAILABLE_PLAYMODES.iter().map(|s| *s).collect(),
            playmodes_display: AVAILABLE_PLAYMODES
                .iter()
                .map(|m| gamemode_display_name(*m))
                .collect()
        }
    }
}



// #[test]
// fn test() {
//     let mut map = HashMap::default();
//     map.set_value("hi", TatakuVariable::new("test"));

//     let count = 1_000;
//     let key = "hi.".repeat(count) + "hi";

//     for _ in 0..count {
//         let mut map2 = HashMap::default();
//         map2.set_value("hi", TatakuVariable::new(map));
//         map = map2
//     }

//     let values = ValueCollection(map);

//     let val = values.get_raw(&key).expect("nope");
//     println!("val: {val:?}");
// }
