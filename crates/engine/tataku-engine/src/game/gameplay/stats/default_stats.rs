#![allow(non_upper_case_globals)]
use crate::prelude::*;

/// hit variance stat
pub const HitVarianceStat: GameModeStat = GameModeStat {
    name: "hit_variance",
    display_name: "Hit Variance",
    description: ""
};

/// hit variance stat group
pub const VarianceStatGroup: StatGroup = StatGroup {
    name: "variance", 
    display_name: "Variance",
    stats: &[
        HitVarianceStat
    ]
};

/// all default stat groups
const DEFAULT_STAT_GROUPS: &[StatGroup] = &[
    VarianceStatGroup,
];

pub fn default_stat_groups() -> Vec<StatGroup> { DEFAULT_STAT_GROUPS.to_vec() }
