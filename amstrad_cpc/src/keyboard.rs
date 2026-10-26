// src/amstrad/keyboard.rs

use winit::keyboard::KeyCode;

/// Convert a modern physical key into coordinates (line, bit) of the CPC matrix.
pub const fn map_key_to_matrix(key: KeyCode) -> Option<(usize, usize)> {
    match key {
        // --- LINE 0 : Cursor & Numerical Pad ---
        KeyCode::ArrowUp => Some((0, 0)),
        KeyCode::ArrowRight => Some((0, 1)),
        KeyCode::ArrowDown => Some((0, 2)),
        KeyCode::Numpad9 => Some((0, 3)),       // f9
        KeyCode::Numpad6 => Some((0, 4)),       // f6
        KeyCode::Numpad3 => Some((0, 5)),       // f3
        KeyCode::NumpadEnter => Some((0, 6)),   // Enter (Pad)
        KeyCode::NumpadDecimal => Some((0, 7)), // f.

        // --- LINE 1 : Left Cursor, Copy & Numerical Pad ---
        KeyCode::ArrowLeft => Some((1, 0)),
        KeyCode::AltLeft => Some((1, 1)), // Copy (mapped on Left Alt)
        KeyCode::Numpad7 => Some((1, 2)), // f7
        KeyCode::Numpad8 => Some((1, 3)), // f8
        KeyCode::Numpad5 => Some((1, 4)), // f5
        KeyCode::Numpad1 => Some((1, 5)), // f1
        KeyCode::Numpad2 => Some((1, 6)), // f2
        KeyCode::Numpad0 => Some((1, 7)), // f0

        // --- LINE 2 : Edition & Modficators ---
        KeyCode::Delete => Some((2, 0)), // Clr (mapped on Suppr)
        KeyCode::BracketLeft => Some((2, 1)), // [
        KeyCode::Enter => Some((2, 2)),  // Main Return
        KeyCode::BracketRight => Some((2, 3)), // ]
        KeyCode::Numpad4 => Some((2, 4)), // f4
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Some((2, 5)), // Shift
        KeyCode::Backslash => Some((2, 6)), // \
        KeyCode::ControlLeft | KeyCode::ControlRight => Some((2, 7)), // Control

        // --- LINE 3 : Righthand Punctuation & P ---
        KeyCode::Equal => Some((3, 0)), // ^
        KeyCode::Minus => Some((3, 1)), // -
        KeyCode::Quote => Some((3, 2)), // @
        KeyCode::KeyP => Some((3, 3)),
        KeyCode::Semicolon => Some((3, 4)), // ; (M on AZERTY)
        KeyCode::Backquote => Some((3, 5)), // :
        KeyCode::Slash => Some((3, 6)),     // /
        KeyCode::Period => Some((3, 7)),    // .

        // --- LINE 4 ---
        KeyCode::Digit0 => Some((4, 0)),
        KeyCode::Digit9 => Some((4, 1)),
        KeyCode::KeyO => Some((4, 2)),
        KeyCode::KeyI => Some((4, 3)),
        KeyCode::KeyL => Some((4, 4)),
        KeyCode::KeyK => Some((4, 5)),
        KeyCode::KeyM => Some((4, 6)), // Comma on AZERTY
        KeyCode::Comma => Some((4, 7)),

        // --- LINE 5 ---
        KeyCode::Digit8 => Some((5, 0)),
        KeyCode::Digit7 => Some((5, 1)),
        KeyCode::KeyU => Some((5, 2)),
        KeyCode::KeyY => Some((5, 3)),
        KeyCode::KeyH => Some((5, 4)),
        KeyCode::KeyJ => Some((5, 5)),
        KeyCode::KeyN => Some((5, 6)),
        KeyCode::Space => Some((5, 7)),

        // --- LINE 6 ---
        KeyCode::Digit6 => Some((6, 0)),
        KeyCode::Digit5 => Some((6, 1)),
        KeyCode::KeyR => Some((6, 2)),
        KeyCode::KeyT => Some((6, 3)),
        KeyCode::KeyG => Some((6, 4)),
        KeyCode::KeyF => Some((6, 5)),
        KeyCode::KeyB => Some((6, 6)),
        KeyCode::KeyV => Some((6, 7)),

        // --- LINE 7 ---
        KeyCode::Digit4 => Some((7, 0)),
        KeyCode::Digit3 => Some((7, 1)),
        KeyCode::KeyE => Some((7, 2)),
        KeyCode::KeyW => Some((7, 3)), // Z on AZERTY
        KeyCode::KeyS => Some((7, 4)),
        KeyCode::KeyD => Some((7, 5)),
        KeyCode::KeyC => Some((7, 6)),
        KeyCode::KeyX => Some((7, 7)),

        // --- LINE 8 ---
        KeyCode::Digit1 => Some((8, 0)),
        KeyCode::Digit2 => Some((8, 1)),
        KeyCode::Escape => Some((8, 2)),
        KeyCode::KeyQ => Some((8, 3)), // A on AZERTY
        KeyCode::Tab => Some((8, 4)),
        KeyCode::KeyA => Some((8, 5)), // Q on AZERTY
        KeyCode::CapsLock => Some((8, 6)),
        KeyCode::KeyZ => Some((8, 7)), // W on AZERTY

        // --- LINE 9 : Joystick 0 & Del Key (Backspace) ---
        KeyCode::Backspace => Some((9, 7)), // Del

        _ => None,
    }
}
