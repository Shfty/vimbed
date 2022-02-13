use std::num::NonZeroUsize;

use crate::{buffer::Buffer, char_len::CharLen};

use super::MotionTrait;

#[derive(Debug, Copy, Clone)]
pub enum CharacterMotion {
    Forward(NonZeroUsize),
    Backward(NonZeroUsize),
    StartOfBuffer,
    EndOfBuffer,
}

impl CharacterMotion {
    pub fn forward(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(CharacterMotion::Forward)
    }

    pub fn backward(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(CharacterMotion::Backward)
    }
}

impl MotionTrait for CharacterMotion {
    fn apply(&self, buffer: &mut Buffer, block_newline: bool) {
        let cursor_offset = buffer.cursor_offset();
        let next_offset = match self {
            CharacterMotion::Forward(dc) => {
                let mut ofs = cursor_offset.saturating_add(dc.get());
                if buffer.cursor.column == buffer.cursor_line().char_len().saturating_sub(1)
                    && block_newline
                {
                    ofs = ofs.saturating_add(1);
                }

                ofs.min(buffer.char_len().saturating_sub(1))
            }
            CharacterMotion::Backward(dc) => {
                let mut ofs = cursor_offset.saturating_sub(dc.get());

                if buffer.cursor.column == 0 && block_newline {
                    ofs = ofs.saturating_sub(1);
                }

                ofs.min(buffer.char_len().saturating_sub(1))
            }
            CharacterMotion::StartOfBuffer => 0,
            CharacterMotion::EndOfBuffer => buffer.buffer.char_len(),
        };

        let (x, y) = buffer.offset_position(next_offset);

        buffer.cursor.column = x;
        buffer.cursor.row = y;

        buffer.cursor.target_column = buffer.cursor.column;
    }
}
