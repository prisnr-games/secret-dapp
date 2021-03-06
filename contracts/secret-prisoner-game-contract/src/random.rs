use std::collections::{HashMap};
use cosmwasm_std::{StdResult, StdError, Storage, debug_print};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use sha2::{Digest, Sha256};

use crate::state::{get_config, Config};
use crate::types::{Color, Shape};

static KEY_ENTROPY_POOL: &[u8] = b"entropy_pool";

fn get_current_entropy_pool<S: Storage>(storage: &S) -> [u8; 32] {
    ReadonlySingleton::new(storage, KEY_ENTROPY_POOL)
        .load()
        .or::<[u8; 32]>(Ok([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]))
        .unwrap()
}

pub fn supply_more_entropy<S: Storage>(
    storage: &mut S,
    additional_entropy: &[u8],
) -> StdResult<()> {
    let current_entropy_pool = get_current_entropy_pool(storage);
    println!("{:?}", current_entropy_pool);

    let mut new_entropy_source = Vec::from(current_entropy_pool);
    new_entropy_source.extend(additional_entropy);

    let new_entropy_pool: [u8; 32] = Sha256::digest(&new_entropy_source).into();

    Singleton::new(storage, KEY_ENTROPY_POOL).save(&new_entropy_pool)
}

pub fn get_random_color<S: Storage>(storage: &S, color_options: &mut Vec<Color>, remove: bool) -> StdResult<Option<Color>> {
    if color_options.len() == 0 {
        return Err(StdError::generic_err("No color options when picking a random color"));
    }

    let config: Config = get_config(storage)?;

    let color_percentage_map: HashMap<Color, u64> = [
        (Color::Red, config.red_weight as u64),
        (Color::Green, config.green_weight as u64),
        (Color::Blue, config.blue_weight as u64),
        (Color::Black, config.black_weight as u64),
    ].iter().cloned().collect();

    let mut total = 0_u64;
    for color in color_options.into_iter() {
        if let Some(pct) = color_percentage_map.get(color) {
            total = total + pct;
        } else {
            // error, using invalid color
            return Err(StdError::generic_err("Invalid color in color options when picking a random color"));
        }
    }
    debug_print(format!("color weight total: {}", total));

    let roll = get_random_number(storage) % total;
    println!("{}", roll);
    debug_print(format!("color roll: {}", roll));

    let mut interval_start = 0_u64;
    let mut picked_color: Option<Color> = None;
    let mut picked_index: Option<usize> = None;
    for (i, color) in color_options.iter().enumerate() {
        if let Some(pct) = color_percentage_map.get(color) {
            if roll < interval_start + pct {
                picked_index = Some(i);
                picked_color = Some(color.clone());
                break;
            }
            interval_start = interval_start + pct;
        }
    }
    if remove && picked_index.is_some() {
        color_options.swap_remove(picked_index.unwrap());
    }
    debug_print(format!("picked color: {:?}", picked_color));
    Ok(picked_color)
}

pub fn get_random_shape<S: Storage>(storage: &S, shape_options: &mut Vec<Shape>, remove: bool) -> StdResult<Option<Shape>> {
    if shape_options.len() == 0 {
        return Err(StdError::generic_err("No shape options when picking a random shape"));
    }

    let config: Config = get_config(storage)?;

    let shape_percentage_map: HashMap<Shape, u64> = [
        (Shape::Triangle, config.triangle_weight as u64),
        (Shape::Square, config.square_weight as u64),
        (Shape::Circle, config.circle_weight as u64),
        (Shape::Star, config.star_weight as u64),
    ].iter().cloned().collect();

    let mut total = 0_u64;
    for shape in shape_options.into_iter() {
        if let Some(pct) = shape_percentage_map.get(shape) {
            total = total + pct;
        } else {
            // error, using invalid shape
            return Err(StdError::generic_err("Invalid shape in shape options when picking a random shape"));
        }
    }
    debug_print(format!("shape weight total: {}", total));

    let roll = get_random_number(storage) % total;
    debug_print(format!("shape roll: {}", roll));

    let mut interval_start = 0_u64;
    let mut picked_shape: Option<Shape> = None;
    let mut picked_index: Option<usize> = None;
    for (i, shape) in shape_options.iter().enumerate() {
        if let Some(pct) = shape_percentage_map.get(shape) {
            if roll < interval_start + pct {
                picked_index = Some(i);
                picked_shape = Some(shape.clone());
                break;
            }
            interval_start = interval_start + pct;
        }
    }
    if remove && picked_index.is_some() {
        shape_options.swap_remove(picked_index.unwrap());
    }
    debug_print(format!("picked shape: {:?}", picked_shape));
    Ok(picked_shape)
}

pub fn get_random_number<S: Storage>(storage: &S) -> u64 {
    let entropy_pool = get_current_entropy_pool(storage);

    let mut rng = ChaChaRng::from_seed(entropy_pool);

    rng.next_u64()
}

pub fn sha_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_slice());
    result
}

