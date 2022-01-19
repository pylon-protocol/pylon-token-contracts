#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cosmwasm_storage::singleton_read;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use pylon_token::staking::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};

use crate::state::{
    read_config, read_staker_info, read_state, remove_staker_info, store_config, store_staker_info,
    store_state, ConfigV2, StakerInfoV2, StateV2,
};
use crate::state::{ConfigV1, StateV1};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &ConfigV2 {
            governance: deps.api.addr_canonicalize(msg.governance.as_str())?,
            pylon_token: deps.api.addr_canonicalize(&msg.pylon_token)?,
            staking_token: vec![deps.api.addr_canonicalize(&msg.staking_token)?],
            distribution_schedule: msg.distribution_schedule,
        },
    )?;

    store_state(
        deps.storage,
        0,
        &StateV2 {
            halted: false,
            started_at: env.block.height,
            last_distributed: env.block.height,
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::MigrateStaking {
            new_staking_contract,
        } => migrate_staking(deps, env, info, new_staking_contract),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: ConfigV2 = read_config(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            // only staking token contract can execute this message
            if config.staking_token[config.staking_token.len() - 1]
                != deps.api.addr_canonicalize(info.sender.as_str())?
            {
                return Err(StdError::generic_err("unauthorized"));
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            bond(deps, env, cw20_sender, cw20_msg.amount)
        }
        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

pub fn bond(deps: DepsMut, env: Env, sender_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let mut resp = Response::new().add_attribute("action", "bond");
    let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(sender_addr.as_str())?;

    let config: ConfigV2 = read_config(deps.storage)?;
    let staking_token_version = (config.staking_token.len() - 1) as u64;
    let mut state: StateV2 = read_state(deps.storage, staking_token_version)?;
    let mut staker_info: StakerInfoV2 = read_staker_info(deps.storage, &sender_addr_raw)?;

    // legacy
    if staking_token_version != staker_info.staking_token_version {
        let legacy_token_version = staker_info.staking_token_version;
        let mut legacy_state = read_state(deps.storage, legacy_token_version)?;
        compute_staker_reward(&legacy_state, &mut staker_info)?;

        let legacy_unbond_amount = staker_info.bond_amount;
        decrease_bond_amount(&mut legacy_state, &mut staker_info, legacy_unbond_amount)?;

        staker_info.staking_token_version = staking_token_version;

        store_state(deps.storage, legacy_token_version, &legacy_state)?;

        // force return to owner
        resp = resp.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.staking_token[legacy_token_version as usize])?
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender_addr.to_string(),
                amount: legacy_unbond_amount,
            })?,
            funds: vec![],
        }));
    }

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;

    // Increase bond_amount
    increase_bond_amount(&mut state, &mut staker_info, amount);

    // Store updated state with staker's staker_info
    store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    store_state(deps.storage, staking_token_version, &state)?;

    Ok(resp.add_attributes(vec![
        ("owner", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

pub fn unbond(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let config: ConfigV2 = read_config(deps.storage)?;
    let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(info.sender.as_str())?;

    let mut staker_info: StakerInfoV2 = read_staker_info(deps.storage, &sender_addr_raw)?;
    let mut state: StateV2 = read_state(deps.storage, staker_info.staking_token_version)?;

    if staker_info.bond_amount < amount {
        return Err(StdError::generic_err("Cannot unbond more than bond amount"));
    }

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;

    // Decrease bond_amount
    decrease_bond_amount(&mut state, &mut staker_info, amount)?;

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.pending_reward.is_zero() && staker_info.bond_amount.is_zero() {
        remove_staker_info(deps.storage, &sender_addr_raw);
    } else {
        store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    }

    // Store updated state
    store_state(deps.storage, staker_info.staking_token_version, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.staking_token[staker_info.staking_token_version as usize])?
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "unbond"),
            ("owner", info.sender.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

// withdraw rewards to executor
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let sender_addr_raw = deps.api.addr_canonicalize(info.sender.as_str())?;

    let config: ConfigV2 = read_config(deps.storage)?;
    let staking_token_version = (config.staking_token.len() - 1) as u64;
    let mut state: StateV2 = read_state(deps.storage, staking_token_version)?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;

    let amount = staker_info.pending_reward;
    staker_info.pending_reward = Uint128::zero();

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.bond_amount.is_zero() {
        remove_staker_info(deps.storage, &sender_addr_raw);
    } else {
        store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    }

    // Store updated state
    store_state(deps.storage, staking_token_version, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.pylon_token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "withdraw"),
            ("owner", info.sender.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn migrate_staking(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_staking_token: String,
) -> StdResult<Response> {
    let mut config: ConfigV2 = read_config(deps.storage)?;
    if config.governance != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut state: StateV2 = read_state(deps.storage, (config.staking_token.len() - 1) as u64)?;

    // compute global reward, sets last_distributed_height to env.block.height
    compute_reward(&config, &mut state, env.block.height);

    let total_distribution_amount: Uint128 =
        config.distribution_schedule.iter().map(|item| item.2).sum();

    let block_height = env.block.height;
    // eliminate distribution slots that have been started
    config
        .distribution_schedule
        .retain(|slot| slot.0 >= block_height);

    let mut distributed_amount = Uint128::zero();
    for s in config.distribution_schedule.iter_mut() {
        if s.1 < block_height {
            // all distributed
            distributed_amount += s.2;
        } else {
            // partially distributed slot
            let num_blocks = s.1 - s.0;
            let distribution_amount_per_block: Decimal = Decimal::from_ratio(s.2, num_blocks);

            let passed_blocks = block_height - s.0;
            let distributed_amount_on_slot =
                distribution_amount_per_block * Uint128::from(passed_blocks as u128);
            distributed_amount += distributed_amount_on_slot;

            // modify distribution slot
            s.1 = block_height;
            s.2 = distributed_amount_on_slot;
        }
    }

    let legacy_token_version = (config.staking_token.len() - 1) as u64;
    let new_token_version = legacy_token_version + 1;
    state.halted = true;
    config
        .staking_token
        .push(deps.api.addr_canonicalize(new_staking_token.as_str())?);

    // update config
    store_config(deps.storage, &config)?;
    // update state
    store_state(deps.storage, legacy_token_version, &state)?;
    store_state(
        deps.storage,
        new_token_version,
        &StateV2 {
            halted: false,
            started_at: block_height,
            last_distributed: block_height,
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        },
    )?;

    let remaining_mine = total_distribution_amount.checked_sub(distributed_amount)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "migrate_staking"),
        ("distributed_amount", &distributed_amount.to_string()),
        ("remaining_amount", &remaining_mine.to_string()),
    ]))
}

fn increase_bond_amount(state: &mut StateV2, staker_info: &mut StakerInfoV2, amount: Uint128) {
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
}

fn decrease_bond_amount(
    state: &mut StateV2,
    staker_info: &mut StakerInfoV2,
    amount: Uint128,
) -> StdResult<()> {
    state.total_bond_amount = state.total_bond_amount.checked_sub(amount)?;
    staker_info.bond_amount = staker_info.bond_amount.checked_sub(amount)?;
    Ok(())
}

// compute distributed rewards and update global reward index
fn compute_reward(config: &ConfigV2, state: &mut StateV2, block_height: u64) {
    if state.halted {
        return;
    }

    if state.total_bond_amount.is_zero() {
        state.last_distributed = block_height;
        return;
    }

    let mut distributed_amount: Uint128 = Uint128::zero();
    for s in config.distribution_schedule.iter() {
        if s.0 > block_height || s.1 < state.last_distributed {
            continue;
        }

        // min(s.1, block_height) - max(s.0, last_distributed)
        let passed_blocks =
            std::cmp::min(s.1, block_height) - std::cmp::max(s.0, state.last_distributed);

        let num_blocks = s.1 - s.0;
        let distribution_amount_per_block: Decimal = Decimal::from_ratio(s.2, num_blocks);
        distributed_amount += distribution_amount_per_block * Uint128::from(passed_blocks as u128);
    }

    state.last_distributed = block_height;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_bond_amount);
}

// withdraw reward to pending reward
fn compute_staker_reward(state: &StateV2, staker_info: &mut StakerInfoV2) -> StdResult<()> {
    let pending_reward = (staker_info.bond_amount * state.global_reward_index)
        .checked_sub(staker_info.bond_amount * staker_info.reward_index)?;

    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {
            token_version,
            block_height,
        } => to_binary(&query_state(deps, token_version, block_height)?),
        QueryMsg::StakerInfo {
            staker,
            block_height,
        } => to_binary(&query_staker_info(deps, staker, block_height)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    let resp = ConfigResponse {
        pylon_token: deps.api.addr_humanize(&config.pylon_token)?.to_string(),
        staking_token: config
            .staking_token
            .iter()
            .map(|token| deps.api.addr_humanize(token).unwrap().to_string())
            .collect(),
        distribution_schedule: config.distribution_schedule,
    };

    Ok(resp)
}

pub fn query_state(
    deps: Deps,
    token_version: Option<u64>,
    block_height: Option<u64>,
) -> StdResult<StateResponse> {
    let config = read_config(deps.storage)?;
    let mut state: StateV2 = read_state(
        deps.storage,
        token_version.unwrap_or((config.staking_token.len() - 1) as u64),
    )?;
    if let Some(block_height) = block_height {
        let config = read_config(deps.storage)?;
        compute_reward(&config, &mut state, block_height);
    }

    Ok(StateResponse {
        halted: state.halted,
        started_at: state.started_at,
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
    })
}

pub fn query_staker_info(
    deps: Deps,
    staker: String,
    block_height: Option<u64>,
) -> StdResult<StakerInfoResponse> {
    let staker_raw = deps.api.addr_canonicalize(&staker)?;

    let mut staker_info: StakerInfoV2 = read_staker_info(deps.storage, &staker_raw)?;
    if let Some(block_height) = block_height {
        let config = read_config(deps.storage)?;
        let mut state = read_state(deps.storage, staker_info.staking_token_version)?;

        compute_reward(&config, &mut state, block_height);
        compute_staker_reward(&state, &mut staker_info)?;
    }

    Ok(StakerInfoResponse {
        staker,
        staking_token_version: staker_info.staking_token_version,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    match msg {
        MigrateMsg::Migrate { governance } => {
            let legacy_config: ConfigV1 = singleton_read(deps.storage, b"config").load()?;
            let genesis = legacy_config.distribution_schedule[0].0;
            store_config(
                deps.storage,
                &ConfigV2 {
                    governance: deps.api.addr_canonicalize(governance.as_str())?,
                    pylon_token: legacy_config.pylon_token,
                    staking_token: vec![legacy_config.staking_token],
                    distribution_schedule: legacy_config.distribution_schedule,
                },
            )?;

            let legacy_state: StateV1 = singleton_read(deps.storage, b"state").load()?;
            store_state(
                deps.storage,
                0,
                &StateV2 {
                    halted: false,
                    started_at: genesis,
                    last_distributed: legacy_state.last_distributed,
                    total_bond_amount: legacy_state.total_bond_amount,
                    global_reward_index: legacy_state.global_reward_index,
                },
            )?;
        }
        MigrateMsg::General {} => {}
    }

    Ok(Response::default())
}
