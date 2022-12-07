use std::collections::HashMap;

pub trait GameModeStat {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn description(&self) -> &'static str { "" }
}

#[derive(Default)]
pub struct StatGroup {
    pub name: String,
    pub stats: Vec<Box<dyn GameModeStat>>,
}
impl StatGroup {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            stats: Vec::new()
        }
    }
    
    pub fn with_stat<S: GameModeStat + 'static>(mut self, m: S) -> Self {
        self.stats.push(Box::new(m));
        self
    }
}

#[derive(Default, Clone)]
pub struct GameplayStats {
    data: HashMap<String, Vec<f32>>,
}
impl GameplayStats {
    pub fn insert<S:GameModeStat>(&mut self, stat: S, value: f32) {
        let key = stat.name().to_owned();

        if let Some(values) = self.data.get_mut(&key) {
            values.push(value)
        } else {
            self.data.insert(key, vec![value]);
        }
    }

    /// group the data into sets of groups
    /// the hashmap is indexed by the group name, and the data is a hashmap of stat name, and values for said stat
    /// note that this will not include stats that dont have at least one value
    pub fn into_groups(&self, groups: &Vec<StatGroup>) -> HashMap<String, HashMap<String, Vec<f32>>> {
        let mut output = HashMap::new();

        for group in groups {
            let mut data = HashMap::new();

            for stat in group.stats.iter() {
                if let Some(val) = self.data.get(&stat.name().to_owned()) {
                    data.insert(stat.name().to_owned(), val.clone());
                }
            }
            output.insert(group.name.clone(), data);
        }

        output
    }
}