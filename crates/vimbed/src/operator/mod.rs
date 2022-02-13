pub mod delete;

use delete::operator_delete;

use nom::IResult;

use crate::motion::Motion;

/// A type that can apply an operator to a buffer
pub trait OperatorTrait {
    fn apply(&self, buffer: &mut crate::buffer::Buffer);
}

/// Repeatable operator
#[derive(Debug, Clone)]
pub struct Operator {
    pub repeat: usize,
    pub variant: OperatorVariant,
    pub motion: Motion,
}

impl Operator {
    pub fn new(repeat: usize, variant: OperatorVariant, motion: Motion) -> Self {
        Operator {
            repeat,
            variant,
            motion,
        }
    }

    pub fn new_one(variant: OperatorVariant, motion: Motion) -> Self {
        Self::new(1, variant, motion)
    }
}

impl OperatorTrait for Operator {
    fn apply(&self, buffer: &mut crate::buffer::Buffer) {
        match self.variant {
            OperatorVariant::Delete => operator_delete(buffer, self.motion),
            _ => (),
        }
    }
}

/// Closed set of built-in operators
#[derive(Debug, Copy, Clone)]
pub enum OperatorVariant {
    Change,
    Delete,
    Yank,
    SwapCase,
    MakeLowercase,
    MakeUppercase,
    //FilterExternal
    //FilterEqualPrg
    //TextFormatting
    //TextFormattingNoMove
    //Rot13
    ShiftRight,
    ShiftLeft,
    //DefineFold
    //OperatorFunc
}

/// Creates a nom parser from a given string tag to the specified operator and motion
pub fn operator<'a, O>(
    tag: &'a str,
    variant: O,
    mut motion: impl FnMut(&str) -> IResult<&str, Motion> + 'a,
) -> impl FnMut(&str) -> IResult<&str, Operator> + 'a
where
    O: Copy + Into<OperatorVariant> + 'a,
{
    let variant = variant.into();
    move |input| {
        let (input, repeat) = nom::character::complete::digit0(input)?;
        let repeat = repeat.parse::<usize>().unwrap_or(1);

        let (input, _) = nom::bytes::streaming::tag(tag)(input)?;

        let (input, motion) = motion(input)?;

        Ok((input, Operator::new(repeat, variant, motion)))
    }
}
