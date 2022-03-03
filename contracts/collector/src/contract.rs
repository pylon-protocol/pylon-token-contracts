#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, coins, to_binary, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cosmwasm_storage::singleton_read;
use cw20::Cw20ExecuteMsg;
use pylon_token::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{Asset, AssetInfo, PairInfo};
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;
use terraswap::querier::{query_balance, query_pair_info, query_token_balance};

use crate::state::{read_config, store_config, Config, CONFIG};

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
            let config = read_config(deps.storage)?;

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
