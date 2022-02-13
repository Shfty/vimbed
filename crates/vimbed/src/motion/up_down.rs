use std::num::NonZeroUsize;

use crate::{buffer::Buffer, char_len::CharLen};

use super::MotionTrait;

#[derive(Debug, Copy, Clone)]
pub enum UpDownMotion {
    FirstLine,
    LastLine,
    Up(NonZeroUsize),
    Down(NonZeroUsize),
}

impl UpDownMotion {
    pub fn up(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(UpDownMotion::Up)
    }

    pub fn down(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(UpDownMotion::Down)
    }
}

impl MotionTrait for UpDownMotion {
    fn apply(&self, buffer: &mut Buffer, block_newline: bool) {
        match self {
            UpDownMotion::FirstLine => buffer.cursor.row = 0,
            UpDownMotion::LastLine => buffer.cursor.row = buffer.lines().count() - 1,
            UpDownMotion::Up(dy) => {
                if buffer.cursor.row > 0 {
                    buffer.cursor.row = buffer.cursor.row.saturating_sub(dy.get());
                }
            }
            UpDownMotion::Down(dy) => {
                let row_end = buffer.lines().count().saturating_sub(1);
                if buffer.cursor.row < row_end {
                    buffer.cursor.row = buffer.cursor.row.saturating_add(dy.get()).min(row_end);
                }
            }
        }

        let line_end = buffer.cursor_line().char_len();
        let line_end_offset = line_end.saturating_sub(if block_newline { 1 } else { 0 });

        if buffer.cursor.column > line_end_offset {
            buffer.cursor.column = line_end_offset;
        }

        let target_column = buffer.cursor.target_column.min(line_end_offset);
        if buffer.cursor.column < target_column {
            buffer.cursor.column = target_column
        }
    }
}
