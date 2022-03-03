#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use pylon_token::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::state::{Config, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            gov: deps.api.addr_validate(msg.gov.as_str())?,
            treasury: deps.api.addr_validate(msg.treasury.as_str())?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Collect {} => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != config.treasury {
                return Err(StdError::generic_err("unauthorized"));
            }

            let ust_balance = deps.querier.query_balance(env.contract.address, "uusd")?;

            Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: config.treasury.to_string(),
                amount: vec![ust_balance],
            })))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;

            to_binary(&ConfigResponse {
                gov: config.gov.to_string(),
                treasury: config.treasury.to_string(),
            })
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    // just override it
    CONFIG.save(
        deps.storage,
        &Config {
            gov: deps.api.addr_validate(msg.gov.as_str())?,
            treasury: deps.api.addr_validate(msg.treasury.as_str())?,
        },
    )?;

    Ok(Response::default())
}
