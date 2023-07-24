use crate::prelude::*;

pub trait GameModeStat {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str { self.name() }
    fn description(&self) -> &'static str { "" }
}

#[derive(Default)]
pub struct StatGroup {
    pub name: String,
    pub display_name: String,
    pub stats: Vec<Box<dyn GameModeStat>>,
}
impl StatGroup {
    pub fn new(name: impl ToString, display_name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            stats: Vec::new()
        }
    }
    
    pub fn with_stat<S: GameModeStat + 'static>(mut self, m: S) -> Self {
        self.stats.push(Box::new(m));
        self
    }
}

#[derive(Default, Clone, Debug)]
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


pub fn default_stat_groups() -> Vec<StatGroup>{
    vec![
        StatGroup::new("variance", "Variance")
            .with_stat(HitVarianceStat)
    ]
}


pub fn default_stats_from_groups(data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> { 
    let mut info = Vec::new();

    if let Some(variance) = data.get(&"variance".to_owned()) {
        if let Some(variance_values) = variance.get(&HitVarianceStat.name().to_owned()) {
            let mut list = Vec::new();

            let mut late_total = 0.0;
            let mut early_total = 0.0;
            let mut total_all = 0.0;
            let mut late_count = 0;
            let mut early_count = 0;
            for i in variance_values {
                total_all += i;

                if *i > 0.0 {
                    late_total += i;
                    late_count += 1;
                } else {
                    early_total += i;
                    early_count += 1;
                }
            }

            let mean = total_all / variance_values.len() as f32;
            let early = early_total / early_count as f32;
            let late = late_total / late_count as f32;

            list.push(MenuStatsEntry::new_list("Variance", variance_values.clone(), Color::PURPLE, true, true, ConcatMethod::StandardDeviation));
            list.push(MenuStatsEntry::new_f32("Mean", mean, Color::WHITE, true, true));

            list.push(MenuStatsEntry::new_f32("Early", early, Color::BLUE, true, true));
            list.push(MenuStatsEntry::new_f32("Late", late, Color::RED, true, true));


            info.push(MenuStatsInfo::new("Hit Variance", GraphType::Scatter, list))
        }
    }

    info
}


pub struct HitVarianceStat;
impl GameModeStat for HitVarianceStat {
    fn name(&self) -> &'static str { "hit_variance" }
    fn display_name(&self) -> &'static str { "Hit Variance" }
}
