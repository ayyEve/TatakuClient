use crate::prelude::*;
use crate::prelude::ui::*;

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
    value: WidgetText,
    
    #[chain] on_input: InputAction<String>,
    #[chain] on_submit: InputAction<String>,
    
    cursor: Cursor,
    pressed: bool,
    hovered: bool,
    active: bool,

    node_id: NodeId,
}
impl TextInput {
    pub fn new(
        placeholder: impl Into<WidgetText>,
        value: impl Into<WidgetText>,
    ) -> Self {
        Self {
            cursor: Cursor::Position(0),

            placeholder: placeholder.into(),
            value: value.into(),
            secure: false,

            on_input: InputAction::default(),
            on_submit: InputAction::default(),

            pressed: false,
            hovered: false,
            active: false,

            node_id: EMPTY_NODE,
        }
    }

    fn get_text(&self) -> Cow<'_, str> {
        let value = self.value.get();
        if value.is_empty() {
            self.placeholder.get()
        } else if self.secure {
            Cow::Owned("*".repeat(value.len()))
        } else {
            value
        }
    }

    fn handle_control_action(
        &mut self, 
        action: ControlAction, 
        shift_pressed: bool
    ) {
        if let Cursor::Selection { start, .. } = self.cursor {
            if action.delete_text() {
                self.replace_selection("");
                self.cursor = Cursor::Position(start);
                return
            }
        }

        let value = self.value.get();
        let len = value.len();
        let indices = value.char_indices();
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
        self.cursor.validate();
    }

    fn replace_selection(&mut self, text: &str) {
        let Cursor::Selection { start, end, .. } = self.cursor 
        else { 
            unreachable!("should not be calling this unless cursor is selection") 
        };
        let value = self.value.get();

        let diff = end - start;
        let (start, split) = value.split_at(start);
        let end = split.split_at(diff).1;

        self.cursor = Cursor::Position(start.len() + text.len());
        self.value.set(format!("{start}{text}{end}"));
    }

    fn add_text(&mut self, text: &str) {
        match &mut self.cursor {
            Cursor::Position(i) => {
                let value = self.value.get();
                let (start, end) = value.split_at(*i);

                self.value.set(format!("{start}{text}{end}"));
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
        
        let len = self.value.get().len();

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
                            let mut value = self.value.get()
                                .clone()
                                .into_owned();

                            value.remove(*n);
                            self.value.set(value);
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
                            let mut value = self.value.get()
                                .clone()
                                .into_owned();

                            value.remove(n);
                            self.value.set(value);

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

                self.cursor.validate();
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

                self.cursor.validate();
                Some(false)
            }
            Key::Up => {
                match &mut self.cursor {
                    Cursor::Position(n) => *n = len,
                    Cursor::Selection { end, .. } => *end = len,
                }
                self.cursor.validate();
                Some(false)
            }
            Key::Down => {
                match &mut self.cursor {
                    Cursor::Position(n) => *n = 0,
                    Cursor::Selection { start, .. } => *start = 0,
                }
                self.cursor.validate();
                Some(false)
            }

        
            Key::Escape => {
                self.active = false;
                Some(false)
            }

            _ => Some(false)
        }
    }

    fn index_rel_pos(&self, text_style: &TextStyle, mut rel_x: f32) -> usize {
        let (font_size, text_scale) = Text::get_font_size_scaled(
            text_style.font_size
        );

        let value = self.value.get();

        for (i, ch) in value.char_indices() {
            let Some(data) = text_style
                .font
                .get_character(font_size, ch) 
            else { continue };
            rel_x -= data.advance_width() * text_scale;
            
            if rel_x <= 0.0 { return i }
        }

        value.len()
    }
}
impl Widget for TextInput {
    fn name(&self) -> CowStr { "text_input_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell) {
        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();
        
        let min_height = half::f16::from_f32(text_style.line_height);
        let min_width = half::f16::from_f32(text_style
            .measure_text(&self.get_text(), None)
            .x
        );

        shell.tree.update_style(
            self.node_id, 
            |style| {
                style.min_width = CssUnit::Pixels(min_width).into();
                style.min_height = CssUnit::Pixels(min_height).into();
            }
        );
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        if !self.on_input.is_built() {
            self.on_input.build(shell.values);
        }
        if !self.on_submit.is_built() {
            self.on_submit.build(shell.values);
        }

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        match &event.event {
            InputType::KeyPress(press) if self.active => {
                if let Some(Key::Enter) = press.as_key() {

                    self.on_submit.run(
                        &self.value.get().into_owned(),
                        self.node_id,
                        shell.messages,
                        shell.actions,
                        shell.values,
                    );
                    
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
                        if let WidgetText::Custom { 
                            custom: BuildableText::Variable { 
                                variable 
                            },
                            cached 
                        } = &self.value {
                            let Ok(variable) = variable
                                .resolve_path(shell.values)
                                .inspect_err(|e| warn!("{e:?}"))
                            else { return };

                            let _ = shell.values
                                .reflect_insert(&variable, cached.clone())
                                .inspect_err(|e| warn!("{e:?}"));
                        }
                        
                        self.on_input.run(
                            &self.value.get().into_owned(),
                            self.node_id,
                            shell.messages,
                            shell.actions,
                            shell.values,
                        );
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
                    let index = self.index_rel_pos(
                        text_style, 
                        pos.x - bounds.pos.x
                    );
                    
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

                    self.cursor.validate();
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
                    self.cursor = Cursor::Position(self.index_rel_pos(
                        text_style,
                        pos.x - bounds.pos.x
                    ));
                }
            }
            InputType::MouseRelease(MouseButton::Left) => {
                self.pressed = false;
                // self.active = false;
            }
            
            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(bounds) = shell.tree.absolute_bounds(self) 
        else { return };

        let text_style = shell.tree.get_text_style(self)
            .unwrap();


        shell.list.push(
            Rectangle::new_bounds(
                bounds,
                shell.general_theme.background_color
            )
            .border(Border::new(
                shell.general_theme.get_color(self.active, self.hovered), 
                2.0
            ))
        );

        let mut text = self.get_text().clone().into_owned();
        shell.list.push(text_style.create_text(text.clone(), bounds));

        if self.active && !self.value.get().is_empty() {
            match self.cursor {
                Cursor::Position(i) => {
                    // TODO: make sure we handle these when setting the index
                    assert!(i <= text.len());
                    assert!(text.is_char_boundary(i));
                    if i < text.len() {
                        let split = text.split_at(i).0;
                        text = split.to_owned();
                    }
                    
                    // TODO: scale with transform?
                    let size = text_style.measure_text(&text, None);
                    
                    let cursor_bar = Rectangle::new(
                        Vector2::new(
                            bounds.pos.x + size.x,
                            bounds.pos.y
                        ),
                        Vector2::new(
                            2.0,
                            bounds.size.y
                        ),
                        shell.general_theme.active_color,
                    );
                    shell.list.push(cursor_bar);
                }
                Cursor::Selection { start, end, .. } => {
                    assert!(end > start);
                    assert!(text.is_char_boundary(start));
                    assert!(text.is_char_boundary(end));

                    let diff = end - start;
                    let (start, split) = text.split_at(start);
                    let offset = text_style
                        .measure_text(start, None);

                    let split = split.split_at(diff).0;
                    let size = text_style
                        .measure_text(split, None);

                    // TODO: scale with transform?
                    let cursor_bar = Rectangle::new(
                        Vector2::new(
                            bounds.pos.x + offset.x,
                            bounds.pos.y
                        ),
                        Vector2::new(
                            size.x,
                            bounds.size.y
                        ),
                        shell.general_theme.active_color.alpha(0.7)
                    );
                    shell.list.push(cursor_bar);
                }
            }
        }
        
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.value.update(shell.values);
        self.placeholder.update(shell.values);
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
    fn validate(&mut self) {
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
        let mut input = TextInput::new("", base_txt);

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
            assert!(input.value.get() == *text);
        }
    }

}
