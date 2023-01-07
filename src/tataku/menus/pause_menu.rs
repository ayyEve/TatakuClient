use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN:f64 = 20.0;
const Y_OFFSET:f64 = 10.0;

pub struct PauseMenu {
    // beatmap: Arc<Mutex<Beatmap>>,
    manager: IngameManager,
    continue_button: MenuButton<Font2, Text>,
    retry_button: MenuButton<Font2, Text>,
    exit_button: MenuButton<Font2, Text>,
    is_fail_menu: bool,

    selected_index: i8,

    window_size: Arc<WindowSize>,

    bg: Option<Image>
}
impl PauseMenu {
    pub async fn new(manager:IngameManager, is_fail_menu: bool) -> PauseMenu {
        let window_size = WindowSize::get();
        let middle = window_size.x /2.0 - BUTTON_SIZE.x/2.0;
        let font = get_font();

        let mut n = 0.0;
        let continue_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Continue", font.clone());
        if !is_fail_menu {n += 1.0}
        let retry_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Retry", font.clone());
        n += 1.0;
        let exit_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Exit", font.clone());

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

    fn exit(&mut self, game:&mut Game) {
        let menu = game.menus.get("beatmap").unwrap().to_owned();
        game.queue_state_change(GameState::InMenu(menu));
    }
}

#[async_trait]
impl AsyncMenu<Game> for PauseMenu {
    fn get_name(&self) -> &str {if self.is_fail_menu {"fail"} else {"pause"}}

    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        self.manager.window_size_changed(window_size).await;
    }
    
    async fn draw(&mut self, args:RenderArgs, list: &mut RenderableCollection) {
        let pos_offset = Vector2::ZERO;
        let depth = 0.0;

        if let Some(bg) = self.bg.clone() {
            list.push(bg)
        }


        // draw buttons
        if !self.is_fail_menu {
            self.continue_button.draw(args, pos_offset, depth, list);
        }
        self.retry_button.draw(args, pos_offset, depth, list);
        self.exit_button.draw(args, pos_offset, depth, list);
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
        if self.exit_button.on_click(pos, button, mods) {self.exit(game)}
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.continue_button.on_mouse_move(pos);
        self.retry_button.on_mouse_move(pos);
        self.exit_button.on_mouse_move(pos);
    }

    async fn on_key_press(&mut self, key:piston::Key, game:&mut Game, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            if self.is_fail_menu {
                self.exit(game);
            } else {
                self.unpause(game);
            }
        }
    }
}
#[async_trait]
impl ControllerInputMenu<Game> for PauseMenu {
    async fn controller_down(&mut self, game:&mut Game, controller: &Box<dyn Controller>, button: u8) -> bool {
        let max = if self.is_fail_menu {2} else {3};

        let mut changed = false;
        if let Some(ControllerButton::DPad_Down) = controller.map_button(button) {
            self.selected_index += 1;
            if self.selected_index >= max {
                self.selected_index = 0;
            }

            changed = true;
        }


        if let Some(ControllerButton::DPad_Up) = controller.map_button(button) {
            if self.selected_index == 0 {
                self.selected_index = max;
            } else if self.selected_index >= max { // original value is 99
                self.selected_index = 0;
            } else {
                self.selected_index -= 1;
            }

            changed = true;
        }

        trace!("changed:{}, index: {}", changed, self.selected_index);

        if changed {
            let mut continue_index = 0;
            if self.is_fail_menu { continue_index = -1 }
            self.continue_button.set_selected(self.selected_index == continue_index);
            self.retry_button.set_selected(self.selected_index == continue_index + 1);
            self.exit_button.set_selected(self.selected_index == continue_index + 2);
        }

        if let Some(ControllerButton::A) = controller.map_button(button) {
            match (self.selected_index, self.is_fail_menu) {
                (0, false) => { // continue
                    self.unpause(game)
                },
                (1, false) | (0, true) => { // retry
                    self.retry(game).await;
                },
                (2, false) | (1, true) => { // close
                    self.exit(game);
                },
                _ => {}
            }
        }

        true
    }
}
