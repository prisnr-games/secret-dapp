use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Coin, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use crate::types::{Chip, GameStage, RoundResult, RoundStage, Guess, Hint};

pub static CONFIG_KEY: &[u8] = b"config";
pub static GAMES_PREFIX: &[u8] = b"games";
pub static PLAYERS_PREFIX: &[u8] = b"players";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: CanonicalAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoundState {
    stage: RoundStage,

    player_a_wager: Coin,
    player_b_wager: Coin,

    player_a_chip: Chip,
    player_b_chip: Chip,
    bag_chip: Chip,

    player_a_first_hint: Option<Hint>,
    player_b_first_hint: Option<Hint>,

    player_a_second_hint: Option<Hint>,
    player_b_second_hint: Option<Hint>,

    player_a_guess: Option<Guess>,
    player_b_guess: Option<Guess>,

    player_a_round_result: Option<RoundResult>,
    player_b_round_result: Option<RoundResult>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    player_a: CanonicalAddr,
    player_b: CanonicalAddr,

    round: u8,
    round_state: RoundState,

    stage: GameStage,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerStatus {
    current_game: u32, // index in games appendstore
    games_won: u32,
    games_lost: u32,
}