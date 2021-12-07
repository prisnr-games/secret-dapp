use std::cmp::{max};
use cosmwasm_std::{
    debug_print, 
    to_binary, Api, Binary, Coin, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage, CanonicalAddr, Uint128, CosmosMsg, BankMsg,
};
use secret_toolkit::{
    permit::{validate, Permission, Permit, RevokedPermits},
    snip721::{
        mint_nft_msg, Metadata, set_viewing_key_msg, register_receive_nft_msg, set_minters_msg, private_metadata_query,
        ViewerInfo, Extension,
    },
};

use crate::msg::{ContractInfo, GameStateResponse, QueryWithPermit, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, space_pad, ResponseStatus::Success};
use crate::random::{get_random_number, supply_more_entropy, sha_256};
use crate::state::{
    create_new_game, set_config, get_config, get_current_game, get_game_state, get_number_of_games,
    GameState, create_new_round, update_game_state, RoundState, Config, set_current_game, get_pool, set_pool,
    StoreContractInfo, set_minter, get_minter,
};
use crate::types::{Chip, Guess, Hint, RoundStage, RoundResult, Target, Color, Shape, is_bitmask_color, GameResult,
RED, GREEN, BLUE, BLACK, TRIANGLE, SQUARE, CIRCLE, STAR, REWARD_NFT, REWARD_POOL};

/// We make sure that responses from `handle` are padded to a multiple of this size.
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";
pub const DEFAULT_STAKES: Uint128 = Uint128(1000000);
pub const DEFAULT_TIMEOUT: u64 = 100; // 100 Blocks (~ 10 minutes)
pub const DENOM: &str = "uscrt";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let prng_seed: Vec<u8> = sha_256(base64::encode(msg.entropy.clone()).as_bytes()).to_vec();
    let viewing_key = base64::encode(&prng_seed);

    let red_weight = msg.red_weight.unwrap_or(25);
    let green_weight = msg.green_weight.unwrap_or(25);
    let blue_weight = msg.blue_weight.unwrap_or(25);
    let black_weight = msg.black_weight.unwrap_or(25);

    let triangle_weight = msg.triangle_weight.unwrap_or(25);
    let square_weight = msg.square_weight.unwrap_or(25);
    let circle_weight = msg.circle_weight.unwrap_or(25);
    let star_weight = msg.star_weight.unwrap_or(25);

    let stakes = msg.stakes.unwrap_or(DEFAULT_STAKES);
    let stakes = stakes.u128();

    // default timeout for each move is 20 blocks
    let timeout = msg.timeout.unwrap_or(DEFAULT_TIMEOUT);

    let admin = deps.api.canonical_address(&env.message.sender)?;
    let contract_address = deps.api.canonical_address(&env.contract.address)?;

    let config = Config {
        admin,
        contract_address,
        red_weight,
        green_weight,
        blue_weight,
        black_weight,
        triangle_weight,
        square_weight,
        circle_weight,
        star_weight,
        stakes,
        timeout,
        viewing_key: viewing_key.clone(),
    };

    set_config(
        &mut deps.storage, 
        config,
    )?;

    let minter = msg.minter.clone();
    let minter = StoreContractInfo {
        address: deps.api.canonical_address(&minter.address)?,
        code_hash: minter.code_hash,
    };
    set_minter(&mut deps.storage, minter)?;

    // is the jackpot pool seeded with funds?
    if env.message.sent_funds.len() == 0 {
        set_pool(&mut deps.storage, 0)?;
    } else if env.message.sent_funds.len() == 1 {
        let funds = &env.message.sent_funds[0];
        if funds.denom != DENOM {
            return Err(StdError::generic_err("Can only seed jackpot pool with scrt"));
        } else {
            set_pool(&mut deps.storage, funds.amount.u128())?;
        }
    } else {
        return Err(StdError::generic_err("Can only seed jackpot pool with scrt"));
    }

    let mut fresh_entropy = to_binary(&msg)?.0;
    fresh_entropy.extend(to_binary(&env)?.0);
    supply_more_entropy(&mut deps.storage, fresh_entropy.as_slice())?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![
            register_receive_nft_msg(
                env.contract_code_hash,
                Some(true),
                None,
                256,
                msg.minter.code_hash.clone(),
                msg.minter.address.clone(),
            )?,
            set_viewing_key_msg(
                viewing_key,
                None,
                256,
                msg.minter.code_hash.clone(),
                msg.minter.address.clone(),
            )?,
            //set_minters_msg(
            //    vec![env.contract.address],
            //    None,
            //    256,
            //    msg.minter.code_hash,
            //    msg.minter.address,
            //)?,
        ],
        log: vec![],
    })
}

fn pad_response(response: StdResult<HandleResponse>) -> StdResult<HandleResponse> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(RESPONSE_BLOCK_SIZE, &mut data.0);
            data
        });
        response
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let mut fresh_entropy = to_binary(&msg)?.0;
    fresh_entropy.extend(to_binary(&env)?.0);
    supply_more_entropy(&mut deps.storage, fresh_entropy.as_slice())?;

    let response = match msg {
        HandleMsg::Join { 
            //stakes, 
            .. 
        } => try_join(deps, env,),
        HandleMsg::Submit { target, color, shape, .. } => try_submit(deps, env, target, color, shape),
        HandleMsg::Guess { target, color, shape, .. } => try_guess(deps, env, target, color, shape),
        HandleMsg::PickReward { reward, .. } => try_pick_reward(deps, env, reward),
        HandleMsg::Withdraw { .. } => try_withdraw(deps, env),
        HandleMsg::BatchReceiveNft { sender, from, token_ids, msg } => try_receive_nft(deps, env, sender, from, token_ids, msg),
        HandleMsg::RevokePermit { permit_name, .. } => revoke_permit(deps, env, permit_name),
    };

    pad_response(response)
}

pub fn try_join<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    //stakes: Option<String>,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;

    // check if already in ongoing game, 
    // if yes, check it is finished otherwise throw error (only one game at a time allowed)
    let current_game_idx = get_current_game(&deps.storage, &player);
    if current_game_idx.is_some() {
        let current_game = get_game_state(&deps.storage, current_game_idx.unwrap())?;
        if !current_game.finished {
            return Err(StdError::generic_err("You must finish current game before beginning a new one"));
        }
    }

    // check that player has sent correct funds to match the stakes
    let stakes = get_config(&deps.storage)?.stakes;
    if env.message.sent_funds.len() != 1 {
        return Err(StdError::generic_err("Incorrect funds sent to join game"));
    }
    let funds = &env.message.sent_funds[0];
    if funds.denom != "uscrt" {
        return Err(StdError::generic_err("Incorrect coin type sent to join game"));
    }
    if funds.amount.u128() != stakes {
        return Err(StdError::generic_err(format!("Incorrect amount sent, must be {} uscrt", stakes)));
    }

    let number_of_games = get_number_of_games(&deps.storage)?;
    let game_ready: bool;
    let mut game_state: Option<GameState> = None;

    // check if a new game needs to be created
    if number_of_games == 0 {
        game_ready = false;
    } else {
        let current_game_state = get_game_state(&deps.storage, number_of_games - 1)?;
        game_ready = current_game_state.player_b.is_none();
        game_state = Some(current_game_state);
    }
    
    if !game_ready {
        // if yes: create a new game state with player_a
        //   create_new_game sets the current game for player to this one
        create_new_game(&mut deps.storage, &player, funds.amount.u128())?;
    } else {
        // if no: add player_b to waiting game_state, create first round and assign chips
        let mut game_state = game_state.unwrap();
        game_state.player_b = Some(player.clone());
        game_state.player_b_wager = Some(funds.amount.u128());

        let new_round = create_new_round(&deps.storage, env.block.height)?;
        game_state.round_state = Some(new_round);
        game_state.round = 1_u8;
        update_game_state(&mut deps.storage, number_of_games - 1, &game_state)?;
        set_current_game(&mut deps.storage, &player, Some(number_of_games - 1))?;
    }

    let game_state_response = get_game_state_response(&deps.storage, player)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Join { status: Success, game_state: Some(game_state_response) })?),
    })
}

