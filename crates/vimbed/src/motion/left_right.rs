use std::num::NonZeroUsize;

use crate::{buffer::Buffer, char_len::CharLen};

use super::{Motion, MotionTrait, WordMotion};

#[derive(Debug, Copy, Clone)]
pub enum LeftRightMotion {
    FirstCharacter,
    FirstNonBlankCharacter,
    LastCharacter,
    Left(NonZeroUsize),
    Right(NonZeroUsize),
}

impl LeftRightMotion {
    pub fn left(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(LeftRightMotion::Left)
    }

    pub fn right(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(LeftRightMotion::Right)
    }
}

impl MotionTrait for LeftRightMotion {
    fn apply(&self, buffer: &mut Buffer, block_newline: bool) {
        let line_end = buffer.cursor_line().char_len();
        let line_end_offset = line_end.saturating_sub(if block_newline { 1 } else { 0 });
        match self {
            LeftRightMotion::FirstCharacter => buffer.cursor.column = 0,
            LeftRightMotion::FirstNonBlankCharacter => {
                buffer.motion(
                    Motion::new_one(LeftRightMotion::FirstCharacter.into()),
                    block_newline,
                );
                buffer.motion(
                    Motion::new_one(WordMotion::forward(1).unwrap().into()),
                    block_newline,
                );
            }
            LeftRightMotion::LastCharacter => buffer.cursor.column = line_end_offset,
            LeftRightMotion::Left(dx) => {
                if buffer.cursor.column > 0 {
                    buffer.cursor.column = buffer.cursor.column.saturating_sub(dx.get());
                }
            }
            LeftRightMotion::Right(dx) => {
                if buffer.cursor.column < line_end_offset {
                    buffer.cursor.column = buffer
                        .cursor
                        .column
                        .saturating_add(dx.get())
                        .min(line_end_offset);
                }
            }
        }

        buffer.cursor.target_column = buffer.cursor.column;
    }
}
