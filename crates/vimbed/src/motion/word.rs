use std::num::NonZeroUsize;

use crate::buffer::Buffer;

use super::MotionTrait;

#[derive(Debug, Copy, Clone)]
pub enum WordMotion {
    Forward(NonZeroUsize),
    Backward(NonZeroUsize),
}

impl WordMotion {
    pub fn forward(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(WordMotion::Forward)
    }

    pub fn backward(count: usize) -> Option<Self> {
        NonZeroUsize::new(count).map(WordMotion::Backward)
    }
}

impl MotionTrait for WordMotion {
    fn apply(&self, buffer: &mut Buffer, _block_newline: bool) {
        let word_offsets = buffer.word_offsets();

        let cursor_word_offset = buffer.cursor_word_offset();

        let next_word_idx = match self {
            WordMotion::Forward(dw) => cursor_word_offset
                .saturating_add(dw.get())
                .min(word_offsets.len().saturating_sub(1)),
            WordMotion::Backward(dw) => cursor_word_offset
                .saturating_sub(dw.get())
                .min(word_offsets.len().saturating_sub(2)),
        };

        let next_word_offset = word_offsets[next_word_idx];

        let (x, y) = buffer.offset_position(next_word_offset);

        buffer.cursor.column = x;
        buffer.cursor.row = y;

        buffer.cursor.target_column = buffer.cursor.column;
    }
}
