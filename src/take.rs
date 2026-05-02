use std::fmt;
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
    Escape,
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
    pub shift: bool,
    pub alt: bool,
}

/// Parsed representation of an entire `.take` file.
#[derive(Debug, Default)]
pub struct TakeFile {
    pub shell: Option<String>,
    pub output: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    pub pace: Option<Duration>,
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Type { text, .. } => write!(f, "Type \"{}\"", text),
            Instruction::Press(key, count) => write!(f, "{:?} x{}", key, count),
            Instruction::Ctrl { key, .. } => match key {
                KeyCombo::Char(c) => write!(f, "Ctrl+{}", c.to_ascii_uppercase()),
                KeyCombo::Digit(d) => write!(f, "Ctrl+{}", d),
            },
            Instruction::Sleep(duration) => write!(f, "Sleep {}ms", duration.as_millis()),
            Instruction::Expect { regex, .. } => write!(f, "Expect /{}/", regex),
            Instruction::ExpectLine { regex, .. } => write!(f, "ExpectLine /{}/", regex),
            Instruction::Hide => write!(f, "Hide"),
            Instruction::Show => write!(f, "Show"),
        }
    }
}
