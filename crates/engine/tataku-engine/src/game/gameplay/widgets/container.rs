use crate::*;
use gameplay::widgets::*;

pub struct GameplayWidgetContainer {
    pub name: CowStr,
    pub visible: bool,

    pub preferred_size: tataku::Vector2,

    pub resolved_pos: tataku::Vector2,

    pub layout: Option<GameplayWidgetLayout>,
    pub default_layout: GameplayWidgetLayout,

    pub inner: Box<dyn GameplayWidget>,
}
impl GameplayWidgetContainer {
    pub fn update(&mut self, shell: &mut GameplayWidgetUpdateShell) {
        if !self.visible { return }
        self.inner.update(shell);
    }

    pub fn draw(
        &mut self,
        shell: &mut GameplayWidgetDrawShell,
    ) {
        if !self.visible { return }

        let transform = tataku::Matrix::identity()
            .trans(self.resolved_pos);

        let mut shell = GameplayWidgetDrawShell {
            transform: shell.transform * transform,
            list: shell.list,
        };

        self.inner.draw(&mut shell);
    }

    pub fn resolved_bounds(&self) -> tataku::Bounds {
        tataku::Bounds::new(self.resolved_pos, self.preferred_size)
    }

    pub fn layout(&self) -> &GameplayWidgetLayout {
        self.layout.as_ref().unwrap_or(&self.default_layout)
    }

    pub fn reset_element(&mut self) {
        self.inner.reset();
    }

    pub fn reload_skin(
        &mut self,
        shell: &mut GameplayWidgetReloadSkinShell
    ) {
        self.inner.reload_skin(shell);
    }
}
