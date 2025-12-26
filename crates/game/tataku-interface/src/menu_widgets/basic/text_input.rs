use crate::prelude::*;
use tataku::{
    Border,
    Vector2,
};
use ui::{
    tree::*,
    widget::*,
};
use widgets::{
    WidgetText,
    InputAction,
};
use input::{ 
    Key,
    KeyInput,
    InputType,
    InputEvent, 
    MouseButton, 
    KeyModifiers,
};

// TODO: should this be context aware? like if you ctrl + whatever on alphanumeric, should it always stop and non-alphanumeric?
/// yes, it probably should
/// 
/// what tokens to stop at when ctrl+(arrow_key|backspace)-ing
const TOKENS: &[char] = &[ 
    '.', ',', 
    '"', '\'', 
    ' ', 
    '(', ')', 
    '[', ']',
    ';', ':',
];

// TODO: add copy/paste support
// TODO: blinky cursor?
// TODO: make sure the forward-select stuff is all correct
// TODO: cache selected text sizes?

#[derive(ChainableInitializer)]
pub struct TextInput {
    #[chain] secure: bool,

    placeholder: WidgetText,
    value: String,
    variable: engine::VariablePathResolver,
    text: widgets::Text,

    load_from_variable: bool,
    
    on_input: Option<InputAction<String>>,
    on_submit: Option<InputAction<String>>,
    
    cursor: Cursor,
    pressed: bool,
    hovered: bool,
    active: bool,

    node_id: NodeId,
}
impl TextInput {
    pub fn new(
        placeholder: WidgetText,
        variable: engine::VariablePathResolver,
    ) -> Self {
        let text = widgets::Text::new("".into());

        Self {
            secure: false,

            placeholder,
            value: String::new(),
            text,
            variable,

            load_from_variable: true,

            on_input: None,
            on_submit: None,

            cursor: Cursor::Position(0),
            pressed: false,
            hovered: false,
            active: false,

            node_id: ui::EMPTY_NODE,
        }
    }

    pub fn on_input(mut self, on_input: Option<impl Into<InputAction<String>>>) -> Self {
        self.on_input = on_input.map(Into::into);
        self
    }

    pub fn on_submit(mut self, on_submit: Option<impl Into<InputAction<String>>>) -> Self {
        self.on_submit = on_submit.map(Into::into);
        self
    }

