use astroport::{asset, pair};
use cosmwasm_std::{Querier, QuerierWrapper, StdResult, Uint128};

pub fn simulate_swap(
    querier: &QuerierWrapper,
    pair: &impl Into<String>,
    offer_asset: asset::AssetInfo,
    amount: impl Into<Uint128>,
) -> StdResult<Uint128> {
    let resp = querier.query_wasm_smart::<pair::SimulationResponse>(
        pair,
        &pair::QueryMsg::Simulation {
            offer_asset: asset::Asset {
                info: offer_asset,
                amount,
            },
        },
    )?;

    Ok(resp.return_amount + resp.commission_amount)
}