fn submission_provably_false(
    hint: Hint,
    other_player_chip_mask: u8,
    other_player_first_hint_mask: u8,
) -> bool {
    let hint_mask = hint.to_bitmask();
    return (hint_mask & other_player_chip_mask > 0) ||
           (hint.is_i_have() && (hint_mask & other_player_first_hint_mask > 0));
}

fn pick_extra_secret<S: Storage>(
    storage: &S,
    other_player_chip: Chip,
    other_player_hint: u8,
    prev_secret: Option<u8>,
) -> StdResult<Option<u8>> {
    if prev_secret.is_none() {
        let roll = get_random_number(storage) % 3;
        if roll == 0 { 
            // share opponent's color
            let color: Color = other_player_chip.color;
            return Ok(Some(Hint::i_have_from_color(color).u8_val()));
        } else if roll == 1 {
            // share opponent's shape
            let shape: Shape = other_player_chip.shape;
            return Ok(Some(Hint::i_have_from_shape(shape).u8_val()));
        } else {
            // share opponent's hint
            return Ok(Some(other_player_hint));
        }
    } else {
        let prev_secret = Hint::from_u8(prev_secret.unwrap())?;
        let roll = get_random_number(storage) % 2;
        if prev_secret.is_i_have() {
            if prev_secret.is_color() {
                // shared opponent's color last time
                if roll == 0 { 
                    // share opponents's hint
                    return Ok(Some(other_player_hint));
                } else {
                    // share opponent's shape
                    let shape: Shape = other_player_chip.shape;
                    return Ok(Some(Hint::i_have_from_shape(shape).u8_val()));

                }
            } else {
                // shared opponent's shape last time
                if roll == 0 { 
                    // share opponent's hint
                    return Ok(Some(other_player_hint));
                } else {
                    // share opponent's color
                    let color: Color = other_player_chip.color;
                    return Ok(Some(Hint::i_have_from_color(color).u8_val()));
                }
            }
        } else {
            // gave away other player's hint last time, so give chip color or shape now
            if roll == 0 { 
                // share opponent's color
                let color: Color = other_player_chip.color;
                return Ok(Some(Hint::i_have_from_color(color).u8_val()));
            } else {
                // share opponent's shape
                let shape: Shape = other_player_chip.shape;
                return Ok(Some(Hint::i_have_from_shape(shape).u8_val()));
            }
        }
    }
}

fn check_timeout<S: Storage>(
    storage: S,
    game_state: GameState,
    block_height: u64,
) -> StdResult<()> {
    // no timeout until both players have joined
    if game_state.round > 0 {
        if game_state.round == 1 && game_state.round_state.is_some() {
            
        }
    }
    Ok(())
}

