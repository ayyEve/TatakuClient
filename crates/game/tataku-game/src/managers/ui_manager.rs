use crate::prelude::*;
use common::reflect::*;
use engine::actions;

use tataku::{
    Vector2,
    Bounds,
};

use ui::{
    tree::Tree,
    widget::{
        Widget,
        shells::*,
    },
    message::{
        Message,
        MessageSource,
    },
};



#[derive(Default2)]
pub struct UiManager {
    messages: Vec<Message>,
    #[default("None".to_owned())]
    current_menu: String,

    /// what menu is currently being drawn?
    #[default(Self::default_tree())]
    pub root_tree: Tree<actions::Action>,

    dialog_counter: usize,
    pub dialogs: Vec<Tree<actions::Action>>,

    pub default_css: &'static str,
}
impl UiManager {
    pub fn new(default_css: &'static str) -> Self {
        Self {
            default_css,
            ..Self::default()
        }
    }

    fn default_tree() -> Tree<actions::Action> {
        Tree::new(
            100, // 100 should be fine right? right??!!?
            MessageSource::Menu,
            ui::EmptyWidget::new_boxed()
        )
    }

    pub fn get_menu(&self) -> &String { &self.current_menu }
    pub fn add_message(&mut self, message: Message) { self.messages.push(message) }

    pub fn set_root<T: Reflect>(
        &mut self,
        root: Box<dyn Widget<actions::Action>>,
        values: &mut T,
        actions: &mut actions::ActionQueue,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        self.root_tree.handle_event(
            &input::TatakuEvent::MenuLeave,
            None,
            values,
            actions,
            &mut self.messages,
        );


        self.current_menu = root.name().into_owned();
        self.messages.retain(|m| !m.source.is_menu());
        self.root_tree.set_node(root, values, self.default_css, text_layout_contexts);
    }


