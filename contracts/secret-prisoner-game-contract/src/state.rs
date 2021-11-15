use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use cosmwasm_std::{
    CanonicalAddr, Coin, ReadonlyStorage, StdError, StdResult, Storage, 
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use crate::types::{Color, Shape, Chip, RoundResult, RoundStage, Guess, Hint};
use crate::random::{get_random_color, get_random_shape, get_random_number};

pub static CONFIG_KEY: &[u8] = b"config";
pub static GAME_PREFIX: &[u8] = b"game";
pub static PLAYER_PREFIX: &[u8] = b"player";
pub static CURRENT_GAME_PREFIX: &[u8] = b"current-game";
pub static WON_PREFIX: &[u8] = b"won";
pub static LOST_PREFIX: &[u8] = b"lost";

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
    pub stage: RoundStage,

    pub player_a_wager: Option<Coin>,
    pub player_b_wager: Option<Coin>,

    pub bag_chip: Chip,
    pub player_a_chip: Chip,
    pub player_b_chip: Chip,

    pub player_a_first_hint: Hint,
    pub player_b_first_hint: Hint,

    pub player_a_first_submit: Option<Hint>,
    pub player_b_first_submit: Option<Hint>,

    pub player_a_second_submit: Option<Hint>,
    pub player_b_second_submit: Option<Hint>,

    pub player_a_guess: Option<Guess>,
    pub player_b_guess: Option<Guess>,

    pub player_a_round_result: Option<RoundResult>,
    pub player_b_round_result: Option<RoundResult>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub player_a: CanonicalAddr,
    pub player_b: Option<CanonicalAddr>,

    pub round: u8, // round 0 means second player has not joined, yet
    pub round_state: Option<RoundState>,
    pub finished: bool,
}

///
/// Game state
/// 

fn store_new_game<S: Storage>(
    storage: &mut S,
    player: &CanonicalAddr,
) -> StdResult<u32> {
    let mut storage = PrefixedStorage::new(GAME_PREFIX, storage);
    let mut storage = AppendStoreMut::<GameState, _>::attach_or_create(&mut storage)?;

    let game_state = GameState {
        player_a: player.clone(),
        player_b: None,
        round: 0_u8,
        round_state: None,
        finished: false,
    };
    storage.push(&game_state)?;
    Ok(storage.len()-1)
}

pub fn create_new_game<S: Storage>(
    storage: &mut S,
    player: &CanonicalAddr,
) -> StdResult<()> {
    let game_idx = store_new_game(storage, player)?;
    set_current_game(storage, player, Some(game_idx))
}

pub fn get_game_state<S: Storage>(
    storage: &S,
    game_idx: u32,
) -> StdResult<GameState> {
    let storage = ReadonlyPrefixedStorage::new(GAME_PREFIX, storage);

    let storage = if let Some(result) = AppendStore::<GameState, _>::attach(&storage) {
        result?
    } else {
        return Err(StdError::generic_err("Error accessing game state storage"));
    };

    storage.get_at(game_idx)
}

pub fn get_number_of_games<S: Storage>(
    storage: &S,
) -> StdResult<u32> {
    let storage = ReadonlyPrefixedStorage::new(GAME_PREFIX, storage);

    let storage = if let Some(result) = AppendStore::<GameState, _>::attach(&storage) {
        result?
    } else {
        return Err(StdError::generic_err("Error accessing game state storage"));
    };

    Ok(storage.len())
}

pub fn update_game_state<S: Storage>(
    storage: &mut S,
    game_idx: u32,
    game_state: &GameState,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(GAME_PREFIX, storage);
    let mut storage = AppendStoreMut::<GameState, _>::attach_or_create(&mut storage)?;

    storage.set_at(game_idx, game_state)
}

pub fn is_game_waiting_for_second_player<S: Storage>(
    storage: &S,
) -> StdResult<bool> {
    let storage = ReadonlyPrefixedStorage::new(GAME_PREFIX, storage);

    let storage = if let Some(result) = AppendStore::<GameState, _>::attach(&storage) {
        result?
    } else {
        return Ok(false);
    };

    // get the state of the last game created
    let game_state = storage.get_at(storage.len()-1)?;
    Ok(game_state.player_b.is_none())
}

