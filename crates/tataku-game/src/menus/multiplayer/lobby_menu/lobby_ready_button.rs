use crate::prelude::*;

const SIZE: Vector2 = Vector2::new(150.0, 50.0);

#[derive(ScrollableGettersSetters)]
pub struct LobbyReadyButton {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,
    ui_scale: Vector2,

    current_lobby: CurrentLobbyDataHelper,

    text: String,
    needs_update: bool,
}

impl LobbyReadyButton {
    pub fn new() -> Self {
        Self {
            pos: Vector2::ZERO,
            size: SIZE,
            hover: false,
            tag: "ready".to_owned(),
            ui_scale: Vector2::ONE,
            needs_update: true,
            text: String::new(),

            current_lobby: CurrentLobbyDataHelper::new()
        }
    }
}

impl ScrollableItem for LobbyReadyButton {
    fn ui_scale_changed(&mut self, scale: Vector2) {
        self.ui_scale = scale;
        self.size = SIZE * scale;
    }
    fn update(&mut self) {
        if self.current_lobby.update() || self.needs_update {
            self.needs_update = false;
            let Some(lobby) = &**self.current_lobby else { return };

            let Some(our_user) = lobby.our_user() else { return };
            if let LobbyUserState::NotReady = our_user.state {
                self.text = "Ready".to_owned();
                self.tag = "ready".to_owned();
            } else if lobby.is_host() {
                // if we're the host and we're ready, set text to start/force start
                let mut ready_count = 0;
                for user in lobby.players.iter() {
                    if user.state == LobbyUserState::Ready { ready_count += 1 }
                }

                self.tag = "start".to_owned();
                if ready_count == lobby.players.len() {
                    self.text = "Start".to_owned();
                } else {
                    self.text = format!("Force start ({ready_count}/{})", lobby.players.len());
                }
            } else {
                self.text = "Unready".to_owned();
                self.tag = "unready".to_owned();
            }

        }
    }

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        // // background and border
        // let bg = Rectangle::new(self.pos + pos_offset, self.size, Color::GRAY.alpha(0.8), Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))).shape(Shape::Round(4.0));
        // list.push(bg);

        // list.push(Text::new(self.pos + pos_offset, 32.0 * self.ui_scale.y, &self.text, Color::BLACK, Font::Main));

        // draw box
        let r = Rectangle::new(
            self.pos + pos_offset,
            self.size,
            Color::new(0.2, 0.2, 0.2, 1.0),
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        );
        
        // draw text
        let mut txt = Text::new(
            Vector2::ZERO,
            12.0,
            self.text.clone(),
            Color::WHITE,
            Font::Main
        );
        txt.center_text(&*r);

        list.push(r);
        list.push(txt);
    }
}
