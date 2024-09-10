use crate::prelude::*;
const WIDTH:f32 = 600.0;
const WIDTH2:f32 = 550.0;
const OFFSET:f32 = 25.0;
const OFFSET2:f32 = 5.0;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[cfg_attr(feature="graphics", Setting(get_items="get_key_items", from_menu="keys_from_menu"))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct ManiaSettings {
    // sv
    #[cfg_attr(feature="graphics", Setting(text="Static SV"))]
    pub static_sv: bool,
    #[cfg_attr(feature="graphics", Setting(text="SV Multiplier", min=0.1, max=10.0))]
    pub sv_multiplier: f32,

    /// how much to change the sv by when a sv change key is pressed
    #[cfg_attr(feature="graphics", Setting(text="SV Change Amount", min=0.1, max=10.0))]
    pub sv_change_delta: f32,

    #[cfg_attr(feature="graphics", Setting(text="Per-Column Judjments"))]
    pub judgements_per_column: bool,
    
    /// how far from the hit position should hit indicators be?
    #[cfg_attr(feature="graphics", Setting(text="Judgment Offset", min=-200.0, max=200.0))]
    pub judgement_indicator_offset: f32,
    
    #[cfg_attr(feature="graphics", Setting(text="Use Skin Judgments"))]
    pub use_skin_judgments: bool,

    // playfield settings
    pub playfield_settings: Vec<ManiaPlayfieldSettings>,
    
    /// col_count [col_num, 0 based]
    /// ie for 4k, key 2: mania_keys\[3]\[1]
    pub keys: Vec<Vec<Key>>,
}
    #[cfg(feature="graphics")]
impl ManiaSettings {
    // pub fn get_key_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {

    //     let info = CollapsibleSettings { 
    //         header_text: "Key Config".to_string(), 
    //         header_text_color: Color::WHITE, 
    //         header_text_align: HorizontalAlign::Center, 
    //         header_height: 50.0,
    //         header_color: Color::GRAY, 
    //         header_color_hover: Color::GRAY, 
    //         header_border: Some(Border::new(Color::BLACK, 2.0)), 
    //         header_border_hover: Some(Border::new(Color::RED, 2.0)), 
    //         header_shape: Shape::Round(10.0), 
    //         auto_height: true, 
    //         first_item_margin: Some(10.0),
    //         initially_expanded: false
    //     };
    //     let mut list = ScrollableArea::new(p, Vector2::new(WIDTH, 0.0), ListMode::Collapsible(info.clone()));

    //     let size = Vector2::new(WIDTH2 - OFFSET2*2.0, 50.0);

    //     // add per-key configs
    //     for i in 0..self.playfield_settings.len() {
    //         let mut info = info.clone();
    //         info.header_text = (i+1).to_string() + "K";
    //         info.initially_expanded = false;
    //         let mut list2 = ScrollableArea::new(Vector2::with_x(OFFSET), Vector2::with_x(WIDTH2), ListMode::Collapsible(info));

    //         // "<prefix>|<playfield_setting_index>key<key_index>"
    //         for (n, key) in self.keys[i].iter().enumerate() {
    //             let mut kb = KeyButton::new(Vector2::with_x(OFFSET2), size, *key, "Key ".to_owned() + &(n+1).to_string(), Font::Main).with_tag(prefix.clone() + "|" + &i.to_string() + "key" + &n.to_string());
    //             let s = sender.clone();
    //             kb.on_change = Arc::new(move |_,_|{let _ = s.send(());});
    //             list2.add_item(Box::new(kb));
    //         }

    //         list.add_item(Box::new(list2));
    //     }

    //     vec![Box::new(list)]
    // }

    // pub fn keys_from_menu(&mut self, prefix: String, list: &ScrollableArea) {
    //     // load per-key configs
    //     for i in 0..self.playfield_settings.len() {
    //         // "<prefix>|<playfield_setting_index>key<key_index>"

    //         for (n, key) in self.keys[i].iter_mut().enumerate() {
    //             let tag = prefix.clone() + "|" + &i.to_string() + "key" + &n.to_string();
    //             if let Some(val) = list.get_tagged(tag).first().map(|i|i.get_value()) {
    //                 *key = *val.downcast_ref::<Key>().expect("couldnt downcast key");
    //             }
    //         }
    //     }
    // }
    
}

impl Default for ManiaSettings {
    fn default() -> Self {
        Self {
            // keys
            keys: vec![
                vec![Key::Space], // 1k
                vec![Key::F, Key::J], // 2k
                vec![Key::F, Key::Space, Key::J], // 3k
                vec![Key::D, Key::F, Key::J, Key::K], // 4k
                vec![Key::D, Key::F, Key::Space, Key::J, Key::K], // 5k
                vec![Key::S, Key::D, Key::F, Key::J, Key::K, Key::L], // 6k
                vec![Key::S, Key::D, Key::F, Key::Space, Key::J, Key::K, Key::L], // 7k
                vec![Key::A, Key::S, Key::D, Key::F, Key::J, Key::K, Key::L, Key::Semicolon], // 8k
                vec![Key::A, Key::S, Key::D, Key::F, Key::Space, Key::J, Key::K, Key::L, Key::Semicolon], // 9k
            ],

            // playfield settings
            playfield_settings: vec![
                ManiaPlayfieldSettings::new("1 Key"),
                ManiaPlayfieldSettings::new("2 Key"),
                ManiaPlayfieldSettings::new("3 Key"),
                ManiaPlayfieldSettings::new("4 Key"),
                ManiaPlayfieldSettings::new("5 Key"),
                ManiaPlayfieldSettings::new("6 Key"),
                ManiaPlayfieldSettings::new("7 Key"),
                ManiaPlayfieldSettings::new("8 Key"),
                ManiaPlayfieldSettings::new("9 Key"),
            ],

            // sv
            static_sv: false,
            sv_multiplier: 1.0,
            sv_change_delta: 0.3,


            // other
            judgements_per_column: false,
            judgement_indicator_offset: 200.0,
            use_skin_judgments: true
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[derive(Reflect)]
#[serde(default)]
pub struct ManiaPlayfieldSettings {
    /// name of this config
    pub name: String,

    /// y pos of the hit area
    /// 
    /// if not upside-down, y is playfield_height - this
    pub hit_pos: f32,

    /// how wide is a note column?
    pub column_width: f32,

    /// how wide is the gap between columns?
    pub column_spacing: f32,

    /// how tall is a note?
    pub note_height: f32,

    /// how offset is the playfield?
    pub x_offset: f32,

    /// how thicc is the note border?
    pub note_border_width: f32,

    /// do the notes scroll up?
    pub upside_down: bool,
}
impl ManiaPlayfieldSettings {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn note_size(&self) -> Vector2 {
        Vector2::new(
            self.column_width,
            self.note_height
        )
    }
}
impl Default for ManiaPlayfieldSettings {
    fn default() -> Self {
        Self {
            name: "unknown".to_owned(),

            hit_pos: 200.0,
            column_width: 100.0,
            column_spacing: 5.0,
            note_height: 50.0,
            x_offset: 0.0,

            note_border_width: 1.4,

            upside_down: false,
        }
    }
}
