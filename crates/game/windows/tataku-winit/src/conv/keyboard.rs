use tataku_engine::*;
use input::Key;

use input::smol_str::SmolStr;


pub(crate) fn key(input: &winit::event::KeyEvent) -> Option<input::KeyInput> {
    use winit::keyboard::KeyLocation::*;
    let mut output = input::KeyInput {
        repeat: input.repeat,
        ..Default::default()
    };

    if let winit::keyboard::Key::<SmolStr>::Character(txt) = &input.logical_key {
        let k = match (txt.as_str(), input.location) {
            ("1", Standard) => Some(Key::Key1),
            ("2", Standard) => Some(Key::Key2),
            ("3", Standard) => Some(Key::Key3),
            ("4", Standard) => Some(Key::Key4),
            ("5", Standard) => Some(Key::Key5),
            ("6", Standard) => Some(Key::Key6),
            ("7", Standard) => Some(Key::Key7),
            ("8", Standard) => Some(Key::Key8),
            ("9", Standard) => Some(Key::Key9),
            ("0", Standard) => Some(Key::Key0),
            ("-", Standard) => Some(Key::Minus),
            ("+", Standard) => Some(Key::Plus),
            ("=", Standard) => Some(Key::Equals),
            ("/", Standard) => Some(Key::Slash),
            ("\\", Standard) => Some(Key::Backslash),
            ("*", Standard) => Some(Key::Asterisk),
            (".", Standard) => Some(Key::Period),
            (",", Standard) => Some(Key::NumpadComma),

            ("1", Numpad) => Some(Key::Numpad1),
            ("2", Numpad) => Some(Key::Numpad2),
            ("3", Numpad) => Some(Key::Numpad3),
            ("4", Numpad) => Some(Key::Numpad4),
            ("5", Numpad) => Some(Key::Numpad5),
            ("6", Numpad) => Some(Key::Numpad6),
            ("7", Numpad) => Some(Key::Numpad7),
            ("8", Numpad) => Some(Key::Numpad8),
            ("9", Numpad) => Some(Key::Numpad9),
            ("0", Numpad) => Some(Key::Numpad0),
            ("-", Numpad) => Some(Key::NumpadSubtract),
            ("+", Numpad) => Some(Key::NumpadAdd),
            ("=", Numpad) => Some(Key::NumpadEquals),
            ("/", Numpad) => Some(Key::NumpadDivide),
            ("*", Numpad) => Some(Key::NumpadMultiply),
            (".", Numpad) => Some(Key::NumpadDecimal),
            (",", Numpad) => Some(Key::NumpadComma),

            ("A"|"a", _) => Some(Key::A),
            ("B"|"b", _) => Some(Key::B),
            ("C"|"c", _) => Some(Key::C),
            ("D"|"d", _) => Some(Key::D),
            ("E"|"e", _) => Some(Key::E),
            ("F"|"f", _) => Some(Key::F),
            ("G"|"g", _) => Some(Key::G),
            ("H"|"h", _) => Some(Key::H),
            ("I"|"i", _) => Some(Key::I),
            ("J"|"j", _) => Some(Key::J),
            ("K"|"k", _) => Some(Key::K),
            ("L"|"l", _) => Some(Key::L),
            ("M"|"m", _) => Some(Key::M),
            ("N"|"n", _) => Some(Key::N),
            ("O"|"o", _) => Some(Key::O),
            ("P"|"p", _) => Some(Key::P),
            ("Q"|"q", _) => Some(Key::Q),
            ("R"|"r", _) => Some(Key::R),
            ("S"|"s", _) => Some(Key::S),
            ("T"|"t", _) => Some(Key::T),
            ("U"|"u", _) => Some(Key::U),
            ("V"|"v", _) => Some(Key::V),
            ("W"|"w", _) => Some(Key::W),
            ("X"|"x", _) => Some(Key::X),
            ("Y"|"y", _) => Some(Key::Y),
            ("Z"|"z", _) => Some(Key::Z),

            ("`", _) => Some(Key::Grave),
            ("@", _) => Some(Key::At),
            ("(", _) => Some(Key::LBracket),
            (")", _) => Some(Key::RBracket),
            ("_", _) => Some(Key::Underline),

            ("'", _) => Some(Key::Apostrophe),
            (":", _) => Some(Key::Colon),
            (";", _) => Some(Key::Semicolon),

            _ => None
        };
        output.key = Some(k?);
        output.text = Some(txt.clone());
        return Some(output)
    }

    let winit::keyboard::Key::Named(named_key) = &input.logical_key 
    else { return None };

    use winit::keyboard::NamedKey;
    let k = match (named_key, input.location) {
        (NamedKey::Escape, _) => Some(Key::Escape),
        (NamedKey::F1, _) => Some(Key::F1),
        (NamedKey::F2, _) => Some(Key::F2),
        (NamedKey::F3, _) => Some(Key::F3),
        (NamedKey::F4, _) => Some(Key::F4),
        (NamedKey::F5, _) => Some(Key::F5),
        (NamedKey::F6, _) => Some(Key::F6),
        (NamedKey::F7, _) => Some(Key::F7),
        (NamedKey::F8, _) => Some(Key::F8),
        (NamedKey::F9, _) => Some(Key::F9),
        (NamedKey::F10, _) => Some(Key::F10),
        (NamedKey::F11, _) => Some(Key::F11),
        (NamedKey::F12, _) => Some(Key::F12),
        (NamedKey::F13, _) => Some(Key::F13),
        (NamedKey::F14, _) => Some(Key::F14),
        (NamedKey::F15, _) => Some(Key::F15),
        (NamedKey::F16, _) => Some(Key::F16),
        (NamedKey::F17, _) => Some(Key::F17),
        (NamedKey::F18, _) => Some(Key::F18),
        (NamedKey::F19, _) => Some(Key::F19),
        (NamedKey::F20, _) => Some(Key::F20),
        (NamedKey::F21, _) => Some(Key::F21),
        (NamedKey::F22, _) => Some(Key::F22),
        (NamedKey::F23, _) => Some(Key::F23),
        (NamedKey::F24, _) => Some(Key::F24),
        (NamedKey::PrintScreen, _) => Some(Key::Snapshot),
        (NamedKey::ScrollLock, _) => Some(Key::Scroll),
        (NamedKey::Pause, _) => Some(Key::Pause),
        (NamedKey::Insert, _) => Some(Key::Insert),
        (NamedKey::Home, _) => Some(Key::Home),
        (NamedKey::Delete, _) => Some(Key::Delete),
        (NamedKey::End, _) => Some(Key::End),
        (NamedKey::PageDown, _) => Some(Key::PageDown),
        (NamedKey::PageUp, _) => Some(Key::PageUp),
        (NamedKey::ArrowLeft, _) => Some(Key::Left),
        (NamedKey::ArrowUp, _) => Some(Key::Up),
        (NamedKey::ArrowRight, _) => Some(Key::Right),
        (NamedKey::ArrowDown, _) => Some(Key::Down),
        (NamedKey::Backspace, _) => Some(Key::Backspace),
        (NamedKey::Space, _) => Some(Key::Space),

        (NamedKey::Compose, _) => Some(Key::Compose),
        (NamedKey::NumLock, _) => Some(Key::Numlock),
        (NamedKey::Tab, _) => Some(Key::Tab),

        (NamedKey::Enter, Numpad) => Some(Key::NumpadEnter),
        (NamedKey::Enter, Standard) => Some(Key::Enter),

        (NamedKey::Convert, _) => Some(Key::Convert),


        (NamedKey::Alt, Left) => Some(Key::LAlt),
        (NamedKey::Control, Left) => Some(Key::LControl),
        (NamedKey::Shift, Left) => Some(Key::LShift),
        (NamedKey::Super, Left) => Some(Key::LWin),

        (NamedKey::Alt, Right) => Some(Key::LAlt),
        (NamedKey::Control, Right) => Some(Key::LControl),
        (NamedKey::Shift, Right) => Some(Key::LShift),
        (NamedKey::Super, Right) => Some(Key::LWin),

        (NamedKey::MediaStop, _) => Some(Key::MediaStop),
        (NamedKey::Power, _) => Some(Key::Power),

        (NamedKey::Copy, _) => Some(Key::Copy),
        (NamedKey::Paste, _) => Some(Key::Paste),
        (NamedKey::Cut, _) => Some(Key::Cut),

        _ => None
    };

    
    output.key = Some(k?);
    Some(output)
}