pub fn try_submit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: String,
    color: Option<String>,
    shape: Option<String>,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;
    debug_print(format!("Player {} submitting hint", env.message.sender));

    if (color.is_none() && shape.is_none()) || (color.is_some() && shape.is_some()) {
        return Err(StdError::generic_err("Hint must be either a color or shape but not both"));
    }

    let hint: Hint;

    match target.as_str() {
        "i_have" => {
            if color.is_some() {
                match color.unwrap().as_str() {
                    "red" => { hint = Hint::IHaveRed },
                    "green" => { hint = Hint::IHaveGreen },
                    "blue" => { hint = Hint::IHaveBlue },
                    "black" => { hint = Hint::IHaveBlack },
                    _ => { return Err(StdError::generic_err("Invalid color")); },
                }
            } else { // shape
                match shape.unwrap().as_str() {
                    "triangle" => { hint = Hint::IHaveTriangle },
                    "square" => { hint = Hint::IHaveSquare },
                    "circle" => { hint = Hint::IHaveCircle },
                    "star" => { hint = Hint::IHaveStar },
                    _ => { return Err(StdError::generic_err("Invalid shape")); },
                }
            }
        },
        "nobody_has" => {
            if color.is_some() {
                match color.unwrap().as_str() {
                    "red" => { hint = Hint::NobodyHasRed },
                    "green" => { hint = Hint::NobodyHasGreen },
                    "blue" => { hint = Hint::NobodyHasBlue },
                    "black" => { hint = Hint::NobodyHasBlack },
                    _ => { return Err(StdError::generic_err("Invalid color")); },
                }
            } else { // shape
                match shape.unwrap().as_str() {
                    "triangle" => { hint = Hint::NobodyHasTriangle },
                    "square" => { hint = Hint::NobodyHasSquare },
                    "circle" => { hint = Hint::NobodyHasCircle },
                    "star" => { hint = Hint::NobodyHasStar },
                    _ => { return Err(StdError::generic_err("Invalid shape")); },
                }
            }
        },
        _ => { return Err(StdError::generic_err("Invalid hint")); }
    }

    // check if already in an ongoing game
    let current_game = get_current_game(&deps.storage, &player);
    if current_game.is_none() {
        return Err(StdError::generic_err("You cannot submit a hint before joining a game"));
    }

    let mut game_state: GameState = get_game_state(&deps.storage, current_game.unwrap())?;

    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }

    if game_state.round == 0 || game_state.round_state.is_none() {
        return Err(StdError::generic_err("First round has not been initialized"));
    }

    if game_state.round >= 3 {
        return Err(StdError::generic_err("Finished round with submissions"))
    }

    let mut round_state: RoundState = game_state.round_state.unwrap();

    match RoundStage::from_u8(round_state.stage)? {
        RoundStage::Initialized => {
            let new_hint = Some(hint.u8_val());

            if player == game_state.player_a && round_state.player_a_first_submit.is_none() {
                round_state.player_a_first_submit = new_hint;
                round_state.player_a_first_submit_block = Some(env.block.height);
                // check if provably false by b, if so reveal a secret from a
                //  this happens if b has color or shape in hint, or
                //  the first hint given in the game contradicts the newly submitted hint
                let other_player_chip = round_state.player_b_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_b_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // calculate secret to give out
                    round_state.player_b_first_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_a_chip.to_humanized()?,
                        round_state.player_a_first_hint, 
                        None
                    )?;
                }
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_first_submit.is_none() {
                round_state.player_b_first_submit = new_hint;
                round_state.player_b_first_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_a_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_a_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // calculate secret to give out
                    round_state.player_a_first_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_b_chip.to_humanized()?,
                        round_state.player_b_first_hint, 
                        None
                    )?;
                }
            } else {
                return Err(StdError::generic_err("Cannot accept a submission from player"));
            }
            round_state.stage = RoundStage::OnePlayerFirstSubmit.u8_val();

            game_state.round_state = Some(round_state);
            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        RoundStage::OnePlayerFirstSubmit => {
            let new_hint = Some(hint.u8_val());

            if player == game_state.player_a && round_state.player_a_first_submit.is_none() {
                round_state.player_a_first_submit = new_hint;
                round_state.player_a_first_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_b_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_b_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // calculate secret to give out
                    round_state.player_b_first_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_a_chip.to_humanized()?,
                        round_state.player_a_first_hint, 
                        None
                    )?;
                }
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_first_submit.is_none() {
                round_state.player_b_first_submit = new_hint;
                round_state.player_b_first_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_a_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_a_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // calculate secret to give out
                    round_state.player_a_first_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_b_chip.to_humanized()?,
                        round_state.player_b_first_hint, 
                        None
                    )?;
                }
            } else {
                return Err(StdError::generic_err("Cannot accept a submission from player"));
            }
            round_state.stage = RoundStage::BothPlayersFirstSubmit.u8_val();

            game_state.round_state = Some(round_state);
            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        RoundStage::BothPlayersFirstSubmit => {
            let new_hint = Some(hint.u8_val());

            if player == game_state.player_a && round_state.player_a_second_submit.is_none() {
                let first_hint = Hint::from_u8(round_state.player_a_first_submit.unwrap())?;
                if (first_hint.is_i_have() && hint.is_i_have()) || 
                   (first_hint.is_nobody_has() && hint.is_nobody_has())
                {
                    return Err(StdError::generic_err("Assertions must have different targets: i_have and nobody_has"));
                }
                if first_hint.to_bitmask() == hint.to_bitmask() {
                    return Err(StdError::generic_err("Second assertion cannot contradict first assertion"));
                }
                round_state.player_a_second_submit = new_hint;
                round_state.player_a_second_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_b_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_b_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // check if a secret was revealed in the first submission, and pick accordingly
                    round_state.player_b_second_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_a_chip.to_humanized()?,
                        round_state.player_a_first_hint, 
                        round_state.player_b_first_extra_secret,
                    )?;
                }
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_second_submit.is_none() {
                let first_hint = Hint::from_u8(round_state.player_b_first_submit.unwrap())?;
                if (first_hint.is_i_have() && hint.is_i_have()) || 
                   (first_hint.is_nobody_has() && hint.is_nobody_has())
                {
                    return Err(StdError::generic_err("Assertions must have different targets: i_have and nobody_has"));
                }
                if first_hint.to_bitmask() == hint.to_bitmask() {
                    return Err(StdError::generic_err("Second assertion cannot contradict first assertion"));
                }
                round_state.player_b_second_submit = new_hint;
                round_state.player_b_second_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_a_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_a_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // check if a secret was revealed in the first submission, and pick accordingly
                    round_state.player_a_second_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_b_chip.to_humanized()?,
                        round_state.player_b_first_hint, 
                        round_state.player_a_first_extra_secret,
                    )?;
                }
            } else {
                return Err(StdError::generic_err("Cannot accept a submission from player"));
            }
            round_state.stage = RoundStage::OnePlayerSecondSubmit.u8_val();

            game_state.round_state = Some(round_state);
            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        RoundStage::OnePlayerSecondSubmit => {
            let new_hint = Some(hint.u8_val());

            if player == game_state.player_a && round_state.player_a_second_submit.is_none() {
                let first_hint = Hint::from_u8(round_state.player_a_first_submit.unwrap())?;
                if (first_hint.is_i_have() && hint.is_i_have()) || 
                   (first_hint.is_nobody_has() && hint.is_nobody_has())
                {
                    return Err(StdError::generic_err("Assertions must have different targets: i_have and nobody_has"));
                }
                if first_hint.to_bitmask() == hint.to_bitmask() {
                    return Err(StdError::generic_err("Second assertion cannot contradict first assertion"));
                }
                round_state.player_a_second_submit = new_hint;
                round_state.player_a_second_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_b_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_b_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // check if a secret was revealed in the first submission, and pick accordingly
                    round_state.player_b_second_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_a_chip.to_humanized()?,
                        round_state.player_a_first_hint, 
                        round_state.player_b_first_extra_secret,
                    )?;
                }
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_second_submit.is_none() {
                let first_hint = Hint::from_u8(round_state.player_b_first_submit.unwrap())?;
                if (first_hint.is_i_have() && hint.is_i_have()) || 
                   (first_hint.is_nobody_has() && hint.is_nobody_has())
                {
                    return Err(StdError::generic_err("Assertions must have different targets: i_have and nobody_has"));
                }
                if first_hint.to_bitmask() == hint.to_bitmask() {
                    return Err(StdError::generic_err("Second assertion cannot contradict first assertion"));
                }
                round_state.player_b_second_submit = new_hint;
                round_state.player_b_second_submit_block = Some(env.block.height);
                let other_player_chip = round_state.player_a_chip.to_humanized()?.to_bitmask();
                let other_player_first_hint = Hint::from_u8(round_state.player_a_first_hint)?.to_bitmask();
                if submission_provably_false(hint, other_player_chip, other_player_first_hint) {
                    // check if a secret was revealed in the first submission, and pick accordingly
                    round_state.player_a_second_extra_secret = pick_extra_secret(
                        &deps.storage, 
                        round_state.player_b_chip.to_humanized()?,
                        round_state.player_b_first_hint, 
                        round_state.player_a_first_extra_secret,
                    )?;
                }
            } else {
                return Err(StdError::generic_err("Cannot accept a submission from player"));
            }
            round_state.stage = RoundStage::BothPlayersSecondSubmit.u8_val();

            game_state.round_state = Some(round_state);
            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        _ => { return Err(StdError::generic_err("Not a submission round")); },
    };

    let game_state_response = get_game_state_response(&deps.storage, player)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Submit { status: Success, game_state: Some(game_state_response) })?),
    })
}

