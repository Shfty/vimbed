use std::num::NonZeroUsize;

use nom::IResult;

use crate::{
    command::{
        command_delete, command_variant, BufferCommand, Command, ContextCommand, COMMAND_BACKSPACE,
        COMMAND_CARRIAGE_RETURN,
    },
    motion::{
        character::CharacterMotion, left_right::LeftRightMotion, motion, up_down::UpDownMotion,
        word::WordMotion, Motion, MotionVariant,
    },
    operator::{operator, Operator, OperatorVariant},
};

use super::{CommandMode, Mode};

fn normal_motion(input: &str) -> IResult<&str, Motion> {
    nom::branch::alt((
        motion(COMMAND_BACKSPACE, CharacterMotion::backward(1).unwrap()),
        motion(COMMAND_CARRIAGE_RETURN, UpDownMotion::down(1).unwrap()),
        motion("h", LeftRightMotion::Left(NonZeroUsize::new(1).unwrap())),
        motion("j", UpDownMotion::down(1).unwrap()),
        motion("k", UpDownMotion::up(1).unwrap()),
        motion("l", LeftRightMotion::Right(NonZeroUsize::new(1).unwrap())),
        motion("0", LeftRightMotion::FirstCharacter),
        motion("^", LeftRightMotion::FirstNonBlankCharacter),
        motion("$", LeftRightMotion::LastCharacter),
        motion("gg", UpDownMotion::FirstLine),
        motion("G", UpDownMotion::LastLine),
        motion("w", WordMotion::forward(1).unwrap()),
        motion("b", WordMotion::backward(1).unwrap()),
    ))(input)
}

// Normal mode input
fn normal_command_motion(input: &str) -> IResult<&str, Command> {
    let (input, motion) = normal_motion(input)?;
    Ok((input, BufferCommand::from(motion).into()))
}

fn normal_operator(input: &str) -> IResult<&str, Operator> {
    nom::branch::alt((operator("d", OperatorVariant::Delete, normal_motion),))(input)
}

fn normal_command_operator(input: &str) -> IResult<&str, Command> {
    let (input, operator) = normal_operator(input)?;
    Ok((input, BufferCommand::from(operator).into()))
}

pub fn normal_command(input: &str) -> IResult<&str, Command> {
    nom::branch::alt((
        command_variant("i", ContextCommand::from(Mode::Insert)),
        command_variant(
            "a",
            Command::from(vec![
                ContextCommand::from(Mode::Insert).into(),
                BufferCommand::from(MotionVariant::from(LeftRightMotion::right(1).unwrap())).into(),
            ]),
        ),
        command_variant(
            "o",
            Command::from(vec![
                ContextCommand::from(Mode::Insert).into(),
                BufferCommand::from(MotionVariant::from(LeftRightMotion::LastCharacter)).into(),
                BufferCommand::Insert("\n").into(),
            ]),
        ),
        command_variant(
            "O",
            Command::from(vec![
                ContextCommand::from(Mode::Insert).into(),
                BufferCommand::from(MotionVariant::from(LeftRightMotion::FirstCharacter)).into(),
                BufferCommand::Insert("\n").into(),
                BufferCommand::from(MotionVariant::from(UpDownMotion::up(1).unwrap())).into(),
            ]),
        ),
        command_variant(
            "s",
            Command::from(vec![
                Command::operator(
                    1,
                    OperatorVariant::Delete,
                    Motion::new_one(CharacterMotion::forward(1).unwrap().into()),
                ),
                ContextCommand::from(Mode::Insert).into(),
            ]),
        ),
        command_variant(
            "S",
            Command::from(vec![
                Command::motion(1, LeftRightMotion::FirstNonBlankCharacter.into()),
                Command::operator(
                    1,
                    OperatorVariant::Delete,
                    Motion::new_one(LeftRightMotion::LastCharacter.into()),
                ),
                ContextCommand::from(Mode::Insert).into(),
            ]),
        ),
        command_variant(
            ":",
            ContextCommand::from(Mode::Command(CommandMode::Command)),
        ),
        command_variant(
            "/",
            ContextCommand::from(Mode::Command(CommandMode::Search)),
        ),
        command_variant(
            "x",
            Command::operator(
                1,
                OperatorVariant::Delete,
                Motion::new_one(CharacterMotion::forward(1).unwrap().into()),
            ),
        ),
        command_variant(
            "dd",
            Command::from(vec![
                Command::motion(1, LeftRightMotion::FirstCharacter.into()),
                Command::operator(
                    1,
                    OperatorVariant::Delete,
                    Motion::new_one(UpDownMotion::down(1).unwrap().into()),
                ),
            ]),
        ),
        command_variant(
            " ",
            BufferCommand::from(MotionVariant::from(CharacterMotion::forward(1).unwrap())),
        ),
        normal_command_motion,
        normal_command_operator,
        command_delete,
    ))(input)
}
