use cosmwasm_std::{
    StdError, StdResult, 
};
use serde::{Deserialize, Serialize};

pub const RED: u8 = 0b10000000u8;
pub const GREEN: u8 = 0b01000000u8;
pub const BLUE: u8 = 0b00100000u8;
pub const BLACK: u8 = 0b00010000u8;
pub const TRIANGLE: u8 = 0b00001000u8;
pub const CIRCLE: u8 = 0b00000100u8;
pub const SQUARE: u8 = 0b00000010u8;
pub const STAR: u8 = 0b00000001u8;

pub const REWARD_NFT: u8 = 1;
pub const REWARD_POOL: u8 = 2;

pub const POWERUP_INSURANCE: u16 = 1;

/*
pub fn is_bitmask_color(mask: u8) -> bool {
    mask & 0xf0 > 0
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GameStage {
    WaitingForSecondPlayer,
    Ongoing,
    Finished,
}

impl GameStage {
    pub fn u8_val(&self) -> u8 {
        match self {
            GameStage::WaitingForSecondPlayer => 0_u8,
            GameStage::Ongoing => 1_u8,
            GameStage::Finished => 2_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<GameStage> {
        match val {
            0_u8 => Ok(GameStage::WaitingForSecondPlayer),
            1_u8 => Ok(GameStage::Ongoing),
            2_u8 => Ok(GameStage::Finished),
            _ => Err(StdError::generic_err("Invalid game stage value")),
        }
    }
}
*/

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GameResult {
    AWon,
    BWon,
    BothLose,
    AJackpotBNft,
    ANftBJackpot,
    NoReward,
}