pub fn try_guess<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: String,
    color: Option<String>,
    shape: Option<String>,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;
    let mut messages: Vec<CosmosMsg> = vec![];

    let guess: Guess;

    if target != "abstain" && (color.is_none() || shape.is_none()) {
        return Err(StdError::generic_err("Invalid guess"));
    }
    let color_type: Color = match color.unwrap().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "black" => Color::Black,
        _ => { return Err(StdError::generic_err("Invalid color")); }
    };

    let shape_type: Shape = match shape.unwrap().as_str() {
        "triangle" => Shape::Triangle,
        "square" => Shape::Square,
        "circle" => Shape::Circle,
        "star" => Shape::Star,
        _ => { return Err(StdError::generic_err("Invalid shape")); }
    };

    match target.as_str() {
        "bag" => {
            guess = Guess {
                target: Target::Bag,
                color: Some(color_type),
                shape: Some(shape_type),
            };
        },
        "opponent" => {
            guess = Guess {
                target: Target::Opponent,
                color: Some(color_type),
                shape: Some(shape_type),
            };
        },
        "abstain" => { 
            guess = Guess {
                target: Target::Abstain,
                color: None,
                shape: None,
            };
        },
        _ => { return Err(StdError::generic_err("Invalid guess")); }
    }

    // check if already in an ongoing game
    let current_game = get_current_game(&deps.storage, &player);
    if current_game.is_none() {
        return Err(StdError::generic_err("You cannot submit a guess before joining a game"));
    }

    let mut game_state: GameState = get_game_state(&deps.storage, current_game.unwrap())?;

    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }

    if game_state.round == 0 || game_state.round_state.is_none() {
        return Err(StdError::generic_err("First round has not been initialized"));
    }

    if game_state.round >= 3 {
        return Err(StdError::generic_err("Finished round with guesses"))
    }

    let mut round_state: RoundState = game_state.round_state.unwrap();

    match RoundStage::from_u8(round_state.stage)? {
        RoundStage::BothPlayersSecondSubmit => {
            let new_guess = Some(guess.to_stored());

            if player == game_state.player_a && round_state.player_a_guess.is_none() {
                round_state.player_a_guess = new_guess;
                round_state.player_a_guess_block = Some(env.block.height);
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_guess.is_none() {
                round_state.player_b_guess = new_guess;
                round_state.player_b_guess_block = Some(env.block.height);
            } else {
                return Err(StdError::generic_err("Cannot accept a submission from player"));
            }
            round_state.stage = RoundStage::OnePlayerGuess.u8_val();

            let round_result: RoundResult;

            if guess.target == Target::Abstain {
                round_result = RoundResult::Abstain;
            } else if guess.target == Target::Bag {
                if guess.color.unwrap() == Color::from_u8(round_state.bag_chip.color)? && guess.shape.unwrap() == Shape::from_u8(round_state.bag_chip.shape)? {
                    round_result = RoundResult::BagCorrect;
                } else {
                    round_result = RoundResult::BagWrong;
                }
            } else { // Target::Opponent
                let opponent_chip: Chip;
                if player == game_state.player_a {
                    opponent_chip = round_state.player_b_chip.clone().to_humanized()?;
                } else {
                    opponent_chip = round_state.player_a_chip.clone().to_humanized()?;
                }
                if guess.color.unwrap() == opponent_chip.color && guess.shape.unwrap() == opponent_chip.shape {
                    round_result = RoundResult::OpponentCorrect;
                } else {
                    round_result = RoundResult::OpponentWrong;
                }
            }

            if player == game_state.player_a {
                round_state.player_a_round_result = Some(round_result.u8_val());
            } else {
                round_state.player_b_round_result = Some(round_result.u8_val());
            }

            game_state.round_state = Some(round_state);
            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        RoundStage::OnePlayerGuess => {
            let new_guess = Some(guess.clone().to_stored());

            if player == game_state.player_a && round_state.player_a_guess.is_none() {
                round_state.player_a_guess = new_guess;
                round_state.player_a_guess_block = Some(env.block.height);
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_guess.is_none() {
                round_state.player_b_guess = new_guess;
                round_state.player_b_guess_block = Some(env.block.height);
            } else {
                return Err(StdError::generic_err("Cannot accept a submission from player"));
            }
            round_state.stage = RoundStage::Finished.u8_val();

            let round_result: RoundResult;

            if guess.target == Target::Abstain {
                round_result = RoundResult::Abstain;
            } else if guess.target == Target::Bag {
                if guess.color.unwrap() == Color::from_u8(round_state.bag_chip.color)? && guess.shape.unwrap() == Shape::from_u8(round_state.bag_chip.shape)? {
                    round_result = RoundResult::BagCorrect;
                } else {
                    round_result = RoundResult::BagWrong;
                }
            } else { // Target::Opponent
                let opponent_chip: Chip;
                if player == game_state.player_a {
                    opponent_chip = round_state.player_b_chip.clone().to_humanized()?;
                } else {
                    opponent_chip = round_state.player_a_chip.clone().to_humanized()?;
                }
                if guess.color.unwrap() == opponent_chip.color && guess.shape.unwrap() == opponent_chip.shape {
                    round_result = RoundResult::OpponentCorrect;
                } else {
                    round_result = RoundResult::OpponentWrong;
                }
            }

            if player == game_state.player_a {
                round_state.player_a_round_result = Some(round_result.u8_val());
            } else {
                round_state.player_b_round_result = Some(round_result.u8_val());
            }

            game_state.round_state = Some(round_state.clone());

            // now only one round
            // check if it goes to pick reward round
            let player_a_round_result = RoundResult::from_u8(round_state.player_a_round_result.unwrap())?;
            let player_b_round_result = RoundResult::from_u8(round_state.player_b_round_result.unwrap())?;
            if (player_a_round_result == RoundResult::BagCorrect && player_b_round_result == RoundResult::BagCorrect) ||
               (player_a_round_result == RoundResult::OpponentCorrect && player_b_round_result == RoundResult::OpponentCorrect) ||
               (player_a_round_result == RoundResult::Abstain && player_b_round_result == RoundResult::Abstain) {
                // advance to the pick reward round
                game_state.round = 3;
            } else {
                // game does not go to pick reward round, so it is finished
                game_state.finished = true;

                // check winners and losers
                if (
                        player_a_round_result == RoundResult::BagCorrect && (
                        player_b_round_result == RoundResult::BagWrong || 
                        player_b_round_result == RoundResult::OpponentWrong || 
                        player_b_round_result == RoundResult::Abstain)
                    ) || (
                        player_a_round_result == RoundResult::OpponentCorrect && (
                        player_b_round_result == RoundResult::BagCorrect ||
                        player_b_round_result == RoundResult::BagWrong ||
                        player_b_round_result == RoundResult::OpponentWrong ||
                        player_b_round_result == RoundResult::Abstain)
                    ) || (
                        player_a_round_result == RoundResult::Abstain && (
                        player_b_round_result == RoundResult::BagWrong ||
                        player_b_round_result == RoundResult::OpponentWrong)
                   ) 
                {
                    // Player A WINS
                    game_state.result = Some(GameResult::AWon.u8_val());
                    let winnings = game_state.player_a_wager.unwrap_or(0) + game_state.player_b_wager.unwrap_or(0);
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        from_address: env.contract.address,
                        to_address: deps.api.human_address(&game_state.player_a)?,
                        amount: vec![Coin {
                            denom: DENOM.to_string(),
                            amount: Uint128(winnings),
                        }],
                    }));
                } else if (
                        player_b_round_result == RoundResult::BagCorrect && (
                        player_a_round_result == RoundResult::BagWrong || 
                        player_a_round_result == RoundResult::OpponentWrong || 
                        player_a_round_result == RoundResult::Abstain)
                    ) || (
                        player_b_round_result == RoundResult::OpponentCorrect && (
                        player_a_round_result == RoundResult::BagCorrect ||
                        player_a_round_result == RoundResult::BagWrong ||
                        player_a_round_result == RoundResult::OpponentWrong ||
                        player_a_round_result == RoundResult::Abstain)
                    ) || (
                        player_b_round_result == RoundResult::Abstain && (
                        player_a_round_result == RoundResult::BagWrong ||
                        player_a_round_result == RoundResult::OpponentWrong)
                    ) 
                {
                    // Player B WINS
                    game_state.result = Some(GameResult::BWon.u8_val());
                    let winnings = game_state.player_a_wager.unwrap_or(0) + game_state.player_b_wager.unwrap_or(0);
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        from_address: env.contract.address,
                        to_address: deps.api.human_address(&game_state.player_b.clone().unwrap())?,
                        amount: vec![Coin {
                            denom: DENOM.to_string(),
                            amount: Uint128(winnings),
                        }],
                    }));
                } else if (
                        player_a_round_result == RoundResult::BagWrong && (
                        player_b_round_result == RoundResult::BagWrong || 
                        player_b_round_result == RoundResult::OpponentWrong)
                    ) || (
                        player_a_round_result == RoundResult::OpponentWrong && (
                        player_b_round_result == RoundResult::BagWrong ||
                        player_b_round_result == RoundResult::OpponentWrong)
                    )
                {
                    // Both LOSE
                    game_state.result = Some(GameResult::BothLose.u8_val());

                    // record the increase in the pool
                    let mut pool = get_pool(&deps.storage)?;
                    pool = pool + game_state.player_a_wager.unwrap_or(0) + game_state.player_b_wager.unwrap_or(0);
                    set_pool(&mut deps.storage, pool)?;
                }
            }

            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        _ => { return Err(StdError::generic_err("Not a guess round")); }
    }
    
    let game_state_response = get_game_state_response(&deps.storage, player)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Guess { status: Success, game_state: Some(game_state_response) })?),
    })
}

