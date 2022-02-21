use cosmwasm_std::{
    from_binary, to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use pylon_token::gov_msg::{AirdropMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, StakingMsg};

use crate::constant::{CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::ContractError;
use crate::states::config::Config;
use crate::states::state::State;

pub type ExecuteResult = Result<Response, ContractError>;

pub mod airdrop;
pub mod poll;
pub mod staking;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

    let response = Response::default().add_attribute("action", "instantiate");

    let config = Config {
        pylon_token: deps.api.addr_canonicalize(msg.voting_token.as_str())?,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: 0u64, // Deprecated
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
        unstaking_period: msg.unstaking_period,
    };
    config.validate()?;

    let state = State {
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
        total_unbondings: Uint128::zero(),
        total_airdrop_count: 0,
        airdrop_update_candidates: vec![],
    };

    Config::save(deps.storage, &config)?;
    State::save(deps.storage, &state)?;

    Ok(response)
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ExecuteResult {
    // only asset contract can execute this message
    let config = Config::load(deps.storage)?;
    if config.pylon_token != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Stake {}) => Ok(Response::new()
            // 1. Update reward
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Airdrop(AirdropMsg::Update {
                    target: Some(cw20_msg.sender.to_string()),
                }))?,
                funds: vec![],
            }))
            // 2. Execute Stake
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Staking(StakingMsg::StakeInternal {
                    sender: cw20_msg.sender.to_string(),
                    amount: cw20_msg.amount,
                }))?,
                funds: vec![],
            }))),
        Ok(Cw20HookMsg::CreatePoll {
            title,
            category,
            description,
            link,
            execute_msgs,
        }) => poll::create(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            title,
            category.into(),
            description,
            link,
            execute_msgs,
        ),
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "update_config");

    let api = deps.api;
    let mut config = Config::load(deps.storage)?;

    if config.owner != api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = api.addr_canonicalize(&owner)?;
    }

    if let Some(quorum) = quorum {
        config.quorum = quorum;
    }

    if let Some(threshold) = threshold {
        config.threshold = threshold;
    }

    if let Some(voting_period) = voting_period {
        config.voting_period = voting_period;
    }

    if let Some(timelock_period) = timelock_period {
        config.timelock_period = timelock_period;
    }

    if let Some(proposal_deposit) = proposal_deposit {
        config.proposal_deposit = proposal_deposit;
    }

    if let Some(period) = snapshot_period {
        config.snapshot_period = period;
    }

    Config::save(deps.storage, &config)?;

    Ok(response)
}
