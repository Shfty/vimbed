use crate::{buffer::Buffer, motion::Motion};

pub fn operator_delete(buffer: &mut Buffer, motion: Motion) {
    let from = buffer.cursor_offset();
    buffer.motion(motion, false);
    let to = buffer.cursor_offset();

    let (from, to) = (from.min(to), from.max(to));
    buffer.replace_range(from..to, "");

    let (x, y) = buffer.offset_position(from);
    buffer.cursor.column = x;
    buffer.cursor.row = y;
}