pub fn try_pick_reward<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    reward: String,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;
    let mut messages: Vec<CosmosMsg> = vec![];
    let mut token_id: Option<String> = None;

    if reward != "nft" && reward != "pool" {
        return Err(StdError::generic_err("Invalid reward selection"));
    }

    // check if already in an ongoing game
    let current_game = get_current_game(&deps.storage, &player);
    if current_game.is_none() {
        return Err(StdError::generic_err("You cannot pick a reward before joining a game"));
    }
    
    let mut game_state: GameState = get_game_state(&deps.storage, current_game.unwrap())?;
    
    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }
    
    if game_state.round == 0 || game_state.round_state.is_none() {
        return Err(StdError::generic_err("First round has not been initialized"));
    }
    
    if game_state.round < 3 {
        return Err(StdError::generic_err("Reward round has not started"))
    }

    // check if other player has picked reward
    let mut waiting_for_other_player: bool = false;
    if (player == game_state.player_a && game_state.player_b_reward_pick.is_none()) ||
       (player == game_state.player_b.clone().unwrap() && game_state.player_a_reward_pick.is_none())
    {
        waiting_for_other_player = true;
    }

    if player == game_state.player_a {
        game_state.player_a_reward_pick_block = Some(env.block.height);
        if reward == "nft" {
            game_state.player_a_reward_pick = Some(REWARD_NFT);
        } else if reward == "pool" {
            game_state.player_a_reward_pick = Some(REWARD_POOL);
        }
    } else if player == game_state.player_b.clone().unwrap() {
        game_state.player_b_reward_pick_block = Some(env.block.height);
        if reward == "nft" {
            game_state.player_b_reward_pick = Some(REWARD_NFT);
        } else if reward == "pool" {
            game_state.player_b_reward_pick = Some(REWARD_POOL);
        }
    }

    if !waiting_for_other_player {
        // determine reward for both
        let player_a_reward_pick = game_state.player_a_reward_pick.unwrap();
        let player_b_reward_pick = game_state.player_b_reward_pick.unwrap();
        if player_a_reward_pick == player_b_reward_pick {
            // both picked the same reward
            // refund wagers
            game_state.result = Some(GameResult::NoReward.u8_val());
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: deps.api.human_address(&game_state.player_a)?,
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(game_state.player_a_wager.unwrap_or(0)),
                }],
            }));
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: deps.api.human_address(&game_state.player_b.clone().unwrap())?,
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(game_state.player_b_wager.unwrap_or(0)),
                }],
            }));
        } else {
            // give out rewards

            // prepare the nft
            token_id = Some(format!("game-badge-{}", current_game.unwrap()));
            let name = format!("prisnr.games");
            let random_bytes: [u8; 8] = get_random_number(&deps.storage).to_be_bytes();
            let rgb = format!("{:x?}{:x?}{:x?}", random_bytes[0], random_bytes[1], random_bytes[2]);
            let random_url = format!(
                "{:x?}{:x?}{:x?}{:x?}{:x?}{:x?}{:x?}{:x?}", 
                random_bytes[0], random_bytes[1], random_bytes[2], random_bytes[3], 
                random_bytes[4], random_bytes[5], random_bytes[6], random_bytes[7]
            );
            let image = format!("https://prisnr.games/nft/{}/{}", random_url, current_game.unwrap());
            let description = format!("Secret Prisoners game badge {}", current_game.unwrap());

            let public_metadata: Option<Metadata> = Some(Metadata{
                extension: Some(Extension{
                    name: Some(name.clone()),
                    description: None,
                    image: None,
                    background_color: Some(rgb.clone()),
                    image_data: None,
                    attributes: None,
                    animation_url: None,
                    youtube_url: None,
                    external_url: None,
                    media: None,
                    protected_attributes: None,
                }),
                token_uri: None,
            });

            let private_metadata: Option<Metadata> = Some(Metadata{
                extension: Some(Extension{
                    name: Some(name),
                    description: Some(description),
                    image: Some(image),
                    background_color: Some(rgb),
                    image_data: None,
                    attributes: None,
                    animation_url: None,
                    youtube_url: None,
                    external_url: None,
                    media: None,
                    protected_attributes: None,
                }),
                token_uri: None,
            });
            let minter: ContractInfo = get_minter(&deps.storage)?.to_humanized(&deps.api)?;

            // jackpot is equal to half of the current pool
            let current_pool = get_pool(&deps.storage)?;
            let jackpot = current_pool / 2;
            if player_a_reward_pick == REWARD_POOL {
                game_state.result = Some(GameResult::AJackpotBNft.u8_val());
                if jackpot > 0 {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        from_address: env.contract.address.clone(),
                        to_address: deps.api.human_address(&game_state.player_a)?,
                        amount: vec![Coin {
                            denom: DENOM.to_string(),
                            amount: Uint128(jackpot),
                        }],
                    }));
                }

                // mint and send NFT to player b
                let nft_owner: Option<HumanAddr> = Some(deps.api.human_address(&game_state.player_b.clone().unwrap())?);
                let cosmos_msg = mint_nft_msg(
                    token_id.clone(), 
                    nft_owner, 
                    public_metadata, 
                    private_metadata, 
                    None, 
                    None, 
                    256, 
                    minter.code_hash, 
                    minter.address,
                )?;
                messages.push(cosmos_msg);
            } else { // player b picked pool
                game_state.result = Some(GameResult::ANftBJackpot.u8_val());
                if jackpot > 0 {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        from_address: env.contract.address.clone(),
                        to_address: deps.api.human_address(&game_state.player_b.clone().unwrap())?,
                        amount: vec![Coin {
                            denom: DENOM.to_string(),
                            amount: Uint128(jackpot),
                        }],
                    }));
                }
                // mint and send NFT to player a
                let nft_owner: Option<HumanAddr> = Some(deps.api.human_address(&game_state.player_a)?);
                let cosmos_msg = mint_nft_msg(
                    token_id.clone(), 
                    nft_owner, 
                    public_metadata, 
                    private_metadata, 
                    None, 
                    None, 
                    256, 
                    minter.code_hash, 
                    minter.address,
                )?;
                messages.push(cosmos_msg);
            }
            set_pool(&mut deps.storage, current_pool - jackpot)?;
        }
        game_state.finished = true;
    }

    update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;

    let game_state_response = get_game_state_response(&deps.storage, player)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::PickReward { status: Success, token_id, game_state: Some(game_state_response) })?),
    })
}

