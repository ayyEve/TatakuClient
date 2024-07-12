#![allow(non_upper_case_globals)]
use crate::prelude::*;

/// hit variance stat
pub const HitVarianceStat: GameModeStat = GameModeStat {
    name: "hit_variance",
    display_name: "Hit Variance",
    description: ""
};

/// hit variance stat group
const VarianceStatGroup: StatGroup = StatGroup {
    name: "variance", 
    display_name: "Variance",
    stats: &[
        HitVarianceStat
    ]
};

/// all default stat groups
const DEFAULT_STAT_GROUPS: &'static [StatGroup] = &[
    VarianceStatGroup,
];

pub fn default_stat_groups() -> Vec<StatGroup> { DEFAULT_STAT_GROUPS.to_vec() }


#[cfg(feature="graphics")]
pub fn default_stats_from_groups(data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> { 
    let mut info = Vec::new();

    if let Some(variance) = data.get(&VarianceStatGroup.name()) {
        if let Some(variance_values) = variance.get(&HitVarianceStat.name()) {
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