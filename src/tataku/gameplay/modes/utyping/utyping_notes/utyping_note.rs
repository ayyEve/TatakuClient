use crate::prelude::*;
use super::super::prelude::*;
// use super::BAR_COLOR;

const NOTE_BORDER_SIZE:f32 = 2.0;
const GRAVITY_SCALING:f32 = 400.0;

// note
#[derive(Clone)]
#[allow(unused)]
pub struct UTypingNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
    // cutoff_time: f32,

    /// what character is this?
    /// string because jap in utf8 is wack
    text: String,
    // romaji: String,
    // char_count: usize,
    branches: Branch,

    hit: bool,
    missed: bool,
    speed: f32,

    settings: Arc<TaikoSettings>,
    playfield: Arc<UTypingPlayfield>,
    bounce_factor: f32,

    /// what char are we trying to hit?
    hit_index: usize,

    image: Option<HitCircleImageHelper>,

    pub judgment: Option<HitJudgment>
}
impl UTypingNote {
    pub async fn new(time:f32, text: String, settings:Arc<TaikoSettings>, playfield: Arc<UTypingPlayfield>) -> Self {
        // let y = settings.hit_position.y;
        // let a = GRAVITY_SCALING * 9.81;
        // let bounce_factor = (2000.0*y.sqrt()) as f32 / (a*(a.powi(2) + 2_000_000.0)).sqrt() * 10.0;
        let bounce_factor = 1.6;

        let branches = Branch::new(&text);

        // let entry = get_things_for_text(&text);
        // let char_count = entry.len();
        // let mut romaji = String::new(); 
        // entry.iter().for_each(|c|romaji += &format!(" {c}"));

        Self {
            time, 
            text, 
            // romaji,
            branches,
            // char_count,
            
            speed: 1.0,
            hit_time: 0.0,
            hit_index: 0,
            hit: false,
            missed: false,

            pos: Vector2::ZERO,
            image: None,
            settings,
            playfield,
            bounce_factor,
            judgment: None
        }
    }


    // dont look at this
    /// check if the char `c` is valid for this character and hit index
    pub fn check_char(&self, c:&char) -> bool {

        self.branches.check_char(*c)
        // let mut val = false;

        // if let Some(char_list) = CHAR_MAPPING.get(&*self.text) {
        //     if let Some(ok_char) = char_list.get(self.hit_index) {
        //         if ok_char == c {
        //             val = true
        //         }
        //     }
        // }

        // val
    }

}

