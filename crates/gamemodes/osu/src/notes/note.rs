use crate::prelude::*;
use tataku::Vector2;

use engine::{
    beatmaps::{
        osu::*,
    },
};

#[derive(Default)]
pub struct Note {
    /// note position
    pub pos: Vector2,
    /// note time in ms
    pub time: f32,

    /// time when the note was hit
    pub hit_time: Option<f32>,
    /// was the note missed?
    pub missed: bool,

    pub hitsound: u8,
    pub hitsamples: HitSamples,

    /// combo color
    color_index: usize,
    #[cfg(feature="graphics")] circle_image: HitCircle,
    #[cfg(feature="graphics")] pub approach_circle: ApproachCircle,

    #[cfg(feature="graphics")] pub shake: Shake,
}
impl Note {
    pub fn new(
        def: &NoteDef,
        combo_num: u16,
        color_index: usize,
    ) -> Self {
        let time = def.time;
        let pos = def.pos;

        let hitsound = def.hitsound;
        let hitsamples = def.hitsamples.clone();

        Self {
            pos,
            time,

            hitsound,
            hitsamples,

            color_index,

            #[cfg(feature="graphics")]
            circle_image: HitCircle::new(combo_num),

            #[cfg(feature="graphics")]
            approach_circle: ApproachCircle::new(),

            ..Self::default()
        }
    }

    pub fn update(&mut self, shell: &mut UpdateShell) {
        #[cfg(feature = "graphics")]
        self.shake.update(shell.time);

        #[cfg(feature = "graphics")]
        self.circle_image.update(shell);
    }

    #[cfg(feature="graphics")]
    pub fn draw(&self, shell: &mut DrawShell) {
        let hit_time = self.hit_time.unwrap_or(self.time + shell.hitwindow_miss);

        // if its not time to draw anything else, leave
        if self.time - shell.time > shell.fade_in_time
            || shell.time - hit_time > shell.fade_out_time
        {
            return
        }

        let alpha = shell.alpha(self.time, hit_time);

        let transform = shell.transform
            * self.shake.transform(2.0)
            * engine::graphics::Transform {
                pos: shell.coords.to_window(self.pos),
                scale: Vector2::ONE * shell.coords.cs,
                ..Default::default()
            }.matrix();

        let mut shell = DrawShell {
            transform,
            alpha,
            list: shell.list,
            ..*shell
        };

        // note
        self.circle_image.draw(self.color_index, &mut shell);

        // timing circle
        self.approach_circle.draw(self.time, self.color_index, &mut shell);
    }

    pub fn reset(&mut self) {
        self.hit_time = None;
        self.missed = false;
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &tataku_graphics::TextureSource,
        skin_manager: &mut dyn tataku_graphics::SkinProvider,
    ) {
        self.circle_image.reload_skin(source, skin_manager);
        self.approach_circle.reload_texture(source, skin_manager);
    }
}
