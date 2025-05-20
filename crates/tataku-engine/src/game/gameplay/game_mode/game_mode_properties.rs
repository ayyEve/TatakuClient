use crate::prelude::*;

pub struct GameModeProperties {
    pub info: &'static GamemodeInfo,
    // pub playmode: Cow<'static, str>,
    pub keys: Vec<(KeyPress, &'static str)>,
    pub end_time: f32,
    pub show_cursor: bool,
    pub timing_bar_things: Vec<(f32, Color)>,

    pub audio_prefix: String,
    pub sound_list: Vec<(String, Vec<AudioLoadData>)>,
}
impl GameModeProperties {
    pub fn playmode(&self) -> &'static str {
        self.info.id
    }
}
impl Default for GameModeProperties {
    fn default() -> Self {
        Self {
            info: &GamemodeInfo::DEFAULT,
            // playmode: Cow::Borrowed("none"),
            keys: Vec::new(),
            end_time: 0.0,
            show_cursor: false,
            timing_bar_things: Vec::new(),
            audio_prefix: String::new(),
            sound_list: Vec::new(),
        }
    }
}


pub trait UiElementLoader: Send + Sync {
    /// Load a ui element
    fn load(
        &mut self, 
        name: &str, 
    );

    /// Change the default layout for a ui element
    fn change_default_layout(
        &mut self,
        name: &str, 
        layout: GameplayWidgetLayout, 
    );
}

pub struct DefaultUiElementLoader {
    pub layouts: HashMap<String, GameplayWidgetLayout>,
    pub elements: Vec<GameplayWidgetContainer>,
    pub playmode: Cow<'static, str>,

    pub widget_builders: Vec<GameplayWidgetBuilder>,

    info: GamemodeInfo,
    settings: Arc<CommonGameplaySettings>,
}
impl DefaultUiElementLoader {
    pub fn new(
        playmode: impl Into<Cow<'static, str>>, 
        layouts: HashMap<String, GameplayWidgetLayout>,
        widget_builders: Vec<GameplayWidgetBuilder>,
        
        info: GamemodeInfo,
        settings: Arc<CommonGameplaySettings>,
    ) -> Self {
        Self {
            widget_builders,
            playmode: playmode.into(),
            elements: Vec::new(),
            layouts,

            info, 
            settings,
        }
    }
}

impl UiElementLoader for DefaultUiElementLoader {
    fn load(
        &mut self, 
        name: &str, 
    ) {
        let Some(builder) = self.widget_builders
            .iter()
            .find(|i| i.name == name)
        else { 
            warn!("gameplay widget {name} not found");
            return 
        };


        let inner = builder.build(&self.info, &self.settings);
        let default_layout = builder.default_layout.clone();

        let mut layout = self.layouts
            .get(&format!("{}_{name}", self.playmode)).cloned()
            .unwrap_or_else(|| default_layout.clone())
            ;

        if layout.scale.x.abs() < 0.01 { layout.scale.x = 1.0 }
        if layout.scale.y.abs() < 0.01 { layout.scale.y = 1.0 }

        self.elements.push(GameplayWidgetContainer {
            layout,
            default_layout,
            element_name: name.to_string(),
            pos_offset: Vector2::ZERO,
            scale: Vector2::ONE,
            inner,
        });
    }

    fn change_default_layout(
        &mut self,
        name: &str, 
        layout: GameplayWidgetLayout, 
    ) {
        // let name = format!("{}_{name}", self.playmode);
        let Some(element) = self.elements.iter_mut().find(|e| e.element_name == name) else { 
            return warn!("ele not found: {name}")
        };
        
        // TODO: is there a better way? this is kinda silly
        if element.default_layout == element.layout {
            element.layout = layout.clone();
        }

        element.default_layout = layout;
    }
}
