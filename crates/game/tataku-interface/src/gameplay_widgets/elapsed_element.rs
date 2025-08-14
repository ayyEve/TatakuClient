use crate::prelude::*;

const FONT_SIZE:f32 = 30.0;
const MAX_CHARS:usize = 7; // 1 for neg, 1 for colon, 2 for secs, 3 for mins
const SIZE:Vector2 = Vector2::new(FONT_SIZE * MAX_CHARS as f32, FONT_SIZE);

struct ElapsedElement {
    // elapsed_image: Option<SkinnedNumber>,
    // elapsed_bounds: Bounds,

    speed: f32,
    start_time: f32,
    end_time: f32,

    elapsed: f32,
}
impl ElapsedElement {
    fn build(
        _: &GamemodeInfo,
        _: &Arc<CommonGameplaySettings>
    ) -> Box<dyn GameplayWidget> {
        Box::new(Self {
            // elapsed_image: SkinnedNumber::new(Color::WHITE, -5000.0, Vector2::ZERO, 0.0, "normal", None, 0).await.ok(),
            // elapsed_bounds: Bounds::new(Vector2::ZERO, SIZE),
            
            speed: 1.0,
            start_time: -1.0,
            end_time: -1.0,
            elapsed: 0.0,
        })
    }
}

impl GameplayWidget for ElapsedElement {
    fn display_name(&self) -> &'static str { "Time Elapsed" }
    fn max_size(&self) -> Vector2 { SIZE }

    fn update(&mut self, manager: &mut dyn GameplayManagerTrait) {
        // if the values arent set yet, set them
        if self.start_time == -1.0 {
            self.speed = manager.mods().get_speed();
            self.end_time = manager.end_time() / self.speed;
            self.start_time = self.end_time - manager.metadata().duration / self.speed;
        }

        self.elapsed = manager.time() / self.speed;
    }

    fn draw(
        &mut self, 
        pos_offset: Vector2, 
        scale: Vector2, 
        _align: Alignment,
        list: &mut RenderableCollection,
    ) {
        // let bounds = Bounds::new(
        //     pos_offset,
        //     SIZE * scale
        // );

        let diff = self.elapsed - self.start_time;
        let secs = (diff / 1000.0).floor();
        let mins = (secs / 60.0).floor() as i16;
        let secs = secs as i16 % 60;

        let text = Text::new(
            pos_offset,
            30.0 * scale.y,
            format!("{mins:02}:{secs:02}"),
            Color::WHITE,
            Font::Main
        );
        // text.center_text(&bounds);
        list.push(text);
    }

}


pub const ELAPSED: GameplayWidgetBuilder = GameplayWidgetBuilder {
    name: "elapsed_timer",
    default_layout: GameplayWidgetLayout::new_default(
        GameplayWidgetAnchor::element("judgement_bar", GameplayWidgetAlign::Left), 
        Alignment::CENTER_LEFT,
        Some(Alignment::CENTER_RIGHT),
        None,
    ),
    build: ElapsedElement::build,
};