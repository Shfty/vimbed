use std::ops::{Deref, DerefMut};

use crate::{
    char_len::CharLen,
    motion::{left_right::LeftRightMotion, up_down::UpDownMotion, Motion, MotionTrait},
    operator::*,
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor {
    pub column: usize,
    pub row: usize,
    pub target_column: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer<'a> {
    pub buffer: &'a mut String,
    pub cursor: Cursor,
}

impl<'a> Deref for Buffer<'a> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a> DerefMut for Buffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl<'a> From<&'a mut String> for Buffer<'a> {
    fn from(buffer: &'a mut String) -> Self {
        Buffer {
            buffer,
            cursor: Default::default(),
        }
    }
}

impl<'a> Buffer<'a> {
    fn lines(&self) -> impl Iterator<Item = &str> + Clone {
        self.split('\n')
    }

    fn line(&self, line: usize) -> Option<&str> {
        self.lines().skip(line).next()
    }

    fn words(&self) -> impl Iterator<Item = &str> + Clone {
        self.split_inclusive(' ')
            .flat_map(|w| w.split_inclusive('\n'))
    }

    pub fn word_offsets(&self) -> Vec<usize> {
        let mut word_offsets = vec![];
        let mut word_head = 0usize;
        for word in self.words() {
            if !word.starts_with(' ') && (!word.starts_with('\n') || word == "\n") {
                word_offsets.push(word_head);
            }
            word_head += word.char_len();
        }
        word_offsets
    }

    pub fn cursor_word_offset(&self) -> usize {
        let word_offsets = self.word_offsets();
        word_offsets
            .iter()
            .position(|i| *i > self.cursor_offset())
            .unwrap_or_else(|| *word_offsets.last().unwrap())
            .saturating_sub(1)
    }

    pub fn cursor_line(&self) -> &str {
        self.line(self.cursor.row).unwrap()
    }

    pub fn cursor_offset(&self) -> usize {
        let chars_before = self
            .lines()
            .take(self.cursor.row)
            .map(|line| line.char_len() + 1)
            .sum::<usize>();
        let chars_current = self.cursor_line().char_len();
        chars_before + self.cursor.column.min(chars_current)
    }

    pub fn offset_position(&self, offset: usize) -> (usize, usize) {
        let y = self[0..offset].chars().filter(|b| *b == '\n').count();

        let x = self[0..offset].split('\n').last().unwrap().char_len();

        (x, y)
    }

    pub fn insert(&mut self, text: &str, block_newline: bool) {
        let ofs = self.cursor_offset();
        let (buffer_l, buffer_r) = self.buffer.split_at(ofs);
        *self.buffer = buffer_l.to_owned() + text + buffer_r;

        if text == "\n" {
            self.motion(
                Motion::new_one(UpDownMotion::down(1).unwrap().into()),
                block_newline,
            );
            self.motion(
                Motion::new_one(LeftRightMotion::FirstCharacter.into()),
                block_newline,
            );
        } else if let Some(motion) = LeftRightMotion::right(text.char_len()) {
            self.motion(Motion::new_one(motion.into()), block_newline);
        }
    }

    pub fn motion<M>(&mut self, motion: M, block_newline: bool)
    where
        M: Into<Motion>,
    {
        let motion = motion.into();
        motion.apply(self, block_newline);
    }

    pub fn operator<O>(&mut self, operator: O)
    where
        O: Into<Operator>,
    {
        let operator = operator.into();
        operator.apply(self)
    }
}
