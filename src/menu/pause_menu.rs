use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN:f64 = 20.0;
const Y_OFFSET:f64 = 10.0;

pub struct PauseMenu {
    // beatmap: Arc<Mutex<Beatmap>>,
    manager: IngameManager,
    continue_button: MenuButton,
    retry_button: MenuButton,
    exit_button: MenuButton,
    is_fail_menu: bool,

    selected_index: i8
}
impl PauseMenu {
    pub fn new(manager:IngameManager, is_fail_menu: bool) -> PauseMenu {
        let middle = Settings::window_size().x /2.0 - BUTTON_SIZE.x/2.0;

        let mut n = 0.0;
        let continue_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Continue");
        if !is_fail_menu {n += 1.0}
        let retry_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Retry");
        n += 1.0;
        let exit_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * n + Y_OFFSET), BUTTON_SIZE, "Exit");

        PauseMenu {
            manager,
            is_fail_menu,
            continue_button,
            retry_button,
            exit_button,
            selected_index: 99,
        }
    }

    pub fn unpause(&mut self, game:&mut Game) {
        // self.beatmap.lock().start();
        // self.manager.lock().start();

        let manager = std::mem::take(&mut self.manager);
        game.queue_state_change(GameState::Ingame(manager));
    }

    fn retry(&mut self, game:&mut Game) {
        self.manager.reset();
        self.unpause(game);
    }

    fn exit(&mut self, game:&mut Game) {
        let menu = game.menus.get("beatmap").unwrap().to_owned();
        game.queue_state_change(GameState::InMenu(menu));
    }
}
impl Menu<Game> for PauseMenu {
    fn get_name(&self) -> &str {if self.is_fail_menu {"fail"} else {"pause"}}
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let pos_offset = Vector2::zero();
        let depth = 0.0;

        // draw buttons
        if !self.is_fail_menu {
            self.continue_button.draw(args, pos_offset, depth, &mut list);
        }
        self.retry_button.draw(args, pos_offset, depth, &mut list);
        self.exit_button.draw(args, pos_offset, depth, &mut list);

        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        // continue map
        if !self.is_fail_menu && self.continue_button.on_click(pos, button, mods) {
            self.unpause(game);
            return;
        }
        
        // retry
        if self.retry_button.on_click(pos, button, mods) {
            self.retry(game);
            return;
        }

        // return to song select
        if self.exit_button.on_click(pos, button, mods) {self.exit(game)}
    }

    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.continue_button.on_mouse_move(pos);
        self.retry_button.on_mouse_move(pos);
        self.exit_button.on_mouse_move(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            if self.is_fail_menu {
                self.exit(game);
            } else {
                self.unpause(game);
            }
        }
    }
}
impl ControllerInputMenu<Game> for PauseMenu {
    fn controller_down(&mut self, game:&mut Game, controller: &Box<dyn Controller>, button: u8) -> bool {
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

        println!("changed:{}, index: {}", changed, self.selected_index);

        if changed {
            let mut continue_index = 0;
            if self.is_fail_menu {continue_index = -1}
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
                    self.retry(game);
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
