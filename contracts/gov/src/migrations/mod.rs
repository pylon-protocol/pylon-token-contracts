use cosmwasm_std::{DepsMut, Response, Uint128};
use cosmwasm_storage::singleton_read;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::states::state::State;

pub type MigrateResult = Result<Response, ContractError>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyState {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
    pub total_airdrop_count: u64,
    pub airdrop_update_candidates: Vec<u64>,
}

pub fn state(deps: DepsMut, total_unbondings: Uint128) -> MigrateResult {
    let state: LegacyState = singleton_read(deps.storage, b"state").load()?;

    State::save(
        deps.storage,
        &State {
            poll_count: state.poll_count,
            total_share: state.total_share,
            total_deposit: state.total_deposit,
            total_unbondings,
            total_airdrop_count: state.total_airdrop_count,
            airdrop_update_candidates: state.airdrop_update_candidates,
        },
    )?;

    Ok(Response::default())
}
