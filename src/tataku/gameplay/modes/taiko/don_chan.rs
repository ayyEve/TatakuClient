use crate::prelude::*;

const DEFAULT_DONCHAN_SIZE:Vector2 = Vector2::new(450.0, 400.0);

pub struct DonChan {
    pub state: DonChanState,

    normal_anim: Option<Animation>,
    combo_anim:  Option<Animation>,
    kiai_anim:   Option<Animation>,
    fail_anim:   Option<Animation>,

    init: bool,
    current_timing_point_time: f32,

    // state checks
    kiai: bool,
    last_combo_milestone: u16,
    last_miss_count: u16,
    last_score: u64,

    // used to check if combo anim has completed
    combo_anim_last_index: usize
}
impl DonChan {
    pub async fn new() -> Self {
        Self {
            state: DonChanState::Normal,

            normal_anim: None,
            combo_anim: None,
            kiai_anim: None,
            fail_anim: None,

            kiai: false,
            init: false,
            current_timing_point_time: 0.0,

            last_combo_milestone: 0,
            last_miss_count: 0,
            last_score: 0,

            combo_anim_last_index: 0,
        }
    }
    fn all_anims(&mut self) -> Vec<&mut Option<Animation>> {
        vec![
            &mut self.normal_anim,
            &mut self.combo_anim,
            &mut self.kiai_anim,
            &mut self.fail_anim,
        ]
    }

    pub fn set_offset(&mut self, offset: f32) {
        for i in self.all_anims() {
            if let Some(anim) = i {
                anim.frame_start_time = offset;
            }
        }
    }
    pub fn update_delays(&mut self, timing_point: &TimingPoint) {
        for i in self.all_anims() {
            if let Some(anim) = i {
                anim.frame_delays.iter_mut().for_each(|d| *d = timing_point.beat_length)
            }
        }
    }

}

#[async_trait]
impl InnerUIElement for DonChan {
    fn display_name(&self) -> &'static str { "DonChan" }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(
            -Vector2::with_y(DEFAULT_DONCHAN_SIZE.y / 2.0), 
            DEFAULT_DONCHAN_SIZE / 2.0
        )
    }

    fn update(&mut self, manager: &mut GameplayManager) {
        let time = manager.time(); 

        // check init
        if !self.init {
            let tp = manager.timing_point_at(0.0, false);
            self.set_offset(tp.time - tp.beat_length * 4.0);
            self.update_delays(tp);
            self.init = true;
        }

        // check timing point change
        let current_tp = manager.current_timing_point();
        if !current_tp.is_inherited() {
            if self.current_timing_point_time != current_tp.time {
                self.current_timing_point_time = current_tp.time;
                self.update_delays(current_tp);
            }
        }

        // check kiai update
        if self.kiai != current_tp.kiai {
            self.kiai = current_tp.kiai
        }

        // TODO: figure out peppy's bullshit for this animation (it might play in reverse after)
        // // check combo milestones
        // let diff = manager.score.combo as i32 - self.last_combo_milestone as i32;
        // if diff >= 25 {
        //     // do combo milestone
        //     self.state = DonChanState::ComboMilestone;
        //     self.last_combo_milestone = manager.score.combo % 25;
        // } else if diff < 0 {
        //     // missed, reset counter
        //     self.last_combo_milestone = 0;
        // }

        // check fail anim
        let xmiss = manager.score.judgments.get("xmiss").copy_or_default();
        if self.last_miss_count < xmiss {
            self.state = DonChanState::Fail;
            self.last_miss_count = xmiss;
        } else if self.last_score != manager.score.score.score && self.state == DonChanState::Fail {
            self.state = DonChanState::Normal;
        }
        self.last_score = manager.score.score.score;
        

        // check if combo milestone anim has finished
        if self.state == DonChanState::ComboMilestone {
            if let Some(anim) = &self.combo_anim {
                if self.combo_anim_last_index > anim.frame_index {
                    // completed
                    info!("combo anim complete");
                    self.state = DonChanState::Normal;
                } else {
                    self.combo_anim_last_index = anim.frame_index
                }
            }
        }

        // update all anims
        for i in self.all_anims() {
            if let Some(anim) = i {
                anim.update(time);
            }
        }
    }

    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection) {
        match self.state {
            DonChanState::Normal => {
                if self.kiai {
                    if let Some(anim) = &self.kiai_anim {
                        let mut anim = anim.clone();
                        anim.pos = pos_offset;
                        anim.scale *= scale;
                        list.push(anim)
                    }
                } else {
                    if let Some(anim) = &self.normal_anim {
                        let mut anim = anim.clone();
                        anim.pos = pos_offset;
                        anim.scale *= scale;
                        list.push(anim)
                    }
                }
            }
            DonChanState::ComboMilestone => {
                if let Some(anim) = &self.combo_anim {
                    let mut anim = anim.clone();
                    anim.pos = pos_offset;
                    anim.scale *= scale;
                    list.push(anim)
                }
            }
            DonChanState::Fail => {
                if let Some(anim) = &self.fail_anim {
                    let mut anim = anim.clone();
                    anim.pos = pos_offset;
                    anim.scale *= scale;
                    list.push(anim)
                }
            }
        }
    }

    fn reset(&mut self) {
        self.init = false;
        self.state = DonChanState::Normal;
        self.current_timing_point_time = 0.0;
        self.kiai = false;
        self.last_combo_milestone = 0;
        self.last_miss_count = 0;
        self.last_score = 0;
        self.combo_anim_last_index = 0;
    }

    async fn reload_skin(&mut self, source: &TextureSource, skin_manager: &mut SkinManager) {
        self.normal_anim = load_anim("idle", source, skin_manager).await;
        self.combo_anim = load_anim("clear", source, skin_manager).await;
        self.kiai_anim = load_anim("kiai", source, skin_manager).await;
        self.fail_anim = load_anim("fail", source, skin_manager).await;
    }
}

async fn load_anim(
    name: &str, 
    source: &TextureSource,
    skin_manager: &mut SkinManager,
) -> Option<Animation> {
    let mut frames = Vec::new();
    let mut current = 0;

    while let Some(tex) = skin_manager.get_texture(format!("pippidon{name}{current}"), source, SkinUsage::Gamemode, false).await {
        current += 1;
        frames.push(tex.tex);
    }

    if frames.len() == 0 {
        None
    } else {
        let delays = vec![50.0; frames.len()];
        let mut anim = Animation::new(
            Vector2::ZERO,
            DEFAULT_DONCHAN_SIZE / 2.0,
            frames,
            delays,
            Vector2::ONE
        );
        anim.origin.x = 0.0;
        anim.origin.y *= 2.0; // since its center, just double it to get the bottom

        Some(anim)
    }

}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DonChanState {
    Normal, // also kiai
    ComboMilestone,
    Fail
}
