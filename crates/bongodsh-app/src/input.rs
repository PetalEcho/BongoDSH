use eframe::egui::{Event, Key};

use bongodsh_core::{InputEvent, KeyId};

pub(crate) fn normalize(event: &Event) -> Option<InputEvent> {
    let Event::Key {
        key,
        physical_key,
        pressed,
        repeat,
        modifiers,
    } = event
    else {
        return None;
    };

    let physical = (*physical_key)?;
    let id = KeyId::new(physical as u32);

    // Releases must pass through even if a modifier was pressed after key-down.
    if !pressed {
        return Some(InputEvent::Release(id));
    }
    if modifiers.command || modifiers.ctrl || modifiers.alt || !is_typing_key(*key) {
        return None;
    }

    Some(InputEvent::Press {
        key: id,
        repeat: *repeat,
    })
}

fn is_typing_key(key: Key) -> bool {
    matches!(
        key,
        Key::A
            | Key::B
            | Key::C
            | Key::D
            | Key::E
            | Key::F
            | Key::G
            | Key::H
            | Key::I
            | Key::J
            | Key::K
            | Key::L
            | Key::M
            | Key::N
            | Key::O
            | Key::P
            | Key::Q
            | Key::R
            | Key::S
            | Key::T
            | Key::U
            | Key::V
            | Key::W
            | Key::X
            | Key::Y
            | Key::Z
            | Key::Num0
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5
            | Key::Num6
            | Key::Num7
            | Key::Num8
            | Key::Num9
            | Key::Space
            | Key::Enter
            | Key::Backspace
            | Key::Tab
            | Key::Colon
            | Key::Comma
            | Key::Backslash
            | Key::Slash
            | Key::Pipe
            | Key::Questionmark
            | Key::Exclamationmark
            | Key::OpenBracket
            | Key::CloseBracket
            | Key::OpenCurlyBracket
            | Key::CloseCurlyBracket
            | Key::Backtick
            | Key::Minus
            | Key::Period
            | Key::Plus
            | Key::Equals
            | Key::Semicolon
            | Key::Quote
    )
}
