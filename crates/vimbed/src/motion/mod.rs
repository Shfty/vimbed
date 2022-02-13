pub mod character;
pub mod left_right;
pub mod up_down;
pub mod word;

use character::CharacterMotion;
use left_right::LeftRightMotion;
use up_down::UpDownMotion;
use word::WordMotion;

use nom::IResult;

use crate::buffer::Buffer;

/// Trait for applying a motion to a buffer
/// TODO: This should probably apply to a new Cursor abstraction
pub trait MotionTrait {
    fn apply(&self, buffer: &mut Buffer, block_newline: bool);
}

/// Repeatable motion
#[derive(Debug, Copy, Clone)]
pub struct Motion {
    repeat: usize,
    variant: MotionVariant,
}

// Motions
pub fn motion<'a, M>(tag: &'a str, variant: M) -> impl FnMut(&str) -> IResult<&str, Motion> + 'a
where
    M: Copy + Into<MotionVariant> + 'a,
{
    let variant = variant.into();
    move |input| {
        let (input, repeat) = nom::character::complete::digit0(input)?;
        let repeat = repeat.parse::<usize>().unwrap_or(1);

        let (input, _) = nom::bytes::streaming::tag(tag)(input)?;
        Ok((input, Motion::new(repeat, variant)))
    }
}

impl Motion {
    pub fn new(repeat: usize, variant: MotionVariant) -> Self {
        Motion { repeat, variant }
    }

    pub fn new_one(variant: MotionVariant) -> Self {
        Self::new(1, variant)
    }
}

impl MotionTrait for Motion {
    fn apply(&self, buffer: &mut Buffer, block_newline: bool) {
        for _ in 0..self.repeat {
            self.variant.apply(buffer, block_newline);
        }
    }
}

/// Closed set of built-in motions
#[derive(Debug, Copy, Clone)]
pub enum MotionVariant {
    LeftRight(LeftRightMotion),
    UpDown(UpDownMotion),
    Word(WordMotion),
    Character(CharacterMotion),
    //TextObject(TextObjectMotion),
}

impl From<LeftRightMotion> for MotionVariant {
    fn from(m: LeftRightMotion) -> Self {
        MotionVariant::LeftRight(m)
    }
}

impl From<UpDownMotion> for MotionVariant {
    fn from(m: UpDownMotion) -> Self {
        MotionVariant::UpDown(m)
    }
}

impl From<WordMotion> for MotionVariant {
    fn from(m: WordMotion) -> Self {
        MotionVariant::Word(m)
    }
}

impl From<CharacterMotion> for MotionVariant {
    fn from(m: CharacterMotion) -> Self {
        MotionVariant::Character(m)
    }
}

impl MotionTrait for MotionVariant {
    fn apply(&self, buffer: &mut Buffer, block_newline: bool) {
        match self {
            MotionVariant::LeftRight(motion) => motion.apply(buffer, block_newline),
            MotionVariant::UpDown(motion) => motion.apply(buffer, block_newline),
            MotionVariant::Word(motion) => motion.apply(buffer, block_newline),
            MotionVariant::Character(motion) => motion.apply(buffer, block_newline),
        }
    }
}