    fn visible_text(&self) -> Cow<'_, str> {
        if self.value.is_empty() {
            self.placeholder.get()
        } else if self.secure {
            Cow::Owned("*".repeat(self.value.len()))
        } else {
            Cow::Borrowed(&self.value)
        }
    }

    fn handle_control_action(
        &mut self,
        action: ControlAction,
        shift_pressed: bool
    ) {
        if let Cursor::Selection { start, .. } = self.cursor
        && action.delete_text() {
            self.replace_selection("");
            self.cursor = Cursor::Position(start);
            return
        }

        let len = self.value.len();
        let indices = self.value.char_indices();

        fn next(
            indices: std::str::CharIndices<'_>,
            index: usize,
            len: usize,
            forwards: bool
        ) -> usize {
            if forwards {
                indices
                    .skip(index)
                    .find(|(_, c)| TOKENS.contains(c))
                    .map_or(len, |(i, _)| i + 1)
            } else {
                // TODO: validate this is actually correct
                // it seems to be
                indices
                    .rev()
                    .skip(len - index)
                    .find(|(_, c)| TOKENS.contains(c))
                    .map_or(0, |(i, _)| i)
            }
        }


        match action {
            ControlAction::Delete => {
                if let Cursor::Position(index) = self.cursor {
                    let end = next(indices, index, len, true);
                    self.cursor = Cursor::Selection {
                        start: index,
                        end,
                        forward_select: true
                    };
                }

                self.replace_selection("");
            }
            ControlAction::Backspace => {
                if let Cursor::Position(index) = self.cursor {
                    let start = next(indices, index, len, false);
                    self.cursor = Cursor::Selection {
                        start,
                        end: index,
                        forward_select: false
                    };
                }

                self.replace_selection("");
            }
            
            ControlAction::CursorLeft => {
                // TODO: start here when you wake up
                match self.cursor {
                    Cursor::Position(i) => {
                        let start = next(
                            indices,
                            i,
                            len,
                            false
                        );

                        if shift_pressed {
                            self.cursor = Cursor::Selection {
                                start,
                                end: i,
                                forward_select: false
                            };
                        } else {
                            self.cursor = Cursor::Position(start);
                        }
                    }
                    Cursor::Selection {
                        start,
                        end,
                        forward_select
                    } => {
                        if shift_pressed {
                            if forward_select {
                                self.cursor = Cursor::Selection {
                                    start,
                                    end: next(
                                        indices,
                                        end,
                                        len,
                                        false
                                    ),
                                    forward_select
                                };
                            } else {
                                self.cursor = Cursor::Selection {
                                    start: next(
                                        indices,
                                        start,
                                        len,
                                        false
                                    ),
                                    end,
                                    forward_select,
                                };
                            }
                        } else {
                            self.cursor = Cursor::Position(next(
                                indices,
                                start,
                                len,
                                false
                            ));
                        }
                    }
                }

            }
            ControlAction::CursorRight => {
                match self.cursor {
                    Cursor::Position(i) => {
                        let end = next(indices, i, len, true);

                        if shift_pressed {
                            self.cursor = Cursor::Selection {
                                start: i,
                                end,
                                forward_select: true
                            };
                        } else {
                            self.cursor = Cursor::Position(end);
                        }
                    }
                    Cursor::Selection {
                        start,
                        end,
                        forward_select
                    } => {
                        if shift_pressed {
                            if forward_select {
                                self.cursor = Cursor::Selection {
                                    start,
                                    end: next(indices, end, len, true),
                                    forward_select
                                };
                            } else {
                                self.cursor = Cursor::Selection {
                                    start: next(
                                        indices,
                                        start,
                                        len,
                                        true
                                    ),
                                    end,
                                    forward_select
                                };
                            }
                        } else {
                            self.cursor = Cursor::Position(next(
                                indices,
                                end,
                                len,
                                true
                            ));
                        }
                    }
                }
            }
            ControlAction::CursorUp => self.cursor = Cursor::Position(len),
            ControlAction::CursorDown => self.cursor = Cursor::Position(0),
        }


        // make sure we use Position if the selection's start and end are the same
        self.cursor.normalize();
    }

    fn replace_selection(&mut self, text: &str) {
        let Cursor::Selection { start, end, .. } = self.cursor
        else {
            unreachable!("should not be calling this unless cursor is selection")
        };

        let diff = end - start;
        let (start, split) = self.value.split_at(start);
        let end = split.split_at(diff).1;

        self.cursor = Cursor::Position(start.len() + text.len());
        self.value = format!("{start}{text}{end}");
    }

    fn add_text(&mut self, text: &str) {
        match &mut self.cursor {
            Cursor::Position(i) => {
                let (start, end) = self.value.split_at(*i);

                self.value = format!("{start}{text}{end}");
                *i += text.len();
            }
            Cursor::Selection { .. } => {
                self.replace_selection(text);
            }
        }
    }
    // returns if the key was consumed, and if so, if the text was updated
    fn handle_key(
        &mut self, 
        key: &KeyInput, 
        mods: KeyModifiers,
    ) -> Option<bool> {
        if let Some(text) = &key.text {
            self.add_text(text);
            return Some(true);
        }
        
        let len = self.value.len();

        match key.as_key()? {
            Key::Backspace if mods.ctrl => {
                self.handle_control_action(
                    ControlAction::Backspace, 
                    mods.shift
                );
                Some(true)
            }
            Key::Delete if mods.ctrl => {
                self.handle_control_action(
                    ControlAction::Delete, 
                    mods.shift
                );
                Some(true)
            }
            Key::Left if mods.ctrl => {
                self.handle_control_action(
                    ControlAction::CursorLeft, 
                    mods.shift
                );
                Some(true)
            }
            Key::Right if mods.ctrl => {
                self.handle_control_action(
                    ControlAction::CursorRight, 
                    mods.shift
                );
                Some(true)
            }
            Key::Up if mods.ctrl => {
                self.handle_control_action(
                    ControlAction::CursorUp, 
                    mods.shift
                );
                Some(true)
            }
            Key::Down if mods.ctrl => {
                self.handle_control_action(
                    ControlAction::CursorDown, 
                    mods.shift
                );
                Some(true)
            }

            Key::Space => {
                self.add_text(" ");
                Some(true)
            }
            Key::Backspace => {
                match &mut self.cursor {
                    Cursor::Position(n) => {
                        if *n > 0 {
                            *n -= 1;

                            self.value.remove(*n);

                            Some(true)
                        } else {
                            Some(false)
                        }
                    }
                    Cursor::Selection { .. } => {
                        self.replace_selection("");
                        Some(true)
                    }
                }
            }
            Key::Delete => {
                match self.cursor {
                    Cursor::Position(n) => {
                        if n < len {
                            self.value.remove(n);

                            Some(true)
                        } else {
                            Some(false)
                        }
                    }
                    Cursor::Selection { .. } => {
                        self.replace_selection("");
                        Some(true)
                    }
                }
            }

            Key::Left => {
                match &mut self.cursor {
                    Cursor::Position(index) => if *index > 0 {
                        let n = *index-1;

                        if mods.shift {
                            self.cursor = Cursor::Selection { 
                                start: n, 
                                end: *index,
                                forward_select: false
                            };
                        } else {
                            *index = n;
                        }
                    }

                    // select going forwards
                    Cursor::Selection { 
                        start, 
                        end, 
                        forward_select: true 
                    } => if *end > 0 {
                        let n = *end - 1;

                        if mods.shift && *start != n {
                            *end = n;
                        } else {
                            self.cursor = Cursor::Position(n);
                        }
                    }

                    // select going backwards
                    Cursor::Selection { 
                        start, 
                        forward_select: false, 
                        .. 
                    } => if *start > 0 {
                        let n = *start - 1;

                        if mods.shift {
                            *start = n;
                        } else {
                            self.cursor = Cursor::Position(n);
                        }
                    }

                }

                self.cursor.normalize();
                Some(false)
            }

            Key::Right => {
                match &mut self.cursor {
                    Cursor::Position(index) => if *index < len {
                        let n = *index + 1;

                        if mods.shift {
                            self.cursor = Cursor::Selection { 
                                start: *index, 
                                end: n,
                                forward_select: true
                            };
                        } else {
                            *index = n;
                        }
                    }
                    Cursor::Selection { 
                        end, 
                        forward_select: true, 
                        .. 
                    } => if *end < len {
                        let n = *end + 1;
                        if mods.shift {
                            *end = n;
                        } else {
                            self.cursor = Cursor::Position(n);
                        }
                    }

                    Cursor::Selection { 
                        start, 
                        end, 
                        forward_select: false 
                    } => if *end < len {
                        let n = *start + 1;
                        if mods.shift && n != *end {
                            *start = n;
                        } else {
                            self.cursor = Cursor::Position(n);
                        }
                    }
                }

                self.cursor.normalize();
                Some(false)
            }
            Key::Up => {
                match &mut self.cursor {
                    Cursor::Position(n) => *n = len,
                    Cursor::Selection { end, .. } => *end = len,
                }
                self.cursor.normalize();
                Some(false)
            }
            Key::Down => {
                match &mut self.cursor {
                    Cursor::Position(n) => *n = 0,
                    Cursor::Selection { start, .. } => *start = 0,
                }
                self.cursor.normalize();
                Some(false)
            }

        
            Key::Escape => {
                self.active = false;
                Some(false)
            }

            _ if mods.ctrl || mods.alt => None,

            _ => Some(false)
        }
    }

    fn position_to_index(&self, pos: f32) -> usize {
        let layout = self.text.text_layout();

        if self.value.is_empty() { // since the layout would be the placeholder
            return 0;
        } else if pos >= layout.full_width() {
            return self.value.len();
        }

        let result = parley::Cluster::from_point(
            layout,
            pos,
            0.0,
        );

        match result {
            Some((cluster, side)) => match side {
                parley::ClusterSide::Left => cluster.text_range().start,
                parley::ClusterSide::Right => cluster.text_range().end - 1,
            },
            None => 0,
        }
    }

    fn index_to_position(&self, index: usize) -> f32 {
        let layout = self.text.text_layout();

        if index >= self.value.len() {
            return layout.full_width();
        };

        parley::Cluster::from_byte_index(
            layout,
            index,
        )
            .and_then(|cluster| cluster.visual_offset())
            .unwrap()
    }
}
impl Widget<actions::Action> for TextInput {
    fn name(&self) -> CowStr { "text_input_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::Single(&self.text)
    }

    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::Single(&mut self.text)
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        let text = self.text.layout(shell)?;

