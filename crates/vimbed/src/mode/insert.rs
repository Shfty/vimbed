use nom::IResult;

use crate::{
    command::{
        command_insert, command_variant, BufferCommand, Command, ContextCommand, COMMAND_BACKSPACE,
        COMMAND_ESCAPE,
    },
    motion::{character::CharacterMotion, left_right::LeftRightMotion, Motion, MotionVariant},
    operator::OperatorVariant,
};

use super::Mode;

pub fn insert_command(input: &str) -> IResult<&str, Command> {
    nom::branch::alt((
        command_variant(
            COMMAND_ESCAPE,
            Command::Multi(vec![
                BufferCommand::from(MotionVariant::from(LeftRightMotion::left(1).unwrap())).into(),
                ContextCommand::from(Mode::Normal).into(),
            ]),
        ),
        command_variant(
            COMMAND_BACKSPACE,
            Command::Multi(vec![
                BufferCommand::from(MotionVariant::from(CharacterMotion::backward(1).unwrap()))
                    .into(),
                Command::operator(
                    1,
                    OperatorVariant::Delete,
                    Motion::new_one(CharacterMotion::forward(1).unwrap().into()),
                ),
            ]),
        ),
        command_insert,
    ))(input)
}
