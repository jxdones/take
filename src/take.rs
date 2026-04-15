use std::time::Duration;

/// Special keys that can be pressed as part of a recording.
#[derive(Debug, PartialEq, Clone)]
pub enum Key {
    Enter,
    Backspace,
    Delete,
    Tab,
    Space,
    Up,
    Down,
    Left,
    Right,
}

/// A single step in a `.take` script.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Type {
        text: String,
        pace: Option<Duration>,
    },
    Press(Key, u32),
    Ctrl {
        key: KeyCombo,
        modifiers: Modifiers,
    },

    Sleep(Duration),
    Expect {
        regex: String,
        timeout: Option<Duration>,
    },
    ExpectLine {
        regex: String,
        timeout: Option<Duration>,
    },

    Hide,
    Show,
}

/// A key payload used by chorded instructions (e.g. Ctrl+<key>).
#[derive(Debug, PartialEq, Clone)]
pub enum KeyCombo {
    Char(char),
    Digit(u8),
}

/// Modifier state for chorded key presses.
#[derive(Debug, PartialEq, Clone)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

/// Parsed representation of an entire `.take` file.
#[derive(Debug, Default)]
pub struct TakeFile {
    pub shell: Option<String>,
    pub output: Option<String>,
    pub pace: Option<Duration>,
    pub instructions: Vec<Instruction>,
}
