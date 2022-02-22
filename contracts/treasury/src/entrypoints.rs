#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::cmp::min;
use std::ops::Mul;

use astroport::{asset, pair};
use cosmwasm_std::{
    coin, coins, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};
use moneymarket::market;
use pylon_token::collector;

use crate::instructions::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::querier;
use crate::states::{Config, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let api = deps.api;

    CONFIG.save(
        deps.storage,
        &Config {
            mine: api.addr_validate(&msg.mine)?,
            controller: api.addr_validate(&msg.controller)?,
            gas_reserve: msg.gas_reserve,
            pylon_collector: api.addr_validate(&msg.pylon_collector)?,
            pylon_governance: api.addr_validate(&msg.pylon_governance)?,
            anchor_moneymarket: api.addr_validate(&msg.anchor_moneymarket)?,
            astroport_pair: api.addr_validate(&msg.astroport_pair)?,
            astroport_generator: api.addr_validate(&msg.astroport_generator)?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let denom = "uusd";

    match msg {
        ExecuteMsg::Harvest {} => {
            let config = CONFIG.load(deps.storage)?;

            // TODO: migrate collector
            let collect_msg = to_binary(&collector::ExecuteMsg::Sweep {
                denom: "".to_string(),
            })?;

            let collect_wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.pylon_collector.to_string(),
                msg: collect_msg,
                funds: vec![],
            });

            let harvest_msg = to_binary(&ExecuteMsg::HarvestInternal {})?;

            let harvest_wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: harvest_msg,
                funds: vec![],
            });

            Ok(Response::new().add_messages(vec![collect_wasm_msg, harvest_wasm_msg]))
        }
        ExecuteMsg::HarvestInternal {} => {
            if env.contract.address != info.sender {
                return Err(StdError::generic_err("unauthorized"));
            }

            let config = CONFIG.load(deps.storage)?;
            let ust_balance = deps
                .querier
                .query_balance(env.contract.address, denom)?
                .amount
                - config.gas_reserve;

            Ok(Response::new().add_messages(vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::StrategyAnchor {
                        amount: ust_balance.multiply_ratio(1, 2),
                    })?,
                    funds: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::StrategyProvideLiquidity {
                        amount: ust_balance.multiply_ratio(1, 4),
                    })?,
                    funds: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::StrategyBuyback {
                        amount: ust_balance.multiply_ratio(1, 4),
                    })?,
                    funds: vec![],
                }),
            ]))
        }
        ExecuteMsg::StrategyAnchor { amount } => {
            let config = CONFIG.load(deps.storage)?;

            // 50% -> aUST
            let convert_amount =
                asset::native_asset(denom.to_string(), amount).deduct_tax(&deps.querier)?;
            let convert_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.anchor_moneymarket.to_string(),
                msg: to_binary(&market::ExecuteMsg::DepositStable {})?,
                funds: vec![convert_amount],
            });

            Ok(Response::new()
                .add_messages(vec![convert_msg])
                .add_attributes(vec![
                    ("action", "strategy_anchor"),
                    (
                        "amount",
                        asset::native_asset(denom.to_string(), amount).to_string(),
                    ),
                ]))
        }
        ExecuteMsg::StrategyProvideLiquidity { amount } => {
            let config = CONFIG.load(deps.storage)?;
            let mine_balance = deps
                .querier
                .query_wasm_smart::<cw20::BalanceResponse>(
                    config.mine,
                    &cw20::Cw20QueryMsg::Balance {
                        address: env.contract.address.to_string(),
                    },
                )?
                .balance;

            // 25% + MINE -> LP + stake
            let convert_amount =
                asset::native_asset(denom.to_string(), amount).deduct_tax(&deps.querier)?;
            let provide_amount = querier::simulate_swap(
                &deps.querier,
                &config.astroport_pair,
                asset::native_asset_info(denom.to_string()),
                convert_amount.amount,
            )?;
            // setup increase allowance message
            let approve_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.mine.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                    spender: config.astroport_pair.to_string(),
                    amount: mine_balance,
                    expires: None,
                })?,
                funds: vec![],
            });
            // setup lp provide message
            let convert_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.astroport_pair.to_string(),
                msg: to_binary(&pair::ExecuteMsg::ProvideLiquidity {
                    assets: [
                        asset::native_asset(denom.to_string(), convert_amount.amount),
                        asset::token_asset(
                            config.mine.clone(),
                            Uint128::from(min(provide_amount.u128(), mine_balance.u128())),
                        ),
                    ],
                    slippage_tolerance: Some(Decimal::from_ratio(1, 10)), // 10%
                    auto_stake: Some(true),
                    receiver: Some(env.contract.address.to_string()),
                })?,
                funds: coins(convert_amount.u128(), denom),
            })?;

            Ok(Response::new()
                .add_messages(vec![approve_msg, convert_msg])
                .add_attributes(vec![
                    ("action", "strategy_provide_liquidity"),
                    (
                        "amount",
                        asset::native_asset(denom.to_string(), amount).to_string(),
                    ),
                ]))
        }
        ExecuteMsg::StrategyBuyback { amount } => {
            let config = CONFIG.load(deps.storage)?;

            // 25% -> MINE buyback

            let convert_amount =
                asset::native_asset(denom.to_string(), amount).deduct_tax(&deps.querier)?;
            let convert_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.astroport_pair.to_string(),
                msg: to_binary(&pair::ExecuteMsg::Swap {
                    offer_asset: asset::native_asset(denom.to_string(), convert_amount.amount),
                    belief_price: None,
                    max_spread: Some(Decimal::from_ratio(1, 100)), // 1%
                    to: Some(config.pylon_governance.to_string()),
                })?,
                funds: vec![convert_amount],
            });

            Ok(Response::new()
                .add_messages(vec![convert_msg])
                .add_attributes(vec![
                    ("action", "strategy_buyback"),
                    (
                        "amount",
                        asset::native_asset(denom.to_string(), amount).to_string(),
                    ),
                ]))
        }
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