pub fn try_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;
    let mut messages: Vec<CosmosMsg> = vec![];

    // check if already in an ongoing game
    let current_game = get_current_game(&deps.storage, &player);
    if current_game.is_none() {
        return Err(StdError::generic_err("You cannot withdraw before joining a game"));
    }
        
    let mut game_state: GameState = get_game_state(&deps.storage, current_game.unwrap())?;

    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }
    
    if game_state.round > 0 || game_state.round_state.is_some() {
        return Err(StdError::generic_err("Cannot withdraw once another player has joined game"));
    }
    
    game_state.finished = true;
    update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;

    if player == game_state.player_a && game_state.player_a_wager.unwrap_or(0) > 0 {
        // refund wager
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: deps.api.human_address(&player)?,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128(game_state.player_a_wager.unwrap_or(0)),
            }],
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Withdraw { status: Success, })?),
    })
}

pub fn try_receive_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    from: HumanAddr,
    token_ids: Vec<String>,
    _msg: Option<String>,
) -> StdResult<HandleResponse> {
    if token_ids.len() != 1 {
        return Err(StdError::generic_err("Can only send one powerup nft at a time"));
    }

    let player = deps.api.canonical_address(&from)?;

    // check if already in an ongoing game
    let current_game = get_current_game(&deps.storage, &player);
    if current_game.is_none() {
        return Err(StdError::generic_err("You cannot send a powerup nft before joining a game"));
    }
        
    let mut game_state: GameState = get_game_state(&deps.storage, current_game.unwrap())?;

    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    let config = get_config(&deps.storage)?;
    let viewer = Some(ViewerInfo {
        address: env.contract.address,
        viewing_key: config.viewing_key.clone(),
    });

    let minter = get_minter(&deps.storage)?.to_humanized(&deps.api)?;
    let priv_meta = private_metadata_query(
        &deps.querier,
        token_ids[0].clone(),
        viewer,
        256,
        minter.code_hash,
        minter.address,
    )?;
    if priv_meta.extension.is_some() {
        let extension = priv_meta.extension.unwrap();
        // TODO: Handle powerup NFT
    } else {
        return Err(StdError::generic_err("Invalid private metadata for powerup nft"));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchReceiveNft { status: Success, game_state: None })?),
    })
}

fn revoke_permit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    permit_name: String,
) -> StdResult<HandleResponse> {
    RevokedPermits::revoke_permit(
        &mut deps.storage,
        PREFIX_REVOKED_PERMITS,
        &env.message.sender,
        &permit_name,
    );

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RevokePermit { status: Success })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query),
    }
}