fn pick_hint<S: Storage>(
    storage: &S,
    color_options: &mut Vec<Color>,
    shape_options: &mut Vec<Shape>,
) -> StdResult<Hint> {
    let hint: Hint;
    let random_number = get_random_number(storage) % 100;
    // 50/50 chance of getting color or shape as hint
    if random_number < 50 {
        // color
        let hint_color = get_random_color(storage, color_options, false)?.unwrap();
        match hint_color {
            Color::Red => { hint = Hint::BagNotRed },
            Color::Green => { hint = Hint::BagNotGreen },
            Color::Blue => { hint = Hint::BagNotBlue },
            Color::Black => { hint = Hint::BagNotBlack },
        }
    } else {
        // shape
        let hint_shape = get_random_shape(storage, shape_options, false)?.unwrap();
        match hint_shape {
            Shape::Triangle => { hint = Hint::BagNotTriangle },
            Shape::Square=> { hint = Hint::BagNotSquare },
            Shape::Circle => { hint = Hint::BagNotCircle },
            Shape::Star => { hint = Hint::BagNotStar },
        }
    }
    Ok(hint)
}

pub fn create_new_round<S: Storage>(
    storage: &S,
    player_a_wager: Option<Coin>,
    player_b_wager: Option<Coin>,
) -> StdResult<RoundState> {
    let mut color_options: Vec<Color> = vec!(
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Black,
    );

    let mut shape_options: Vec<Shape> = vec!(
        Shape::Triangle,
        Shape::Square,
        Shape::Circle,
        Shape::Star,
    );

    let bag_chip = Chip {
        color: get_random_color(storage, &mut color_options, true)?.unwrap(),
        shape: get_random_shape(storage, &mut shape_options, true)?.unwrap(),
    };

    let player_a_first_hint: Hint = pick_hint(storage, &mut color_options, &mut shape_options)?;
    let player_b_first_hint: Hint = pick_hint(storage, &mut color_options, &mut shape_options)?;

    let player_a_chip = Chip {
        color: get_random_color(storage, &mut color_options, true)?.unwrap(),
        shape: get_random_shape(storage, &mut shape_options, true)?.unwrap(),
    };

    let player_b_chip = Chip {
        color: get_random_color(storage, &mut color_options, true)?.unwrap(),
        shape: get_random_shape(storage, &mut shape_options, true)?.unwrap(),
    };

    Ok(RoundState {
        stage: RoundStage::Initialized,
        player_a_wager,
        player_b_wager,
        bag_chip,
        player_a_chip,
        player_b_chip,
        player_a_first_hint,
        player_b_first_hint,
        player_a_first_submit: None,
        player_b_first_submit: None,
        player_a_second_submit: None,
        player_b_second_submit: None,
        player_a_guess: None,
        player_b_guess: None,
        player_a_round_result: None,
        player_b_round_result: None,
    })
}

///
/// Player Status
/// 

pub fn set_current_game<S: Storage>(
    storage: &mut S,
    player: &CanonicalAddr,
    game_idx: Option<u32>,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(CURRENT_GAME_PREFIX, storage);
    set_bin_data(&mut storage, player.as_slice(), &game_idx)
}

pub fn get_current_game<S: ReadonlyStorage>(
    storage: &S,
    player: &CanonicalAddr,
) -> Option<u32> {
    let storage = ReadonlyPrefixedStorage::new(CURRENT_GAME_PREFIX, storage);
    get_bin_data(&storage, player.as_slice()).unwrap_or_else(|_| None)
}

pub fn set_won<S: Storage>(
    storage: &mut S,
    player: &CanonicalAddr,
    won: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(WON_PREFIX, storage);
    set_bin_data(&mut storage, player.as_slice(), &won)
}

pub fn get_won<S: ReadonlyStorage>(
    storage: &S,
    player: &CanonicalAddr,
) -> u32 {
    let storage = ReadonlyPrefixedStorage::new(WON_PREFIX, storage);
    get_bin_data(&storage, player.as_slice()).unwrap_or_else(|_| 0_u32)
}

pub fn set_lost<S: Storage>(
    storage: &mut S,
    player: &CanonicalAddr,
    lost: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(LOST_PREFIX, storage);
    set_bin_data(&mut storage, player.as_slice(), &lost)
}

pub fn get_lost<S: ReadonlyStorage>(
    storage: &S,
    player: &CanonicalAddr,
) -> u32 {
    let storage = ReadonlyPrefixedStorage::new(LOST_PREFIX, storage);
    get_bin_data(&storage, player.as_slice()).unwrap_or_else(|_| 0_u32)
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