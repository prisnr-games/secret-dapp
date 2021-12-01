use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128,};
use secret_toolkit::permit::Permit;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub rounds_per_game: u8,

    // default even weights for colors and shapes (25,25,25,25)
    pub red_weight: Option<u16>,
    pub green_weight: Option<u16>,
    pub blue_weight: Option<u16>,
    pub black_weight: Option<u16>,
    pub triangle_weight: Option<u16>,
    pub square_weight: Option<u16>,
    pub circle_weight: Option<u16>,
    pub star_weight: Option<u16>,

    // stakes for a game in uscrt (default = 1000000)
    pub stakes: Option<Uint128>,
    //pub low_stakes: Coin,
    //pub medium_stakes: Coin,
    //pub high_stakes: Coin,

    // timeout per turn, in # of blocks
    pub timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Join a new game
    Join {
        // one of {"low", "medium", "high"} or None means no money bet
        //stakes: Option<String>,
        padding: Option<String>, 
    },

    Submit {
        // one of {"nobody_has", "bag_not"}
        target: String,
        // one of {"triangle", "square", "circle", "star"}
        shape: Option<String>,
        // one of {"red", "green", "blue", "black"}
        color: Option<String>,
        padding: Option<String>, 
    },

    Guess {
        // one of {"bag", "opponent", "abstain"}
        target: String,
        // one of {"triangle", "square", "circle", "star"} or None if "abstain"
        shape: Option<String>,
        // one of {"red", "green", "blue", "black"} or None if "abstain"
        color: Option<String>,
        padding: Option<String>,
    },

    PickReward {
        // one of {"nft", "pool"}
        reward: String,
        padding: Option<String>,
    },

    // Permit
    RevokePermit {
        permit_name: String,
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameStateResponse {
    pub round: Option<u8>,
    pub wager: Option<Uint128>,
    pub chip_color: Option<String>,
    pub chip_shape: Option<String>,
    pub hint: Option<String>,
    pub first_submit: Option<String>,
    pub opponent_first_submit: Option<String>,
    pub first_extra_secret: Option<String>,
    pub second_submit: Option<String>,
    pub opponent_second_submit: Option<String>,
    pub second_extra_secret: Option<String>,
    pub guess: Option<String>,
    pub opponent_guess: Option<String>,
    pub round_result: Option<String>,
    pub opponent_round_result: Option<String>,
    pub finished: Option<bool>,
    pub result: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Join {
        status: ResponseStatus,
        game_state: Option<GameStateResponse>,
    },

    Submit {
        status: ResponseStatus,
        game_state: Option<GameStateResponse>,
    },

    Guess {
        status: ResponseStatus,
        game_state: Option<GameStateResponse>,
    },

    PickReward {
        status: ResponseStatus,
        game_state: Option<GameStateResponse>,
    },

    // Permit
    RevokePermit {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    // GameState returns the player's view on current game
    GameState {},

    // PlayerStats returns how many wins/losses for player
    PlayerStats {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GameState {
        round: Option<u8>,
        wager: Option<Uint128>,
        chip_color: Option<String>,
        chip_shape: Option<String>,
        hint: Option<String>,
        first_submit: Option<String>,
        opponent_first_submit: Option<String>,
        first_extra_secret: Option<String>,
        second_submit: Option<String>,
        opponent_second_submit: Option<String>,
        second_extra_secret: Option<String>,
        guess: Option<String>,
        opponent_guess: Option<String>,
        round_result: Option<String>,
        opponent_round_result: Option<String>,
        finished: Option<bool>,
        result: Option<String>,
    },
    PlayerStats {
        info: String // TODO:
    },
}

// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(block_size: usize, message: &mut Vec<u8>) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}