fn permit_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: Permit,
    query: QueryWithPermit,
) -> Result<Binary, StdError> {
    // Validate permit content
    let token_address = deps.api.human_address(
        &get_config(&deps.storage)?.contract_address
    )?;

    let account = validate(deps, PREFIX_REVOKED_PERMITS, &permit, token_address)?;

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::GameState {} => {
            if !permit.check_permission(&Permission::Owner) {
                return Err(StdError::generic_err(format!(
                    "No permission to query game state, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_game_state(deps, &account)
        }
        QueryWithPermit::PlayerStats {} => {
            if !permit.check_permission(&Permission::Owner) {
                return Err(StdError::generic_err(format!(
                    "No permission to query player stats, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_player_stats(deps, &account)
        }
    }
}

fn color_to_string(color: Color) -> String {
    match color {
        Color::Red => "color:red".to_string(),
        Color::Green => "color:green".to_string(),
        Color::Blue => "color:blue".to_string(),
        Color::Black => "color:black".to_string(),
    }
}

fn shape_to_string(shape: Shape) -> String {
    match shape {
        Shape::Triangle => "shape:triangle".to_string(),
        Shape::Square => "shape:square".to_string(),
        Shape::Circle => "shape:circle".to_string(),
        Shape::Star => "shape:star".to_string(),
    }
}

fn hint_to_string(hint: Hint) -> String {
    match hint {
        Hint::NobodyHasRed => "nobody_has|color:red".to_string(),
        Hint::NobodyHasGreen => "nobody_has|color:green".to_string(),
        Hint::NobodyHasBlue => "nobody_has|color:blue".to_string(),
        Hint::NobodyHasBlack => "nobody_has|color:black".to_string(),
        Hint::NobodyHasTriangle => "nobody_has|shape:triangle".to_string(),
        Hint::NobodyHasSquare => "nobody_has|shape:square".to_string(),
        Hint::NobodyHasCircle => "nobody_has|shape:circle".to_string(),
        Hint::NobodyHasStar => "nobody_has|shape:star".to_string(),
        Hint::IHaveRed => "i_have|color:red".to_string(),
        Hint::IHaveGreen => "i_have|color:green".to_string(),
        Hint::IHaveBlue => "i_have|color:blue".to_string(),
        Hint::IHaveBlack => "i_have|color:black".to_string(),
        Hint::IHaveTriangle => "i_have|shape:triangle".to_string(),
        Hint::IHaveSquare => "i_have|shape:square".to_string(),
        Hint::IHaveCircle => "i_have|shape:circle".to_string(),
        Hint::IHaveStar => "i_have|shape:star".to_string(),
    }
}

fn target_to_string(target: Target) -> String {
    match target {
        Target::Abstain => "abstain".to_string(),
        Target::Bag => "bag".to_string(),
        Target::Opponent => "opponent".to_string(),
    }
}

fn guess_to_string(guess: Guess) -> String {
    let target_str = target_to_string(guess.target);
    let mut color_str = "".to_string();
    let mut shape_str = "".to_string();
    if guess.color.is_some() {
        color_str = color_to_string(guess.color.unwrap());
    }
    if guess.shape.is_some() {
        shape_str = shape_to_string(guess.shape.unwrap());
    }
    let guess_str = format!("{}|{}|{}", target_str, color_str, shape_str);
    guess_str
}

fn round_result_to_string(round_result: RoundResult) -> String {
    match round_result {
        RoundResult::BagCorrect => "bag|correct".to_string(),
        RoundResult::BagWrong => "bag|wrong".to_string(),
        RoundResult::OpponentCorrect => "opponent|correct".to_string(),
        RoundResult::OpponentWrong => "opponent|wrong".to_string(),
        RoundResult::Abstain => "abstain".to_string(),
    }
}

fn bitmask_to_string(bitmask: u8) -> String {
    match bitmask {
        RED => "color:red".to_string(),
        GREEN => "color:green".to_string(),
        BLUE => "color:blue".to_string(),
        BLACK => "color:black".to_string(),
        TRIANGLE => "shape:triangle".to_string(),
        SQUARE => "shape:square".to_string(),
        CIRCLE => "shape:circle".to_string(),
        STAR => "shape:star".to_string(),
        _ => "".to_string(),
    }
}

fn get_game_state_response<S: Storage>(
    storage: &S,
    player: CanonicalAddr,
) -> StdResult<GameStateResponse> {
    let mut round: Option<u8> = None;
    let mut wager: Option<Uint128> = None;
    let mut chip_color: Option<String> = None;
    let mut chip_shape: Option<String> = None;
    let mut hint: Option<String> = None;
    let mut first_round_start_block: Option<u64> = None;
    let mut first_submit: Option<String> = None;
    let mut first_submit_block: Option<u64> = None;
    let mut opponent_first_submit: Option<String> = None;
    let mut first_extra_secret: Option<String> = None;
    let mut second_submit_turn_start_block: Option<u64> = None;
    let mut second_submit: Option<String> = None;
    let mut second_submit_block: Option<u64> = None;
    let mut opponent_second_submit: Option<String> = None;
    let mut second_extra_secret: Option<String> = None;
    let mut guess_turn_start_block: Option<u64> = None;
    let mut guess: Option<String> = None;
    let mut guess_block: Option<u64> = None;
    let mut opponent_guess: Option<String> = None;
    let mut round_result: Option<String> = None;
    let mut opponent_round_result: Option<String> = None;
    let mut pick_reward_round_start_block: Option<u64> = None;
    let mut finished: Option<bool> = None;
    let mut result: Option<String> = None;

    let current_game = get_current_game(storage, &player);
    if current_game.is_some() {
        let game_state: GameState = get_game_state(storage, current_game.unwrap())?;
        if player == game_state.player_a {
            wager = Some(Uint128(game_state.player_a_wager.unwrap_or(0)));
            round = Some(game_state.round);
            finished = Some(game_state.finished);
            if game_state.result.is_some() {
                let game_result = GameResult::from_u8(game_state.result.unwrap())?;
                if game_result == GameResult::AWon {
                    result = Some("you won wager".to_string());
                } else if game_result == GameResult::BWon || game_result == GameResult::BothLose {
                    result = Some("you lost wager".to_string());
                } else if game_result == GameResult::AJackpotBNft {
                    result = Some("you won jackpot".to_string());
                } else if game_result == GameResult::ANftBJackpot {
                    result = Some("you won nft".to_string());
                } else if game_result == GameResult::NoReward {
                    result = Some("you lost reward".to_string());
                }
            }
            if game_state.round_state.is_some() {
                let round_state = game_state.round_state.unwrap();
                first_round_start_block = Some(round_state.round_start_block);
                let chip = round_state.player_a_chip;
                chip_color = Some(color_to_string(Color::from_u8(chip.color)?));
                chip_shape = Some(shape_to_string(Shape::from_u8(chip.shape)?));
                let initial_hint = round_state.player_a_first_hint;
                hint = Some(hint_to_string(Hint::from_u8(initial_hint)?));
                if round_state.player_a_first_submit.is_some() {
                    first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_first_submit.unwrap())?));
                    first_submit_block = round_state.player_a_first_submit_block;
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_b_first_submit.is_some() {
                        if round_state.player_a_first_extra_secret.is_some() {
                            first_extra_secret = Some(hint_to_string(Hint::from_u8(round_state.player_a_first_extra_secret.unwrap())?));
                        } else {
                            opponent_first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_first_submit.unwrap())?));
                        }
                        second_submit_turn_start_block = Some(max(
                            round_state.player_a_first_submit_block.unwrap(),
                            round_state.player_b_first_submit_block.unwrap()
                        ));
                    }
                }
                if round_state.player_a_second_submit.is_some() {
                    second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_second_submit.unwrap())?));
                    second_submit_block = round_state.player_a_second_submit_block;
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_b_second_submit.is_some() {
                        if round_state.player_a_second_extra_secret.is_some() {
                            second_extra_secret = Some(hint_to_string(Hint::from_u8(round_state.player_a_second_extra_secret.unwrap())?));
                        } else {
                            opponent_second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_second_submit.unwrap())?));
                        }
                        guess_turn_start_block = Some(max(
                            round_state.player_a_second_submit_block.unwrap(),
                            round_state.player_b_second_submit_block.unwrap()
                        ));
                    }
                }
                if round_state.player_a_guess.is_some() {
                    guess = Some(guess_to_string(round_state.player_a_guess.unwrap().to_humanized()?));
                    guess_block = round_state.player_a_guess_block;
                    // player cannot see opponent's guess until made own guess
                    if round_state.player_b_guess.is_some() {
                        opponent_guess = Some(guess_to_string(round_state.player_b_guess.unwrap().to_humanized()?));
                        if game_state.round == 3 {
                            // went to pick reward round, send block when started
                            pick_reward_round_start_block = Some(max(
                                round_state.player_a_guess_block.unwrap(),
                                round_state.player_b_guess_block.unwrap()
                            ));
                        }
                    }
                }
                if round_state.player_a_round_result.is_some() {
                    round_result = Some(round_result_to_string(RoundResult::from_u8(round_state.player_a_round_result.unwrap())?));
                    // player cannot see opponent's round result until own round result if available
                    if round_state.player_b_round_result.is_some() {
                        opponent_round_result = Some(round_result_to_string(RoundResult::from_u8(round_state.player_b_round_result.unwrap())?));
                    }
                }
            }
        } else if player == game_state.player_b.unwrap() {
            wager = Some(Uint128(game_state.player_b_wager.unwrap_or(0)));
            round = Some(game_state.round);
            finished = Some(game_state.finished);
            if game_state.result.is_some() {
                let game_result = GameResult::from_u8(game_state.result.unwrap())?;
                if game_result == GameResult::BWon {
                    result = Some("you won wager".to_string());
                } else if game_result == GameResult::AWon || game_result == GameResult::BothLose {
                    result = Some("you lost wager".to_string());
                } else if game_result == GameResult::AJackpotBNft {
                    result = Some("you won nft".to_string());
                } else if game_result == GameResult::ANftBJackpot {
                    result = Some("you won jackpot".to_string());
                } else if game_result == GameResult::NoReward {
                    result = Some("you lost reward".to_string());
                }
            }
            if game_state.round_state.is_some() {
                let round_state = game_state.round_state.unwrap();
                first_round_start_block = Some(round_state.round_start_block);
                let chip = round_state.player_b_chip;
                chip_color = Some(color_to_string(Color::from_u8(chip.color)?));
                chip_shape = Some(shape_to_string(Shape::from_u8(chip.shape)?));
                let initial_hint = round_state.player_b_first_hint;
                hint = Some(hint_to_string(Hint::from_u8(initial_hint)?));
                if round_state.player_b_first_submit.is_some() {
                    first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_first_submit.unwrap())?));
                    first_submit_block = round_state.player_b_first_submit_block;
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_a_first_submit.is_some() {
                        if round_state.player_b_first_extra_secret.is_some() {
                            first_extra_secret = Some(hint_to_string(Hint::from_u8(round_state.player_b_first_extra_secret.unwrap())?));
                        } else {
                            opponent_first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_first_submit.unwrap())?));
                        }
                        second_submit_turn_start_block = Some(max(
                            round_state.player_a_first_submit_block.unwrap(),
                            round_state.player_b_first_submit_block.unwrap()
                        ));
                    }
                }
                if round_state.player_b_second_submit.is_some() {
                    second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_second_submit.unwrap())?));
                    second_submit_block = round_state.player_b_second_submit_block;
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_a_second_submit.is_some() {
                        if round_state.player_b_second_extra_secret.is_some() {
                            second_extra_secret = Some(hint_to_string(Hint::from_u8(round_state.player_b_second_extra_secret.unwrap())?));
                        } else {
                            opponent_second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_second_submit.unwrap())?));
                        }
                        guess_turn_start_block = Some(max(
                            round_state.player_a_second_submit_block.unwrap(),
                            round_state.player_b_second_submit_block.unwrap()
                        ));
                    }
                }
                if round_state.player_b_guess.is_some() {
                    guess = Some(guess_to_string(round_state.player_b_guess.unwrap().to_humanized()?));
                    guess_block = round_state.player_b_guess_block;
                    // player cannot see opponent's guess until made own guess
                    if round_state.player_a_guess.is_some() {
                        opponent_guess = Some(guess_to_string(round_state.player_a_guess.unwrap().to_humanized()?));
                        if game_state.round == 3 {
                            // went to pick reward round, send block when started
                            pick_reward_round_start_block = Some(max(
                                round_state.player_a_guess_block.unwrap(),
                                round_state.player_b_guess_block.unwrap()
                            ));
                        }
                    }
                }
                if round_state.player_b_round_result.is_some() {
                    round_result = Some(round_result_to_string(RoundResult::from_u8(round_state.player_b_round_result.unwrap())?));
                    // player cannot see opponent's round result until own round result if available
                    if round_state.player_a_round_result.is_some() {
                        opponent_round_result = Some(round_result_to_string(RoundResult::from_u8(round_state.player_a_round_result.unwrap())?));
                    }
                }
            }
        }
    }

    Ok(GameStateResponse {
        round,
        wager,
        chip_color,
        chip_shape,
        hint,
        first_round_start_block,
        first_submit,
        first_submit_block,
        opponent_first_submit,
        first_extra_secret,
        second_submit_turn_start_block,
        second_submit,
        second_submit_block,
        opponent_second_submit,
        second_extra_secret,
        guess_turn_start_block,
        guess,
        guess_block,
        opponent_guess,
        round_result,
        opponent_round_result,
        pick_reward_round_start_block,
        finished,
        result,
    })
}