#[async_trait]
impl HitObject for UTypingNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {
        self.time + hw_miss
    }
    async fn update(&mut self, beatmap_time: f32) {
        let delta_time = beatmap_time - self.hit_time;
        let y = 
            if self.hit {GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2) - (delta_time * self.bounce_factor)} 
            else if self.missed {GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2)} 
            else {0.0};
        
        self.pos = self.playfield.hit_position + Vector2::new((self.time - beatmap_time) * self.speed, y);

        self.image.ok_do_mut(|i|i.set_pos(self.pos));
    }
    async fn draw(&mut self, _time: f32, list: &mut RenderableCollection) {
        if self.pos.x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > 10000000.0 { return }

        let size = Vector2::new(self.settings.note_radius, self.settings.note_radius);

        if let Some(image) = &mut self.image {
            image.draw(list);
        } else {
            list.push(Circle::new(
                self.pos,
                self.settings.note_radius,
                Color::TRANSPARENT_WHITE,
                Some(Border::new(Color::RED, NOTE_BORDER_SIZE))
            ));
        }


        // draw text to hit
        let mut t = Text::new(
            self.pos,
            32.0,
            // self.romaji.clone(), //
            self.text.clone(),
            Color::BLACK,
            Font::Fallback
        );
        let mut rect = Bounds::new(self.pos - size / 2.0, size);
        t.center_text(&rect);
        rect.size.y = 32.0;

        // add romaji variants
        let height = self.settings.note_radius * self.settings.big_note_multiplier * 2.0 + self.settings.playfield_height_padding;
        rect.pos.y = self.pos.y + height / 2.0;

        let prefix = self.branches.current_text();
        let complete_color = Color::RED;
        let completed_len = prefix.len();
        let incomplete_color = Color::WHITE;

        let lines = self.branches.get_strs();

        const MAX_COUNT: usize = 5;
        let over_max = lines.len() > MAX_COUNT;

        for i in 0..lines.len().min(MAX_COUNT) {
            let i = &lines[i];
            let len = i.len();
            
            // draw text to hit
            let mut t = Text::new(
                self.pos,
                32.0,
                i.clone(),
                Color::BLACK,
                Font::Fallback
            );

            t.text_colors = (0..completed_len).map(|_|complete_color).chain((0..(len - completed_len)).map(|_|incomplete_color)).collect();
            t.center_text(&rect);
            rect.pos.y += t.measure_text().y + 5.0;
            list.push(t);
        }
        if over_max {
            // draw text to hit
            let mut t = Text::new(
                self.pos,
                32.0,
                "...".to_owned(),
                Color::BLACK,
                Font::Fallback
            );
            t.center_text(&rect);
            list.push(t);
        }

        list.push(t);
    }

    async fn reset(&mut self) {
        self.pos = Vector2::ZERO;
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
        self.hit_index = 0;
        self.branches.reset();
    }

    async fn reload_skin(&mut self, skin_manager: &mut SkinManager) {
        self.image = HitCircleImageHelper::new(&self.settings, skin_manager).await;
    }
}

// utyping hitobject stuff
impl UTypingNote {
    // pub fn x_at(&self, time:f32) -> f32 {
    //     (self.time() - time) * self.speed
    // }
    pub fn time_at(&self, x: f32) -> f32 {
        -(x / self.speed) + self.time()
    }

    pub fn was_hit(&self) -> bool {
        self.hit || self.missed
    }

    pub fn miss(&mut self, time: f32) {
        self.missed = true;
        self.hit_time = time;
    }
    pub fn hit(&mut self, time: f32, c: char) {
        self.hit_index += 1;

        if self.branches.add_char(c) {
            self.hit = true;
            self.hit_time = time;
        }
    }

    pub fn complete(&self) -> bool {
        self.hit
    }

    // returns the list of chars for the first branch for this character
    pub fn get_chars(&self) -> Vec<char> {
        self.branches.get_first()
        // get_things_for_text(&self.text)
    }

    pub fn update_playfield(&mut self, playfield: Arc<UTypingPlayfield>) {
        self.playfield = playfield;
    }
}


#[derive(Clone)]
struct HitCircleImageHelper {
    circle: Image,
    overlay: Image,
}
impl HitCircleImageHelper {
    async fn new(_settings: &Arc<TaikoSettings>, skin_manager: &mut SkinManager) -> Option<Self> {
        let scale = 1.0;
        let hitcircle = "taikohitcircle";


        let mut circle = skin_manager.get_texture(hitcircle, true).await;
        if let Some(circle) = &mut circle {
            let scale = Vector2::ONE * scale;

            circle.pos = Vector2::ZERO;
            circle.scale = scale;
            // circle.color = color;
        }

        let mut overlay = skin_manager.get_texture(hitcircle.to_owned() + "overlay", true).await;
        if let Some(overlay) = &mut overlay {
            let scale = Vector2::ONE * scale;

            overlay.pos = Vector2::ZERO;
            overlay.scale = scale;
            // overlay.color = color;
        }

        if overlay.is_none() || circle.is_none() { return None }

        Some(Self {
            circle: circle.unwrap(),
            overlay: overlay.unwrap(),
        })
    }

    fn set_pos(&mut self, pos: Vector2) {
        self.circle.pos  = pos;
        self.overlay.pos = pos;
    }
    fn draw(&mut self, list: &mut RenderableCollection) {
        list.push(self.circle.clone());
        list.push(self.overlay.clone());
    }
}
