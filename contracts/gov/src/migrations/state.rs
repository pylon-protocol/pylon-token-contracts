use cosmwasm_std::{CanonicalAddr, Decimal, DepsMut, Env, Response, Uint128};
use cosmwasm_storage::singleton_read;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::states::config::Config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub owner: CanonicalAddr,
    pub pylon_token: CanonicalAddr,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

pub fn migrate(deps: DepsMut, _env: Env, unstaking_period: u64) -> super::MigrateResult {
    let config: LegacyConfig = singleton_read(deps.storage, b"config").load()?;
    Config::save(
        deps.storage,
        &Config {
            owner: config.owner,
            pylon_token: config.pylon_token,
            quorum: config.quorum,
            threshold: config.threshold,
            voting_period: config.voting_period,
            timelock_period: config.timelock_period,
            expiration_period: config.expiration_period,
            proposal_deposit: config.proposal_deposit,
            snapshot_period: config.snapshot_period,
            unstaking_period,
        },
    )?;

    Ok(Response::default())
}
