use crate::prelude::*;


// TODO: operations (so can scroll to items etc)

pub struct UiManager {
    messages: Vec<Message>,
    current_menu: MenuType,

    /// what menu is currently being drawn?
    pub root_tree: Tree,

    dialog_counter: usize,
    pub dialogs: Vec<Tree>,
}
impl UiManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_menu: MenuType::Internal("None"),
            root_tree: Tree::with_capacity(100, MessageOwner::Menu), // 100 should be fine right? right??!!?

            dialog_counter: 0,
            dialogs: Vec::new(),
        }
    }

    pub fn get_menu(&self) -> MenuType { self.current_menu.clone() }
    pub fn add_message(&mut self, message: Message) { self.messages.push(message) }

    pub fn set_root<T: Reflect>(
        &mut self, 
        root: Box<dyn Widget>,
        values: &mut T
    ) {
        self.current_menu = MenuType::from_menu(&*root);
        self.messages.retain(|m| !m.owner.is_menu());
        self.root_tree.set_node(root, values);
    }


    pub fn add_dialog(
        &mut self, 
        dialog: Box<dyn Widget>,
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        let dialog = Box::new(DialogWidget::new(
            dialog.name(),
            false,
            false,
            dialog
        ));

        let num = self.dialog_counter;

        let mut tree = Tree::with_capacity(50, MessageOwner::Dialog(num));
        tree.set_node(dialog, values);
        tree.node.handle_message(
            &Message::new(
                tree.owner, 
                "set_num",
                MessageValue::Number(num),
            ), 
            values, 
            actions
        );


        // FIXME: 
        tree.update_bounds(self.root_tree.bounds);

        self.dialogs.push(tree);
        self.dialog_counter += 1;
    }

    pub fn close_latest(
        &mut self,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) -> bool {
        let Some(last) = self.dialogs.last_mut() else { return false };
        last.node.handle_message(
            &Message::new(
                last.owner,
                "close",
                MessageValue::Click
            ), 
            values,
            actions, 
        );
        true
    }
    pub fn force_close_all(
        &mut self, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue
    ) {
        for i in self.dialogs.iter_mut() {
            i.node.handle_message(
                &Message::new(
                    i.owner,
                    "force_close",
                    MessageValue::Click
                ), 
                values,
                actions, 
            );
        }
        self.dialogs.clear();
        self.dialog_counter = 0;
    }


    fn handle_inputs(
        &mut self,
        input_state: &mut CurrentInputState,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        // check dialogs first
        for dialog in self.dialogs.iter_mut().rev() {
            dialog.handle_inputs(
                input_state, 
                values, 
                actions, 
                &mut self.messages
            );
        }

        self.root_tree.handle_inputs(
            input_state, 
            values, 
            actions, 
            &mut self.messages
        );
    }

    pub fn update(
        &mut self,
        input_state: &mut CurrentInputState,
        mut tataku_events: Vec<(TatakuEventType, Option<TatakuValue>)>,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.handle_inputs(input_state, values, actions);

        for m in self.messages.take() {
            let Some(tree) = [&mut self.root_tree]
                .into_iter()
                .chain(self.dialogs.iter_mut())
                .find(|t| t.owner.is_eq(m.owner)) 
                else { 
                    warn!("no tree for message {m:?}");
                    continue
                };

            tree.node.handle_message(
                &m, 
                values, 
                actions
            );
        }

        for i in input_state.keys_down.0.iter() {
            let Some(key) = i.as_key() else { continue };

            tataku_events.push((TatakuEventType::KeyPress(CustomMenuKeyEvent {
                key,
                control: input_state.mods.ctrl,
                alt: input_state.mods.alt,
                shift: input_state.mods.shift,
            }), None));
        }
        for i in input_state.keys_up.0.iter() {
            let Some(key) = i.as_key() else { continue };

            tataku_events.push((TatakuEventType::KeyRelease(CustomMenuKeyEvent {
                key,
                control: input_state.mods.ctrl,
                alt: input_state.mods.alt,
                shift: input_state.mods.shift,
            }), None));
        }

        for (event, param) in tataku_events {
            // update all trees
            for tree in self.dialogs.iter_mut().chain([&mut self.root_tree]) {
                tree.node.handle_event(
                    event, 
                    param.clone(), 
                    values
                );
            }
        }

        // update dialogs
        for dialog in self.dialogs.iter_mut().rev() {
            dialog.update(values, actions, &mut self.messages, skin_manager);
        }

        // update the root widget
        self.root_tree.update(values, actions, &mut self.messages, skin_manager);
        
        
        // im leaving this in

        // what u doing here still
        // chaos >:3
        // nooooooooo
        // i cant even type because of you
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        for i in [&mut self.root_tree].into_iter().chain(self.dialogs.iter_mut()) {
            i.draw(list);
        }
    }

    fn tree_with_node(&mut self, node: NodeId) -> Option<(usize, &mut Tree)> {
        [&mut self.root_tree]
            .into_iter()
            .chain(self.dialogs.iter_mut())
            .enumerate()
            .find(|(_, tree)| tree.has_node(node))
    }

    pub fn handle_ui_action(
        &mut self, 
        action: UiAction,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        let node = action.node;
        let action = action.action;

        let Some((mut num, tree)) = self.tree_with_node(node) else {
            warn!("couldnt find tree with provided node id!");
            return
        };

        match action {
            UiActionType::Refresh => tree.mark_refresh("Refresh"),

            UiActionType::MarkDirty => {
                tree.mark_dirty(node);
                tree.mark_refresh("MarkDirty");
            }

            UiActionType::ContextChanged => {
                tree.update_context(node);
                // tree.mark_refresh("context changed");
            }

            UiActionType::UpdateStyle(style) => {
                tree.set_style(node, *style);
                tree.mark_refresh("UpdateStyle");
            }

            UiActionType::UpdateStyleWith(f) => {
                let Some(mut style) = tree.get_style(node).cloned() else { return };
                f(&mut style);
                tree.set_style(node, style);
                tree.mark_refresh("UpdateStyleWith");
            }

            UiActionType::UpdateDisplay(display) => {
                let Some(mut style) = tree.get_style(node).cloned() else { return warn!("style not found for node: {node:?}")};
                style.display = display;
                tree.set_style(node, style);
                tree.mark_refresh("UpdateDisplay");
            }

            UiActionType::DialogAction(action) if num > 0 => {
                num -= 1; // 0 is the menu, so subtract 1 to get the dialog index
                match action {
                    DialogAction::Close => {
                        tree.node.handle_message(
                            &Message::new(tree.owner, "force_close", MessageValue::Click), 
                            values, 
                            actions,
                        );
                        self.dialogs.remove(num);
                    }

                    DialogAction::MoveDialog(pos) => {
                        let old_bounds = tree.bounds;
                        tree.update_bounds(Bounds::new(
                            pos, 
                            old_bounds.size,
                        ));
                    }

                    DialogAction::ResizeDialog(size) => {
                        let old_bounds = tree.bounds;
                        tree.update_bounds(Bounds::new(
                            old_bounds.pos, 
                            size,
                        ));
                        tree.mark_refresh("resize dialog");
                    }
                }
            } 
            
            UiActionType::DialogAction(action) => {
                warn!("trying to run dialog action {action:?} on menu!");
            }
        }
    }



    pub fn reload_skin(
        &mut self, 
        values: &mut dyn Reflect,
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.root_tree.reload_skin(
            values,
            &mut self.messages,
            skin_manager
        );
        
        for i in self.dialogs.iter_mut() {
            i.reload_skin(
                values,
                &mut self.messages,
                skin_manager
            );
        }
    }

    pub fn window_size_changed(&mut self, window_size: Vector2) {
        self.root_tree.update_bounds(Bounds::new(Vector2::ZERO, window_size));
        
        // TODO: account for draggables
        for i in self.dialogs.iter_mut() {
            i.update_bounds(Bounds::new(Vector2::ZERO, window_size));
        }
    }
}
