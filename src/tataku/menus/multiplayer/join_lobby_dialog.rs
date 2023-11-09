use crate::prelude::*;

pub struct JoinLobbyDialog {
    lobby_id: u32,
    layout_manager: LayoutManager, 
    scrollable: ScrollableArea,
    should_close: bool,
}
impl JoinLobbyDialog {
    pub fn new(lobby_id: u32) -> Self {
        let layout_manager = LayoutManager::new();

        const WIDTH:f32 = 500.0; 
        let mut scrollable = ScrollableArea::new(
            Style {
                min_size: Size {
                    width: Dimension::Points(500.0),
                    height: Dimension::Auto,
                },
                ..Default::default()
            }, 
            ListMode::VerticalList, 
            &layout_manager
        );

        let style = Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Points(50.0)
            },
            ..Default::default()
        };
        // password
        scrollable.add_item(Box::new(TextInput::new(style.clone(), "Password", "", &layout_manager, Font::Main).with_tag("password")));

        // done and close buttons 
        {
            let mut button_scrollable = ScrollableArea::new(
                style.clone(), 
                ListMode::Grid(GridSettings::new(Vector2::ZERO, HorizontalAlign::Center)),
                &layout_manager 
            );
            let style = Style {
                size: LayoutManager::small_button(),
                ..Default::default()
            };
            button_scrollable.add_item(Box::new(MenuButton::new(style.clone(), "Done", &button_scrollable.layout_manager, Font::Main).with_tag("done")));
            button_scrollable.add_item(Box::new(MenuButton::new(style.clone(), "Close", &button_scrollable.layout_manager, Font::Main).with_tag("close")));
            scrollable.add_item(Box::new(button_scrollable));
        }
        // scrollable.set_size(Vector2::new(WIDTH, scrollable.get_elements_height()));

        Self {
            lobby_id,
            layout_manager,
            scrollable,
            should_close: false,
        }
    }

    fn get_value<T: 'static + Clone>(&self, tag: impl ToString) -> T {
        self.scrollable
            .get_tagged(tag.to_string())
            .first()
            .unwrap()
            .get_value()
            .downcast_ref::<T>()
            .unwrap()
            .clone()
    }
}

#[async_trait]
impl Dialog<Game> for JoinLobbyDialog {
    fn name(&self) -> &'static str { "join_lobby_dialog" }
    fn title(&self) -> &'static str { "Join Lobby" }
    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.should_close = true; }

    fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.scrollable.size()) }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.scrollable.on_mouse_move(pos);
    }
    async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
        self.scrollable.on_scroll(delta)
    }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
        if let Some(tagged) = self.scrollable.on_click_tagged(pos, button, *mods) {
            info!("tagged: {tagged}");

            match &*tagged {
                "done" => {
                    let id = self.lobby_id;
                    let password = self.get_value::<String>("password");
                    tokio::spawn(async move { OnlineManager::join_lobby(id, password).await; });
                    self.should_close = true;
                }
                "close" => {
                    self.should_close = true;
                }

                _ => {}
            }

            return true;
        }

        false
    }
    async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.scrollable.on_click_release(pos, button);
        self.scrollable.get_hover()
    }

    async fn on_text(&mut self, text:&String) -> bool {
        self.scrollable.on_text(text.clone());
        self.scrollable.get_selected_index().is_some()
    }
    async fn on_key_press(&mut self, key:Key, mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.scrollable.on_key_press(key, *mods)
    }
    async fn on_key_release(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.scrollable.on_key_release(key);
        self.scrollable.get_selected_index().is_some()
    }

    async fn draw(&mut self, offset:Vector2, list: &mut RenderableCollection) {
        self.scrollable.draw(offset, list);
    }
}
