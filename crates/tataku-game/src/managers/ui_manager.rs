use crate::prelude::*;

// TODO: operations (so can scroll to items etc)
pub struct UiManager {
    messages: Vec<Message>,
    current_menu: String,

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
            current_menu: "None".to_owned(),
            root_tree: Tree::new(
                100, // 100 should be fine right? right??!!?
                MessageOwner::Menu, 
                EmptyWidget::new_boxed()
            ), 

            dialog_counter: 0,
            dialogs: Vec::new(),
        }
    }

    pub fn get_menu(&self) -> &String { &self.current_menu }
    pub fn add_message(&mut self, message: Message) { self.messages.push(message) }

    pub fn set_root<T: Reflect>(
        &mut self, 
        root: Box<dyn Widget>,
        values: &mut T
    ) {
        self.current_menu = root.name().into_owned();
        self.messages.retain(|m| !m.owner.is_menu());
        self.root_tree.set_node(root, values);
    }


    pub fn add_dialog(
        &mut self, 
        dialog: Box<dyn Widget>,
        options: DialogCreateOptions,
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        if !options.allow_multiple {
            // check if dialog already exists, if so, dont add it
            let name = dialog.name();
            if self.dialogs
                .iter()
                .any(|n| n.get_node().name() == name)
            { 
                debug!("not adding dialog {}, already exists", dialog.name());
                return 
            }
        }

        let dialog = DialogWidget::new(
            options.title,
            options.draggable,
            options.resizable,
            dialog
        ).boxed();



        debug!("adding dialog: {}", dialog.name());
        let num = self.dialog_counter;
        self.dialog_counter += 1;
        let mut tree = Tree::new(
            50, 
            MessageOwner::Dialog(num), 
            EmptyWidget::new_boxed()
        );

        tree.set_node(dialog, values);
        tree.handle_message(
            &Message::new(
                tree.owner, 
                "set_num",
                MessageValue::Number(num),
            ), 
            values, 
            actions,
            &mut self.messages,
        );


        let mut bounds = self.root_tree.bounds;
        // if the dialog is resizable or draggable, make it a quarter of the screen size
        if options.draggable || options.resizable {
            let half = bounds.size / 2.0;
            let quarter = half / 2.0;
            bounds = Bounds::new(
                bounds.pos + quarter,
                half
            );
        }

        tree.update_bounds(bounds, values);

        self.dialogs.push(tree);
    }

    fn all_trees(&mut self) -> impl Iterator<Item = &mut Tree> {
        [&mut self.root_tree]
            .into_iter()
            .chain(self.dialogs.iter_mut())
    }
    pub fn close_latest(
        &mut self,
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) -> bool {
        let Some(last) = self.dialogs.last_mut() else { return false };

        last.handle_message(
            &Message::new(
                last.owner,
                "force_close",
                MessageValue::Click
            ), 
            values,
            actions, 
            &mut self.messages,
        );
        
        true
    }
    pub fn force_close_all(
        &mut self, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue
    ) {
        for i in self.dialogs.iter_mut() {
            i.handle_message(
                &Message::new(
                    i.owner,
                    "force_close",
                    MessageValue::Click
                ), 
                values,
                actions, 
                &mut self.messages,
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
            if dialog.handle_inputs(
                input_state, 
                values, 
                actions, 
                &mut self.messages
            ) { 
                return 
            }
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

            tree.handle_message(
                &m, 
                values, 
                actions,
                &mut self.messages,
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
            for tree in self.dialogs
                .iter_mut()
                .chain([&mut self.root_tree])
            {
                tree.handle_event(
                    event, 
                    param.as_ref(),
                    values, 
                    actions,
                    &mut self.messages,
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

    pub fn draw(
        &mut self, 
        values: &ValueCollection,
        list: &mut RenderableCollection,
    ) {
        self.root_tree.draw(values, list);
        for i in self.dialogs.iter_mut().rev() {
            i.draw(values, list);
        }
    }

    fn tree_with_node(&mut self, node: NodeId) -> Option<(usize, &mut Tree)> {
        self.all_trees()
            .enumerate()
            .find(|(_, tree)| tree.has_node(node))
    }

    pub fn handle_ui_action(
        &mut self, 
        action: UiAction,
        values: &mut dyn Reflect,
        _actions: &mut ActionQueue,
    ) {
        let node = action.node;
        let action = action.action;

        let Some((mut num, tree)) = self.tree_with_node(node) 
        else {
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
                let Some(mut style) = tree.get_style(node).cloned() 
                else { return };

                f(&mut style);
                tree.set_style(node, style);
                tree.mark_refresh("UpdateStyleWith");
            }

            UiActionType::UpdateDisplay(display) => {
                let Some(mut style) = tree.get_style(node).cloned() else { 
                    return warn!("style not found for node: {node:?}");
                };
                style.display = display;
                tree.set_style(node, style);
                tree.mark_refresh("UpdateDisplay");
            }

            UiActionType::DialogAction(action) if num > 0 => {
                num -= 1; // 0 is the menu, so subtract 1 to get the dialog index
                match action {
                    DialogAction::Close => {
                        // let mut messages = Vec::new();
                        // tree.handle_message(
                        //     &Message::new(
                        //         tree.owner, 
                        //         "force_close", 
                        //         MessageValue::Click
                        //     ), 
                        //     values, 
                        //     actions,
                        //     &mut messages
                        // );
                        self.dialogs.remove(num);
                        // self.messages.extend(messages);
                    }

                    DialogAction::MoveDialog(pos) => {
                        let old_bounds = tree.bounds;
                        tree.update_bounds(
                            Bounds::new(
                                pos, 
                                old_bounds.size,
                            ), values
                        );
                    }

                    DialogAction::ResizeDialog(size) => {
                        let old_bounds = tree.bounds;
                        tree.update_bounds(
                            Bounds::new(
                                old_bounds.pos, 
                                size,
                            ),
                            values
                        );
                        tree.mark_refresh("resize dialog");
                    }
                
                    DialogAction::BringToFront => {
                        let d = self.dialogs.remove(num);
                        self.dialogs.push(d);
                    }
                }
            } 
            
            UiActionType::DialogAction(action) => {
                warn!("trying to run dialog action {action:?} on menu!");
            }
        }
    }


    pub fn window_size_changed(
        &mut self, 
        window_size: Vector2, 
        values: &dyn Reflect,
    ) {
        let old_bounds = self.root_tree.bounds;
        let new_bounds = Bounds::new(Vector2::ZERO, window_size);
        self.root_tree.update_bounds(new_bounds, values);
        
        // only resize dialogs that were the size of the old bounds (aka fullscreen dialogs)
        for i in self.dialogs.iter_mut() {
            if i.bounds == old_bounds {
                i.update_bounds(new_bounds, values);
            }
        }
    }

    pub fn reload_skin(
        &mut self, 
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.root_tree.reload_skin(
            values,
            &mut self.messages,
            actions,
            skin_manager,
        );
        
        for i in self.dialogs.iter_mut() {
            i.reload_skin(
                values,
                &mut self.messages,
                actions,
                skin_manager
            );
        }
    }

}
