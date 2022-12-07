use crate::prelude::*;
// use super::BAR_COLOR;

const NOTE_BORDER_SIZE:f64 = 2.0;
const GRAVITY_SCALING:f32 = 400.0;


lazy_static::lazy_static! {
    static ref CHAR_MAPPING: Arc<HashMap<&'static str, Vec<char>>> = {
        // many of these are probably wrong because i typed them out on my eng kb
        let list = vec![
            ("", vec![]),
            ("ん", vec!['n']),

            // vowel only
            ("あ", vec!['a']),
            ("い", vec!['i']),
            ("う", vec!['u']),
            ("え", vec!['e']),
            ("お", vec!['o']),

            // starts with 'b'
            ("ば", vec!['b','a']),
            ("び", vec!['b','i']),
            ("ぶ", vec!['b','u']),
            ("べ", vec!['b','e']),
            ("ぼ", vec!['b','o']),

            // starts with 'n'
            ("な", vec!['n','a']),
            ("に", vec!['n','i']),
            ("ぬ", vec!['n','u']),
            ("ね", vec!['n','e']),
            ("の", vec!['n','o']),

            // starts with 'w'
            ("わ", vec!['w','a']),
            ("ゐ", vec!['w','i']), // doesnt exist but dont care
            // ("𛄟", vec!['w','u']),
            ("ゑ", vec!['w','e']), // doesnt exist but dont care
            ("を", vec!['w','o']),
            
            // starts with 'r'
            ("ら", vec!['r','a']),
            ("り", vec!['r','i']),
            ("る", vec!['r','u']),
            ("れ", vec!['r','e']),
            ("ろ", vec!['r','o']),
            
            // starts with 'y'
            ("や", vec!['y','a']),
            // ("い", vec!['y','i']),
            ("ゆ", vec!['y','u']),
            // ("いぇ", vec!['y','e']),
            ("よ", vec!['y','o']),
            
            // starts with 'm'
            ("ま", vec!['m','a']),
            ("み", vec!['m','i']),
            ("む", vec!['m','u']),
            ("め", vec!['m','e']),
            ("も", vec!['m','o']),

            // starts with 'h'
            ("は", vec!['h','a']),
            ("ひ", vec!['h','i']),
            ("ふ", vec!['f','u']), // fu
            ("へ", vec!['h','e']),
            ("ほ", vec!['h','o']),

            // starts with 't'
            ("た", vec!['t','a']),
            ("ち", vec!['c','h','i']), // chi
            ("つ", vec!['t','s','u']), // tsu
            ("っ", vec!['t','u']),
            ("て", vec!['t','e']),
            ("と", vec!['t','o']),

            // starts with 's'
            ("さ", vec!['s','a']),
            ("し", vec!['s', 'h','i']), // shi
            ("じ", vec!['s', 'h','i']), // shi
            ("す", vec!['s','u']),
            ("せ", vec!['s','e']),
            ("そ", vec!['s','o']),

            // starts with 'k'
            ("か", vec!['k','a']),
            ("き", vec!['k','i']),
            ("く", vec!['k','u']),
            ("け", vec!['k','e']),
            ("こ", vec!['k','o']),

            // starts with 'g'
            ("が", vec!['g','a']),
            ("ぎ", vec!['g','i']),
            ("ぐ", vec!['g','u']),
            ("げ", vec!['g','e']),
            ("ご", vec!['g','o']),

            // starts with 'd'
            ("だ", vec!['d','a']),
            ("ぢ", vec!['d','i']),
            ("づ", vec!['d','u']),
            ("で", vec!['d','e']),
            ("ど", vec!['d','o']),

            // starts with 'z'
            ("ざ", vec!['z','a']),
            // ("じ", vec!['z','i']),
            ("ず", vec!['z','u']),
            ("ぜ", vec!['z','e']),
            ("ぞ", vec!['z','o']),
        ];

        let mut map = HashMap::new();
        for (key, val) in list {
            if !map.insert(key, val).is_none() {
                panic!("duplicate entry '{}'", key);
            }
        }

        Arc::new(map)
    };
}

fn get_things_for_text(s: &String) -> Vec<char> {
    s.split("").map(|s|CHAR_MAPPING.get(s).expect(&format!("map missing char '{}'", s))).flatten().map(|c|*c).collect()
}


// note
#[derive(Clone)]
#[allow(unused)]
pub struct UTypingNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
    cutoff_time: f32,

    /// what character is this?
    /// string because jap in utf8 is wack
    text: String,
    romaji: String,
    char_count: usize,

    hit: bool,
    missed: bool,
    speed: f32,

    settings: Arc<TaikoSettings>,
    bounce_factor: f32,

    /// what char are we trying to hit?
    hit_index: usize,

    image: Option<HitCircleImageHelper>
}
impl UTypingNote {
    pub async fn new(time:f32, text: String, cutoff_time: f32, settings:Arc<TaikoSettings>, diff_calc_only:bool) -> Self {
        // let y = settings.hit_position.y;
        // let a = GRAVITY_SCALING * 9.81;
        // let bounce_factor = (2000.0*y.sqrt()) as f32 / (a*(a.powi(2) + 2_000_000.0)).sqrt() * 10.0;
        let bounce_factor = 1.6;

        let entry = get_things_for_text(&text);
        let char_count = entry.len();
        let mut romaji = String::new(); 
        entry.iter().for_each(|c|romaji += &format!(" {c}"));

        Self {
            time, 
            text, 
            romaji,
            
            speed: 1.0,
            hit_time: 0.0,
            hit_index: 0,
            hit: false,
            missed: false,

            pos: Vector2::zero(),
            image: if diff_calc_only {None} else {HitCircleImageHelper::new(&settings, time as f64).await},
            settings,
            bounce_factor,
            cutoff_time,
            char_count,
        }
    }