    pub fn add_dialog(
        &mut self,
        dialog: Box<dyn Widget<actions::Action>>,
        options: actions::menu::DialogCreateOptions,
        values: &mut dyn Reflect,
        actions: &mut actions::ActionQueue,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        let name = dialog.name();
        if !options.allow_multiple {
            // check if dialog already exists, if so, dont add it
            if self.dialogs
                .iter()
                .any(|n| n.get_node().name() == name)
            {
                debug!("not adding dialog {name}, already exists");
                return
            }
        }

        let dialog = interface::DialogWidget::new(
            options.title,
            options.draggable,
            options.resizable,
            options.background,
            dialog
        ).boxed();



        debug!("adding dialog: {name}");
        let num = self.dialog_counter;
        self.dialog_counter += 1;
        let mut tree = Tree::new(
            50,
            MessageSource::Dialog(num),
            ui::EmptyWidget::new_boxed()
        );

        tree.set_node(dialog, values, self.default_css, text_layout_contexts);
        tree.handle_message(
            &Message::new(
                tree.source,
                "set_num",
                None,
                Box::new(num),
            ),
            values,
            actions,
            &mut self.messages,
        );
        tree.handle_event(
            &input::TatakuEvent::MenuEnter,
            None,
            values,
            actions,
            &mut self.messages
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

    fn all_trees(&mut self) -> impl Iterator<Item = &mut Tree<actions::Action>> {
        [&mut self.root_tree]
            .into_iter()
            .chain(self.dialogs.iter_mut())
    }
    pub fn close_latest(
        &mut self,
        values: &mut dyn Reflect,
        actions: &mut actions::ActionQueue,
    ) -> bool {
        let Some(last) = self.dialogs.last_mut()
        else { return false };

        last.handle_event(
            &input::TatakuEvent::MenuLeave,
            None,
            values,
            actions,
            &mut self.messages,
        );
        last.handle_message(
            &Message::new(
                last.source,
                "force_close",
                None,
                Box::new(()),
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
        actions: &mut actions::ActionQueue
    ) {
        for i in self.dialogs.iter_mut() {
            i.handle_message(
                &Message::new(
                    i.source,
                    "force_close",
                    None,
                    Box::new(()),
                ),
                values,
                actions,
                &mut self.messages,
            );
        }
        self.dialogs.clear();
        self.dialog_counter = 0;
    }

    pub fn update(
        &mut self,
        input_state: &mut ui::CurrentInputState,
        mut tataku_events: Vec<(input::TatakuEvent, Option<tataku::TatakuValue>)>,
        values: &mut dyn Reflect,
        actions: &mut actions::ActionQueue,
        skin_manager: &mut dyn graphics::SkinProvider,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        self.handle_inputs(input_state, values, actions);

        for m in self.messages.take() {
            let Some(tree) = [&mut self.root_tree]
                .into_iter()
                .chain(self.dialogs.iter_mut())
                .find(|t| t.source == m.source)
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

        for event in input_state.events.iter() {
            let mods = input_state.mods;
            let control = mods.contains(input::KeyModifiers::CTRL);
            let alt = mods.contains(input::KeyModifiers::ALT);
            let shift = mods.contains(input::KeyModifiers::SHIFT);

            match event {
                input::InputType::KeyPress(input) => {
                    let Some(key) = input.key else { continue };

                    tataku_events.push((input::TatakuEvent::KeyPress(input::CustomMenuKeyEvent {
                        key,
                        control,
                        alt,
                        shift,
                    }), None));
                }
                input::InputType::KeyRelease(input) => {
                    let Some(key) = input.key else { continue };

                    tataku_events.push((input::TatakuEvent::KeyRelease(input::CustomMenuKeyEvent {
                        key,
                        control,
                        alt,
                        shift,
                    }), None));
                }

                _ => {}
            }
        }

        for (event, param) in tataku_events {
            // update all trees
            for tree in self.dialogs
                .iter_mut()
                .chain([&mut self.root_tree])
            {
                tree.handle_event(
                    &event,
                    param.as_ref(),
                    values,
                    actions,
                    &mut self.messages,
                );
            }
        }

        // update dialogs
        for dialog in self.dialogs.iter_mut().rev() {
            dialog.update(
                values,
                actions,
                &mut self.messages,
                skin_manager,
                
                self.default_css,
                text_layout_contexts
            );
        }

        // update the root widget
        self.root_tree.update(
            values,
            actions,
            &mut self.messages,
            skin_manager,

            self.default_css,
            text_layout_contexts
        );


        // im leaving this in

        // what u doing here still
        // chaos >:3
        // nooooooooo
        // i cant even type because of you
    }

    fn handle_inputs(
        &mut self,
        input_state: &mut ui::CurrentInputState,
        values: &mut dyn Reflect,
        actions: &mut actions::ActionQueue,
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


    pub fn draw_menu(
        &mut self,
        values: &ValueCollection,
        list: &mut graphics::RenderableCollection,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        self.root_tree.draw(values, list, text_layout_contexts);
    }
    pub fn draw_dialogs(
        &mut self,
        values: &ValueCollection,
        list: &mut graphics::RenderableCollection,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        for i in self.dialogs.iter_mut().rev() {
            i.draw(values, list, text_layout_contexts);
        }
    }

    fn tree_source(&mut self, source: MessageSource) -> Option<(usize, &mut Tree<actions::Action>)> {
        self.all_trees()
            .enumerate()
            .find(|(_, tree)| tree.source == source)
    }

    pub fn handle_ui_action(
        &mut self,
        action: actions::ui::UiAction,
        values: &mut dyn Reflect,
        _actions: &mut actions::ActionQueue,
    ) {
        use actions::ui::UiActionType as UiActionType;
        use actions::dialog::DialogAction as DialogAction;

        let node = action.node;
        let source = action.source;
        let action = action.action;

        let Some((mut num, tree)) = self.tree_source(source)
        else {
            warn!("couldnt find tree with provided node id!");
            return
        };

        match action {
            // UiActionType::Refresh => tree.mark_for_relayout("Refresh"),

            // UiActionType::MarkDirty => {
            //     tree.mark_dirty(node);
            //     tree.mark_for_relayout("MarkDirty");
            // }

            UiActionType::ContextChanged => {
                tree.update_context(node);
            }

            // UiActionType::UpdateStyleWith(f) => {
            //     tree.update_style(node, |s| f(s));
            // }

            // UiActionType::OverrideDisplay(display) => {
            //     tree.override_display(node, display);
            //     tree.mark_for_relayout("UpdateDisplay");
            // }

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
                        tree.mark_for_relayout();
                    }

                    DialogAction::BringToFront => {
                        let d = self.dialogs.remove(num);
                        self.dialogs.push(d);
                    }
                }
            }

            UiActionType::DialogAction(action) => {
                warn!("trying to run dialog action {action:?} on menu!");
            },
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
        actions: &mut actions::ActionQueue,
        skin_manager: &mut dyn graphics::SkinProvider,
        text_layout_contexts: &mut TextLayoutContexts,
    ) {
        self.root_tree.reload_skin(
            values,
            &mut self.messages,
            actions,
            skin_manager,

            self.default_css,
            text_layout_contexts,
        );

        for i in self.dialogs.iter_mut() {
            i.reload_skin(
                values,
                &mut self.messages,
                actions,
                skin_manager,
                self.default_css,
                text_layout_contexts,
            );
        }
    }

}
