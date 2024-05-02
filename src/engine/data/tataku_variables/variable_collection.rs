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


#[test]
fn test() {
    let mut map = ValueCollectionMapHelper::default();
    map.set("hi", TatakuVariable::new("test"));

    let count = 1_000;
    let key = "hi.".repeat(count) + "hi";

    for _ in 0..count {
        let mut map2 = ValueCollectionMapHelper::default();
        map2.set("hi", TatakuVariable::new(map.finish()));
        map = map2
    }

    let TatakuValue::Map(map) = map.finish() else { panic!() };
    let values = ValueCollection(map);

    let val = values.get_raw(&key).expect("nope");
    println!("val: {val:?}");
}
