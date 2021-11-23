use cosmwasm_std::{
    //debug_print, 
    to_binary, Api, Binary, Coin, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage, CanonicalAddr, Uint128,
};
use secret_toolkit::permit::{validate, Permission, Permit, RevokedPermits};

use crate::msg::{GameStateResponse, QueryWithPermit, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, space_pad, ResponseStatus::Success};
use crate::random::{supply_more_entropy};
use crate::state::{
    create_new_game, set_config, get_config, get_current_game, get_game_state, get_number_of_games,
    GameState, create_new_round, update_game_state, RoundState, Config, set_current_game,
};
use crate::types::{Chip, Guess, Hint, RoundStage, RoundResult, Target, Color, Shape};

/// We make sure that responses from `handle` are padded to a multiple of this size.
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let red_weight = msg.red_weight.unwrap_or(25);
    let green_weight = msg.green_weight.unwrap_or(25);
    let blue_weight = msg.blue_weight.unwrap_or(25);
    let black_weight = msg.black_weight.unwrap_or(25);

    let triangle_weight = msg.triangle_weight.unwrap_or(25);
    let square_weight = msg.square_weight.unwrap_or(25);
    let circle_weight = msg.circle_weight.unwrap_or(25);
    let star_weight = msg.star_weight.unwrap_or(25);

    let stakes = msg.stakes.unwrap_or(Uint128(1000000));
    let stakes = stakes.u128();

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
    };

    set_config(
        &mut deps.storage, 
        config,
    )?;

    //debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
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
        HandleMsg::Forfeit { .. } => try_forfeit(deps, env),
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
        create_new_game(&mut deps.storage, &player)?;
    } else {
        // if no: add player_b to waiting game_state, create first round and assign chips
        let mut game_state = game_state.unwrap();
        game_state.player_b = Some(player.clone());
        // TODO: add player wager parameters
        let new_round = create_new_round(&deps.storage, None, None)?;
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

pub fn try_submit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: String,
    color: Option<String>,
    shape: Option<String>,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;

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
        "bag_not" => {
            if color.is_some() {
                match color.unwrap().as_str() {
                    "red" => { hint = Hint::BagNotRed },
                    "green" => { hint = Hint::BagNotGreen },
                    "blue" => { hint = Hint::BagNotBlue },
                    "black" => { hint = Hint::BagNotBlack },
                    _ => { return Err(StdError::generic_err("Invalid color")); },
                }
            } else { // shape
                match shape.unwrap().as_str() {
                    "triangle" => { hint = Hint::BagNotTriangle },
                    "square" => { hint = Hint::BagNotSquare },
                    "circle" => { hint = Hint::BagNotCircle },
                    "star" => { hint = Hint::BagNotStar },
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

    if game_state.round == 0 || game_state.round_state.is_none() {
        return Err(StdError::generic_err("First round has not been initialized"));
    }

    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }

    let mut round_state: RoundState = game_state.round_state.unwrap();

    match RoundStage::from_u8(round_state.stage)? {
        RoundStage::Initialized => {
            let new_hint = Some(hint.u8_val());

            if player == game_state.player_a && round_state.player_a_first_submit.is_none() {
                round_state.player_a_first_submit = new_hint;
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_first_submit.is_none() {
                round_state.player_b_first_submit = new_hint;
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
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_first_submit.is_none() {
                round_state.player_b_first_submit = new_hint;
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
                round_state.player_a_second_submit = new_hint;
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_second_submit.is_none() {
                round_state.player_b_second_submit = new_hint;
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
                round_state.player_a_second_submit = new_hint;
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_second_submit.is_none() {
                round_state.player_b_second_submit = new_hint;
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

    if game_state.round == 0 || game_state.round_state.is_none() {
        return Err(StdError::generic_err("First round has not been initialized"));
    }

    if game_state.finished {
        return Err(StdError::generic_err("Game is finished, join a new game"));
    }

    let mut round_state: RoundState = game_state.round_state.unwrap();

    match RoundStage::from_u8(round_state.stage)? {
        RoundStage::BothPlayersSecondSubmit => {
            let new_guess = Some(guess.to_stored());

            if player == game_state.player_a && round_state.player_a_guess.is_none() {
                round_state.player_a_guess = new_guess;
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_guess.is_none() {
                round_state.player_b_guess = new_guess;
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
            } else if player == game_state.player_b.clone().unwrap() && round_state.player_b_guess.is_none() {
                round_state.player_b_guess = new_guess;
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

            game_state.round_state = Some(round_state);

            // now only one round
            // TODO: Is this the last round?? if not, increment and reset
            game_state.finished = true;

            update_game_state(&mut deps.storage, current_game.unwrap(), &game_state)?;
        },
        _ => { return Err(StdError::generic_err("Not a guess round")); }
    }
    
    let game_state_response = get_game_state_response(&deps.storage, player)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Guess { status: Success, game_state: Some(game_state_response) })?),
    })
}

pub fn try_forfeit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let player = deps.api.canonical_address(&env.message.sender)?;
    Ok(HandleResponse::default())
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
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Black => "black".to_string(),
    }
}

fn shape_to_string(shape: Shape) -> String {
    match shape {
        Shape::Triangle => "triangle".to_string(),
        Shape::Square => "square".to_string(),
        Shape::Circle => "circle".to_string(),
        Shape::Star => "star".to_string(),
    }
}

fn hint_to_string(hint: Hint) -> String {
    match hint {
        Hint::BagNotRed => "bag_not|red".to_string(),
        Hint::BagNotGreen => "bag_not|green".to_string(),
        Hint::BagNotBlue => "bag_not|blue".to_string(),
        Hint::BagNotBlack => "bag_not|black".to_string(),
        Hint::BagNotTriangle => "bag_not|triangle".to_string(),
        Hint::BagNotSquare => "bag_not|square".to_string(),
        Hint::BagNotCircle => "bag_not|circle".to_string(),
        Hint::BagNotStar => "bag_not|star".to_string(),
        Hint::IHaveRed => "i_have|red".to_string(),
        Hint::IHaveGreen => "i_have|green".to_string(),
        Hint::IHaveBlue => "i_have|blue".to_string(),
        Hint::IHaveBlack => "i_have|black".to_string(),
        Hint::IHaveTriangle => "i_have|triangle".to_string(),
        Hint::IHaveSquare => "i_have|square".to_string(),
        Hint::IHaveCircle => "i_have|circle".to_string(),
        Hint::IHaveStar => "i_have|star".to_string(),
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

fn get_game_state_response<S: Storage>(
    storage: &S,
    player: CanonicalAddr,
) -> StdResult<GameStateResponse> {
    let mut round: Option<u8> = None;
    let mut wager: Option<Coin> = None;
    let mut chip_color: Option<String> = None;
    let mut chip_shape: Option<String> = None;
    let mut hint: Option<String> = None;
    let mut first_submit: Option<String> = None;
    let mut second_submit: Option<String> = None;
    let mut opponent_first_submit: Option<String> = None;
    let mut opponent_second_submit: Option<String> = None;
    let mut guess: Option<String> = None;
    let mut opponent_guess: Option<String> = None;
    let mut round_result: Option<String> = None;
    let mut opponent_round_result: Option<String> = None;
    let mut finished: Option<bool> = None;

    let current_game = get_current_game(storage, &player);
    if current_game.is_some() {
        let game_state: GameState = get_game_state(storage, current_game.unwrap())?;
        if player == game_state.player_a {
            round = Some(game_state.round);
            finished = Some(game_state.finished);
            if game_state.round_state.is_some() {
                let round_state = game_state.round_state.unwrap();
                wager = round_state.player_a_wager;
                let chip = round_state.player_a_chip;
                chip_color = Some(color_to_string(Color::from_u8(chip.color)?));
                chip_shape = Some(shape_to_string(Shape::from_u8(chip.shape)?));
                let initial_hint = round_state.player_a_first_hint;
                hint = Some(hint_to_string(Hint::from_u8(initial_hint)?));
                if round_state.player_a_first_submit.is_some() {
                    first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_first_submit.unwrap())?));
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_b_first_submit.is_some() {
                        opponent_first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_first_submit.unwrap())?));
                    }
                }
                if round_state.player_a_second_submit.is_some() {
                    second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_second_submit.unwrap())?));
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_b_second_submit.is_some() {
                        opponent_second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_second_submit.unwrap())?));
                    }
                }
                if round_state.player_a_guess.is_some() {
                    guess = Some(guess_to_string(round_state.player_a_guess.unwrap().to_humanized()?));
                    // player cannot see opponent's guess until made own guess
                    if round_state.player_b_guess.is_some() {
                        opponent_guess = Some(guess_to_string(round_state.player_b_guess.unwrap().to_humanized()?));
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
            round = Some(game_state.round);
            finished = Some(game_state.finished);
            if game_state.round_state.is_some() {
                let round_state = game_state.round_state.unwrap();
                wager = round_state.player_b_wager;
                let chip = round_state.player_b_chip;
                chip_color = Some(color_to_string(Color::from_u8(chip.color)?));
                chip_shape = Some(shape_to_string(Shape::from_u8(chip.shape)?));
                let initial_hint = round_state.player_b_first_hint;
                hint = Some(hint_to_string(Hint::from_u8(initial_hint)?));
                if round_state.player_b_first_submit.is_some() {
                    first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_first_submit.unwrap())?));
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_a_first_submit.is_some() {
                        opponent_first_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_first_submit.unwrap())?));
                    }
                }
                if round_state.player_b_second_submit.is_some() {
                    second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_b_second_submit.unwrap())?));
                    // player cannot see opponent's submission until made own submission
                    if round_state.player_a_second_submit.is_some() {
                        opponent_second_submit = Some(hint_to_string(Hint::from_u8(round_state.player_a_second_submit.unwrap())?));
                    }
                }
                if round_state.player_b_guess.is_some() {
                    guess = Some(guess_to_string(round_state.player_b_guess.unwrap().to_humanized()?));
                    // player cannot see opponent's guess until made own guess
                    if round_state.player_a_guess.is_some() {
                        opponent_guess = Some(guess_to_string(round_state.player_a_guess.unwrap().to_humanized()?));
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
        first_submit,
        second_submit,
        opponent_first_submit,
        opponent_second_submit,
        guess,
        opponent_guess,
        round_result,
        opponent_round_result,
        finished,
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
        first_submit: game_state_response.first_submit,
        second_submit: game_state_response.second_submit,
        opponent_first_submit: game_state_response.opponent_first_submit,
        opponent_second_submit: game_state_response.opponent_second_submit,
        guess: game_state_response.guess,
        opponent_guess: game_state_response.opponent_guess,
        round_result: game_state_response.round_result,
        opponent_round_result: game_state_response.opponent_round_result,
        finished: game_state_response.finished,
    };
    to_binary(&response)
}

fn query_player_stats<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
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
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

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
}
*/