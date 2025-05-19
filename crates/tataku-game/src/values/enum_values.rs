use crate::prelude::*;

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Debug, Clone, Default)]
pub struct EnumValues {
    pub sort_by: Vec<SortBy>,
    pub group_by: Vec<GroupBy>,
    pub score_methods: Vec<ScoreRetreivalMethod>,

    pub playmodes: Vec<String>,
    pub playmodes_display: Vec<String>,
}
impl EnumValues {
    pub fn new(infos: &GamemodeInfos) -> Self {
        let (playmodes, playmodes_display) = infos.by_num.iter()
            .map(|g| (g.id.to_string(), g.display_name.to_string()))
            .unzip();

        Self {
            sort_by: SortBy::list(),
            group_by: GroupBy::list(),
            score_methods: ScoreRetreivalMethod::list(),

            playmodes,
            playmodes_display,
        }
    }
}