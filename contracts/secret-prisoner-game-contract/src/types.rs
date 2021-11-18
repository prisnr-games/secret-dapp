use cosmwasm_std::{
    StdError, StdResult, 
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GameStage {
    WaitingForSecondPlayer,
    Ongoing,
    Finished,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum RoundStage {
    Initialized,
    OnePlayerFirstSubmit,
    BothPlayersFirstSubmit,
    OnePlayerSecondSubmit,
    BothPlayersSecondSubmit,
    OnePlayerGuess,
    Finished,
}

impl RoundStage {
    pub fn u8_val(&self) -> u8 {
        match self {
            RoundStage::Initialized => 0_u8,
            RoundStage::OnePlayerFirstSubmit => 1_u8,
            RoundStage::BothPlayersFirstSubmit => 2_u8,
            RoundStage::OnePlayerSecondSubmit => 3_u8,
            RoundStage::BothPlayersSecondSubmit => 4_u8,
            RoundStage::OnePlayerGuess => 5_u8,
            RoundStage::Finished => 6_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<RoundStage> {
        match val {
            0_u8 => Ok(RoundStage::Initialized),
            1_u8 => Ok(RoundStage::OnePlayerFirstSubmit),
            2_u8 => Ok(RoundStage::BothPlayersFirstSubmit),
            3_u8 => Ok(RoundStage::OnePlayerSecondSubmit),
            4_u8 => Ok(RoundStage::BothPlayersSecondSubmit),
            5_u8 => Ok(RoundStage::OnePlayerGuess),
            6_u8 => Ok(RoundStage::Finished),
            _ => Err(StdError::generic_err("Invalid round stage value")),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum RoundResult {
    BagCorrect,
    BagWrong,
    OpponentCorrect,
    OpponentWrong,
    Abstain,
}

impl RoundResult {
    pub fn u8_val(&self) -> u8 {
        match self {
            RoundResult::BagCorrect => 0_u8,
            RoundResult::BagWrong => 1_u8,
            RoundResult::OpponentCorrect => 2_u8,
            RoundResult::OpponentWrong => 3_u8,
            RoundResult::Abstain => 4_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<RoundResult> {
        match val {
            0_u8 => Ok(RoundResult::BagCorrect),
            1_u8 => Ok(RoundResult::BagWrong),
            2_u8 => Ok(RoundResult::OpponentCorrect),
            3_u8 => Ok(RoundResult::OpponentWrong),
            4_u8 => Ok(RoundResult::Abstain),
            _ => Err(StdError::generic_err("Invalid round result value")),
        }
    }
}

#[derive(Hash, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Color {
    Red,
    Green,
    Blue,
    Black,
}

#[derive(Hash, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Shape {
    Triangle,
    Square,
    Circle,
    Star,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Chip {
    pub color: Color,
    pub shape: Shape,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Hint {
    BagNotRed,
    BagNotGreen,
    BagNotBlue,
    BagNotBlack,
    BagNotTriangle,
    BagNotSquare,
    BagNotCircle,
    BagNotStar,
    IHaveRed,
    IHaveGreen,
    IHaveBlue,
    IHaveBlack,
    IHaveTriangle,
    IHaveSquare,
    IHaveCircle,
    IHaveStar,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Target {
    Bag,
    Opponent,
    Abstain,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Guess {
    pub target: Target,
    pub color: Option<Color>,
    pub shape: Option<Shape>,
}