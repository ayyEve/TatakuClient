use crate::prelude::*;
use crate::prelude::ui::*;

pub struct ModDialog {
    mod_groups: Vec<GameplayModGroup>,

    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl ModDialog {
    pub async fn new(groups: Vec<GameplayModGroup>) -> Self {
        let mut new_groups = default_mod_groups();

        // see if any groups are named the same and merge them
        'outer: for g in groups {
            for n in new_groups.iter_mut() {
                if n.name == g.name {
                    n.mods.extend(g.mods.into_iter());
                    continue 'outer;
                }
            }

            new_groups.push(g);
        }

        // // create the scrollable and add the mod buttons to it
        // let window_size = WindowSize::get();
        // let mut scroll = ScrollableArea::new(Vector2::with_y(20.0), window_size.0, ListMode::VerticalList);
        // let pos = Vector2::new(50.0, 0.0);

        // let manager = ModManager::get();
        // for group in new_groups {
        //     scroll.add_item(Box::new(MenuSection::new(pos, 50.0, &group.name, Color::WHITE, Font::Main)));
            
        //     for m in group.mods {
        //         scroll.add_item(Box::new(ModButton::new(pos, m, &manager)));
        //     }
        // }

        Self {
            mod_groups: new_groups,
            // scroll,
            // window_size,
            // selected_index: 0,


            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE,
        }
    }

    fn build_view(&self, shell: &mut LayoutShell<'_>) -> Box<dyn Widget> {
        // let mods = shell.values.reflect_get::<ModManager>("global.mods").unwrap();
        let owner = shell.owner;

        let mut items:Vec<Box<dyn Widget>> = Vec::new();
        for group in self.mod_groups.clone() {
            items.push(TextWidget::new(group.name).width(FILL).boxed());
            items.push(TextWidget::new(" ").width(FILL).boxed());

            for m in group.mods {
                let cond = BuildableCondition::Unbuilt(format!("global.mods.{}", m.name));

                items.push(row!(
                    Checkbox::new(m.name, cond).on_toggle(move |_| Message::new(owner, m, MessageValue::Click)).font_size(30.0).width(FILL).boxed(),
                    TextWidget::new(m.description).width(FILL).font_size(30.0).boxed();
                    width = FILL
                ));
            }
        }

        Container::new(items)
            .id("mods_list")
            .scrollable(true)
            .drag_scroll(true)
            .flex_direction(FlexDirection::Column)
            .vertical_overflow(taffy::Overflow::Scroll)
            .boxed()
    }
}
impl Widget for ModDialog {
    fn name(&self) -> Cow<'static,str> { "mod_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.node.update_styles(tree, resolver, display_override);
    }
    
    fn layout(
        &mut self, 
        shell: &mut LayoutShell<'_>
    ) -> TaffyResult<NodeId> {
        self.node = self.build_view(shell);
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::default(), 
            &[ child ]
        )?;

        Ok(self.node_id)
    }

    fn input(
        &mut self, 
        event: &InputEvent,
        shell: &mut InputShell<'_>,
    ) {
        self.node.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.node.draw(shell);
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>,
        actions: &mut ActionQueue,
    ) { 
        self.node.update(shell, actions)
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        let MessageTag::GameplayMod(m) = message.tag else { return };

        actions.push(ModAction::ToggleMod(m.name.to_owned()));
        for m in m.removes {
            actions.push(ModAction::RemoveMod((*m).to_owned()));
        }
        // self.toggle_mod(m, values);
    }

}


// struct ModButton {
//     m: GameplayMod,

//     node_id: NodeId,
//     node: Box<dyn Widget>
// }
// impl ModButton {
//     fn new(m: GameplayMod) -> Self {
//         let cond = ElementCondition::Unbuilt("".into());

//         let node = row!(
//             Checkbox::new(m.name, cond).on_toggle(move|_| Message::new(owner, m, MessageValue::Click)).font_size(30.0).width(FILL).boxed(),
//             TextWidget::new(m.description).width(FILL).font_size(30.0).boxed();
//             width = FILL
//         );

//         Self {
//             m,
//             node_id: EMPTY_NODE,
//             node,
//         }
//     }
// }
// impl Widget for ModButton {
//     fn name(&self) -> Cow<'static, str> { format!("mod_{}", self.m.name) }
//     fn node_id(&self) -> NodeId { self.node_id }

//     fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
//         let child = self.node.layout(shell)?;
//         self.node_id = shell.tree.new_with_children(
//             Style::DEFAULT, 
//             &[ child ]
//         )?;

