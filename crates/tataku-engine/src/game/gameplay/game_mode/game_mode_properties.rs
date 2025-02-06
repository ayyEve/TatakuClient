use crate::prelude::*;

// #[async_trait]
// pub trait GameModeProperties: Send + Sync {
//     /// playmode for this game mode
//     fn playmode(&self) -> Cow<'static, str>;

//     /// should the cursor be visible (ie, osu yes, taiko/mania no)
//     fn show_cursor(&self) -> bool { false }
    
//     /// what ms does this map end?
//     fn end_time(&self) -> f32;

//     /// what key presses are valid, as well as what they should be named as
//     /// used for the key counter
//     fn get_possible_keys(&self) -> Vec<(KeyPress, &str)>;

//     /// setup any gamemode specific ui elements for this gamemode
//     /// ie combo and leaderboard, since the pos is different per-mode
//     async fn get_ui_elements(
//         &self, 
//         _loader: &mut dyn UiElementLoader,
//     ) {}

//     fn get_playfield(&self) -> Bounds;
    
//     /// f32 is hitwindow, color is color for that window
//     fn timing_bar_things(&self) -> Vec<(f32, Color)>;
    
//     fn get_info(&self) -> GameModeInfo;
// }

pub struct GameModeProperties {
    pub info: &'static GameModeInfo,
    // pub playmode: Cow<'static, str>,
    pub keys: Vec<(KeyPress, &'static str)>,
    pub end_time: f32,
    pub show_cursor: bool,
    pub timing_bar_things: Vec<(f32, Color)>,

    pub audio_prefix: String,
}
impl GameModeProperties {
    pub fn playmode(&self) -> &'static str {
        self.info.id
    }
}
impl Default for GameModeProperties {
    fn default() -> Self {
        Self {
            info: &GameModeInfo::DEFAULT,
            // playmode: Cow::Borrowed("none"),
            keys: Vec::new(),
            end_time: 0.0,
            show_cursor: false,
            timing_bar_things: Vec::new(),
            audio_prefix: String::new(),
        }
    }
}



#[async_trait]
pub trait UiElementLoader: Send + Sync {
    /// Load a ui element
    async fn load(
        &mut self, 
        name: &str, 
        default_layout: UiElementLayout, 
        inner: Box<dyn InnerUIElement>
    );

    /// Change the default layout for a ui element
    async fn change_default_layout(
        &mut self,
        name: &str, 
        layout: UiElementLayout, 
    );
}

#[derive(Default)]
pub struct DefaultUiElementLoader {
    pub layouts: HashMap<String, UiElementLayout>,
    pub elements: Vec<UIElement>,
    pub playmode: Cow<'static, str>,
}
impl DefaultUiElementLoader {
    pub fn new(
        playmode: impl Into<Cow<'static, str>>, 
        layouts: HashMap<String, UiElementLayout>
    ) -> Self {
        Self {
            playmode: playmode.into(),
            elements: Vec::new(),
            layouts,
        }
    }
}
#[async_trait]
impl UiElementLoader for DefaultUiElementLoader {
    async fn load(
        &mut self, 
        name: &str, 
        default_layout: UiElementLayout, 
        inner: Box<dyn InnerUIElement>
    ) {
        let mut layout = self.layouts
            .get(&format!("{}_{name}", self.playmode)).cloned()
            .unwrap_or_else(|| default_layout.clone())
            ;

        if layout.scale.x.abs() < 0.01 { layout.scale.x = 1.0 }
        if layout.scale.y.abs() < 0.01 { layout.scale.y = 1.0 }

        self.elements.push(UIElement {
            layout,
            default_layout,
            element_name: name.to_string(),
            pos_offset: Vector2::ZERO,
            scale: Vector2::ONE,
            inner,
        });
    }

    async fn change_default_layout(
        &mut self,
        name: &str, 
        layout: UiElementLayout, 
    ) {
        // let name = format!("{}_{name}", self.playmode);
        let Some(element) = self.elements.iter_mut().find(|e| e.element_name == name) else { 
            return warn!("ele not found: {name}")
        };
        
        // TODO: is there a better way? this is kinda silly
        if element.default_layout == element.layout {
            element.layout = layout.clone()
        }

        element.default_layout = layout;
    }
}
