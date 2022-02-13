use std::io::{Stdout, Write};

use crossterm::{
    cursor::{self, CursorShape},
    terminal, QueueableCommand, Result as CrosstermResult,
};

use vimbed::{
    char_len::CharLen,
    context::{BufferId, Context},
    mode::{CommandMode, Mode},
};

pub fn render(
    stdout: &mut Stdout,
    ctx: &Context,
    input_buffer: &String,
    (width, height): (u16, u16),
) -> CrosstermResult<()> {
    // Clear
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;

    let mut last_line = 0;

    let buffer_edit = ctx.buffer(BufferId::Edit);
    let buffer_command = ctx.buffer(BufferId::Command);
    let buffer_search = ctx.buffer(BufferId::Search);

    // Draw text
    for (line, text) in buffer_edit.as_str().split('\n').enumerate() {
        stdout.queue(cursor::MoveTo(0, line as u16))?;
        let mut text = text.to_string();
        text.truncate(width as usize);
        write!(stdout, " {:4} {}", line + 1, text)?;
        last_line = line;
    }

    // Fill remaining buffer with markers
    for line in last_line.saturating_add(1)..height.saturating_sub(2) as usize {
        stdout.queue(cursor::MoveTo(0, line as u16))?;
        write!(stdout, "~")?;
    }

    // Draw status bar
    let left_status = format!("{} >", ctx.mode,);
    let right_status = format!(
        "< {:3} < {:3}% < {:3}:{:2}",
        buffer_edit.cursor_offset(),
        ((buffer_edit.cursor.row as f32 / (buffer_edit.lines().count() - 1) as f32) * 100.0)
            as usize,
        buffer_edit.cursor.row + 1,
        buffer_edit.cursor.column + 1,
    );

    stdout.queue(cursor::MoveTo(0, height - 2))?;
    write!(stdout, "{}", left_status)?;

    stdout.queue(cursor::MoveTo(
        width - right_status.char_len() as u16,
        height - 2,
    ))?;
    write!(stdout, "{}", right_status)?;

    // Draw command bar
    stdout.queue(cursor::MoveTo(0, height - 1))?;
    write!(
        stdout,
        "{}{}",
        match ctx.mode {
            Mode::Command(CommandMode::Command) => ":",
            Mode::Command(CommandMode::Search) => "/",
            _ => "",
        },
        match ctx.mode {
            Mode::Command(CommandMode::Command) => buffer_command.cursor_line(),
            Mode::Command(CommandMode::Search) => buffer_search.cursor_line(),
            _ => "",
        }
    )?;

    stdout.queue(cursor::MoveTo(
        width - input_buffer.char_len() as u16 - 9,
        height - 1,
    ))?;
    write!(stdout, "{}", input_buffer)?;

    // Move cursor to context position and set shape
    match ctx.mode {
        Mode::Command(sub_mode) => {
            let buffer = match sub_mode {
                CommandMode::Command => ctx.buffer(BufferId::Command),
                CommandMode::Search => ctx.buffer(BufferId::Search),
            };

            stdout.queue(cursor::MoveTo(1 + buffer.cursor.column as u16, height - 1))?;

            stdout.queue(cursor::SetCursorShape(
                if buffer.cursor.column == buffer.lines().last().unwrap_or("").char_len() {
                    CursorShape::Block
                } else {
                    CursorShape::Line
                },
            ))?;
        }
        _ => {
            stdout.queue(cursor::MoveTo(
                6 + buffer_edit.cursor.column as u16,
                buffer_edit.cursor.row as u16,
            ))?;

            stdout.queue(cursor::SetCursorShape(match ctx.mode {
                Mode::Insert => CursorShape::Line,
                _ => CursorShape::Block,
            }))?;
        }
    }

    // Flush commands
    stdout.flush()
}
