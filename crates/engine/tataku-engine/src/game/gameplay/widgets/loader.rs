use crate::*;
use gameplay::widgets::*;

pub struct UiElementLoader {
    pub layouts: HashMap<String, GameplayWidgetLayout>,
    pub elements: Vec<GameplayWidgetContainer>,
    pub playmode: CowStr,

    pub widget_builders: Vec<GameplayWidgetBuilder>,

    info: gameplay::GamemodeInfo,
    settings: Arc<settings::common_gameplay::CommonGameplaySettings>,
}
impl UiElementLoader {
    pub fn new(
        playmode: impl Into<CowStr>,
        layouts: HashMap<String, GameplayWidgetLayout>,
        widget_builders: Vec<GameplayWidgetBuilder>,

        info: gameplay::GamemodeInfo,
        settings: Arc<settings::common_gameplay::CommonGameplaySettings>,
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

    pub fn load(
        &mut self,
        name: &'static str,
    ) {
        let Some(builder) = self.widget_builders
            .iter()
            .find(|i| i.name == name)
        else {
            warn!("gameplay widget {name} not found");
            return;
        };

        let inner = builder.build(&self.info, &self.settings);
        let default_layout = builder.default_layout.clone();

        let layout = self.layouts
            .get(&format!("{}_{name}", self.playmode)).cloned()
            ;

        // todo: move this somewhere more sensible
        // if layout.scale.x.abs() < 0.01 { layout.scale.x = 1.0 }
        // if layout.scale.y.abs() < 0.01 { layout.scale.y = 1.0 }

        self.elements.push(GameplayWidgetContainer {
            name: Cow::Borrowed(name),
            visible: true,
            resolved_pos: tataku::Vector2::ZERO,
            layout,
            default_layout,
            inner,
        });
    }

    pub fn change_default_layout(
        &mut self,
        name: &'static str,
        layout: GameplayWidgetLayout,
    ) {
        // let name = format!("{}_{name}", self.playmode);
        let Some(element) = self.elements.iter_mut().find(|e| e.name == name) else {
            return warn!("ele not found: {name}")
        };

        element.default_layout = layout;
    }
}
