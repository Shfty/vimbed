use std::fmt::Debug;

use nom::{error::Error, Err};

use crate::{
    buffer::Buffer,
    command::{command, BufferCommand, Command, ContextCommand},
    mode::{command::command_command, insert::insert_command, normal::normal_command, Mode},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BufferId {
    Edit,
    Command,
    Search,
}

/// Result type for handling nom errors
type Result<'i> = std::result::Result<(), Err<Error<&'i str>>>;

// Vim application context
pub struct Context<'a> {
    pub mode: Mode,

    pub buffer_edit: Buffer<'a>,
    pub buffer_command: Buffer<'a>,
    pub buffer_search: Buffer<'a>,

    pub fn_command: Option<Box<dyn FnMut(&str)>>,
}

impl Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VimContext")
            .field("mode", &self.mode)
            .field("buffer_edit", &self.buffer_edit)
            .field("buffer_command", &self.buffer_command)
            .field("buffer_search", &self.buffer_search)
            .finish()
    }
}

impl<'a> Context<'a> {
    pub fn new(
        buffer_edit: &'a mut String,
        buffer_command: &'a mut String,
        buffer_search: &'a mut String,
    ) -> Self {
        let buffer_edit = buffer_edit.into();
        let buffer_command = buffer_command.into();
        let buffer_search = buffer_search.into();

        Context {
            mode: Default::default(),
            buffer_edit,
            buffer_command,
            buffer_search,
            fn_command: Default::default(),
        }
    }

    pub fn with_command_callback<F>(mut self, f: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.fn_command = Some(Box::new(f));
        self
    }

    pub fn command(&mut self) {
        if let Some(ref mut f) = self.fn_command {
            f(self
                .buffer_command
                .lines()
                .skip(self.buffer_command.lines().count() - 1)
                .next()
                .unwrap());
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }

    pub fn buffer(&self, buffer: BufferId) -> &Buffer<'a> {
        match buffer {
            BufferId::Edit => &self.buffer_edit,
            BufferId::Command => &self.buffer_command,
            BufferId::Search => &self.buffer_search,
        }
    }

    pub fn buffer_mut(&mut self, buffer: BufferId) -> &mut Buffer<'a> {
        match buffer {
            BufferId::Edit => &mut self.buffer_edit,
            BufferId::Command => &mut self.buffer_command,
            BufferId::Search => &mut self.buffer_search,
        }
    }

    pub fn active_buffer_id(&self) -> BufferId {
        match self.mode {
            Mode::Normal | Mode::Insert => BufferId::Edit,
            Mode::Command(command_mode) => match command_mode {
                crate::mode::CommandMode::Command => BufferId::Command,
                crate::mode::CommandMode::Search => BufferId::Search,
            },
        }
    }

    pub fn active_buffer(&self) -> &Buffer<'a> {
        self.buffer(self.active_buffer_id())
    }

    pub fn active_buffer_mut(&mut self) -> &mut Buffer<'a> {
        self.buffer_mut(self.active_buffer_id())
    }

    pub fn block_newline(&self) -> bool {
        match self.mode {
            Mode::Normal => true,
            Mode::Insert => false,
            Mode::Command(_) => false,
        }
    }

    pub fn input_str<'i>(&mut self, input: &'i str) -> Result<'i> {
        let (_, command) = command(input)?;
        self.input_command(command)
    }

    fn input_command<'i>(&mut self, command: Command<'i>) -> Result<'i> {
        match command {
            Command::Context(c) => match c {
                ContextCommand::RunCommand => {
                    self.command();
                    return Ok(());
                }
                ContextCommand::ChangeMode(mode) => {
                    self.set_mode(mode);
                    return Ok(());
                }
            },
            Command::Buffer(c) => {
                let block_newline = self.block_newline();
                let buffer = self.active_buffer_mut();
                match c {
                    BufferCommand::Motion(m) => buffer.motion(m, block_newline),
                    BufferCommand::Insert(s) => buffer.insert(s, block_newline),
                    BufferCommand::Operator(o) => buffer.operator(o),
                };
                return Ok(());
            }
            Command::Raw(input) => match self.mode {
                Mode::Normal => self.input_command(normal_command(input)?.1),
                Mode::Insert => self.input_command(insert_command(input)?.1),
                Mode::Command(_) => self.input_command(command_command(input)?.1),
            },
            Command::Multi(m) => {
                for command in m {
                    self.input_command(command)?;
                }
                return Ok(());
            }
        }
    }
}
