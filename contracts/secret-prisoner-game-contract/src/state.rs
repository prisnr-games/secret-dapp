use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use cosmwasm_std::{
    CanonicalAddr, Coin, ReadonlyStorage, StdError, StdResult, Storage, 
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use crate::types::{Color, RED, GREEN, BLUE, BLACK, TRIANGLE, SQUARE, CIRCLE, STAR, Shape, Chip, RoundResult, RoundStage, Guess, Hint, StoredChip, StoredGuess};
use crate::random::{get_random_color, get_random_shape, get_random_number};

pub static CONFIG_KEY: &[u8] = b"config";
pub static POOL_KEY: &[u8] = b"pool";
pub static GAME_PREFIX: &[u8] = b"game";
pub static PLAYER_PREFIX: &[u8] = b"player";
pub static CURRENT_GAME_PREFIX: &[u8] = b"current-game";
pub static WON_PREFIX: &[u8] = b"won";
pub static LOST_PREFIX: &[u8] = b"lost";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: CanonicalAddr,
    pub contract_address: CanonicalAddr,
    pub red_weight: u16,
    pub green_weight: u16,
    pub blue_weight: u16,
    pub black_weight: u16,
    pub triangle_weight: u16,
    pub square_weight: u16,
    pub circle_weight: u16,
    pub star_weight: u16,
    pub stakes: u128,
    pub timeout: u64,
}

pub fn set_config<S: Storage>(
    storage: &mut S,
    config: Config,
) -> StdResult<()> {
    set_bin_data(storage, CONFIG_KEY, &config)
}

pub fn get_config<S: ReadonlyStorage>(storage: &S) -> StdResult<Config> {
    get_bin_data(storage, CONFIG_KEY)
}

///
/// Pool size
/// 

pub fn set_pool<S: Storage>(
    storage: &mut S,
    amount: u128,
) -> StdResult<()> {
    set_bin_data(storage, POOL_KEY, &amount)
}

pub fn get_pool<S: ReadonlyStorage>(
    storage: &S,
) -> StdResult<u128> {
    get_bin_data(storage, POOL_KEY)
}

///
/// Game state
///

#[derive(Serialize, Deserialize, Clone)]
pub struct RoundState {
    pub stage: u8,
    // block height when the round started
    pub round_start_block: u64,

    pub bag_chip: StoredChip,
    pub player_a_chip: StoredChip,
    pub player_b_chip: StoredChip,

    pub player_a_first_hint: u8,
    pub player_b_first_hint: u8,

    pub player_a_first_submit: Option<u8>,
    pub player_b_first_submit: Option<u8>,

    pub player_a_second_submit: Option<u8>,
    pub player_b_second_submit: Option<u8>,

    pub player_a_guess: Option<StoredGuess>,
    pub player_b_guess: Option<StoredGuess>,

    pub player_a_round_result: Option<u8>,
    pub player_b_round_result: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub player_a: CanonicalAddr,
    pub player_b: Option<CanonicalAddr>,

    pub player_a_wager: Option<u128>,
    pub player_b_wager: Option<u128>,

    pub player_a_last_move_block: Option<u64>,
    pub player_b_last_move_block: Option<u64>,

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
    wager: u128,
) -> StdResult<u32> {
    let mut storage = PrefixedStorage::new(GAME_PREFIX, storage);
    let mut storage = AppendStoreMut::<GameState, _>::attach_or_create(&mut storage)?;

    let game_state = GameState {
        player_a: player.clone(),
        player_b: None,
        player_a_wager: Some(wager),
        player_b_wager: None,
        player_a_last_move_block: None,
        player_b_last_move_block: None,
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
    wager: u128,
) -> StdResult<()> {
    let game_idx = store_new_game(storage, player, wager)?;
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
        return Ok(0_u32);
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

pub fn create_new_round<S: Storage>(
    storage: &S,
    block: u64,
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

    let player_a_chip = Chip {
        color: get_random_color(storage, &mut color_options, true)?.unwrap(),
        shape: get_random_shape(storage, &mut shape_options, true)?.unwrap(),
    };

    let player_b_chip = Chip {
        color: get_random_color(storage, &mut color_options, true)?.unwrap(),
        shape: get_random_shape(storage, &mut shape_options, true)?.unwrap(),
    };

    let player_a_first_hint: Hint;
    let player_b_first_hint: Hint;
    let available_hints_mask: u8 = !(bag_chip.to_bitmask() | player_a_chip.to_bitmask() | player_b_chip.to_bitmask());
    let available_color: Hint;
    let available_shape: Hint;
    match available_hints_mask & 0xf0u8 {
        RED => { available_color = Hint::NobodyHasRed },
        GREEN => { available_color = Hint::NobodyHasGreen },
        BLUE => { available_color = Hint::NobodyHasBlue },
        BLACK => { available_color = Hint::NobodyHasBlack },
        _ => { return Err(StdError::generic_err("Error calculating available color hint"));}
    }
    match available_hints_mask & 0x0fu8 {
        TRIANGLE => { available_shape = Hint::NobodyHasTriangle },
        SQUARE => { available_shape = Hint::NobodyHasSquare },
        CIRCLE => { available_shape = Hint::NobodyHasCircle },
        STAR => { available_shape = Hint::NobodyHasStar },
        _ => { return Err(StdError::generic_err("Error calculating available shape hint"));}
    }

    let roll = get_random_number(storage) % 2;
    if roll == 0 {
        // give player a color hint, give player b shape hint
        player_a_first_hint = available_color;
        player_b_first_hint = available_shape;
    } else {
        // give player a shape hint, give player a color hint
        player_a_first_hint = available_shape;
        player_b_first_hint = available_color;
    }

    Ok(RoundState {
        stage: RoundStage::Initialized.u8_val(),
        round_start_block: block,
        bag_chip: bag_chip.to_stored(),
        player_a_chip: player_a_chip.to_stored(),
        player_b_chip: player_b_chip.to_stored(),
        player_a_first_hint: player_a_first_hint.u8_val(),
        player_b_first_hint: player_b_first_hint.u8_val(),
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