impl GameResult {
    pub fn u8_val(&self) -> u8 {
        match self {
            GameResult::AWon => 0_u8,
            GameResult::BWon => 1_u8,
            GameResult::BothLose => 2_u8,
            GameResult::AJackpotBNft => 3_u8,
            GameResult::ANftBJackpot => 4_u8,
            GameResult::NoReward => 5_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<GameResult> {
        match val {
            0_u8 => Ok(GameResult::AWon),
            1_u8 => Ok(GameResult::BWon),
            2_u8 => Ok(GameResult::BothLose),
            3_u8 => Ok(GameResult::AJackpotBNft),
            4_u8 => Ok(GameResult::ANftBJackpot),
            5_u8 => Ok(GameResult::NoReward),
            _ => Err(StdError::generic_err("Invalid game result value")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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

#[derive(Debug, Hash, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Color {
    Red,
    Green,
    Blue,
    Black,
}

impl Color {
    pub fn u8_val(&self) -> u8 {
        match self {
            Color::Red => 0_u8,
            Color::Green => 1_u8,
            Color::Blue => 2_u8,
            Color::Black => 3_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<Color> {
        match val {
            0_u8 => Ok(Color::Red),
            1_u8 => Ok(Color::Green),
            2_u8 => Ok(Color::Blue),
            3_u8 => Ok(Color::Black),
            _ => Err(StdError::generic_err("Invalid color value")),
        }
    }

    pub fn to_bitmask(&self) -> u8 {
        match self {
            Color::Red => RED,
            Color::Green => GREEN,
            Color::Blue => BLUE,
            Color::Black => BLACK,
        }
    }

    pub fn from_bitmask(mask: u8) -> StdResult<Color> {
        match mask {
            RED => Ok(Color::Red),
            GREEN => Ok(Color::Green),
            BLUE => Ok(Color::Blue),
            BLACK => Ok(Color::Black),
            _ => { return Err(StdError::generic_err("Invalid color bitmask")); }
        }
    }
}

#[derive(Debug, Hash, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Shape {
    Triangle,
    Square,
    Circle,
    Star,
}

impl Shape {
    pub fn u8_val(&self) -> u8 {
        match self {
            Shape::Triangle => 0_u8,
            Shape::Square => 1_u8,
            Shape::Circle => 2_u8,
            Shape::Star => 3_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<Shape> {
        match val {
            0_u8 => Ok(Shape::Triangle),
            1_u8 => Ok(Shape::Square),
            2_u8 => Ok(Shape::Circle),
            3_u8 => Ok(Shape::Star),
            _ => Err(StdError::generic_err("Invalid shape value")),
        }
    }

    pub fn to_bitmask(&self) -> u8 {
        match self {
            Shape::Triangle => TRIANGLE,
            Shape::Square => SQUARE,
            Shape::Circle => CIRCLE,
            Shape::Star => STAR,
        }
    }

    pub fn from_bitmask(mask: u8) -> StdResult<Shape> {
        match mask {
            TRIANGLE => Ok(Shape::Triangle),
            SQUARE => Ok(Shape::Square),
            CIRCLE => Ok(Shape::Circle),
            STAR => Ok(Shape::Star),
            _ => Err(StdError::generic_err("Invalid shape value")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chip {
    pub color: Color,
    pub shape: Shape,
}

impl Chip {
    pub fn to_stored(&self) -> StoredChip {
        StoredChip {
            color: self.color.u8_val(),
            shape: self.shape.u8_val(),
        }
    }

    pub fn to_bitmask(&self) -> u8 {
        let mut mask: u8 = 0;
        match self.color {
            Color::Red => mask |= RED,
            Color::Green => mask |= GREEN,
            Color::Blue => mask |= BLUE,
            Color::Black => mask |= BLACK,
        }
        match self.shape {
            Shape::Triangle => mask |= TRIANGLE,
            Shape::Square => mask |= SQUARE,
            Shape::Circle => mask |= CIRCLE,
            Shape::Star => mask |= STAR,
        }
        mask
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredChip {
    pub color: u8,
    pub shape: u8,
}

impl StoredChip {
    pub fn to_humanized(&self) -> StdResult<Chip> {
        let chip = Chip {
            color: Color::from_u8(self.color)?,
            shape: Shape::from_u8(self.shape)?,
        };
        Ok(chip)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Hint {
    NobodyHasRed,
    NobodyHasGreen,
    NobodyHasBlue,
    NobodyHasBlack,
    NobodyHasTriangle,
    NobodyHasSquare,
    NobodyHasCircle,
    NobodyHasStar,
    IHaveRed,
    IHaveGreen,
    IHaveBlue,
    IHaveBlack,
    IHaveTriangle,
    IHaveSquare,
    IHaveCircle,
    IHaveStar,
}

impl Hint {
    pub fn u8_val(&self) -> u8 {
        match self {
            Hint::NobodyHasRed => 0_u8,
            Hint::NobodyHasGreen => 1_u8,
            Hint::NobodyHasBlue => 2_u8,
            Hint::NobodyHasBlack => 3_u8,
            Hint::NobodyHasTriangle => 4_u8,
            Hint::NobodyHasSquare => 5_u8,
            Hint::NobodyHasCircle => 6_u8,
            Hint::NobodyHasStar => 7_u8,
            Hint::IHaveRed => 8_u8,
            Hint::IHaveGreen => 9_u8,
            Hint::IHaveBlue => 10_u8,
            Hint::IHaveBlack => 11_u8,
            Hint::IHaveTriangle => 12_u8,
            Hint::IHaveSquare => 13_u8,
            Hint::IHaveCircle => 14_u8,
            Hint::IHaveStar => 15_u8,
        }
    }

    pub fn is_i_have(&self) -> bool {
        return self.u8_val() > 7;
    }

    pub fn is_nobody_has(&self) -> bool {
        return !self.is_i_have();
    }

    pub fn is_color(&self) -> bool {
        return self.u8_val() < 4 || (self.u8_val() > 7 && self.u8_val() < 12);
    }

    /*
    pub fn is_shape(&self) -> bool {
        return !self.is_color();
    }
    */

    pub fn from_u8(val: u8) -> StdResult<Hint> {
        match val {
            0_u8 => Ok(Hint::NobodyHasRed),
            1_u8 => Ok(Hint::NobodyHasGreen),
            2_u8 => Ok(Hint::NobodyHasBlue),
            3_u8 => Ok(Hint::NobodyHasBlack),
            4_u8 => Ok(Hint::NobodyHasTriangle),
            5_u8 => Ok(Hint::NobodyHasSquare),
            6_u8 => Ok(Hint::NobodyHasCircle),
            7_u8 => Ok(Hint::NobodyHasStar),
            8_u8 => Ok(Hint::IHaveRed),
            9_u8 => Ok(Hint::IHaveGreen),
            10_u8 => Ok(Hint::IHaveBlue),
            11_u8 => Ok(Hint::IHaveBlack),
            12_u8 => Ok(Hint::IHaveTriangle),
            13_u8 => Ok(Hint::IHaveSquare),
            14_u8 => Ok(Hint::IHaveCircle),
            15_u8 => Ok(Hint::IHaveStar),
            _ => Err(StdError::generic_err("Invalid hint value")),
        }
    }

    pub fn to_bitmask(&self) -> u8 {
        match self {
            Hint::NobodyHasRed => RED,
            Hint::NobodyHasGreen => GREEN,
            Hint::NobodyHasBlue => BLUE,
            Hint::NobodyHasBlack => BLACK,
            Hint::NobodyHasTriangle => TRIANGLE,
            Hint::NobodyHasSquare => SQUARE,
            Hint::NobodyHasCircle => CIRCLE,
            Hint::NobodyHasStar => STAR,
            Hint::IHaveRed => RED,
            Hint::IHaveGreen => GREEN,
            Hint::IHaveBlue => BLUE,
            Hint::IHaveBlack => BLACK,
            Hint::IHaveTriangle => TRIANGLE,
            Hint::IHaveSquare => SQUARE,
            Hint::IHaveCircle => CIRCLE,
            Hint::IHaveStar => STAR,
        }
    }

    pub fn i_have_from_color(color: Color) -> Hint {
        match color {
            Color::Red => Hint::IHaveRed,
            Color::Green => Hint::IHaveGreen,
            Color::Blue => Hint::IHaveBlue,
            Color::Black => Hint::IHaveBlack,
        }
    }

    pub fn i_have_from_shape(shape: Shape) -> Hint {
        match shape {
            Shape::Triangle => Hint::IHaveTriangle,
            Shape::Square => Hint::IHaveSquare,
            Shape::Circle => Hint::IHaveCircle,
            Shape::Star => Hint::IHaveStar,
        }
    }

    /*
    pub fn nobody_has_from_color(color: Color) -> Hint {
        match color {
            Color::Red => Hint::NobodyHasRed,
            Color::Green => Hint::NobodyHasGreen,
            Color::Blue => Hint::NobodyHasBlue,
            Color::Black => Hint::NobodyHasBlack,
        }
    }

    pub fn nobody_has_from_shape(shape: Shape) -> Hint {
        match shape {
            Shape::Triangle => Hint::NobodyHasTriangle,
            Shape::Square => Hint::NobodyHasSquare,
            Shape::Circle => Hint::NobodyHasCircle,
            Shape::Star => Hint::NobodyHasStar,
        }
    }
    */
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Target {
    Bag,
    Opponent,
    Abstain,
}

impl Target {
    pub fn u8_val(&self) -> u8 {
        match self {
            Target::Bag => 0_u8,
            Target::Opponent => 1_u8,
            Target::Abstain => 2_u8,
        }
    }

    pub fn from_u8(val: u8) -> StdResult<Target> {
        match val {
            0_u8 => Ok(Target::Bag),
            1_u8 => Ok(Target::Opponent),
            2_u8 => Ok(Target::Abstain),
            _ => Err(StdError::generic_err("Invalid target value")),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Guess {
    pub target: Target,
    pub color: Option<Color>,
    pub shape: Option<Shape>,
}

impl Guess {
    pub fn to_stored(&self) -> StoredGuess {
        let color: Option<u8> = match &self.color {
            Some(color) => Some(color.u8_val()),
            None => None,
        };
        let shape: Option<u8> = match &self.shape {
            Some(shape) => Some(shape.u8_val()),
            None => None,
        };
        StoredGuess {
            target: self.target.u8_val(),
            color,
            shape,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredGuess {
    pub target: u8,
    pub color: Option<u8>,
    pub shape: Option<u8>,
}

impl StoredGuess {
    pub fn to_humanized(&self) -> StdResult<Guess> {
        let color: Option<Color> = match self.color {
            Some(color) => Some(Color::from_u8(color)?),
            None => None,
        };
        let shape: Option<Shape> = match self.shape {
            Some(shape) => Some(Shape::from_u8(shape)?),
            None => None,
        };
        let guess = Guess {
            target: Target::from_u8(self.target)?,
            color,
            shape,
        };
        Ok(guess)
    }
}