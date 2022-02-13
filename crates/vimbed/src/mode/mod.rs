pub mod command;
pub mod insert;
pub mod normal;

use std::fmt::Display;

// Top-level mode
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command(CommandMode),
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command(_) => "Command",
        })
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

// Command mode submode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandMode {
    Command,
    Search,
}
