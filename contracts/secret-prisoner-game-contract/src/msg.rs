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

    // contract info for the powerup nft minter
    pub minter: ContractInfo,

    pub entropy: String,
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

    // Submit an assertion for the opponent
    Submit {
        // one of {"nobody_has", "bag_not"}
        target: String,
        // one of {"triangle", "square", "circle", "star"}
        shape: Option<String>,
        // one of {"red", "green", "blue", "black"}
        color: Option<String>,
        padding: Option<String>, 
    },

    // Guess the arbiter's or the opponent's chip
    Guess {
        // one of {"bag", "opponent", "abstain"}
        target: String,
        // one of {"triangle", "square", "circle", "star"} or None if "abstain"
        shape: Option<String>,
        // one of {"red", "green", "blue", "black"} or None if "abstain"
        color: Option<String>,
        padding: Option<String>,
    },

    // Pick a reward if round 3 has been entered
    PickReward {
        // one of {"nft", "jackpot"}
        reward: String,
        padding: Option<String>,
    },

    // Withdraw from a game if no opponent has joined
    Withdraw {
        padding: Option<String>,
    },

    // Check if opponent has timed out and force endgame, if so
    ForceEndgame {
        padding: Option<String>,
    },

    BatchReceiveNft {
        sender: HumanAddr,
        from: HumanAddr,
        token_ids: Vec<String>,
        msg: Option<String>,
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
    pub powerup: Option<String>,
    pub first_round_start_block: Option<u64>,
    pub first_submit: Option<String>,
    pub first_submit_block: Option<u64>,
    pub opponent_first_submit: Option<String>,
    pub first_extra_secret: Option<String>,
    pub second_submit_turn_start_block: Option<u64>,
    pub second_submit: Option<String>,
    pub second_submit_block: Option<u64>,
    pub opponent_second_submit: Option<String>,
    pub second_extra_secret: Option<String>,
    pub guess_turn_start_block: Option<u64>,
    pub guess: Option<String>,
    pub guess_block: Option<u64>,
    pub opponent_guess: Option<String>,
    pub round_result: Option<String>,
    pub opponent_round_result: Option<String>,
    pub pick_reward_round_start_block: Option<u64>,
    pub finished: Option<bool>,
    pub result: Option<String>,
    pub opponent_powerup: Option<String>,
    pub pick: Option<String>,
    pub jackpot_reward: Option<Uint128>,
    pub nft_token_id: Option<String>,
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

    Withdraw {
        status: ResponseStatus,
        game_state: Option<GameStateResponse>,
    },

    ForceEndgame {
        status: ResponseStatus,
        game_state: Option<GameStateResponse>,
    },

    BatchReceiveNft {
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
    PoolSize { },

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
    PoolSize {
        amount: Uint128,
        denom: String,
    },

    GameState {
        round: Option<u8>,
        wager: Option<Uint128>,
        chip_color: Option<String>,
        chip_shape: Option<String>,
        hint: Option<String>,
        powerup: Option<String>,
        first_round_start_block: Option<u64>,
        first_submit: Option<String>,
        first_submit_block: Option<u64>,
        opponent_first_submit: Option<String>,
        first_extra_secret: Option<String>,
        second_submit_turn_start_block: Option<u64>,
        second_submit: Option<String>,
        second_submit_block: Option<u64>,
        opponent_second_submit: Option<String>,
        second_extra_secret: Option<String>,
        guess_turn_start_block: Option<u64>,
        guess: Option<String>,
        guess_block: Option<u64>,
        opponent_guess: Option<String>,
        round_result: Option<String>,
        opponent_round_result: Option<String>,
        pick_reward_round_start_block: Option<u64>,
        finished: Option<bool>,
        result: Option<String>,
        opponent_powerup: Option<String>,
        pick: Option<String>,
        jackpot_reward: Option<Uint128>,
        nft_token_id: Option<String>,
    },

    PlayerStats {
        info: String // TODO:
    },
}

/// code hash and address of a contract
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct ContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: HumanAddr,
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