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
}

#[derive(Hash, Serialize, Deserialize, Clone, Eq, PartialEq)]
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
}

#[derive(Serialize, Deserialize, Clone)]
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

impl Hint {
    pub fn u8_val(&self) -> u8 {
        match self {
            Hint::BagNotRed => 0_u8,
            Hint::BagNotGreen => 1_u8,
            Hint::BagNotBlue => 2_u8,
            Hint::BagNotBlack => 3_u8,
            Hint::BagNotTriangle => 4_u8,
            Hint::BagNotSquare => 5_u8,
            Hint::BagNotCircle => 6_u8,
            Hint::BagNotStar => 7_u8,
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

    pub fn from_u8(val: u8) -> StdResult<Hint> {
        match val {
            0_u8 => Ok(Hint::BagNotRed),
            1_u8 => Ok(Hint::BagNotGreen),
            2_u8 => Ok(Hint::BagNotBlue),
            3_u8 => Ok(Hint::BagNotBlack),
            4_u8 => Ok(Hint::BagNotTriangle),
            5_u8 => Ok(Hint::BagNotSquare),
            6_u8 => Ok(Hint::BagNotCircle),
            7_u8 => Ok(Hint::BagNotStar),
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
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
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