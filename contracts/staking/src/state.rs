use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, Bucket, ReadonlyBucket};

static KEY_CONFIG: &[u8] = b"config";

static PREFIX_STATE: &[u8] = b"state";
static PREFIX_REWARD: &[u8] = b"reward";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigV1 {
    pub pylon_token: CanonicalAddr,
    pub staking_token: CanonicalAddr,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigV2 {
    pub governance: CanonicalAddr,
    pub pylon_token: CanonicalAddr,
    pub staking_token: Vec<CanonicalAddr>, // index = token_version
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

pub fn store_config(storage: &mut dyn Storage, config: &ConfigV2) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<ConfigV2> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateV1 {
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateV2 {
    pub halted: bool,
    pub started_at: u64,
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal,
}

pub fn store_state(
    storage: &mut dyn Storage,
    token_version: u64,
    state: &StateV2,
) -> StdResult<()> {
    Bucket::<StateV2>::new(storage, PREFIX_STATE).save(&token_version.to_be_bytes(), state)
}

pub fn read_state(storage: &dyn Storage, token_version: u64) -> StdResult<StateV2> {
    ReadonlyBucket::<StateV2>::new(storage, PREFIX_STATE).load(&token_version.to_be_bytes())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfoV1 {
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
    pub staking_token_version: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfoV2 {
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
    pub staking_token_version: u64,
}

/// returns return staker_info of the given owner
pub fn store_staker_info(
    storage: &mut dyn Storage,
    owner: &CanonicalAddr,
    staker_info: &StakerInfoV2,
) -> StdResult<()> {
    Bucket::<StakerInfoV2>::new(storage, PREFIX_REWARD).save(owner.as_slice(), staker_info)
}

/// remove staker_info of the given owner
pub fn remove_staker_info(storage: &mut dyn Storage, owner: &CanonicalAddr) {
    Bucket::<StakerInfoV2>::new(storage, PREFIX_REWARD).remove(owner.as_slice())
}

/// returns rewards owned by this owner
/// (read-only version for queries)
pub fn read_staker_info(storage: &dyn Storage, owner: &CanonicalAddr) -> StdResult<StakerInfoV2> {
    let config = read_config(storage)?;

    match ReadonlyBucket::<StakerInfoV1>::new(storage, PREFIX_REWARD).may_load(owner.as_slice())? {
        Some(staker_info) => Ok(StakerInfoV2 {
            reward_index: staker_info.reward_index,
            bond_amount: staker_info.bond_amount,
            pending_reward: staker_info.pending_reward,
            staking_token_version: staker_info.staking_token_version.unwrap_or_default(),
        }),
        None => Ok(StakerInfoV2 {
            reward_index: Decimal::zero(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            staking_token_version: (config.staking_token.len() - 1) as u64,
        }),
    }
}