        self.node_id = shell.tree.new_with_children(&[text])?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        self.text.init_style(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        match &event.event {
            InputType::KeyPress(press) if self.active => {
                if let Some(Key::Enter) = press.as_key() {

                    if let Some(on_submit) = &self.on_submit {
                        on_submit.run(
                            &self.value,
                            self.node_id,
                            shell.source,
                            shell.messages,
                            shell.actions,
                            shell.values,
                        );
                    }

                    shell.event_consumed = true;
                    self.active = false;
                    return;
                }

                if let Some(Key::Tab) = press.as_key() {
                    shell.event_consumed = true;
                    self.active = false;
                    // TODO: event to select the next element
                    return;
                }


                if let Some(text_changed) = self.handle_key(
                    press, 
                    event.key_mods
                ) {
                    shell.event_consumed = true;

                    if text_changed {

                        // todo: fixme:
                        // if let WidgetText::Custom {
                        //     custom: BuildableText::Variable {
                        //         variable
                        //     },
                        //     cached
                        // } = &self.value {
                        //     let Ok(variable) = variable
                        //         .resolve_path(shell.values)
                        //         .inspect_err(|e| warn!("{e:?}"))
                        //     else { return };

                        //     let _ = shell.values
                        //         .reflect_insert(&variable, cached.clone())
                        //         .inspect_err(|e| warn!("{e:?}"));
                        // }

                        if let Some(on_input) = &self.on_input {
                            on_input.run(
                                &self.value,
                                self.node_id,
                                shell.source,
                                shell.messages,
                                shell.actions,
                                shell.values,
                            );
                        }
                    }
                }
            }


            InputType::MouseMove(pos) => {
                let Some(ctx) = shell.tree.get_context(self.node_id) 
                else { return };

                let pos = ctx.inverse_global_transform * *pos;
                let bounds = shell.tree.bounds(self.node_id).unwrap();
                self.hovered = bounds.contains(pos);

                if self.pressed {
                    use std::cmp::Ordering;

                    let index = self.position_to_index(pos.x - bounds.pos.x);

                    match self.cursor {
                        Cursor::Position(i) => {
                            match index.cmp(&i) {
                                Ordering::Equal => {}
                                Ordering::Less => self.cursor = Cursor::Selection { 
                                    start: index, 
                                    end: i, 
                                    forward_select: false
                                },
                                Ordering::Greater => self.cursor = Cursor::Selection { 
                                    start: i, 
                                    end: index, 
                                    forward_select: true
                                }
                            }
                        }
                        Cursor::Selection { 
                            start, 
                            end, 
                            forward_select 
                        } => {
                            if forward_select {
                                if index >= start {
                                    self.cursor = Cursor::Selection { 
                                        start, 
                                        end: index, 
                                        forward_select: true 
                                    };
                                } else {
                                    self.cursor = Cursor::Selection { 
                                        start: index, 
                                        end: start, 
                                        forward_select: false 
                                    };
                                }
                            } else if index <= end {
                                self.cursor = Cursor::Selection { 
                                    start: index, 
                                    end, 
                                    forward_select: false 
                                };
                            } else {
                                self.cursor = Cursor::Selection { 
                                    start: end, 
                                    end: index, 
                                    forward_select: true 
                                };
                            }
                        }
                    }

                    self.cursor.normalize();
                }
            }
            InputType::MousePress(MouseButton::Left) => {
                self.active = self.hovered;
                self.pressed = self.active;

                let Some(ctx) = shell.tree.get_context(self.node_id) 
                else { return };

                let pos = ctx.inverse_global_transform * event.mouse_pos;
                let bounds = shell.tree
                    .content_bounds(self.node_id)
                    .unwrap();

                if self.pressed {
                    shell.event_consumed = true;

                    let index = self.position_to_index(pos.x - bounds.pos.x);

                    self.cursor = Cursor::Position(index);
                }
            }
            InputType::MouseRelease(MouseButton::Left) => {
                self.pressed = false;
                // self.active = false;
            }
            
            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        shell.list.push(graphics::Rectangle::new_bounds(
            bounds,
            shell.general_theme.background_color
        ).border(Border::new(
            shell.general_theme.get_color(self.active, self.hovered), 
            2.0
        )));

        self.text.draw(shell);

        if !self.active || self.value.is_empty() { return; }

        let layout = self.text.text_layout();
        let height = layout.height(); // since there is only one line

        let y_offset = (bounds.size.y - height) / 2.0;

        match self.cursor {
            Cursor::Position(i) => {
                let pos = self.index_to_position(i);

                let cursor_bar = graphics::Rectangle::new(
                    Vector2::new(
                        bounds.pos.x + pos,
                        bounds.pos.y + y_offset
                    ),
                    Vector2::new(
                        2.0,
                        height
                    ),
                    shell.general_theme.active_color,
                );
                shell.list.push(cursor_bar);
            }
            Cursor::Selection { start, end, .. } => {
                assert!(end > start);

                let start = self.index_to_position(start);
                let end = self.index_to_position(end);

                let cursor_bar = graphics::Rectangle::new(
                    Vector2::new(
                        bounds.pos.x + start,
                        bounds.pos.y + y_offset
                    ),
                    Vector2::new(
                        end - start,
                        height,
                    ),
                    shell.general_theme.active_color.with_alpha_f32(0.7)
                );
                shell.list.push(cursor_bar);
            }
        }
        
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        self.placeholder.update(shell.values);

        let variable = match self.variable.resolve_path(shell.values) {
            Ok(v) => v,
            Err(e) => {
                warn!("invalid variable path: {e:?}");
                return;
            }
        };

        if self.load_from_variable {
            self.load_from_variable = false;

            match shell.values.reflect_get::<String>(&variable) {
                Ok(value) => {
                    self.value = value.to_string();
                },
                Err(e) => {
                    warn!("invalid text input path: {e:?}");
                },
            }
        } else if let Err(e) = shell.values.reflect_insert(&variable, self.value.clone()) {
            warn!("failed to insert text input to reflect: {e:?}");
        }

        self.text.text.set(self.visible_text().into_owned());
        self.text.update(shell);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cursor {
    Position(usize),
    Selection {
        start: usize,
        end: usize,
        forward_select: bool,
    }
}
impl Cursor {
    fn normalize(&mut self) {
        use std::cmp::Ordering;
        if let Self::Selection { start, end, .. } = *self {
            match start.cmp(&end) {
                Ordering::Less => {}
                Ordering::Equal => *self = Cursor::Position(start),
                Ordering::Greater => *self = Cursor::Selection { 
                    start: end, 
                    end: start, 
                    forward_select: true 
                },
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ControlAction {
    Delete,
    Backspace,
    CursorLeft,
    CursorRight,
    CursorUp,
    CursorDown
}
impl ControlAction {
    fn delete_text(self) -> bool {
        matches!(self, Self::Delete | Self::Backspace)
    }
}

#[test]
/// these tests verify cursor navigation and text input all function as expected
fn test() {
    #[derive(Debug, Clone)]
    struct TestCase {
        input_cursor: Cursor,
        input_event: (KeyInput, KeyModifiers),
        expected: Cursor,
        expected_text: Option<String>,
    }
    impl TestCase {
        fn new(
            input_cursor: Cursor,
            input_event: (KeyInput, KeyModifiers),
            expected: Cursor,
            expected_text: Option<&str>,
        ) -> Self {
            Self {
                input_cursor,
                input_event,
                expected,
                expected_text: expected_text.map(|i| i.to_string())
            }
        }
    }

    let base_txt = "some test text";
    let len = base_txt.len();

    let none = KeyModifiers::default();
    let ctrl = KeyModifiers { ctrl: true, ..Default::default() };
    let shift = KeyModifiers { shift: true, ..Default::default() };
    let ctrl_shift = KeyModifiers { ctrl: true, shift: true, ..Default::default() };

    let tests: &[&[TestCase]] = &[
        // test left out of bounds (left arrow)
        &[
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Left), none),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Left), ctrl),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Left), shift),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Left), ctrl_shift),
                Cursor::Position(0),
                None
            ),
        ],

        // test left out of bounds (down arrow)
        &[
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Down), none),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Down), ctrl),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Down), shift),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Down), ctrl_shift),
                Cursor::Position(0),
                None
            ),
        ],

        // test right out of bounds (right arrow)
        &[
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Right), none),
                Cursor::Position(len),
                None
            ),
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Right), ctrl),
                Cursor::Position(len),
                None
            ),
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Right), shift),
                Cursor::Position(len),
                None
            ),
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Right), ctrl_shift),
                Cursor::Position(len),
                None
            ),
        ],

        // test right out of bounds (up arrow)
        &[
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Up), none),
                Cursor::Position(len),
                None
            ),
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Up), ctrl),
                Cursor::Position(len),
                None
            ),
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Up), shift),
                Cursor::Position(len),
                None
            ),
            TestCase::new(
                Cursor::Position(len),
                (KeyInput::test_key(Key::Up), ctrl_shift),
                Cursor::Position(len),
                None
            ),
        ],


        // test left direction
        &[
            TestCase::new(
                Cursor::Position(5),
                (KeyInput::test_key(Key::Left), none),
                Cursor::Position(4),
                None
            ),
            TestCase::new(
                Cursor::Position(3),
                (KeyInput::test_key(Key::Left), ctrl),
                Cursor::Position(0),
                None
            ),
            TestCase::new(
                Cursor::Position(5),
                (KeyInput::test_key(Key::Left), shift),
                Cursor::Selection { start: 4, end: 5, forward_select: false },
                None
            ),
            TestCase::new(
                Cursor::Position(5),
                (KeyInput::test_key(Key::Left), ctrl_shift),
                Cursor::Selection { start: 0, end: 5, forward_select: false },
                None
            ),
        ],
        // test right direction

        &[
            TestCase::new(
                Cursor::Position(1),
                (KeyInput::test_key(Key::Right), none),
                Cursor::Position(2),
                None
            ),
            TestCase::new(
                Cursor::Position(0),
                (KeyInput::test_key(Key::Right), ctrl),
                Cursor::Position(4),
                None
            ),

            TestCase::new(
                Cursor::Position(1),
                (KeyInput::test_key(Key::Right), shift),
                Cursor::Selection { start: 1, end: 2, forward_select: true },
                None
            ),

            TestCase::new(
                Cursor::Position(1),
                (KeyInput::test_key(Key::Right), ctrl_shift),
                Cursor::Selection { start: 1, end: 4, forward_select: true },
                None
            ),
        ],


        // test text insert
        &[
            // index
            TestCase::new(
                Cursor::Position(4),
                (KeyInput::test_key_text("A"), none),
                Cursor::Position(5),
                Some("someA test text")
            ),

            // selection
            TestCase::new(
                Cursor::Selection { start: 0, end: 5, forward_select: true },
                (KeyInput::test_key_text("A"), none),
                Cursor::Position(1),
                Some("Atest text")
            ),
        ]
        
    ];

    for i in tests.iter().copied().flatten() {
        let mut input = TextInput::new("".into(), base_txt.into());

        input.cursor = i.input_cursor;
        input.handle_key(&i.input_event.0, i.input_event.1);

        println!();
        println!(
            "{:?} + {} ({:?}) -> {:?}", 
            i.input_cursor, 
            i.input_event.0.text.as_ref().map(|k| k.to_string()).unwrap_or(format!("{:?}", i.input_event.0.logical)), 
            i.input_event.1, 
            i.expected
        );
        assert_eq!(input.cursor, i.expected);
        if let Some(text) = &i.expected_text {
            println!("{text}");
            assert!(input.value == *text);
        }
    }

}
