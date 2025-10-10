use crate::prelude::*;
use common::reflect::*;

use tataku::Vsync;

use engine::{
    window::FullscreenMonitor,
    data::{
        SortBy,
        ScoreRetreivalMethod,
    },
    gameplay::{
        GamemodeInfo, 
        GamemodeInfos,
    },
    settings::{
        settings::SelectedTheme,
        display::PerformanceMode,
    },
};

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Debug, Clone, Default)]
pub struct EnumValues {
    // static enums
    pub sort_by: Vec<SortBy>,
    pub group_by: Vec<GroupBy>,
    pub vsync: Vec<Vsync>,
    pub performance_mode: Vec<PerformanceMode>,
    pub score_methods: Vec<ScoreRetreivalMethod>,

    pub playmodes: HashMap<String, String>,

    // modifiable enums
    pub skins: Vec<String>,
    pub themes: Vec<SelectedTheme>,
    pub monitors: Vec<FullscreenMonitor>,
}
impl EnumValues {
    pub fn new(infos: &GamemodeInfos) -> Self {
        let playmodes = infos
            .by_num
            .iter()
            .map(|g| (g.id.into(), g.display_name.into()))
            .collect();

        Self {
            sort_by: SortBy::list(),
            group_by: GroupBy::list(),
            score_methods: ScoreRetreivalMethod::list(),
            vsync: Vsync::list(),
            performance_mode: PerformanceMode::list(),
            playmodes,

            skins: Vec::new(),
            themes: vec![ SelectedTheme::Tataku, SelectedTheme::Osu ],
            monitors: Vec::new(),
        }
    }
}
