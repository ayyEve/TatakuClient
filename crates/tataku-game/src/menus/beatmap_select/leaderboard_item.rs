use crate::prelude::*;


pub const LEADERBOARD_PADDING:f32 = 100.0;
pub const LEADERBOARD_POS:Vector2 = Vector2::new(10.0, LEADERBOARD_PADDING);


#[derive(Clone)]
pub struct LeaderboardComponent {
    pub num: usize,
    pub score: IngameScore,
    score_mods: String,
    acc: f32,
}
impl LeaderboardComponent {
    pub fn new(
        num: usize, 
        score: IngameScore,
        infos: &GamemodeInfos,
    ) -> Self {

        let info = infos.get_info(&score.playmode).unwrap();
        let score_mods = ModManager::short_mods_string(
            &score.mods, 
            false, 
            info
        );
        let acc = info.calc_acc(&score) * 100.0;


        Self {
            num,
            score,
            score_mods,
            acc
        }
    }
    pub fn view(&self) -> IcedElement {
        use crate::prelude::iced_elements::*;

        let score_mods = &self.score_mods;
        let acc = self.acc;

        let now = chrono::Utc::now().timestamp() as u64;
        let time_diff = now as i64 - self.score.time as i64;
        let time_diff_str = if time_diff < 60 * 5 {
            format!(" | {time_diff}s")
        } else {
            String::new()
        };

        // TODO: cache this ??
        Button::new(col!(
            Text::new(format!("{}: {}", self.score.username, format_number(self.score.score.score)))
                .width(Fill)
                .size(16.0),
            Text::new(format!("{}x, {acc:.2}%, {score_mods}{time_diff_str}", format_number(self.score.max_combo)))
                .width(Fill)
                .size(16.0);
        ))
            .width(Fill)
            .on_press(Message::new(MessageOwner::Menu, "score", MessageType::Number(self.num)))
            .into_element()
    }
}