    // dont look at this
    /// check if the char `c` is valid for this character and hit index
    fn check_char(&self, c:&char) -> bool {
        let mut val = false;

        if let Some(char_list) = CHAR_MAPPING.get(&*self.text) {
            if let Some(ok_char) = char_list.get(self.hit_index) {
                if ok_char == c {
                    val = true
                }
            }
        }

        val
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
        
        self.pos = self.settings.hit_position + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, y as f64);

        if let Some(image) = &mut self.image {
            image.set_pos(self.pos)
        }
    }
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();

        if self.pos.x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > args.window_size[0] as f64 {return list}

        let size = Vector2::new(self.settings.note_radius, self.settings.note_radius);

        if let Some(image) = &mut self.image {
            image.draw(&mut list);
        } else {
            list.push(Box::new(Circle::new(
                Color::TRANSPARENT_WHITE,
                self.time as f64,
                self.pos,
                self.settings.note_radius,
                Some(Border::new(Color::RED, NOTE_BORDER_SIZE))
            )));
        }

        // draw text to hit
        let mut t = Text::new(
            Color::BLACK,
            self.time as f64,
            self.pos,
            32,
            self.romaji.clone(), //self.text.clone(),
            get_font()
        );
        t.center_text(Rectangle::bounds_only(self.pos - size / 2.0, size));
        list.push(Box::new(t));

        list
    }

    async fn reset(&mut self) {
        self.pos = Vector2::zero();
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
        self.hit_index = 0;
    }
}

// utyping hitobject stuff
#[allow(unused)]
impl UTypingNote {
    pub fn get_points(&mut self, time: f32, c: char, hit_windows:Vec<f32>) -> ScoreHit {
        // check already hit
        if self.hit || self.missed {return ScoreHit::None}

        // check time
        if time + hit_windows[0] < self.time {return ScoreHit::None}

        if self.check_char(&c) {
            self.hit_index += 1;

            if self.hit_index == self.char_count {
                self.hit = true;
                self.hit_time = time;
            }

            ScoreHit::X300
        } else {
            self.missed = true;
            self.hit_time = time;
            ScoreHit::Miss
        }
    }

    pub fn x_at(&self, time:f32) -> f32 {
        (self.time() - time) * self.speed
    }
    pub fn time_at(&self, x: f32) -> f32 {
        -(x / self.speed) + self.time()
    }

    pub fn was_hit(&self) -> bool {
        self.hit || self.missed
    }
    pub fn force_hit(&mut self) {
        self.hit = true
    }

    pub fn check_missed(&mut self, time: f32, miss_time: f32) -> bool {
        if self.hit || self.missed {return false}

        if time >= self.end_time(miss_time) {
            self.missed = true;
            self.hit_time = time;
            true
        } else {
            false
        }
    }
}


#[derive(Clone)]
struct HitCircleImageHelper {
    circle: Image,
    overlay: Image,
}
impl HitCircleImageHelper {
    async fn new(_settings: &Arc<TaikoSettings>, depth: f64) -> Option<Self> {
        let scale = 1.0;
        let hitcircle = "taikohitcircle";


        let mut circle = SkinManager::get_texture(hitcircle, true).await;
        if let Some(circle) = &mut circle {
            circle.depth = depth;
            circle.initial_pos = Vector2::zero();
            circle.initial_scale = Vector2::one() * scale;
            // circle.initial_color = color;
            
            circle.current_pos = circle.initial_pos;
            circle.current_scale = circle.initial_scale;
            circle.current_color = circle.initial_color;
        }

        let mut overlay = SkinManager::get_texture(hitcircle.to_owned() + "overlay", true).await;
        if let Some(overlay) = &mut overlay {
            overlay.depth = depth - 0.0000001;
            overlay.initial_pos = Vector2::zero();
            overlay.initial_scale = Vector2::one() * scale;
            // overlay.initial_color = color;
            
            overlay.current_pos = overlay.initial_pos;
            overlay.current_scale = overlay.initial_scale;
            overlay.current_color = overlay.initial_color;
        }

        if overlay.is_none() || circle.is_none() {return None}

        Some(Self {
            circle: circle.unwrap(),
            overlay: overlay.unwrap(),
        })
    }

    fn set_pos(&mut self, pos: Vector2) {
        self.circle.current_pos  = pos;
        self.overlay.current_pos = pos;
    }
    fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        list.push(Box::new(self.circle.clone()));
        list.push(Box::new(self.overlay.clone()));
    }
}
