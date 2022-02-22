use astroport::{asset, pair};
use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Uint128};

pub fn simulate_swap(
    querier: &QuerierWrapper,
    pair: &Addr,
    offer_asset: asset::AssetInfo,
    amount: impl Into<Uint128>,
) -> StdResult<Uint128> {
    let resp: pair::SimulationResponse = querier.query_wasm_smart(
        pair,
        &pair::QueryMsg::Simulation {
            offer_asset: asset::Asset {
                info: offer_asset,
                amount: amount.into(),
            },
        },
    )?;

    Ok(resp.return_amount + resp.commission_amount)
}
