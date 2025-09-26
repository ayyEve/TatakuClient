use crate::*;
use tataku::Vector2;
use common::{
    Score,
    replays::*,
    network::spectator::SpectatorFrame,
};

use engine::{
    actions::action::ActionQueue,
    actions::game::GameplayTypeInfo,
    gameplay::{
        mods::ModManager,
        judgments::HitJudgment,
        helpers::{
            IngameScore,
            KeyCounter,
            TimingPointHelper,
        },
    }
};

/// how much time should pass at beatmap start before audio begins playing (and the map "starts")
pub const LEAD_IN_TIME:f32 = 1000.0;

pub trait GameplayManagerTrait {
    fn time(&self) -> f32;
    fn end_time(&self) -> f32;

    fn score(&self) -> &IngameScore;
    fn score_mut(&mut self) -> &mut IngameScore;
    fn all_scores(&self) -> Vec<&IngameScore>;
    fn all_non_user_scores(&self) -> &[IngameScore];
    
    fn mods(&self) -> &ModManager;
    fn metadata(&self) -> &BeatmapMeta;
    fn key_counter(&self) -> &KeyCounter;
    fn spectators(&mut self) -> &mut online::SpectatorList;
    fn timing_points(&self) -> &TimingPointHelper;
    fn properties(&self) -> &gameplay::GamemodeProperties;

    fn judgments(&self) -> &Vec<HitJudgment>;

    fn health(&self) -> &dyn gameplay::health_manager::HealthManager;

    fn hitbar_timings(&self) -> Vec<(f32, f32)>;

    fn bounds(&self) -> tataku::Bounds;
    
    fn handle_action(
        &mut self, 
        action: actions::gameplay::GameplayAction,
        settings: &Settings,
    );
    fn handle_gamemode_action(
        &mut self, 
        action: gameplay::action::GamemodeAction,
        settings: &Settings
    );

    fn set_mode(&mut self, mode: GameplayType);
    fn get_mode(&self) -> &GameplayType;


    fn start(&mut self);
    fn pause(&mut self);
    fn reset(&mut self);
    fn fail(&mut self);
}

// TODO: move this???
pub trait DifficultyProvider: Send + Sync {
    fn get_diff(
        &mut self, 
        map: &Arc<BeatmapMeta>, 
        playmode: &str, 
        mods: &ModManager
    ) -> tataku::TatakuResult<f32>;
}


/// What gameplay method should we use for this gameplay manager?
#[derive(Clone, Debug, Default)]
pub enum GameplayType {
    /// Just regular gameplay
    #[default]
    Normal,

    /// This manager is handling gameplay preview
    Preview,

    /// This manager is watching a replay
    Replaying {
        /// What score+replay are we watching?
        score: Score,

        /// What frame index are we at?
        current_frame: usize,
    },

    #[cfg(feature="gameplay")]
    /// We're handling spectating someone
    Spectator {
        /// What is the current spec state
        state: SpectatorState,
        /// List of buffered spectator frames
        frames: VecDeque<SpectatorFrame>,

        host_id: u32,
        host_username: ArcStr,

        /// list of buffered replay frames
        replay_frames: Vec<ReplayFrame>,
        /// what replay frame are we on
        current_frame: usize,

        /// Up to what time do we have data for?
        ///
        /// ie, up to what time we can show gameplay
        good_until: f32,

        /// List of (id,username) for other spectators
        spectators: HashMap<u32, ArcStr>,

        /// List of score frames to help sync the host score with our score
        ///
        /// TODO: ideally this wouldnt be necessary though
        buffered_score_frames: Vec<(f32, Score)>,
    },

    #[cfg(feature="gameplay")]
    /// The player is in a multiplayer match
    Multiplayer {
        /// when was escape pressed last
        last_escape_press: tataku::Instant,
        score_send_timer: tataku::Instant,
    },
}
impl GameplayType {

    // convenience fns
    pub fn is_preview(&self) -> bool { matches!(self, &Self::Preview) }
    #[cfg(feature="gameplay")]
    pub fn is_multi(&self) -> bool { matches!(self, &Self::Multiplayer { .. }) }
    pub fn is_replay(&self) -> bool { matches!(self, &Self::Replaying {..}) }

    pub fn should_load_scores(&self) -> bool {
        match self {
            Self::Normal | Self::Replaying {..} => true,
            Self::Preview {..} => false,

            #[cfg(feature="gameplay")]
            Self::Spectator {..} => true,

            #[cfg(feature="gameplay")]
            Self::Multiplayer {..} => false,
        }
    }

    #[cfg(feature="gameplay")]
    pub fn should_send_spec_frames(&self) -> bool {
        match self {
            // send spec frames for normal gameplay and multi, not for anything else
            Self::Normal | Self::Multiplayer {..} => true,
            Self::Replaying {..} | Self::Spectator {..} | Self::Preview {..} => false,
        }
    }

    #[cfg(feature="gameplay")]
    pub fn skip_input(&self) -> bool {
        matches!(self, Self::Replaying { .. } | Self::Preview { .. } | Self::Spectator { .. })
    }
}

