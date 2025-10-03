use crate::prelude::*;
use common::{
    Score,
    replays::ReplayFrame,
};

use tataku::{
    Color,
    Bounds,
    Vector2,
};

use engine::{
    actions,
    gameplay,
    Settings,
    BeatmapMeta,
    Notification,
    beatmaps::Beatmap,

    gameplay::{
        *,
        mods::*,
        judgments::*,
        gameplay_manager::*,
    }
};

pub struct GameplaySimulator {
    simulate_score: Score,
    gameplay: GameplayManager,
}
impl GameplaySimulator {
    pub fn new(
        score: Score,

        infos: &GamemodeInfos,
        beatmaps: &BeatmapManager,
        settings: &engine::Settings,
        database: &engine::DatabaseShell,
    ) -> tataku::Result<Self> {
        if score.replay.is_none() {
            // TOdO: not panic
            panic!("no replay");
        }

        let map = beatmaps
            .get_by_hash(&score.beatmap_hash)
            .unwrap();
        let info = infos.get_info(&map.mode)?;

        let mods = ModManager::new(
            score.mods.iter(), 
            score.speed, 
            info
        );

        let gameplay = GameplayManager::create(
            infos, 
            &score.playmode, 
            &map, 
            mods, 
            settings,
            database,
        )?;

        Ok(Self {
            simulate_score: score,
            gameplay,
        })
    }

    fn run(mut self) -> Result<SimulationOk, SimulationError> {
        let replay = self.simulate_score.replay.as_ref().unwrap();

        let notes = self.gameplay.gamemode.all_notes();
        let mut events = notes
            .iter()
            .map(|i| Event::Note(i.time()))
            .chain(replay.frames.iter().copied().map(Event::Input))
            .collect::<Vec<_>>();

        events.sort();
        
        let mut font = ui::widget::TextLayoutContexts::new();
        let mut actions = actions::ActionQueue::new();

        // for time in times {
        //     match time {
        //         Time::Input(time) | Time::Note(time) => {
        //             self.gameplay.song_time = time;
        //             self.gameplay.update(values, &mut font_contexts, &mut actions);
        //         }

        //         _ => {}
        //     }
        // }

        Ok(SimulationOk {
            score: IngameScore::new(self.simulate_score, true, false)
        })
    }

}


enum Event {
    // time is from an input event
    Input(ReplayFrame),

    // time is from a note
    Note(f32),

    // // time is from a long note (slider/hold/etc)
    // NoteRange { 
    //     start: f32,
    //     end: f32,
    // },
}
impl Event {
    fn time(&self) -> f32 {
        match self {
            Self::Input(f) => f.time,
            Self::Note(t) => *t,
        }
    }
}
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time() == other.time()
    }
}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for Event {}
impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time().total_cmp(&other.time())
    }
}


enum SimulationError {
    Failed,
}

struct SimulationOk {
    score: IngameScore,
}