use cosmwasm_std::{
    //debug_print, 
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage,
};
use secret_toolkit::permit::{validate, Permission, Permit, RevokedPermits};

use crate::msg::{QueryWithPermit, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, space_pad, ResponseStatus::Success};
use crate::random::{supply_more_entropy};
use crate::state::{
    create_new_game, set_config, get_config, get_current_game, get_game_state, is_game_waiting_for_second_player, get_number_of_games,
    GameState, create_new_round, update_game_state,
};

/// We make sure that responses from `handle` are padded to a multiple of this size.
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    set_config(
        &mut deps.storage, 
        deps.api.canonical_address(&env.message.sender)?, 
        deps.api.canonical_address(&env.contract.address)?,
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

    // check if already in ongoing game, if yes throw error (only one game at a time allowed)
    if get_current_game(&deps.storage, &player).is_some() {
        return Err(StdError::generic_err("Finish current game before beginning a new one"));
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
        create_new_game(&mut deps.storage, &player)?;
    } else {
        // if no: add player_b to waiting game_state, create first round and assign chips
        let mut game_state = game_state.unwrap();
        game_state.player_b = Some(player);
        // TODO: add player wager parameters
        let new_round = create_new_round(&deps.storage, None, None)?;
        game_state.round_state = Some(new_round);
        game_state.round = 1_u8;
        update_game_state(&mut deps.storage, number_of_games - 1, &game_state)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Join { status: Success })?),
    })
}

pub fn try_submit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: String,
    color: String,
    shape: String,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    Ok(HandleResponse::default())
}

pub fn try_guess<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: String,
    color: Option<String>,
    shape: Option<String>,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    Ok(HandleResponse::default())
}

pub fn try_forfeit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
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

fn query_game_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
) -> StdResult<Binary> {
    let response = QueryAnswer::GameState {
        info: "TODO".to_string()
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