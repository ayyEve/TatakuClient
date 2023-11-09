use crate::prelude::*;

pub struct CreateLobbyDialog {
    scrollable: ScrollableArea,
    layout_manager: LayoutManager,
    should_close: bool,
}
impl CreateLobbyDialog {
    pub fn new() -> Self {
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

        // name
        scrollable.add_item(Box::new(TextInput::new(style.clone(), "Lobby Name", "", &layout_manager, Font::Main).with_tag("name")));

        // password
        scrollable.add_item(Box::new(TextInput::new(style.clone(), "Password", "", &layout_manager, Font::Main).with_tag("password")));

        // private
        scrollable.add_item(Box::new(Checkbox::new(style.clone(), "Private", false, &layout_manager, Font::Main).with_tag("private")));

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
            scrollable,
            layout_manager,
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
impl Dialog<Game> for CreateLobbyDialog {
    fn name(&self) -> &'static str { "create_lobby_dialog" }
    fn title(&self) -> &'static str { "Create a Lobby" }
    fn should_close(&self) -> bool { self.should_close }
    fn get_bounds(&self) -> Bounds { Bounds::new(Vector2::ZERO, self.scrollable.size()) }
    async fn force_close(&mut self) { self.should_close = true; }

    async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
        self.scrollable.on_mouse_move(pos);
    }
    async fn on_mouse_scroll(&mut self, delta:f32, _g:&mut Game) -> bool {
        self.scrollable.on_scroll(delta)
    }
    async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _game:&mut Game) -> bool {
        if let Some(tagged) = self.scrollable.on_click_tagged(pos, button, *mods) {

            match &*tagged {
                "done" => {
                    let name = self.get_value::<String>("name");
                    let password = self.get_value::<String>("password");
                    let private = self.get_value::<bool>("private");
                    let players = 16;
                    tokio::spawn(async move {
                        OnlineManager::create_lobby(name, password, private, players).await
                    });
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