impl From<GameplayTypeInfo> for GameplayType {
    fn from(value: GameplayTypeInfo) -> Self {
        match value {
            GameplayTypeInfo::Normal => Self::Normal,
            GameplayTypeInfo::Preview => Self::Preview,
            GameplayTypeInfo::Replay(score) => Self::Replaying { score: *score, current_frame: 0 },
            
            #[cfg(feature="gameplay")]
            GameplayTypeInfo::Multiplayer => Self::Multiplayer { 
                last_escape_press: tataku::Instant::now(), 
                score_send_timer: tataku::Instant::now() 
            },
            #[cfg(feature="gameplay")]
            GameplayTypeInfo::Spectator(a) => Self::Spectator {
                state: SpectatorState::None,
                frames: a.pending_frames,
                host_id: a.host_id,
                host_username: a.host_username,
                replay_frames: Vec::new(),
                current_frame: 0,
                good_until: 0.0,
                spectators: a.spectators,
                buffered_score_frames: Vec::new()
            },

            #[cfg(not(feature="gameplay"))]
            _ => unimplemented!()
        }
    }
}



pub struct GameplayDrawShell<'a> {
    pub time: f32,
    pub gameplay_mode: &'a GameplayType,
    pub current_timing_point: &'a beatmaps::TimingPoint,
    pub mods: &'a ModManager,
    pub score: &'a IngameScore,
    pub window_size: Vector2,
}

pub struct GameplayUpdateShell<'a> {
    /// current map time
    pub time: f32,

    /// current game speed
    pub game_speed: f32,

    /// NOTE: use an action to set this, dont manually set it
    pub completed: bool,

    /// current mods
    pub mods: &'a ModManager,

    /// the current timing point
    pub current_timing_point: &'a beatmaps::TimingPoint,

    /// the current gameplay mode
    pub gameplay_mode: &'a GameplayType,

    /// our current score
    pub score: &'a IngameScore,

    // all timing points
    pub timing_points: &'a TimingPointHelper,

    /// list of actions to be performed
    pub actions: Vec<gameplay::Action>,

    /// Game settings
    pub settings: &'a Settings,

    /// Action queue
    pub action_queue: &'a mut ActionQueue,

    pub window_size: Vector2,
}
impl GameplayUpdateShell<'_> {
    /// does the manager believe the map has been completed?
    pub fn complete(&self) -> bool { self.completed }

    pub fn add_action(&mut self, action: impl Into<gameplay::Action>) {
        self.actions.push(action.into());
    }
    pub fn add_replay_action(&mut self, action: ReplayAction) {
        self.actions.push(gameplay::Action::ReplayAction(ReplayFrame::new(self.time, action)));
    }

    /// check and add to hit timings if found
    pub fn check_judgment<'j>(
        &mut self,
        windows: &'j [(HitJudgment, Range<f32>)],
        time: f32,
        note_time: f32
    ) -> Option<&'j HitJudgment> {
        if let Some(hj) = self.check_judgment_only(windows, time, note_time) {
            self.actions.push(gameplay::Action::AddJudgment(*hj));
            self.actions.push(gameplay::Action::AddTiming { hit_time: time, note_time });
            // return the hit judgment we got
            Some(hj)
        } else {
            None
        }
    }

    pub fn check_judgment_condition<'j>(
        &mut self,
        windows: &'j [(HitJudgment, Range<f32>)],
        time: f32,
        note_time: f32,
        cond: impl Fn() -> bool,
        if_bad: &'j HitJudgment
    ) -> Option<&'j HitJudgment> {
        if let Some(hj) = self.check_judgment_only(windows, time, note_time) {
            if cond() {
                self.actions.push(gameplay::Action::AddJudgment(*hj));
                self.actions.push(gameplay::Action::AddTiming { hit_time: time, note_time });
                // return the hit judgment we got
                Some(hj)
            } else {
                self.actions.push(gameplay::Action::AddJudgment(*if_bad));
                // return the hit judgment we got
                Some(if_bad)
            }
        } else {
            None
        }
    }

    /// only check if the note + hit fit into a window, and if so, return the corresponding judgment
    pub fn check_judgment_only<'j>(
        &self,
        windows: &'j [(HitJudgment, Range<f32>)],
        time: f32,
        note_time: f32
    ) -> Option<&'j HitJudgment> {
        let diff = (time - note_time).abs() / self.game_speed;
        for (hj, window) in windows.iter() {
            if window.contains(&diff) {
                // return the hit judgment we got
                return Some(hj)
            }
        }

        None
    }

    pub fn add_judgment(&mut self, judgment: HitJudgment) {
        self.actions.push(gameplay::Action::AddJudgment(judgment));
    }
    #[cfg(feature="graphics")]
    pub fn add_indicator(&mut self, indicator: impl gameplay::judgments::JudgementIndicator + 'static) {
        self.actions.push(gameplay::Action::AddIndicator(Box::new(indicator)));
    }
    pub fn add_stat(&mut self, stat: gameplay::stats::GameModeStat, value: f32) {
        self.actions.push(gameplay::Action::AddStat { stat, value });
    }

    pub fn play_hitsounds(&mut self, sounds: &[gameplay::Hitsound], repeat: bool) {
        for i in sounds {
            self.action_queue.push(actions::audio::AudioAction::new(
                i.get_id(),
                actions::audio::AudioActionType::Play {
                    volume: i.volume,
                    repeat,
                    restart: true
                }).into()
            );
        }
    }

}


#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum SpectatorState {
    None, // Default
    Buffering, // waiting for data
    Watching, // host playing
    Paused, // host paused
    MapChanging, // host is changing map
}

#[derive(Debug)]
pub enum SpectatorManagerAction {
    QuitSpec,
}