fn query_game_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
) -> StdResult<Binary> {
    let player = deps.api.canonical_address(account)?;
    let game_state_response = get_game_state_response(&deps.storage, player)?;

    let response = QueryAnswer::GameState {
        round: game_state_response.round,
        wager: game_state_response.wager,
        chip_color: game_state_response.chip_color,
        chip_shape: game_state_response.chip_shape,
        hint: game_state_response.hint,
        first_round_start_block: game_state_response.first_round_start_block,
        first_submit: game_state_response.first_submit,
        first_submit_block: game_state_response.first_submit_block,
        opponent_first_submit: game_state_response.opponent_first_submit,
        first_extra_secret: game_state_response.first_extra_secret,
        second_submit_turn_start_block: game_state_response.second_submit_turn_start_block,
        second_submit: game_state_response.second_submit,
        second_submit_block: game_state_response.second_submit_block,
        opponent_second_submit: game_state_response.opponent_second_submit,
        second_extra_secret: game_state_response.second_extra_secret,
        guess_turn_start_block: game_state_response.guess_turn_start_block,
        guess: game_state_response.guess,
        guess_block: game_state_response.guess_block,
        opponent_guess: game_state_response.opponent_guess,
        round_result: game_state_response.round_result,
        opponent_round_result: game_state_response.opponent_round_result,
        pick_reward_round_start_block: game_state_response.pick_reward_round_start_block,
        finished: game_state_response.finished,
        result: game_state_response.result,
    };
    to_binary(&response)
}

fn query_player_stats<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _account: &HumanAddr,
) -> StdResult<Binary> {
    let response = QueryAnswer::PlayerStats {
        info: "TODO".to_string()
    };
    to_binary(&response)
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins,};
    use crate::msg::{InitMsg};
    use crate::random::{get_random_color,};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            admin: None,
            rounds_per_game: 2,
            red_weight: Some(25),
            blue_weight: Some(25),
            green_weight: Some(25),
            black_weight: Some(25),
            triangle_weight: Some(25),
            square_weight: Some(25),
            circle_weight: Some(25),
            star_weight: Some(25),
            stakes: Some(Uint128(1000000)),
            timeout: Some(20),
            entropy: "random".to_string(),
        };
        let env = mock_env("creator", &coins(1000, DENOM));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        //let res = query(&deps, QueryMsg::PlayerStats {}).unwrap();
        //let value: CountResponse = from_binary(&res).unwrap();
        //assert_eq!(17, value.count);
    }

    #[test]
    fn random_color_shape() {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("creator", &coins(1000, DENOM));

        let msg = InitMsg {
            admin: None,
            rounds_per_game: 2,
            red_weight: Some(25),
            blue_weight: Some(25),
            green_weight: Some(25),
            black_weight: Some(25),
            triangle_weight: Some(25),
            square_weight: Some(25),
            circle_weight: Some(25),
            star_weight: Some(25),
            stakes: Some(Uint128(1000000)),
            timeout: Some(20),
            entropy: "random".to_string(),
        };
        let _res = init(&mut deps, env.clone(), msg).unwrap();

        let msg = HandleMsg::Join { padding: None, };

        let mut fresh_entropy = to_binary(&msg).unwrap().0;
        fresh_entropy.extend(to_binary(&env).unwrap().0);
        println!("{:?}", fresh_entropy);

        supply_more_entropy(&mut deps.storage, fresh_entropy.as_slice()).unwrap();
        let mut color_options = vec![
            Color::Red,
            Color::Green,
            Color::Blue,
            Color::Black,
        ];
        println!("{:?}", color_options);
        
        let color = get_random_color(&deps.storage, &mut color_options, true);
        println!("{:?}", color);

        let mut fresh_entropy = to_binary(&msg).unwrap().0;
        fresh_entropy.extend(to_binary(&env).unwrap().0);
        println!("{:?}", fresh_entropy);

        supply_more_entropy(&mut deps.storage, fresh_entropy.as_slice()).unwrap();

        let mut color_options = vec![
            Color::Red,
            Color::Green,
            Color::Blue,
            Color::Black,
        ];
        println!("{:?}", color_options);
        
        let color = get_random_color(&deps.storage, &mut color_options, true);
        println!("{:?}", color);
    }
    */
/*
    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // not anyone can reset
        let unauth_env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env("creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
*/
/*
}
*/