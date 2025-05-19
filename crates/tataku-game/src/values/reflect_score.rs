use crate::prelude::*;

// a reflect-friendly score
#[derive(Debug, Clone, Default)]
#[derive(Reflect)]
pub struct ReflectScore {
    // score fields
    pub username: String,
    pub beatmap_hash: Md5Hash,
    pub playmode: String,

    pub time: u64,

    pub score: u64,
    pub combo: u16,
    pub max_combo: u16,
    pub accuracy: f32,
    pub performance: f32,
    /// (user_hit_time - correct_time)
    pub hit_timings: Vec<f32>,

    //

    // #[reflect(skip)] gamemode_info: GamemodeInfo,

    pub id: usize,
    pub health: f32,
    pub judgments: Vec<ReflectJudgment>,
    pub stat_data: Vec<ReflectStat>,

    pub mods: ReflectMods,
}
impl ReflectScore {
    pub fn new(
        score: &IngameScore,
        info: &GamemodeInfo,
    ) -> Self {
        Self {
            username: score.username.clone(),
            beatmap_hash: score.beatmap_hash,
            playmode: score.playmode.clone(),
            time: score.time,
            score: score.score.score,
            combo: score.combo,
            max_combo: score.max_combo,
            accuracy: score.accuracy * 100.0,
            performance: score.performance,
            hit_timings: score.hit_timings.clone(),
            id: score.id,
            health: score.health,

            // gamemode_info: info,
            judgments: info
                .judgments
                .iter()
                .map(|j| score
                    .judgments
                    .get(j.id)
                    .map(|c| ReflectJudgment::new(*j, *c))
                    .unwrap_or_else(|| ReflectJudgment::new(*j, 0))
                )
                .collect(),

            stat_data: score.stat_data
                .iter()
                .map(|(s, d)| ReflectStat::new(s.clone(), d.clone()))
                .collect(),

            mods: ReflectMods::new(
                &score.mods,
                score.speed,
                info,
            ),
        }
    }

    pub fn update(&mut self, score: &IngameScore) {
        self.score = score.score.score;
        self.combo = score.combo;
        self.max_combo = score.max_combo;
        self.accuracy = score.accuracy * 100.0;
        self.performance = score.performance;
        self.hit_timings = score.hit_timings.clone();
        self.health = score.health;

        for j in self.judgments.iter_mut() {
            let Some(count) = score.judgments.get(j.judgment.id) else { continue };
            j.count = *count;
        }

        for stat in self.stat_data.iter_mut() {
            let Some(s) = score.stat_data.get(&stat.name) else { continue };
            stat.data = s.clone()
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Reflect)]
pub struct ReflectJudgment {
    judgment: HitJudgment,
    count: u16,
}
impl ReflectJudgment {
    pub fn new(
        judgment: HitJudgment,
        count: u16,
    ) -> Self {
        println!("{judgment:?}");
        Self {
            judgment,
            count,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[derive(Reflect)]
pub struct ReflectMods {
    mods: Vec<GameplayMod>,
    short_list: String,
    speed: f32,
}
impl ReflectMods {
    pub fn new(
        mods: &[ModDefinition],
        speed: GameSpeed,
        info: &GamemodeInfo,
    ) -> Self {
        let all_mods = info.mods
            .iter()
            .flat_map(|g| g.mods.iter().map(|m| (m.name, m)))
            .collect::<HashMap<_,_>>();

        let (mod_list, short_mods) = 
            mods
            .iter()
            .filter_map(|m| all_mods.get(&*m.name))
            .map(|m| (**m, m.short_name))
            .unzip::<_,_, Vec<_>, Vec<&str>>();

        Self {
            mods: mod_list,
            short_list: short_mods.join(" "),
            speed: speed.as_f32(),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Reflect)]
pub struct ReflectStat {
    name: String,
    data: Vec<f32>
}
impl ReflectStat {
    pub fn new(name: String, data: Vec<f32>) -> Self {
        Self {
            name,
            data
        }
    }
}