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

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum RoundResult {
    BagCorrect,
    BagWrong,
    OpponentCorrect,
    OpponentWrong,
    Abstain,
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
pub enum Guess {
    BagCorrect,
    BagWrong,
    OpponentCorrect,
    OpponentWrong,
    Abstain,
}