//         Ok(self.node_id)
//     }

//     fn input(&mut self, event: &InputEvent, shell: &mut InputShell<'_>) {
//         self.node.input(event, shell);
//     }

//     fn draw(&self, shell: &mut DrawShell<'_> ,) {
//         self.node.draw(shell)
//     }
    
// }



// impl Dialog for ModDialog {
//     fn get_num(&self) -> usize { self.num }
//     fn set_num(&mut self, num: usize) { self.num = num }

//     fn should_close(&self) -> bool { self.should_close }
//     fn force_close(&mut self) { self.should_close = true; }

    



//     // async fn update(&mut self, _g: &mut Game) {
//     //     self.scroll.update();
//     // }
    
//     // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
//     //     self.draw_background(Color::BLACK, offset, list);
//     //     self.scroll.draw(offset, list);
//     // }

//     // async fn on_key_press(&mut self, key:Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
//     //     match key {
//     //         Key::Up => {
//     //             self.deincrement_index();
//     //             true
//     //         }
//     //         Key::Down => {
//     //             self.increment_index();
//     //             true
//     //         }
//     //         Key::Space | Key::Return => {
//     //             self.toggle_current();
//     //             true
//     //         }

//     //         _ => false
//     //     }
//     // }

//     // async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
//     //     self.scroll.on_mouse_move(pos);
//     // }

//     // async fn on_mouse_down(&mut self, pos:Vector2, button:MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
//     //     self.scroll.on_click(pos, button, *mods);
//     //     true
//     // }

//     // async fn on_mouse_up(&mut self, pos:Vector2, button:MouseButton, _mods:&KeyModifiers, _g:&mut Game) -> bool {
//     //     self.scroll.on_click_release(pos, button);
//     //     true
//     // }

// }

// #[derive(ScrollableGettersSetters)]
// struct ModButton {
//     size: Vector2,
//     pos: Vector2,
//     hover: bool,
//     selected: bool,

//     gameplay_mod: GameplayMod,
//     mod_name: String,
//     enabled: bool,

//     mods: ModManagerHelper
// }
// impl ModButton {
//     fn new(pos: Vector2, gameplay_mod: GameplayMod, current_mods: &ModManager) -> Self {
//         let enabled = current_mods.has_mod(gameplay_mod.name);
//         let mod_name = gameplay_mod.display_name.to_owned();

//         Self {
//             size: Vector2::new(500.0, 50.0),
//             pos, 
//             hover: false,
//             selected: false,

//             gameplay_mod,
//             mod_name,

//             enabled,
//             mods: ModManagerHelper::new()
//         }
//     }

//     fn toggle(&self) {
//         let name = self.gameplay_mod.name;
//         let removes:HashSet<String> = self.gameplay_mod.removes.iter().map(|m|(*m).to_owned()).collect();
//         tokio::spawn(async move {
//             let mut manager = ModManager::get_mut();
//             manager.toggle_mod(name);
//             manager.mods.retain(|m|!removes.contains(m));
//         });
//     }
// }
// impl ScrollableItem for ModButton {
//     fn update(&mut self) {
//         if self.mods.update() {
//             self.enabled = self.mods.has_mod(self.gameplay_mod)
//         }
//     }

//     fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
//         let pos_offset = self.pos + pos_offset;
//         let cb_size = Vector2::new(200.0, 50.0);

//         let mut checkbox = Checkbox::new(Vector2::ZERO, cb_size, &self.mod_name, self.enabled, Font::Main);
//         checkbox.set_hover(self.hover);
//         checkbox.set_selected(self.selected);

//         let font_size = 30.0;
//         let desc_pos = pos_offset + cb_size.x_portion() + Vector2::new(10.0, (cb_size.y - font_size) / 2.0);
//         let desc_text = Text::new(
//             desc_pos, 
//             font_size, 
//             self.gameplay_mod.description, 
//             Color::WHITE, 
//             Font::Main
//         );

//         checkbox.draw(pos_offset, list);
//         list.push(desc_text);
//     }

//     fn on_key_press(&mut self, key:Key, _mods:KeyModifiers) -> bool {
//         if key == Key::Space {
//             self.toggle()
//         }

//         self.get_selected()
//     }

//     fn on_click(&mut self, _pos:Vector2, _btn: MouseButton, _mods:KeyModifiers) -> bool {
//         if self.hover {
//             self.toggle();
//         }

//         self.hover
//     }
// }
