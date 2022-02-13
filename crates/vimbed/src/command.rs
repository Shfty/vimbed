use std::str::FromStr;

use nom::IResult;

use crate::{
    mode::Mode,
    motion::{
        character::CharacterMotion, left_right::LeftRightMotion, up_down::UpDownMotion, Motion,
        MotionVariant,
    },
    operator::{Operator, OperatorVariant},
};

use super::motion::motion;

pub const COMMAND_LEFT: &'static str = "<Left>";
pub const COMMAND_DOWN: &'static str = "<Down>";
pub const COMMAND_UP: &'static str = "<Up>";
pub const COMMAND_RIGHT: &'static str = "<Right>";
pub const COMMAND_HOME: &'static str = "<Home>";
pub const COMMAND_END: &'static str = "<End>";
pub const COMMAND_CARRIAGE_RETURN: &'static str = "<CR>";
pub const COMMAND_ESCAPE: &'static str = "<ESC>";
pub const COMMAND_BACKSPACE: &'static str = "<BS>";
pub const COMMAND_LEADER: &'static str = "<Leader>";
pub const COMMAND_DELETE: &'static str = "<Delete>";

#[derive(Debug, Clone)]
pub enum ContextCommand {
    ChangeMode(Mode),
    RunCommand,
}

impl From<Mode> for ContextCommand {
    fn from(m: Mode) -> Self {
        ContextCommand::ChangeMode(m)
    }
}

#[derive(Debug, Clone)]
pub enum BufferCommand<'a> {
    Insert(&'a str),
    Motion(Motion),
    Operator(Operator),
}

impl<'a> From<&'a str> for BufferCommand<'a> {
    fn from(s: &'a str) -> Self {
        BufferCommand::Insert(s)
    }
}

impl From<Motion> for BufferCommand<'_> {
    fn from(m: Motion) -> Self {
        BufferCommand::Motion(m)
    }
}

impl From<MotionVariant> for BufferCommand<'_> {
    fn from(v: MotionVariant) -> Self {
        BufferCommand::from(Motion::new_one(v))
    }
}

impl From<Operator> for BufferCommand<'_> {
    fn from(o: Operator) -> Self {
        BufferCommand::Operator(o)
    }
}

// Top-level input
#[derive(Debug, Clone)]
pub enum Command<'a> {
    Context(ContextCommand),
    Buffer(BufferCommand<'a>),
    Raw(&'a str),
    Multi(Vec<Command<'a>>),
}

impl<'a> From<BufferCommand<'a>> for Command<'a> {
    fn from(c: BufferCommand<'a>) -> Self {
        Command::Buffer(c)
    }
}

impl From<ContextCommand> for Command<'_> {
    fn from(c: ContextCommand) -> Self {
        Command::Context(c)
    }
}

impl<'a> From<&'a str> for Command<'a> {
    fn from(s: &'a str) -> Self {
        Command::Raw(s)
    }
}

impl<'a> From<Vec<Command<'a>>> for Command<'a> {
    fn from(v: Vec<Command<'a>>) -> Self {
        Command::Multi(v)
    }
}

impl Command<'_> {
    pub fn motion(repeat: usize, variant: MotionVariant) -> Self {
        BufferCommand::Motion(Motion::new(repeat, variant)).into()
    }

    pub fn operator(repeat: usize, variant: OperatorVariant, motion: Motion) -> Self {
        BufferCommand::Operator(Operator {
            repeat,
            variant,
            motion,
        })
        .into()
    }
}

// nom parsers
pub fn command_insert(input: &str) -> IResult<&str, Command> {
    let (input, output) = nom::bytes::complete::take_till1(|input| input == '<')(input)?;
    Ok((input, BufferCommand::Insert(output).into()))
}

pub fn command_raw(input: &str) -> IResult<&str, Command> {
    let (input, output) = nom::bytes::complete::take_till1(|input| input == '<')(input)?;
    Ok((input, output.into()))
}

pub fn command_variant<'a, 'b, V>(
    tag: &'a str,
    variant: V,
) -> impl FnMut(&str) -> IResult<&str, Command<'b>> + 'a
where
    V: Clone + Into<Command<'b>> + 'a,
{
    move |input| {
        let (input, _) = nom::bytes::streaming::tag(tag)(input)?;
        Ok((input, variant.clone().into()))
    }
}

pub fn command_motion(input: &str) -> IResult<&str, Command> {
    let (input, output) = nom::branch::alt((
        motion(COMMAND_LEFT, LeftRightMotion::left(1).unwrap()),
        motion(COMMAND_DOWN, UpDownMotion::down(1).unwrap()),
        motion(COMMAND_UP, UpDownMotion::up(1).unwrap()),
        motion(COMMAND_RIGHT, LeftRightMotion::right(1).unwrap()),
        motion(COMMAND_HOME, LeftRightMotion::FirstCharacter),
        motion(COMMAND_END, LeftRightMotion::LastCharacter),
    ))(input)?;

    Ok((input, BufferCommand::from(output).into()))
}

/// Matches angle bracket delimited commands with no corresponding BaseCommand
fn command_special_raw(input: &str) -> IResult<&str, Command> {
    let (input, output) = nom::combinator::recognize(nom::sequence::delimited(
        nom::bytes::complete::tag("<"),
        nom::character::complete::alphanumeric0,
        nom::bytes::complete::tag(">"),
    ))(input)?;

    Ok((input, output.into()))
}

pub fn command(input: &str) -> IResult<&str, Command> {
    nom::branch::alt((command_special_raw, command_raw))(input)
}

pub fn command_delete(input: &str) -> IResult<&str, Command> {
    command_variant(
        COMMAND_DELETE,
        Command::operator(
            1,
            OperatorVariant::Delete,
            Motion::new_one(CharacterMotion::forward(1).unwrap().into()),
        ),
    )(input)
}
