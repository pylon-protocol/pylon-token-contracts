#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, WasmMsg,
};

use astroport::asset::AssetInfo;
use astroport::{asset, pair};
use cw20::Cw20ExecuteMsg;
use moneymarket::market;
use pylon_token::collector;

use crate::instructions::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse,
};
use crate::states::{Config, State, CONFIG, STATE};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
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

    STATE.save(
        deps.storage,
        &State {
            prev_harvest_time: env.block.time.seconds(),
        },
    )?;

    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let denom = "uusd";

    match msg {
        ExecuteMsg::Harvest {} => {
            let config = CONFIG.load(deps.storage)?;
            if config.controller != info.sender {
                return Err(StdError::generic_err("unauthorized"));
            }

            // update harvest time
            let mut state = STATE.load(deps.storage)?;
            state.prev_harvest_time = env.block.time.seconds();
            STATE.save(deps.storage, &state)?;

            let collect_msg = to_binary(&collector::ExecuteMsg::Collect {})?;
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
                .query_balance(env.contract.address.clone(), denom)?
                .amount
                - config.gas_reserve;

            Ok(Response::new().add_messages(vec![
                // 50% -> aUST
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::StrategyAnchor {
                        amount: ust_balance.multiply_ratio(1u128, 2u128), // 50%
                    })?,
                    funds: vec![],
                }),
                // 25% -> UST + MINE => LP + Staking
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::StrategyProvideLiquidity {
                        amount: ust_balance.multiply_ratio(1u128, 4u128), // 25%
                    })?,
                    funds: vec![],
                }),
                // 25% -> UST => MINE -> Gov
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::StrategyBuyback {
                        amount: ust_balance.multiply_ratio(1u128, 4u128), // 25%
                    })?,
                    funds: vec![],
                }),
            ]))
        }
        ExecuteMsg::StrategyAnchor { amount } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != env.contract.address && info.sender != config.controller {
                return Err(StdError::generic_err("unauthorized"));
            }

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
                        &asset::native_asset(denom.to_string(), amount).to_string(),
                    ),
                ]))
        }
        ExecuteMsg::StrategyProvideLiquidity { amount } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != env.contract.address && info.sender != config.controller {
                return Err(StdError::generic_err("unauthorized"));
            }

            let self_mine_balance = {
                let resp: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                    config.mine.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: env.contract.address.to_string(),
                    },
                )?;

                resp.balance
            };

            let token_resp: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                config.mine.clone(),
                &cw20::Cw20QueryMsg::Balance {
                    address: config.astroport_pair.to_string(),
                },
            )?;
            let pair_mine_balance = token_resp.balance;

            let native_resp = deps.querier.query_balance(&config.astroport_pair, denom)?;
            let pair_ust_balance = native_resp.amount;

            // 25% + MINE -> LP + stake
            let convert_ust =
                asset::native_asset(denom.to_string(), amount).deduct_tax(&deps.querier)?;

            // setup increase allowance message
            let approve_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.mine.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                    spender: config.astroport_pair.to_string(),
                    amount: self_mine_balance,
                    expires: None,
                })?,
                funds: vec![],
            });

            // setup lp provide message
            let provide_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.astroport_pair.to_string(),
                msg: to_binary(&pair::ExecuteMsg::ProvideLiquidity {
                    assets: [
                        asset::native_asset(denom.to_string(), convert_ust.amount),
                        asset::token_asset(
                            config.mine,
                            convert_ust
                                .amount
                                .multiply_ratio(pair_mine_balance, pair_ust_balance),
                        ),
                    ],
                    slippage_tolerance: Some(Decimal::from_ratio(1u128, 100u128)), // 1%
                    auto_stake: Some(true),
                    receiver: Some(env.contract.address.to_string()),
                })?,
                funds: vec![convert_ust],
            });

            Ok(Response::new()
                .add_messages(vec![approve_msg, provide_msg])
                .add_attributes(vec![
                    ("action", "strategy_provide_liquidity"),
                    (
                        "amount",
                        &asset::native_asset(denom.to_string(), amount).to_string(),
                    ),
                ]))
        }
        ExecuteMsg::StrategyBuyback { amount } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != env.contract.address && info.sender != config.controller {
                return Err(StdError::generic_err("unauthorized"));
            }

            // 25% -> MINE buyback

            let convert_amount =
                asset::native_asset(denom.to_string(), amount).deduct_tax(&deps.querier)?;
            let convert_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.astroport_pair.to_string(),
                msg: to_binary(&pair::ExecuteMsg::Swap {
                    offer_asset: asset::native_asset(denom.to_string(), convert_amount.amount),
                    belief_price: None,
                    max_spread: Some(Decimal::from_ratio(1u128, 100u128)), // 1%
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
                        &asset::native_asset(denom.to_string(), amount).to_string(),
                    ),
                ]))
        }
        ExecuteMsg::Withdraw { target } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != config.controller && info.sender != config.pylon_governance {
                return Err(StdError::generic_err("unauthorized"));
            }

            Ok(Response::new().add_message(match target.info {
                AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: info.sender.to_string(),
                        amount: target.amount,
                    })?,
                    funds: vec![],
                }),
                AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(target.amount.u128(), denom),
                }),
            }))
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;

            Ok(to_binary(&ConfigResponse {
                mine: config.mine.to_string(),
                controller: config.controller.to_string(),
                gas_reserve: config.gas_reserve,
                pylon_collector: config.pylon_collector.to_string(),
                pylon_governance: config.pylon_governance.to_string(),
                anchor_moneymarket: config.anchor_moneymarket.to_string(),
                astroport_pair: config.astroport_pair.to_string(),
                astroport_generator: config.astroport_generator.to_string(),
            })?)
        }
        QueryMsg::State {} => {
            let config = CONFIG.load(deps.storage)?;
            let state = STATE.load(deps.storage)?;

            let ust_balance = deps.querier.query_balance(config.pylon_collector, "uusd")?;

            Ok(to_binary(&StateResponse {
                prev_harvest_time: state.prev_harvest_time,
                pending_ust: ust_balance.amount,
            })?)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
