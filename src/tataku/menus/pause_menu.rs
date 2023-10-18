use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN:f32 = 20.0;
const Y_OFFSET:f32 = 10.0;

pub struct PauseMenu {
    // beatmap: Arc<Mutex<Beatmap>>,
    manager: Box<IngameManager>,
    continue_button: MenuButton,
    retry_button: MenuButton,
    exit_button: MenuButton,
    is_fail_menu: bool,

    selected_index: i8,

    window_size: Arc<WindowSize>,

    bg: Option<Image>
}
impl PauseMenu {
    pub async fn new(manager: Box<IngameManager>, is_fail_menu: bool) -> PauseMenu {
        let window_size = WindowSize::get();
        let middle = window_size.x /2.0 - BUTTON_SIZE.x/2.0;

        let mut n = 0.0;
        let continue_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Continue", Font::Main);
        if !is_fail_menu {n += 1.0}
        let retry_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Retry", Font::Main);
        n += 1.0;
        let exit_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Exit", Font::Main);

        let mut bg = if is_fail_menu {
            SkinManager::get_texture("fail-background", true).await
        } else {
            SkinManager::get_texture("pause-overlay", true).await
        };

        if let Some(bg) = &mut bg {
            bg.fit_to_bg_size(window_size.0, true);
        }

        PauseMenu {
            manager,
            is_fail_menu,
            continue_button,
            retry_button,
            exit_button,
            selected_index: 99,
            window_size,
            bg
        }
    }

    pub fn unpause(&mut self, game:&mut Game) {
        // self.beatmap.lock().start();
        // self.manager.lock().start();

        let manager = std::mem::take(&mut self.manager);
        game.queue_state_change(GameState::Ingame(manager));
    }

    async fn retry(&mut self, game:&mut Game) {
        self.manager.reset().await;
        self.unpause(game);
    }

    async fn exit(&mut self, game:&mut Game) {
        // let menu = game.menus.get("beatmap").unwrap().to_owned();
        game.queue_state_change(GameState::InMenu(Box::new(BeatmapSelectMenu::new().await)));
    }
}

#[async_trait]
impl AsyncMenu<Game> for PauseMenu {
    fn get_name(&self) -> &str {if self.is_fail_menu {"fail"} else {"pause"}}

    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        self.manager.window_size_changed(window_size).await;
    }
    
    async fn draw(&mut self, list: &mut RenderableCollection) {
        let pos_offset = Vector2::ZERO;

        if let Some(bg) = self.bg.clone() {
            list.push(bg)
        }

        // draw buttons
        if !self.is_fail_menu {
            self.continue_button.draw(pos_offset, list);
        }
        self.retry_button.draw(pos_offset, list);
        self.exit_button.draw(pos_offset, list);
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        // continue map
        if !self.is_fail_menu && self.continue_button.on_click(pos, button, mods) {
            self.unpause(game);
            return;
        }
        
        // retry
        if self.retry_button.on_click(pos, button, mods) {
            self.retry(game).await;
            return;
        }

        // return to song select
        if self.exit_button.on_click(pos, button, mods) { self.exit(game).await }
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.continue_button.on_mouse_move(pos);
        self.retry_button.on_mouse_move(pos);
        self.exit_button.on_mouse_move(pos);
    }

    async fn on_key_press(&mut self, key:Key, game:&mut Game, _mods:KeyModifiers) {
        if key == Key::Escape {
            if self.is_fail_menu {
                self.exit(game).await;
            } else {
                self.unpause(game);
            }
        }
    }


    async fn controller_down(&mut self, game:&mut Game, _controller: &GamepadInfo, button: ControllerButton) -> bool {
        let max = if self.is_fail_menu {2} else {3};

        let mut changed = false;

        match button {
            ControllerButton::DPadDown => {
                self.selected_index += 1;
                if self.selected_index >= max {
                    self.selected_index = 0;
                }

                changed = true;
            }

            ControllerButton::DPadUp => {
                if self.selected_index == 0 {
                    self.selected_index = max;
                } else if self.selected_index >= max { // original value is 99
                    self.selected_index = 0;
                } else {
                    self.selected_index -= 1;
                }

                changed = true;
            }

            ControllerButton::South => {
                match (self.selected_index, self.is_fail_menu) {
                    // continue
                    (0, false) => self.unpause(game),
                    // retry
                    (1, false) | (0, true) => self.retry(game).await,
                    // close
                    (2, false) | (1, true) => self.exit(game).await,
                    _ => {}
                }
            }

            _ => {}
        }

        trace!("changed:{}, index: {}", changed, self.selected_index);

        if changed {
            let mut continue_index = 0;
            if self.is_fail_menu { continue_index = -1 }
            self.continue_button.set_selected(self.selected_index == continue_index);
            self.retry_button.set_selected(self.selected_index == continue_index + 1);
            self.exit_button.set_selected(self.selected_index == continue_index + 2);
        }

        true
    }

}
