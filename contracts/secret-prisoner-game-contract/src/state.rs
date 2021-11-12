use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

use cosmwasm_std::{
    CanonicalAddr, Coin, ReadonlyStorage, StdError, StdResult, Storage,
};
use crate::types::{Chip, GameStage, RoundResult, RoundStage, Guess, Hint};

pub static CONFIG_KEY: &[u8] = b"config";
pub static GAMES_PREFIX: &[u8] = b"games";
pub static PLAYERS_PREFIX: &[u8] = b"players";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: CanonicalAddr,
    pub contract_address: CanonicalAddr,
}

pub fn set_config<S: Storage>(
    storage: &mut S,
    admin: CanonicalAddr,
    contract_address: CanonicalAddr,
) -> StdResult<()> {
    let config = Config {
        admin,
        contract_address,
    };
    set_bin_data(storage, CONFIG_KEY, &config)
}

pub fn get_config<S: ReadonlyStorage>(storage: &S) -> StdResult<Config> {
    get_bin_data(storage, CONFIG_KEY)
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

//
// Bin data storage setters and getters
//

pub fn set_bin_data<T: Serialize, S: Storage>(
    storage: &mut S,
    key: &[u8],
    data: &T,
) -> StdResult<()> {
    let bin_data =
        bincode2::serialize(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;
    storage.set(key, &bin_data);
    Ok(())
}

pub fn get_bin_data<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<T> {
    let bin_data = storage.get(key);
    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(bincode2::deserialize::<T>(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}